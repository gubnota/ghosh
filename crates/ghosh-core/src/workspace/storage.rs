use super::{
    document::Document,
    model::{
        Manifest, Node, entity_id, invalid, node_id_collision, relative_path, validate_node_id,
    },
};
use crate::contracts::{AppError, Cancellation, ErrorCode, Result};
use cap_std::{
    ambient_authority,
    fs::{Dir, OpenOptions},
};
use fs2::FileExt;
use sha2::{Digest, Sha256};
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};

pub const MAX_DOCUMENT: u64 = 1_048_576;
pub fn hash(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

pub struct Lease {
    _file: File,
}
impl Lease {
    pub fn acquire(lock_dir: &Path, key: &str) -> Result<Self> {
        fs::create_dir_all(lock_dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(lock_dir, fs::Permissions::from_mode(0o700))?;
        }
        let dir = Dir::open_ambient_dir(lock_dir, ambient_authority())?;
        let mut options = OpenOptions::new();
        options.read(true).write(true).create(true).truncate(false);
        let file = dir
            .open_with(format!("{}.lock", hash(key.as_bytes())), &options)?
            .into_std();
        FileExt::try_lock_exclusive(&file).map_err(|e| {
            AppError::new(if e.kind() == std::io::ErrorKind::WouldBlock {
                ErrorCode::Busy
            } else {
                ErrorCode::Io
            })
        })?;
        Ok(Self { _file: file })
    }
}

pub fn canonical_package(path: &Path) -> Result<PathBuf> {
    if path.extension().and_then(|s| s.to_str()) != Some("ghosh")
        || fs::symlink_metadata(path)?.file_type().is_symlink()
    {
        return Err(AppError::new(ErrorCode::InvalidPath));
    }
    let canonical = path.canonicalize()?;
    if !canonical.is_dir() {
        return Err(AppError::new(ErrorCode::InvalidPath));
    }
    Ok(canonical)
}
/// Capability-relative IO prevents symlink substitution from escaping this root.
pub fn safe_path(dir: &Dir, path: &str) -> Result<()> {
    relative_path(path)?;
    let mut current = PathBuf::new();
    for component in Path::new(path).components() {
        current.push(component);
        match dir.symlink_metadata(&current) {
            Ok(meta) if meta.file_type().is_symlink() => {
                return Err(AppError::new(ErrorCode::InvalidPath));
            }
            Ok(_) => (),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
            Err(e) => return Err(e.into()),
        }
    }
    Ok(())
}
pub fn read_text(dir: &Dir, path: &str) -> Result<String> {
    safe_path(dir, path)?;
    let file = dir.open(path)?;
    if !file.metadata()?.is_file() || file.metadata()?.len() > MAX_DOCUMENT {
        return Err(invalid());
    }
    let mut text = String::new();
    file.take(MAX_DOCUMENT + 1)
        .read_to_string(&mut text)
        .map_err(|_| invalid())?;
    if text.len() as u64 > MAX_DOCUMENT {
        return Err(invalid());
    }
    Ok(text)
}

pub struct Loaded {
    pub doc: Document,
    pub manifest: Option<Manifest>,
    pub id: String,
    pub name: String,
    pub schema: u32,
    pub nodes: Vec<Node>,
    pub stores: BTreeMap<String, String>,
    pub revision: String,
}
pub fn load(dir: &Dir) -> Result<Loaded> {
    let text = read_text(dir, "ghosh.toml")?;
    let doc = Document::parse(&text)?;
    let schema = doc.schema()?;
    if schema > 2 {
        let id = doc
            .ast
            .get("workspace")
            .and_then(|v| v.get("id"))
            .and_then(|v| v.as_str())
            .ok_or_else(invalid)?
            .to_owned();
        if ulid::Ulid::from_string(&id).is_err() {
            return Err(invalid());
        }
        let name = doc
            .ast
            .get("workspace")
            .and_then(|v| v.get("name"))
            .and_then(|v| v.as_str())
            .unwrap_or("Newer workspace")
            .to_owned();
        return Ok(Loaded {
            doc,
            manifest: None,
            id,
            name,
            schema,
            nodes: vec![],
            stores: BTreeMap::new(),
            revision: hash(text.as_bytes()),
        });
    }
    let manifest = doc.manifest()?;
    for path in manifest.paths.all() {
        safe_path(dir, path)?;
        if !dir.metadata(path)?.is_dir() {
            return Err(AppError::new(ErrorCode::InvalidPath));
        }
    }
    let files = config_files(dir, &manifest)?;
    validated_files(files)
}
fn validated_files(files: BTreeMap<String, String>) -> Result<Loaded> {
    if files.len() > 1024
        || files.values().any(|s| s.len() as u64 > MAX_DOCUMENT)
        || files.values().map(String::len).sum::<usize>() > 16 * 1024 * 1024
    {
        return Err(invalid());
    }
    let doc = Document::parse(files.get("ghosh.toml").ok_or_else(invalid)?)?;
    let schema = doc.schema()?;
    let manifest = doc.manifest()?;
    let revision = fingerprint(&files);
    let mut nodes = Vec::new();
    let mut node_ids = BTreeSet::new();
    let mut stores = BTreeMap::new();
    let mut tunnels = BTreeSet::from(["direct".to_string()]);
    for (path, text) in &files {
        if path == "ghosh.toml" {
            continue;
        }
        // Sources are inert and may be invalid drafts edited outside Ghosh.
        // Their bytes still invalidate execution reviews and participate in CAS.
        if super::automation::source_kind(&manifest, path).is_some() {
            continue;
        }
        let parsed = Document::parse(text)?;
        let node_file = Path::new(path).parent() == Some(Path::new(&manifest.paths.nodes));
        if parsed.schema()? > if node_file { manifest.schema } else { 1 } {
            return Err(AppError::new(ErrorCode::UnsupportedSchema));
        }
        let file_id = Path::new(path)
            .file_stem()
            .and_then(|s| s.to_str())
            .ok_or_else(invalid)?;
        if node_file {
            validate_node_id(file_id)?;
        } else if !entity_id(file_id) {
            return Err(invalid());
        }
        if parsed.ast.get("id").and_then(|v| v.as_str()) != Some(file_id) {
            return Err(invalid());
        }
        if node_file {
            let node: Node = toml_edit::de::from_document(parsed.ast).map_err(|_| invalid())?;
            node.validate(file_id)?;
            if !node_ids.insert(node.id.to_ascii_lowercase()) {
                return Err(node_id_collision());
            }
            nodes.push(node);
        } else if Path::new(path).parent() == Some(Path::new(&manifest.paths.profiles)) {
            let store = parsed
                .ast
                .get("data_store")
                .and_then(|v| v.as_str())
                .ok_or_else(invalid)?;
            let uuid = uuid::Uuid::parse_str(store)
                .map_err(|_| invalid())?
                .to_string();
            if stores.values().any(|other| other == &uuid) {
                return Err(AppError::new(ErrorCode::MissingReference));
            }
            stores.insert(file_id.into(), uuid);
        } else if Path::new(path).parent() == Some(Path::new(&manifest.paths.tunnels)) {
            if file_id == "direct" {
                return Err(invalid());
            }
            tunnels.insert(file_id.into());
        }
    }
    if manifest
        .workspace
        .default_profile
        .as_ref()
        .is_some_and(|id| !stores.contains_key(id))
        || manifest
            .workspace
            .default_tunnel
            .as_ref()
            .is_some_and(|id| !tunnels.contains(id))
    {
        return Err(AppError::new(ErrorCode::MissingReference));
    }
    for node in &nodes {
        if !tunnels.contains(&node.tunnel) {
            return Err(AppError::new(ErrorCode::MissingReference));
        }
        let mut chain = BTreeSet::new();
        let mut current = node;
        while let Some(next) = current.jump.as_deref().filter(|s| !s.is_empty()) {
            if !chain.insert(current.id.as_str()) {
                return Err(AppError::new(ErrorCode::MissingReference));
            }
            current = nodes
                .iter()
                .find(|n| n.id == next)
                .ok_or_else(|| AppError::new(ErrorCode::MissingReference))?;
        }
    }
    Ok(Loaded {
        id: manifest.workspace.id.clone(),
        name: manifest.workspace.name.clone(),
        manifest: Some(manifest),
        doc,
        schema,
        nodes,
        stores,
        revision,
    })
}

fn config_files(dir: &Dir, model: &Manifest) -> Result<BTreeMap<String, String>> {
    let mut files = BTreeMap::from([("ghosh.toml".into(), read_text(dir, "ghosh.toml")?)]);
    let mut total = files["ghosh.toml"].len();
    // Future service documents remain opaque: only manifest/node/profile identity
    // and references are activated in M1.1. All TOML bytes participate in revision.
    for folder in [
        &model.paths.nodes,
        &model.paths.profiles,
        &model.paths.tunnels,
        &model.paths.certs,
        &model.paths.recipes,
        &model.paths.playbooks,
        &model.paths.scripts,
    ] {
        safe_path(dir, folder)?;
        for entry in dir.read_dir(folder)? {
            let entry = entry?;
            let filename = entry.file_name();
            let candidate = Path::new(folder).join(&filename);
            let source =
                super::automation::source_kind(model, candidate.to_str().ok_or_else(invalid)?)
                    .is_some();
            let entity = ![&model.paths.playbooks, &model.paths.scripts].contains(&folder)
                && Path::new(&filename).extension().and_then(|s| s.to_str()) == Some("toml");
            if !source && !entity {
                continue;
            }
            let path = Path::new(folder)
                .join(filename)
                .to_str()
                .ok_or_else(invalid)?
                .to_owned();
            let text = read_text(dir, &path)?;
            total += text.len();
            if files.len() >= 1024 || total > 16 * 1024 * 1024 {
                return Err(invalid());
            }
            files.insert(path, text);
        }
    }
    Ok(files)
}
fn fingerprint(files: &BTreeMap<String, String>) -> String {
    let mut digest = Sha256::new();
    for (path, text) in files {
        digest.update((path.len() as u64).to_be_bytes());
        digest.update(path);
        digest.update((text.len() as u64).to_be_bytes());
        digest.update(text);
    }
    format!("{:x}", digest.finalize())
}
pub fn current_revision(dir: &Dir) -> Result<String> {
    let text = read_text(dir, "ghosh.toml")?;
    let doc = Document::parse(&text)?;
    if doc.schema()? > 2 {
        return Ok(hash(text.as_bytes()));
    }
    let model = doc.manifest()?;
    Ok(fingerprint(&config_files(dir, &model)?))
}

/// Prepare and validate the entire graph without publishing any file.
/// The typed workspace service, never IPC, chooses this relative path.
pub(super) fn preview_file(dir: &Dir, path: &str, text: Option<&str>) -> Result<Loaded> {
    let model = Document::parse(&read_text(dir, "ghosh.toml")?)?.manifest()?;
    let mut files = config_files(dir, &model)?;
    match text {
        Some(text) => {
            files.insert(path.into(), text.into());
        }
        None => {
            if files.remove(path).is_none() {
                return Err(invalid());
            }
        }
    }
    validated_files(files)
}

/// One-file transaction: a complete validated candidate, then revision CAS and
/// one rename. Removals are retained beside the entity with a non-config suffix.
/// Multi-file migrations must use their own journal, not repeat this primitive.
pub(super) fn atomic_file(
    dir: &Dir,
    expected: &str,
    path: &str,
    text: Option<&str>,
    cancel: &Cancellation,
) -> Result<Loaded> {
    cancel.check()?;
    if current_revision(dir).ok().as_deref() != Some(expected) {
        return Err(AppError::new(ErrorCode::Conflict));
    }
    safe_path(dir, path)?;
    let candidate = preview_file(dir, path, text)?;
    let parent_path = Path::new(path).parent().ok_or_else(invalid)?;
    let parent = dir.open_dir(if parent_path.as_os_str().is_empty() {
        Path::new(".")
    } else {
        parent_path
    })?;
    let filename = Path::new(path).file_name().ok_or_else(invalid)?;
    let temp = format!(".ghosh-{}.tmp", uuid::Uuid::new_v4());
    let result = (|| {
        if let Some(text) = text {
            let mut options = OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use cap_std::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            let mut file = parent.open_with(&temp, &options)?;
            file.write_all(text.as_bytes())?;
            file.sync_all()?;
        }
        cancel.check()?;
        safe_path(dir, path)?;
        if current_revision(dir).ok().as_deref() != Some(expected) {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        if text.is_some() {
            parent.rename(&temp, &parent, filename)?;
        } else {
            parent.rename(
                filename,
                &parent,
                format!(".ghosh-{}.removed", uuid::Uuid::new_v4()),
            )?;
        }
        // Publication is the commit point; late cancellation cannot undo it.
        parent.try_clone()?.into_std_file().sync_all()?;
        Ok(candidate)
    })();
    let _ = parent.remove_file(&temp);
    result
}

/// Backup names are chosen by the typed migration, never by an IPC caller.
pub(super) fn backup_manifest(dir: &Dir, expected: &str, name: &str) -> Result<()> {
    if current_revision(dir).ok().as_deref() != Some(expected) {
        return Err(AppError::new(ErrorCode::Conflict));
    }
    let text = read_text(dir, "ghosh.toml")?;
    let temp = format!(".ghosh-{}.tmp", uuid::Uuid::new_v4());
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use cap_std::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = dir.open_with(&temp, &options)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        // Hard link publishes without replacement. An existing exact backup is reusable.
        match dir.hard_link(&temp, dir, name) {
            Ok(()) => (),
            Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {
                if read_text(dir, name)? != text {
                    return Err(AppError::new(ErrorCode::Conflict));
                }
            }
            Err(error) => return Err(error.into()),
        }
        dir.try_clone()?.into_std_file().sync_all()?;
        Ok(())
    })();
    let _ = dir.remove_file(&temp);
    result
}

