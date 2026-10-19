pub mod automation;
pub mod document;
pub mod identity;
pub mod inventory;
pub mod model;
pub mod storage;
pub mod watch;

use crate::{
    contracts::{AppError, Cancellation, ErrorCode, NodeSummary, Result, WorkspaceSnapshot},
    policy::{Principal, authorize_edit},
};
use cap_std::{ambient_authority, fs::Dir};
use std::path::{Path, PathBuf};
use storage::{Lease, Loaded};

pub struct Workspace {
    root: PathBuf,
    dir: Dir,
    loaded: Loaded,
    trusted: bool,
    _path_lease: Lease,
    _id_lease: Lease,
}
impl Workspace {
    pub fn open(path: &Path, lock_dir: &Path) -> Result<Self> {
        let root = storage::canonical_package(path)?;
        let path_lease = Lease::acquire(lock_dir, &format!("path:{}", root.display()))?;
        let dir = Dir::open_ambient_dir(&root, ambient_authority())?;
        let loaded = storage::load(&dir)?;
        let id_lease = Lease::acquire(lock_dir, &format!("id:{}", loaded.id))?;
        let trusted = loaded.schema <= 2;
        Ok(Self {
            root,
            dir,
            loaded,
            trusted,
            _path_lease: path_lease,
            _id_lease: id_lease,
        })
    }
    pub fn create(
        parent: &Path,
        name: &str,
        lock_dir: &Path,
        cancel: &Cancellation,
    ) -> Result<Self> {
        // Serialize cooperating creators before the final directory exists.
        let target = parent
            .canonicalize()?
            .join(format!("{}.ghosh", name.trim()));
        let lease = Lease::acquire(lock_dir, &format!("create:{}", target.display()))?;
        let root = storage::create(parent, name, cancel)?;
        let opened = Self::open(&root, lock_dir);
        drop(lease);
        opened
    }
    pub fn root(&self) -> &Path {
        &self.root
    }
    pub fn snapshot(&self) -> WorkspaceSnapshot {
        let external_changed = self.check_root().is_err()
            || storage::current_revision(&self.dir)
                .map(|r| r != self.loaded.revision)
                .unwrap_or(true);
        let mut notices = vec![
            "Opening a workspace never runs hooks, scripts, network requests, or Keychain access."
                .into(),
        ];
        if self.loaded.schema > 2 {
            notices.push("Newer schema: read-only. Upgrade Ghosh to edit this package.".into());
        }
        if external_changed {
            notices.push("Files changed on disk. Reload to validate them before editing.".into());
        }
        notices.push(
            "Use a local filesystem. Cloud/network folders have no distributed writer guarantee."
                .into(),
        );
        WorkspaceSnapshot {
            id: self.loaded.id.clone(),
            name: self.loaded.name.clone(),
            path: self.root.to_string_lossy().into(),
            revision: self.loaded.revision.clone(),
            schema: self.loaded.schema,
            read_only: self.loaded.schema > 2,
            trusted: self.trusted,
            external_changed,
            nodes: self
                .loaded
                .nodes
                .iter()
                .map(|n| NodeSummary {
                    id: n.id.clone(),
                    host: n.host.clone(),
                    port: n.port,
                    user: n.user.clone(),
                    labels: n.labels.clone(),
                })
                .collect(),
            notices,
        }
    }
    fn check(&self, id: &str, revision: &str) -> Result<()> {
        if id != self.loaded.id {
            return Err(AppError::new(ErrorCode::Denied));
        }
        if revision != self.loaded.revision {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        if storage::current_revision(&self.dir).ok().as_deref() != Some(revision) {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        self.check_root()
    }
    fn check_root(&self) -> Result<()> {
        // Detect moved/replaced packages; anchored IO never silently follows a replacement.
        if self.root.canonicalize().ok().as_deref() != Some(self.root.as_path()) {
            return Err(AppError::new(ErrorCode::InvalidPath));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            let live = std::fs::metadata(&self.root)?;
            let held = self.dir.dir_metadata()?;
            use cap_std::fs::MetadataExt as CapMetadataExt;
            if live.dev() != held.dev() || live.ino() != held.ino() {
                return Err(AppError::new(ErrorCode::Conflict));
            }
        }
        Ok(())
    }
    pub fn rename(
        &mut self,
        principal: Principal,
        id: &str,
        revision: &str,
        name: &str,
        cancel: &Cancellation,
    ) -> Result<WorkspaceSnapshot> {
        authorize_edit(principal)?;
        self.check(id, revision)?;
        if self.loaded.schema > 2 {
            return Err(AppError::new(ErrorCode::UnsupportedSchema));
        }
        let mut edited = document::Document::parse(&self.loaded.doc.text())?;
        edited.rename(name)?;
        let model = edited.manifest()?;
        let committed = storage::atomic_manifest(&self.dir, revision, &edited.text(), cancel)?;
        // Update only after commit. Reload may fail if an external writer races us.
        self.loaded.doc = edited;
        self.loaded.name = model.workspace.name.clone();
        self.loaded.manifest = Some(model);
        self.loaded.revision = committed;
        Ok(self.snapshot())
    }
    pub fn reload(
        &mut self,
        principal: Principal,
        id: &str,
        revision: &str,
    ) -> Result<WorkspaceSnapshot> {
        authorize_edit(principal)?;
        if id != self.loaded.id {
            return Err(AppError::new(ErrorCode::Denied));
        }
        if revision != self.loaded.revision {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        self.check_root()?;
        let fresh = storage::load(&self.dir)?;
        if fresh.id != self.loaded.id {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        for (id, store) in &self.loaded.stores {
            if fresh.stores.get(id).is_some_and(|other| other != store) {
                return Err(model::invalid());
            }
        }
        self.loaded = fresh;
        if self.loaded.schema > 2 {
            self.trusted = false;
        }
        Ok(self.snapshot())
    }
    pub fn set_trusted(
        &mut self,
        principal: Principal,
        id: &str,
        revision: &str,
        trusted: bool,
    ) -> Result<WorkspaceSnapshot> {
        authorize_edit(principal)?;
        self.check(id, revision)?;
        if self.loaded.schema > 2 {
            return Err(AppError::new(ErrorCode::UnsupportedSchema));
        }
        self.trusted = trusted;
        Ok(self.snapshot())
    }
}
