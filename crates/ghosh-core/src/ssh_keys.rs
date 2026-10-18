//! User-local SSH identity metadata. Private bytes never implement Serialize or Debug.
use crate::{
    contracts::*,
    policy::{Principal, authorize_edit},
};
use base64::{Engine, engine::general_purpose::STANDARD};
use cap_std::{
    ambient_authority,
    fs::{Dir, OpenOptions},
};
use fs2::FileExt;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    fs::{self, File},
    io::{Read, Write},
    path::{Path, PathBuf},
};
use ts_rs::TS;
use zeroize::Zeroizing;

const MAX_KEY: u64 = 65536;
const MAX_REGISTRY: u64 = 262144;
fn unavailable() -> AppError {
    AppError::new(ErrorCode::KeyUnavailable)
}
fn digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum KeyStorage {
    Reference,
    Managed,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum KeyStatus {
    Available,
    Missing,
    Changed,
    Unsafe,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct PublicKeyMetadata {
    pub algorithm: String,
    pub fingerprint: String,
}
/// An adapter may inspect only the explicit, bounded, owner-only staging file.
pub trait KeyGenerator {
    fn generate(&self, path: &Path) -> Result<()>;
}
pub trait KeyInspector {
    fn inspect(&self, path: &Path) -> Result<PublicKeyMetadata>;
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct KeyImportRequest {
    pub path: String,
    pub label: String,
    pub storage: KeyStorage,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct KeyImportPreview {
    pub request: KeyImportRequest,
    pub digest: String,
    pub public: PublicKeyMetadata,
    pub encrypted: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct KeyCommitRequest {
    pub preview: KeyImportPreview,
    pub revision: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct KeySummary {
    pub id: String,
    pub label: String,
    pub path: String,
    pub storage: KeyStorage,
    pub public: PublicKeyMetadata,
    pub encrypted: bool,
    pub status: KeyStatus,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct KeysSnapshot {
    pub revision: String,
    pub default_key: Option<String>,
    pub keys: Vec<KeySummary>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct KeyRegistryRequest {
    pub revision: String,
    pub id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct KeyGenerateRequest {
    pub revision: String,
    pub label: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct KeyRelinkRequest {
    pub revision: String,
    pub id: String,
    pub preview: KeyImportPreview,
}

#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Stamp {
    dev: u64,
    ino: u64,
    len: u64,
    modified: i64,
    nanos: i64,
}
impl Stamp {
    fn of(meta: &fs::Metadata) -> Self {
        use std::os::unix::fs::MetadataExt;
        Self {
            dev: meta.dev(),
            ino: meta.ino(),
            len: meta.len(),
            modified: meta.mtime(),
            nanos: meta.mtime_nsec(),
        }
    }
    fn matches(&self, meta: &fs::Metadata) -> bool {
        let other = Self::of(meta);
        self.dev == other.dev
            && self.ino == other.ino
            && self.len == other.len
            && self.modified == other.modified
            && self.nanos == other.nanos
    }
}
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    summary: KeySummary,
    stamp: Stamp,
}
#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct Registry {
    default_key: Option<String>,
    entries: Vec<Entry>,
}

pub struct FileKeyRegistry {
    root: PathBuf,
    dir: Dir,
    root_stamp: Stamp,
    _lease: File,
    registry: Registry,
    revision: String,
}
struct GenerationStage {
    dir: Dir,
    name: String,
}
impl Drop for GenerationStage {
    fn drop(&mut self) {
        let _ = self.dir.remove_dir_all(&self.name);
    }
}
struct Staged<'a> {
    dir: &'a Dir,
    name: String,
}
impl Drop for Staged<'_> {
    fn drop(&mut self) {
        let _ = self.dir.remove_file(&self.name);
    }
}

fn regular_private(meta: &fs::Metadata) -> bool {
    use std::os::unix::fs::MetadataExt;
    // This is not an ACL against another process running as the same user.
    meta.is_file()
        && meta.uid() == unsafe { libc::geteuid() }
        && matches!(meta.mode() & 0o7777, 0o400 | 0o600)
        && meta.nlink() == 1
}
fn read_private(path: &Path) -> Result<(Zeroizing<Vec<u8>>, Stamp)> {
    use std::os::unix::fs::OpenOptionsExt;
    if !path.is_absolute() || path.canonicalize().map_err(|_| unavailable())? != path {
        return Err(unavailable());
    }
    let mut file = fs::OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)
        .map_err(|_| unavailable())?;
    let meta = file.metadata().map_err(|_| unavailable())?;
    if !regular_private(&meta) || meta.len() == 0 || meta.len() > MAX_KEY {
        return Err(unavailable());
    }
    let stamp = Stamp::of(&meta);
    let mut bytes = Zeroizing::new(Vec::new());
    (&mut file)
        .take(MAX_KEY + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| unavailable())?;
    if bytes.len() as u64 > MAX_KEY
        || !stamp.matches(&file.metadata()?)
        || path.canonicalize().map_err(|_| unavailable())? != path
        || !stamp.matches(&fs::symlink_metadata(path).map_err(|_| unavailable())?)
    {
        return Err(unavailable());
    }
    Ok((bytes, stamp))
}
fn valid_label(label: &str) -> bool {
    !label.trim().is_empty() && label.len() <= 80 && !label.chars().any(char::is_control)
}
fn valid_public(public: &PublicKeyMetadata) -> bool {
    ["ED25519", "RSA", "ECDSA", "DSA", "ED25519-SK", "ECDSA-SK"]
        .contains(&public.algorithm.as_str())
        && public
            .fingerprint
            .strip_prefix("SHA256:")
            .is_some_and(|value| {
                base64::engine::general_purpose::STANDARD_NO_PAD
                    .decode(value)
                    .is_ok_and(|bytes| bytes.len() == 32)
            })
}
fn encrypted(bytes: &[u8]) -> Result<bool> {
    let text = std::str::from_utf8(bytes).map_err(|_| unavailable())?;
    if text.starts_with("-----BEGIN OPENSSH PRIVATE KEY-----") {
        let encoded = Zeroizing::new(
            text.lines()
                .skip(1)
                .take_while(|line| !line.starts_with("-----END "))
                .collect::<String>(),
        );
        let decoded = Zeroizing::new(
            STANDARD
                .decode(encoded.as_bytes())
                .map_err(|_| unavailable())?,
        );
        let body = decoded
            .strip_prefix(b"openssh-key-v1\0")
            .ok_or_else(unavailable)?;
        let len = u32::from_be_bytes(
            body.get(..4)
                .ok_or_else(unavailable)?
                .try_into()
                .map_err(|_| unavailable())?,
        ) as usize;
        let cipher = body
            .get(4..4usize.checked_add(len).ok_or_else(unavailable)?)
            .ok_or_else(unavailable)?;
        return Ok(cipher != b"none");
    }
    if text.starts_with("-----BEGIN ENCRYPTED PRIVATE KEY-----") {
        return Ok(true);
    }
    if [
        "-----BEGIN RSA PRIVATE KEY-----",
        "-----BEGIN EC PRIVATE KEY-----",
        "-----BEGIN PRIVATE KEY-----",
    ]
    .iter()
    .any(|header| text.starts_with(header))
    {
        return Ok(text.lines().any(|line| line == "Proc-Type: 4,ENCRYPTED"));
    }
    Err(unavailable())
}
impl FileKeyRegistry {
    /// Native-only resolution after an explicit trusted operation. The caller
    /// must bind this registry revision and identity to the reviewed operation.
    pub fn resolve(
        &self,
        access: &crate::policy::SecretAccess,
        id: &str,
        revision: &str,
    ) -> Result<Zeroizing<Vec<u8>>> {
        let _ = access.workspace_id();
        self.live()?;
        self.check(revision)?;
        let entry = self
            .registry
            .entries
            .iter()
            .find(|entry| entry.summary.id == id)
            .ok_or_else(unavailable)?;
        if entry.summary.encrypted {
            return Err(AppError::new(ErrorCode::UnsupportedConnection));
        }
        let (bytes, stamp) = read_private(Path::new(&entry.summary.path))?;
        if !entry
            .stamp
            .matches(&fs::symlink_metadata(&entry.summary.path).map_err(|_| unavailable())?)
            || stamp.dev != entry.stamp.dev
            || stamp.ino != entry.stamp.ino
        {
            return Err(unavailable());
        }
        self.check(revision)?;
        Ok(bytes)
    }
    pub fn open(root: &Path) -> Result<Self> {
        use std::os::unix::fs::{DirBuilderExt, MetadataExt, OpenOptionsExt};
        if !root.exists() {
            fs::DirBuilder::new().mode(0o700).create(root)?;
        }
        let canonical = root.canonicalize()?;
        let meta = fs::symlink_metadata(root)?;
        if canonical != root
            || !meta.is_dir()
            || meta.uid() != unsafe { libc::geteuid() }
            || meta.mode() & 0o077 != 0
        {
            return Err(unavailable());
        }
        let dir = Dir::open_ambient_dir(root, ambient_authority())?;
        let lease = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .mode(0o600)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(root.join("registry.lock"))?;
        if !regular_private(&lease.metadata()?) {
            return Err(unavailable());
        }
        lease
            .try_lock_exclusive()
            .map_err(|_| AppError::new(ErrorCode::Busy))?;
        let mut this = Self {
            root: root.into(),
            dir,
            root_stamp: Stamp::of(&meta),
            _lease: lease,
            registry: Registry::default(),
            revision: digest(b""),
        };
        let bytes = this.registry_bytes()?;
        if !bytes.is_empty() {
            this.registry = serde_json::from_slice(&bytes).map_err(|_| unavailable())?;
            if this.registry.entries.len() > 256 {
                return Err(unavailable());
            }
            let mut ids = std::collections::BTreeSet::new();
            for entry in &this.registry.entries {
                if !crate::workspace::model::key_id(&entry.summary.id)
                    || !valid_label(&entry.summary.label)
                    || !valid_public(&entry.summary.public)
                    || !ids.insert(&entry.summary.id)
                    || !Path::new(&entry.summary.path).is_absolute()
                    || (entry.summary.storage == KeyStorage::Managed
                        && entry.summary.path
                            != this
                                .root
                                .join(format!("{}.key", entry.summary.id))
                                .to_string_lossy())
                {
                    return Err(unavailable());
                }
            }
            if this
                .registry
                .default_key
                .as_ref()
                .is_some_and(|id| !ids.contains(id))
            {
                return Err(unavailable());
            }
        }
        this.revision = digest(&bytes);
        Ok(this)
    }
    fn live(&self) -> Result<()> {
        use std::os::unix::fs::MetadataExt;
        let meta = fs::symlink_metadata(&self.root)?;
        if meta.file_type().is_symlink()
            || meta.dev() != self.root_stamp.dev
            || meta.ino() != self.root_stamp.ino
            || meta.mode() & 0o077 != 0
        {
            return Err(unavailable());
        }
        Ok(())
    }
    fn registry_bytes(&self) -> Result<Vec<u8>> {
        self.live()?;
        match self.dir.symlink_metadata("registry.json") {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Vec::new()),
            Err(e) => Err(e.into()),
            Ok(meta) => {
                if meta.file_type().is_symlink() || !meta.is_file() || meta.len() > MAX_REGISTRY {
                    return Err(unavailable());
                }
                let file = self.dir.open("registry.json")?.into_std();
                if !regular_private(&file.metadata()?) {
                    return Err(unavailable());
                }
                let mut bytes = Vec::new();
                file.take(MAX_REGISTRY + 1).read_to_end(&mut bytes)?;
                if bytes.len() as u64 > MAX_REGISTRY {
                    return Err(unavailable());
                }
                Ok(bytes)
            }
        }
    }
    fn check(&self, revision: &str) -> Result<()> {
        if revision != self.revision || digest(&self.registry_bytes()?) != self.revision {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        Ok(())
    }
    fn stage(&self, bytes: &[u8]) -> Result<Staged<'_>> {
        use cap_std::fs::OpenOptionsExt;
        self.live()?;
        let name = format!(".stage-{}", uuid::Uuid::new_v4());
        let mut options = OpenOptions::new();
        options.write(true).create_new(true).mode(0o600);
        let mut file = self.dir.open_with(&name, &options)?;
        let staged = Staged {
            dir: &self.dir,
            name,
        };
        file.write_all(bytes)?;
        file.sync_all()?;
        Ok(staged)
    }
    fn persist(&mut self, registry: Registry, revision: &str) -> Result<()> {
        self.check(revision)?;
        let bytes = serde_json::to_vec(&registry).map_err(|_| unavailable())?;
        if bytes.len() as u64 > MAX_REGISTRY {
            return Err(AppError::new(ErrorCode::Capacity));
        }
        let staged = self.stage(&bytes)?;
        self.check(revision)?;
        self.dir.rename(&staged.name, &self.dir, "registry.json")?;
        drop(staged);
        // The rename is the commit point. A sync error must not leave stale in-memory state.
        self.registry = registry;
        self.revision = digest(&bytes);
        self.dir.open(".")?.sync_all()?;
        Ok(())
    }
    pub fn snapshot(&self, principal: Principal) -> Result<KeysSnapshot> {
        authorize_edit(principal)?;
        self.check(&self.revision)?;
        let keys = self
            .registry
            .entries
            .iter()
            .map(|entry| {
                let mut key = entry.summary.clone();
                key.status = match fs::symlink_metadata(&key.path) {
                    Err(e) if e.kind() == std::io::ErrorKind::NotFound => KeyStatus::Missing,
                    Err(_) => KeyStatus::Unsafe,
                    Ok(meta)
                        if !regular_private(&meta)
                            || Path::new(&key.path).canonicalize().ok().as_deref()
                                != Some(Path::new(&key.path)) =>
                    {
                        KeyStatus::Unsafe
                    }
                    Ok(meta) if !entry.stamp.matches(&meta) => KeyStatus::Changed,
                    Ok(_) => KeyStatus::Available,
                };
                key
            })
            .collect();
        Ok(KeysSnapshot {
            revision: self.revision.clone(),
            default_key: self.registry.default_key.clone(),
            keys,
        })
    }
    fn inspect(
        &self,
        request: &KeyImportRequest,
        inspector: &dyn KeyInspector,
    ) -> Result<(KeyImportPreview, Zeroizing<Vec<u8>>, Stamp)> {
        if !valid_label(&request.label) || request.path.len() > 4096 {
            return Err(unavailable());
        }
        let (bytes, stamp) = read_private(Path::new(&request.path))?;
        let encrypted = encrypted(&bytes)?;
        let staged = self.stage(&bytes)?;
        let public = inspector.inspect(&self.root.join(&staged.name))?;
        if !valid_public(&public) {
            return Err(unavailable());
        }
        let framed = serde_json::to_vec(&(request, &public, encrypted, digest(&bytes)))
            .map_err(|_| unavailable())?;
        Ok((
            KeyImportPreview {
                request: request.clone(),
                digest: digest(&framed),
                public,
                encrypted,
            },
            bytes,
            stamp,
        ))
    }
    pub fn preview(
        &self,
        principal: Principal,
        request: KeyImportRequest,
        inspector: &dyn KeyInspector,
    ) -> Result<KeyImportPreview> {
        authorize_edit(principal)?;
        self.check(&self.revision)?;
        Ok(self.inspect(&request, inspector)?.0)
    }
    pub fn commit(
        &mut self,
        principal: Principal,
        request: KeyCommitRequest,
        inspector: &dyn KeyInspector,
    ) -> Result<KeysSnapshot> {
        authorize_edit(principal)?;
        self.check(&request.revision)?;
        if self.registry.entries.len() >= 256 {
            return Err(AppError::new(ErrorCode::Capacity));
        }
        let (preview, bytes, mut stamp) = self.inspect(&request.preview.request, inspector)?;
        if preview.digest != request.preview.digest
            || preview.public != request.preview.public
            || preview.encrypted != request.preview.encrypted
        {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        if self
            .registry
            .entries
            .iter()
            .any(|entry| entry.summary.public.fingerprint == preview.public.fingerprint)
        {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        let id = uuid::Uuid::new_v4().to_string();
        let mut path = preview.request.path.clone();
        // Stage and publish the key first. A registry failure leaves an unreferenced
        // owner-only key, never a reference to a missing/uncommitted identity.
        if preview.request.storage == KeyStorage::Managed {
            let staged = self.stage(&bytes)?;
            self.check(&request.revision)?;
            let name = format!("{id}.key");
            // hard_link provides no-replace publication on the same volume.
            self.dir.hard_link(&staged.name, &self.dir, &name)?;
            drop(staged);
            path = self.root.join(&name).to_string_lossy().into_owned();
            stamp = Stamp::of(&self.dir.open(&name)?.into_std().metadata()?);
            self.dir.open(".")?.sync_all()?;
        }
        let mut registry = self.registry.clone();
        registry.entries.push(Entry {
            summary: KeySummary {
                id,
                label: preview.request.label,
                path,
                storage: preview.request.storage,
                public: preview.public,
                encrypted: preview.encrypted,
                status: KeyStatus::Available,
            },
            stamp,
        });
        self.persist(registry, &request.revision)?;
        self.snapshot(principal)
    }
    /// Generate a fresh unencrypted Ed25519 identity. Explicit local UI action only.
    /// Private output remains in owner-only staging and never enters a DTO.
    pub fn generate(
        &mut self,
        principal: Principal,
        request: KeyGenerateRequest,
        generator: &dyn KeyGenerator,
        inspector: &dyn KeyInspector,
    ) -> Result<KeysSnapshot> {
        authorize_edit(principal)?;
        self.check(&request.revision)?;
        if !valid_label(&request.label) || self.registry.entries.len() >= 256 {
            return Err(unavailable());
        }
        self.live()?;
        let name = format!(".generate-{}", uuid::Uuid::new_v4());
        // cap-std create_dir uses the process umask; chmod immediately before key creation.
        self.dir.create_dir(&name)?;
        let stage = GenerationStage {
            dir: self.dir.try_clone()?,
            name,
        };
        use std::os::unix::fs::PermissionsExt;
        self.dir.set_permissions(
            &stage.name,
            cap_std::fs::Permissions::from_std(fs::Permissions::from_mode(0o700)),
        )?;
        let path = self.root.join(&stage.name).join("identity");
        generator.generate(&path)?;
        let preview = self.preview(
            principal,
            KeyImportRequest {
                path: path.to_string_lossy().into_owned(),
                label: request.label,
                storage: KeyStorage::Managed,
            },
            inspector,
        )?;
        if preview.encrypted || preview.public.algorithm != "ED25519" {
            return Err(unavailable());
        }
        self.commit(
            principal,
            KeyCommitRequest {
                preview,
                revision: request.revision,
            },
            inspector,
        )
    }
    /// Restore the path of an existing reference without changing its identity.
    pub fn relink(
        &mut self,
        principal: Principal,
        request: KeyRelinkRequest,
        inspector: &dyn KeyInspector,
    ) -> Result<KeysSnapshot> {
        authorize_edit(principal)?;
        self.check(&request.revision)?;
        let index = self
            .registry
            .entries
            .iter()
            .position(|entry| entry.summary.id == request.id)
            .ok_or_else(|| AppError::new(ErrorCode::MissingReference))?;
        let existing = &self.registry.entries[index];
        if existing.summary.storage != KeyStorage::Reference
            || request.preview.request.storage != KeyStorage::Reference
        {
            return Err(unavailable());
        }
        let (preview, _, stamp) = self.inspect(&request.preview.request, inspector)?;
        if preview.digest != request.preview.digest
            || preview.public != request.preview.public
            || preview.encrypted != request.preview.encrypted
            || preview.public != existing.summary.public
        {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        let mut registry = self.registry.clone();
        let entry = &mut registry.entries[index];
        entry.summary.path = preview.request.path;
        entry.summary.label = preview.request.label;
        entry.summary.encrypted = preview.encrypted;
        entry.stamp = stamp;
        self.persist(registry, &request.revision)?;
        self.snapshot(principal)
    }
    pub fn set_default(
        &mut self,
        principal: Principal,
        request: KeyRegistryRequest,
    ) -> Result<KeysSnapshot> {
        authorize_edit(principal)?;
        self.check(&request.revision)?;
        if let Some(id) = request.id.as_ref() {
            let snapshot = self.snapshot(principal)?;
            let key = snapshot
                .keys
                .iter()
                .find(|entry| &entry.id == id)
                .ok_or_else(|| AppError::new(ErrorCode::MissingReference))?;
            if key.status != KeyStatus::Available {
                return Err(unavailable());
            }
        }
        let mut registry = self.registry.clone();
        registry.default_key = request.id;
        self.persist(registry, &request.revision)?;
        self.snapshot(principal)
    }
    /// Removes metadata only. Managed files are retained for explicit recovery.
    pub fn forget(
        &mut self,
        principal: Principal,
        request: KeyRegistryRequest,
    ) -> Result<KeysSnapshot> {
        authorize_edit(principal)?;
        self.check(&request.revision)?;
        let id = request.id.ok_or_else(unavailable)?;
        let mut registry = self.registry.clone();
        let before = registry.entries.len();
        registry.entries.retain(|entry| entry.summary.id != id);
        if registry.entries.len() == before {
            return Err(AppError::new(ErrorCode::MissingReference));
        }
        if registry.default_key.as_ref() == Some(&id) {
            registry.default_key = None;
        }
        self.persist(registry, &request.revision)?;
        self.snapshot(principal)
    }
}