pub fn atomic_manifest(
    dir: &Dir,
    expected: &str,
    text: &str,
    cancel: &Cancellation,
) -> Result<String> {
    cancel.check()?;
    if current_revision(dir).ok().as_deref() != Some(expected) {
        return Err(AppError::new(ErrorCode::Conflict));
    }
    let model = Document::parse(text)?.manifest()?;
    let mut committed_files = config_files(dir, &model)?;
    committed_files.insert("ghosh.toml".into(), text.into());
    let committed_revision = fingerprint(&committed_files);
    let temp = format!(".ghosh-{}.tmp", uuid::Uuid::new_v4());
    let result = (|| {
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use cap_std::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = dir.open_with(&temp, &options)?;
        file.write_all(text.as_bytes())?;
        file.sync_all()?;
        cancel.check()?;
        safe_path(dir, "ghosh.toml")?;
        if current_revision(dir).ok().as_deref() != Some(expected) {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        dir.rename(&temp, dir, "ghosh.toml")?;
        // Rename is the commit point: cancellation after it cannot undo success.
        dir.try_clone()?.into_std_file().sync_all()?;
        Ok(committed_revision)
    })();
    let _ = dir.remove_file(&temp);
    result
}

/// Create in a sibling staging directory, fsync, then publish one directory.
/// Interrupted staging packages have a random hidden name and are never opened.
pub fn create(parent: &Path, name: &str, cancel: &Cancellation) -> Result<PathBuf> {
    super::model::validate_name(name)?;
    if name == "." || name == ".." || name.contains(['/', '\\']) || name.starts_with('.') {
        return Err(AppError::new(ErrorCode::InvalidPath));
    }
    cancel.check()?;
    let parent = parent.canonicalize()?;
    let dir = Dir::open_ambient_dir(&parent, ambient_authority())?;
    let filename = format!("{}.ghosh", name.trim());
    if dir.try_exists(&filename)? {
        return Err(AppError::new(ErrorCode::Conflict));
    }
    let stage = format!(".ghosh-create-{}", uuid::Uuid::new_v4());
    dir.create_dir(&stage)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(parent.join(&stage), fs::Permissions::from_mode(0o700))?;
    }
    let result = (|| {
        let staged = dir.open_dir(&stage)?;
        let id = ulid::Ulid::new().to_string();
        let created = time::OffsetDateTime::now_utc()
            .format(&time::format_description::well_known::Rfc3339)
            .map_err(|_| invalid())?;
        let model = Manifest {
            schema: 1,
            workspace: super::model::WorkspaceMetadata {
                name: name.trim().into(),
                id,
                created: created.parse().map_err(|_| invalid())?,
                default_profile: None,
                default_tunnel: Some("direct".into()),
                default_key: None,
            },
            paths: Default::default(),
            mcp: Default::default(),
        };
        let text = toml_edit::ser::to_string_pretty(&model).map_err(|_| invalid())?;
        for folder in model.paths.all() {
            cancel.check()?;
            staged.create_dir_all(folder)?;
        }
        staged.create_dir_all("trust")?;
        for (path, bytes) in [
            ("ghosh.toml", text.as_bytes()),
            (
                ".gitignore",
                b"/state/\n/data/\n*.tmp\n.DS_Store\n".as_slice(),
            ),
            ("trust/known_hosts", b"".as_slice()),
        ] {
            cancel.check()?;
            let mut file = staged.create(path)?;
            file.write_all(bytes)?;
            file.sync_all()?;
        }
        staged.try_clone()?.into_std_file().sync_all()?;
        cancel.check()?;
        if dir.try_exists(&filename)? {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        #[cfg(any(target_os = "macos", target_os = "linux"))]
        rustix::fs::renameat_with(
            &dir,
            &stage,
            &dir,
            &filename,
            rustix::fs::RenameFlags::NOREPLACE,
        )
        .map_err(|e| AppError::from(std::io::Error::from(e)))?;
        #[cfg(not(any(target_os = "macos", target_os = "linux")))]
        dir.rename(&stage, &dir, &filename)?;
        dir.try_clone()?.into_std_file().sync_all()?;
        Ok(parent.join(filename))
    })();
    if result.is_err() {
        let _ = dir.remove_dir_all(&stage);
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn case_only_node_files_are_rejected_even_on_case_sensitive_filesystems() {
        let mut files = BTreeMap::from([(
            "ghosh.toml".into(),
            "schema=1\n[workspace]\nid='01ARZ3NDEKTSV4RRFFQ69G5FAV'\nname='Names'\ncreated=2026-10-22T00:00:00Z\n[paths]\nnodes='inventory'\n".into(),
        )]);
        for id in ["RTX3070", "rtx3070"] {
            files.insert(
                format!("inventory/{id}.toml"),
                format!(
                    "schema=1\nid='{id}'\nhost='127.0.0.1'\nuser='ops'\n[auth]\nmethod='system'\n"
                ),
            );
        }
        let error = validated_files(files).err().unwrap();
        assert_eq!(error.code, ErrorCode::Conflict);
        assert!(error.message.contains("different capitalization"));
    }
}
