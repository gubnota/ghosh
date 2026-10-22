use ghosh_core::{
    approvals::{ApprovalContext, ApprovalDecisionRequest, Approvals},
    contracts::*,
    operations::{Jobs, OperationsSnapshot},
    policy::Principal,
    secrets::SecretStore,
    ssh_keys::*,
    workspace::{Workspace, identity::*, inventory::*, model::Node, watch::WorkspaceWatch},
};
use std::{
    collections::VecDeque,
    fs,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};
use tauri::{AppHandle, Emitter};
mod ansible;
pub(crate) mod automation;
mod history;
mod local_exec;
mod monitor;
mod source_lint;
pub(crate) use source_lint::lint_source;
mod runs;
mod ssh;
use history::WorkspaceHistory;

struct Active {
    workspace: Workspace,
    context: ApprovalContext,
    _watch: WorkspaceWatch,
}
// macOS can deliver document events before Tauri runs setup.
#[derive(Default)]
pub struct DocumentOpenQueue {
    pending: Mutex<VecDeque<String>>,
}
impl DocumentOpenQueue {
    fn enqueue(&self, path: String) -> Result<()> {
        let mut pending = self.pending.lock().map_err(poisoned)?;
        if pending.len() >= 10 {
            return Err(AppError::new(ErrorCode::Busy));
        }
        pending.push_back(path);
        Ok(())
    }
    pub fn request(&self, path: String, app: &AppHandle) -> Result<()> {
        self.enqueue(path)?;
        // A missing window/listener is harmless: the UI drains the queue on startup.
        app.emit_to("main", "workspace-open-requested", ())
            .map_err(|_| AppError::new(ErrorCode::Internal))
    }
    fn next(&self) -> Result<Option<String>> {
        Ok(self.pending.lock().map_err(poisoned)?.pop_front())
    }
}
pub struct Services {
    ssh_reviews: Mutex<Vec<ssh::Review>>,
    runs: Mutex<runs::RunRegistry>,
    monitor: Mutex<Option<monitor::Monitor>>,
    pub terminals: ghosh_term::Terminals,
    pub jobs: Jobs,
    approvals: Mutex<Approvals>,
    keys: Mutex<Result<FileKeyRegistry>>,
    active: Mutex<Option<Active>>,
    history: Mutex<WorkspaceHistory>,
    pending: Arc<DocumentOpenQueue>,
    data_dir: PathBuf,
    pub _secrets: Box<dyn SecretStore>,
}
fn poisoned<T>(_: std::sync::PoisonError<T>) -> AppError {
    AppError::new(ErrorCode::Internal)
}
impl Services {
    pub fn new(data_dir: PathBuf, pending: Arc<DocumentOpenQueue>) -> Result<Self> {
        fs::create_dir_all(&data_dir)?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            fs::set_permissions(&data_dir, fs::Permissions::from_mode(0o700))?;
        }
        let history = WorkspaceHistory::load(&data_dir);
        // Damaged key metadata must not block unrelated workspace recovery.
        let keys = FileKeyRegistry::open(&data_dir.canonicalize()?.join("ssh"));
        Ok(Self {
            ssh_reviews: Mutex::new(Vec::new()),
            runs: Mutex::new(runs::RunRegistry::default()),
            monitor: Mutex::new(None),
            terminals: ghosh_term::Terminals::default(),
            jobs: Jobs::default(),
            approvals: Mutex::new(Approvals::default()),
            keys: Mutex::new(keys),
            active: Mutex::new(None),
            history: Mutex::new(history),
            pending,
            data_dir,
            _secrets: crate::platform::secret_store(),
        })
    }
    // Caller holds active first. Rotate even when the attempted mutation fails:
    // an in-flight review cannot become valid again after trust/reload transitions.
    fn invalidate_reviews(&self, active: &mut Option<Active>) -> Result<()> {
        if self.run_active()? {
            return Err(AppError::new(ErrorCode::Busy));
        }
        if let Some(active) = active {
            active.context.session_id = uuid::Uuid::new_v4().to_string();
        }
        self.approvals.lock().map_err(poisoned)?.synchronize(None);
        self.ssh_reviews.lock().map_err(poisoned)?.clear();
        *self.monitor.lock().map_err(poisoned)? = None;
        Ok(())
    }
    pub fn workspace_key(&self, request: RevisionRequest) -> Result<Option<String>> {
        let active = self.active.lock().map_err(poisoned)?;
        active
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .workspace
            .workspace_key(Principal::LocalUi, &request)
    }
    pub fn preview_workspace_key(
        &self,
        request: WorkspaceKeyRequest,
    ) -> Result<WorkspaceKeyPreview> {
        let active = self.active.lock().map_err(poisoned)?;
        self.validate_key_choice(request.default_key.as_deref())?;
        active
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .workspace
            .preview_workspace_key(Principal::LocalUi, &request)
    }
    fn validate_key_choice(&self, id: Option<&str>) -> Result<()> {
        if let Some(id) = id {
            let keys = self.keys()?;
            if !keys
                .keys
                .iter()
                .any(|key| key.id == id && key.status == KeyStatus::Available)
            {
                return Err(AppError::new(ErrorCode::KeyUnavailable));
            }
        }
        Ok(())
    }
    pub fn commit_workspace_key(&self, request: WorkspaceKeyCommit) -> Result<WorkspaceSnapshot> {
        let mut active = self.active.lock().map_err(poisoned)?;
        self.validate_key_choice(request.request.default_key.as_deref())?;
        self.invalidate_reviews(&mut active)?;
        active
            .as_mut()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .workspace
            .commit_workspace_key(Principal::LocalUi, &request, &Cancellation::default())
    }
    pub fn identity_preview(&self, request: NodeIdentityRequest) -> Result<IdentityPreview> {
        let active = self.active.lock().map_err(poisoned)?;
        request.node.validate(&request.node.id)?;
        let default = active
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .workspace
            .workspace_key(Principal::LocalUi, &request.context)?;
        let mut identity = select_identity(&request.node, default.as_deref(), &self.keys()?);
        if matches!(
            request.node.effective_auth_method(),
            ghosh_core::workspace::model::AuthMethod::System
        ) {
            match self.system_identity(&request.node) {
                Ok(key) => {
                    identity.public = Some(key.public);
                    identity.status = Some(KeyStatus::Available);
                }
                Err(_) => identity.status = Some(KeyStatus::Missing),
            }
        }
        Ok(identity)
    }
    pub fn generate_key(&self, request: KeyGenerateRequest) -> Result<KeysSnapshot> {
        self.with_keys(|keys| {
            keys.generate(
                Principal::LocalUi,
                request,
                &crate::platform::ssh_keys::OpenSshGenerator,
                &crate::platform::ssh_keys::OpenSshInspector,
            )
        })
    }
    pub fn relink_key(&self, request: KeyRelinkRequest) -> Result<KeysSnapshot> {
        self.with_keys(|keys| {
            keys.relink(
                Principal::LocalUi,
                request,
                &crate::platform::ssh_keys::OpenSshInspector,
            )
        })
    }
    /// Inspect only fixed conventional candidates after an explicit local UI request.
    pub fn system_key_candidates(&self) -> Result<Vec<KeyImportPreview>> {
        let home =
            std::env::var_os("HOME").ok_or_else(|| AppError::new(ErrorCode::KeyUnavailable))?;
        let home = PathBuf::from(home);
        let mut candidates = vec![];
        for (relative, label) in [
            (".ssh/id_ed25519", "System Ed25519"),
            (".ssh/id_rsa", "System RSA"),
            (".orbstack/ssh/id_ed25519", "OrbStack Linux"),
        ] {
            let request = KeyImportRequest {
                label: label.into(),
                path: home.join(relative).to_string_lossy().into_owned(),
                storage: KeyStorage::Reference,
            };
            if let Ok(preview) = self.preview_key(request) {
                candidates.push(preview);
            }
        }
        Ok(candidates)
    }
    pub fn keys(&self) -> Result<KeysSnapshot> {
        self.with_keys(|keys| keys.snapshot(Principal::LocalUi))
    }
    fn with_keys<T>(&self, action: impl FnOnce(&mut FileKeyRegistry) -> Result<T>) -> Result<T> {
        let mut keys = self.keys.lock().map_err(poisoned)?;
        action(keys.as_mut().map_err(|error| error.clone())?)
    }
    pub fn preview_key(&self, request: KeyImportRequest) -> Result<KeyImportPreview> {
        self.with_keys(|keys| {
            keys.preview(
                Principal::LocalUi,
                request,
                &crate::platform::ssh_keys::OpenSshInspector,
            )
        })
    }
    pub fn commit_key(&self, request: KeyCommitRequest) -> Result<KeysSnapshot> {
        self.with_keys(|keys| {
            keys.commit(
                Principal::LocalUi,
                request,
                &crate::platform::ssh_keys::OpenSshInspector,
            )
        })
    }
    pub fn default_key(&self, request: KeyRegistryRequest) -> Result<KeysSnapshot> {
        self.with_keys(|keys| keys.set_default(Principal::LocalUi, request))
    }
    pub fn forget_key(&self, request: KeyRegistryRequest) -> Result<KeysSnapshot> {
        self.with_keys(|keys| keys.forget(Principal::LocalUi, request))
    }
    pub fn pending(&self) -> Result<Option<String>> {
        self.pending.next()
    }
    pub fn recent(&self) -> Result<Vec<RecentWorkspace>> {
        Ok(self.history.lock().map_err(poisoned)?.recent.clone())
    }
    pub fn forget_recent(&self, path: Option<String>) -> Result<Vec<RecentWorkspace>> {
        self.edit_history(|history| history.forget(path.as_deref()))?;
        self.recent()
    }
    fn edit_history(&self, edit: impl FnOnce(&mut WorkspaceHistory)) -> Result<()> {
        let mut stored = self.history.lock().map_err(poisoned)?;
        let mut next = stored.clone();
        edit(&mut next);
        next.save(&self.data_dir)?;
        *stored = next;
        Ok(())
    }
    fn remember(&self, snapshot: &WorkspaceSnapshot) -> Result<()> {
        self.edit_history(|history| history.remember(&snapshot.name, &snapshot.path))
    }
    pub fn close(&self, request: RevisionRequest) -> Result<()> {
        let mut active = self.active.lock().map_err(poisoned)?;
        if self.jobs.snapshot()?.jobs.iter().any(|job| {
            matches!(
                job.state,
                ghosh_core::operations::JobState::Running
                    | ghosh_core::operations::JobState::Cancelling
            )
        }) {
            return Err(AppError::new(ErrorCode::Busy));
        }
        let snapshot = active
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .workspace
            .snapshot();
        if snapshot.id != request.workspace_id || snapshot.revision != request.revision {
            return Err(AppError::new(ErrorCode::Conflict));
        }
        self.edit_history(|history| history.last = None)?;
        self.invalidate_reviews(&mut active)?;
        *self.monitor.lock().map_err(poisoned)? = None;
        self.terminals.close_all();
        *active = None;
        Ok(())
    }
    fn startup_path(&self) -> Result<Option<String>> {
        // Explicit native opens win over automatic restoration.
        Ok(self
            .pending
            .next()?
            .or(self.history.lock().map_err(poisoned)?.last.clone()))
    }
    pub fn startup(&self, app: &AppHandle) -> Result<Option<WorkspaceSnapshot>> {
        let mut active = self.active.lock().map_err(poisoned)?;
        if let Some(current) = active.as_ref() {
            return Ok(Some(current.workspace.snapshot()));
        }
        let Some(path) = self.startup_path()? else {
            return Ok(None);
        };
        let workspace = Workspace::open(Path::new(&path), &self.data_dir.join("locks"))?;
        self.activate(workspace, &mut active, app).map(Some)
    }
    fn activate(
        &self,
        workspace: Workspace,
        active: &mut Option<Active>,
        app: &AppHandle,
    ) -> Result<WorkspaceSnapshot> {
        let mut snapshot = workspace.snapshot();
        let payload = WorkspaceEvent {
            workspace_id: snapshot.id.clone(),
            revision: snapshot.revision.clone(),
        };
        let handle = app.clone();
        let watch = WorkspaceWatch::start(workspace.root(), move || {
            let _ = handle.emit_to("main", "workspace-files-changed", &payload);
        })?;
        if self.remember(&snapshot).is_err() {
            snapshot.notices.push(
                "Recent workspace history could not be saved. The workspace is still open.".into(),
            );
        }
        *self.monitor.lock().map_err(poisoned)? = None;
        self.terminals.close_all();
        *active = Some(Active {
            context: ApprovalContext::new(snapshot.id.clone(), snapshot.revision.clone()),
            workspace,
            _watch: watch,
        });
        Ok(snapshot)
    }
    pub fn open(&self, path: &Path, app: &AppHandle) -> Result<WorkspaceSnapshot> {
        let mut active = self.active.lock().map_err(poisoned)?;
        self.invalidate_reviews(&mut active)?;
        let canonical = path.canonicalize()?;
        if let Some(current) = active.as_mut()
            && current.workspace.root() == canonical
        {
            let mut snapshot = current.workspace.snapshot();
            if self.remember(&snapshot).is_err() {
                snapshot
                    .notices
                    .push("Recent workspace history could not be saved.".into());
            }
            return Ok(snapshot);
        }
        let workspace = Workspace::open(path, &self.data_dir.join("locks"))?;
        self.activate(workspace, &mut active, app)
    }
    pub fn create(&self, request: CreateRequest, app: &AppHandle) -> Result<WorkspaceSnapshot> {
        let mut active = self.active.lock().map_err(poisoned)?;
        self.invalidate_reviews(&mut active)?;
        let workspace = Workspace::create(
            Path::new(&request.parent),
            &request.name,
            &self.data_dir.join("locks"),
            &Cancellation::default(),
        )?;
        self.activate(workspace, &mut active, app)
    }
    pub fn snapshot(&self) -> Result<Option<WorkspaceSnapshot>> {
        Ok(self
            .active
            .lock()
            .map_err(poisoned)?
            .as_ref()
            .map(|a| a.workspace.snapshot()))
    }
    pub fn node_details(&self, request: RevisionRequest) -> Result<Vec<Node>> {
        self.active
            .lock()
            .map_err(poisoned)?
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .workspace
            .node_details(Principal::LocalUi, &request)
    }
    pub fn preview_node(&self, request: NodeMutationRequest) -> Result<NodeChangePreview> {
        self.active
            .lock()
            .map_err(poisoned)?
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .workspace
            .preview_node(Principal::LocalUi, &request)
    }
    pub fn commit_node(&self, request: NodeCommitRequest) -> Result<WorkspaceSnapshot> {
        let mut guard = self.active.lock().map_err(poisoned)?;
        self.invalidate_reviews(&mut guard)?;
        guard
            .as_mut()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .workspace
            .commit_node(Principal::LocalUi, &request, &Cancellation::default())
    }
    pub fn rename(&self, request: RenameRequest) -> Result<WorkspaceSnapshot> {
        let mut guard = self.active.lock().map_err(poisoned)?;
        self.invalidate_reviews(&mut guard)?;
        let active = guard
            .as_mut()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?;
        let mut snapshot = active.workspace.rename(
            Principal::LocalUi,
            &request.workspace_id,
            &request.revision,
            &request.name,
            &Cancellation::default(),
        )?;
        if self.remember(&snapshot).is_err() {
            snapshot
                .notices
                .push("Recent workspace history could not be updated.".into());
        }
        Ok(snapshot)
    }
    pub fn reload(&self, request: RevisionRequest) -> Result<WorkspaceSnapshot> {
        let mut guard = self.active.lock().map_err(poisoned)?;
        self.invalidate_reviews(&mut guard)?;
        guard
            .as_mut()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .workspace
            .reload(Principal::LocalUi, &request.workspace_id, &request.revision)
    }
    pub fn trust(&self, request: TrustRequest) -> Result<WorkspaceSnapshot> {
        let mut guard = self.active.lock().map_err(poisoned)?;
        self.invalidate_reviews(&mut guard)?;
        guard
            .as_mut()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .workspace
            .set_trusted(
                Principal::LocalUi,
                &request.workspace_id,
                &request.revision,
                request.trusted,
            )
    }
    // Lock order is always active -> approvals. Snapshot rechecks disk state;
    // events are only invalidation hints, never authorization evidence.
    fn synchronize_approvals(&self, active: &Option<Active>, approvals: &mut Approvals) {
        let context = active.as_ref().and_then(|active| {
            let snapshot = active.workspace.snapshot();
            (snapshot.trusted && !snapshot.read_only && !snapshot.external_changed).then(|| {
                let mut context = active.context.clone();
                context.revision = snapshot.revision;
                context
            })
        });
        approvals.synchronize(context);
    }
    pub fn operations(&self) -> Result<OperationsSnapshot> {
        let active = self.active.lock().map_err(poisoned)?;
        let mut approvals = self.approvals.lock().map_err(poisoned)?;
        self.synchronize_approvals(&active, &mut approvals);
        let snapshot = approvals.snapshot();
        drop(approvals);
        drop(active);
        Ok(OperationsSnapshot {
            jobs: self.jobs.snapshot()?,
            approvals: snapshot,
        })
    }
    pub fn decide(&self, request: ApprovalDecisionRequest) -> Result<()> {
        let active = self.active.lock().map_err(poisoned)?;
        let mut approvals = self.approvals.lock().map_err(poisoned)?;
        self.synchronize_approvals(&active, &mut approvals);
        approvals.decide(Principal::LocalUi, request)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use ghosh_core::approvals::{
        ApprovalBinding, ApprovalDecision, ApprovalState, Requester, content_digest,
    };

    #[test]
    fn explicit_open_precedes_restore_and_close_releases_workspace() {
        let (tmp, service, _) = approval_fixture();
        let snapshot = service.snapshot().unwrap().unwrap();
        service.remember(&snapshot).unwrap();
        service.pending.enqueue("/explicit.ghosh".into()).unwrap();
        assert_eq!(
            service.startup_path().unwrap().as_deref(),
            Some("/explicit.ghosh")
        );
        assert_eq!(
            service.startup_path().unwrap().as_deref(),
            Some(snapshot.path.as_str())
        );
        assert!(
            service
                .close(RevisionRequest {
                    workspace_id: snapshot.id.clone(),
                    revision: "stale".into()
                })
                .is_err()
        );
        assert!(service.snapshot().unwrap().is_some());
        let job = service
            .jobs
            .begin(ghosh_core::operations::JobKind::NodeConnect, false)
            .unwrap();
        assert!(
            service
                .close(RevisionRequest {
                    workspace_id: snapshot.id.clone(),
                    revision: snapshot.revision.clone()
                })
                .is_err()
        );
        drop(job);
        service
            .close(RevisionRequest {
                workspace_id: snapshot.id.clone(),
                revision: snapshot.revision.clone(),
            })
            .unwrap();
        assert!(service.snapshot().unwrap().is_none());
        assert!(service.startup_path().unwrap().is_none());
        assert_eq!(service.recent().unwrap().len(), 1);
        let reopened =
            Workspace::open(Path::new(&snapshot.path), &tmp.path().join("locks")).unwrap();
        drop(reopened);
        drop(service);
        let restored = Services::new(
            tmp.path().join("data"),
            Arc::new(DocumentOpenQueue::default()),
        )
        .unwrap();
        assert!(restored.startup_path().unwrap().is_none());
        restored.forget_recent(None).unwrap();
        assert!(Path::new(&snapshot.path).exists());
        assert!(restored.recent().unwrap().is_empty());
    }

    fn approval_fixture() -> (tempfile::TempDir, Services, String) {
        let tmp = tempfile::tempdir().unwrap();
        let service = Services::new(
            tmp.path().join("data"),
            Arc::new(DocumentOpenQueue::default()),
        )
        .unwrap();
        let mut workspace = Workspace::create(
            tmp.path(),
            "Operations",
            &tmp.path().join("locks"),
            &Cancellation::default(),
        )
        .unwrap();
        let snapshot = workspace.snapshot();
        workspace
            .set_trusted(Principal::LocalUi, &snapshot.id, &snapshot.revision, true)
            .unwrap();
        let context = ApprovalContext::new(snapshot.id, snapshot.revision);
        let watch = WorkspaceWatch::start(workspace.root(), || {}).unwrap();
        *service.active.lock().unwrap() = Some(Active {
            workspace,
            context: context.clone(),
            _watch: watch,
        });
        service.operations().unwrap();
        let id = service
            .approvals
            .lock()
            .unwrap()
            .request(
                Principal::NativeService,
                ApprovalBinding {
                    context,
                    requester: Requester::Mcp,
                    tool: "node.run".into(),
                    source_digest: content_digest(b"source"),
                    targets: vec!["edge1".into()],
                    route_digest: content_digest(b"route"),
                    identity_digest: content_digest(b"identity"),
                },
            )
            .unwrap();
        (tmp, service, id)
    }

    #[test]
    fn ssh_review_session_never_revives_after_trust_or_reload() {
        let (_tmp, service, _) = approval_fixture();
        let original = service
            .active
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .context
            .session_id
            .clone();
        let snapshot = service.snapshot().unwrap().unwrap();
        for trusted in [false, true] {
            service
                .trust(TrustRequest {
                    workspace_id: snapshot.id.clone(),
                    revision: snapshot.revision.clone(),
                    trusted,
                })
                .unwrap();
            assert_ne!(
                service
                    .active
                    .lock()
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .context
                    .session_id,
                original
            );
        }
        let before_reload = service
            .active
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .context
            .session_id
            .clone();
        service
            .reload(RevisionRequest {
                workspace_id: snapshot.id,
                revision: snapshot.revision,
            })
            .unwrap();
        assert_ne!(
            service
                .active
                .lock()
                .unwrap()
                .as_ref()
                .unwrap()
                .context
                .session_id,
            before_reload
        );
    }

    #[test]
    fn approval_decision_rechecks_disk_without_waiting_for_watch_or_poll() {
        let (_tmp, service, id) = approval_fixture();
        let snapshot = service.snapshot().unwrap().unwrap();
        let manifest = Path::new(&snapshot.path).join("ghosh.toml");
        let source = fs::read_to_string(&manifest).unwrap();
        fs::write(manifest, format!("{source}\n# outside edit\n")).unwrap();
        assert_eq!(
            service
                .decide(ApprovalDecisionRequest {
                    id,
                    decision: ApprovalDecision::AllowOnce
                })
                .unwrap_err()
                .code,
            ErrorCode::Denied
        );
        assert_eq!(
            service.operations().unwrap().approvals.requests[0].state,
            ApprovalState::Invalidated
        );
    }

    #[test]
    fn fast_untrust_retrust_and_save_cannot_revive_existing_requests() {
        for save in [false, true] {
            let (_tmp, service, id) = approval_fixture();
            service
                .decide(ApprovalDecisionRequest {
                    id: id.clone(),
                    decision: ApprovalDecision::AllowSession,
                })
                .unwrap();
            let snapshot = service.snapshot().unwrap().unwrap();
            if save {
                service
                    .rename(RenameRequest {
                        workspace_id: snapshot.id,
                        revision: snapshot.revision,
                        name: "Changed".into(),
                    })
                    .unwrap();
            } else {
                for trusted in [false, true] {
                    service
                        .trust(TrustRequest {
                            workspace_id: snapshot.id.clone(),
                            revision: snapshot.revision.clone(),
                            trusted,
                        })
                        .unwrap();
                }
            }
            assert_eq!(
                service.operations().unwrap().approvals.requests[0].state,
                ApprovalState::Invalidated
            );
            assert!(
                service
                    .decide(ApprovalDecisionRequest {
                        id,
                        decision: ApprovalDecision::AllowOnce
                    })
                    .is_err()
            );
        }
    }

    #[test]
    fn node_commit_invalidates_grants_and_publishes_only_the_reviewed_definition() {
        let (_tmp, service, id) = approval_fixture();
        service
            .decide(ApprovalDecisionRequest {
                id,
                decision: ApprovalDecision::AllowSession,
            })
            .unwrap();
        let snapshot = service.snapshot().unwrap().unwrap();
        let source = "schema = 1\nid = 'worker'\nhost = '192.0.2.10'\nuser = 'ops'\n[auth]\nmethod = 'agent'\n";
        let mutation = NodeMutationRequest {
            context: RevisionRequest {
                workspace_id: snapshot.id,
                revision: snapshot.revision,
            },
            change: NodeChange::Import {
                source: source.into(),
                replace: false,
            },
        };
        let preview = service.preview_node(mutation.clone()).unwrap();
        assert!(service.snapshot().unwrap().unwrap().nodes.is_empty());
        let committed = service
            .commit_node(NodeCommitRequest {
                mutation,
                digest: preview.digest,
            })
            .unwrap();
        assert!(committed.trusted);
        assert_eq!(committed.nodes[0].id, "worker");
        assert_eq!(
            service.operations().unwrap().approvals.requests[0].state,
            ApprovalState::Invalidated
        );
    }

    #[test]
    fn early_document_requests_survive_until_consumer_is_ready() {
        let queue = Arc::new(DocumentOpenQueue::default());
        queue.enqueue("first.ghosh".into()).unwrap();
        queue.enqueue("second.ghosh".into()).unwrap();
        let consumer = Arc::clone(&queue);
        drop(queue);
        assert_eq!(consumer.next().unwrap().as_deref(), Some("first.ghosh"));
        assert_eq!(consumer.next().unwrap().as_deref(), Some("second.ghosh"));
        assert!(consumer.next().unwrap().is_none());
    }

    #[test]
    fn document_request_queue_is_bounded_without_losing_accepted_requests() {
        let queue = DocumentOpenQueue::default();
        for n in 0..10 {
            queue.enqueue(format!("{n}.ghosh")).unwrap();
        }
        assert!(queue.enqueue("overflow.ghosh".into()).is_err());
        for n in 0..10 {
            assert_eq!(queue.next().unwrap(), Some(format!("{n}.ghosh")));
        }
        assert!(queue.next().unwrap().is_none());
    }
}
