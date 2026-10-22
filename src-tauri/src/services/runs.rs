use super::*;
use ghosh_core::{
    automation::*,
    operations::{JobKind, JobState},
    runs::*,
    workspace::automation::AutomationReadRequest,
};
use std::{
    collections::BTreeSet,
    io::{Read, Write},
    time::{SystemTime, UNIX_EPOCH},
};

#[derive(Default)]
pub(super) struct RunRegistry {
    pub active: Option<String>,
    pub progress: Option<RunProgress>,
}
fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}
fn invalid() -> AppError {
    AppError::new(ErrorCode::InvalidDocument)
}
fn dir(service: &Services) -> Result<PathBuf> {
    let path = service.data_dir.join("runs");
    if !path.exists() {
        fs::create_dir(&path)?;
    }
    if !fs::symlink_metadata(&path)?.file_type().is_dir() {
        return Err(AppError::new(ErrorCode::InvalidPath));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(path)
}
fn save(service: &Services, report: &RunReport, event: Option<&TaskResult>) -> Result<()> {
    let root = dir(service)?;
    let temporary = root.join(format!(".{}.tmp", uuid::Uuid::new_v4()));
    let result = (|| {
        let mut options = fs::OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options.open(&temporary)?;
        file.write_all(&serde_json::to_vec(report).map_err(|_| invalid())?)?;
        file.sync_all()?;
        fs::rename(&temporary, root.join(format!("{}.json", report.id)))?;
        if let Some(event) = event {
            // Append metadata only. Output is bounded in the atomic snapshot.
            let mut options = fs::OpenOptions::new();
            options.append(true).create(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options
                    .mode(0o600)
                    .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
            }
            let mut events = options.open(root.join(format!("{}.jsonl", report.id)))?;
            if !events.metadata()?.is_file() {
                return Err(invalid());
            }
            let line = serde_json::json!({"node":event.node_id,"task":event.name,"phase":event.phase,"state":event.state,"exit_code":event.exit_code});
            serde_json::to_writer(&mut events, &line).map_err(|_| invalid())?;
            events.write_all(b"\n")?;
            events.sync_all()?;
        }
        #[cfg(unix)]
        fs::File::open(&root)?.sync_all()?;
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(temporary);
    }
    result
}
fn load(service: &Services, id: &str) -> Result<RunReport> {
    if uuid::Uuid::parse_str(id).is_err() {
        return Err(invalid());
    }
    let path = dir(service)?.join(format!("{id}.json"));
    let mut options = fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path)?;
    if !file.metadata()?.is_file() {
        return Err(invalid());
    }
    let mut bytes = vec![];
    file.take(2 * 1024 * 1024 + 1).read_to_end(&mut bytes)?;
    if bytes.len() > 2 * 1024 * 1024 {
        return Err(invalid());
    }
    let mut report: RunReport = serde_json::from_slice(&bytes).map_err(|_| invalid())?;
    if report.id != id {
        return Err(invalid());
    }
    if report.state == RunState::Running {
        let registry = service.runs.lock().map_err(poisoned)?;
        if registry.active.as_deref() == Some(id) {
            report.progress = registry.progress.clone();
        }
    }
    if report.state == RunState::Running
        && service.runs.lock().map_err(poisoned)?.active.as_deref() != Some(id)
    {
        report.state = RunState::OutcomeUnknown;
        report.finished_at = Some(now());
        report.message = "Ghosh stopped before this run finished. Inspect targets before retrying; no automatic replay occurred.".into();
    }
    Ok(report)
}
struct VerifiedExecutor<'a> {
    service: &'a Services,
    request: &'a AutomationReadRequest,
    source: &'a str,
    input: &'a str,
    inner: &'a mut dyn Executor,
    node: &'a str,
}
impl Executor for VerifiedExecutor<'_> {
    fn step_started(&mut self, name: &str, _handler: bool) {
        if let Ok(mut registry) = self.service.runs.lock()
            && let Some(progress) = &mut registry.progress
        {
            progress.node_id = self.node.into();
            progress.step = name.into();
            progress.phase.clear();
            progress.output.clear();
            progress.truncated = false;
        }
    }
    fn step_finished(&mut self, handler: bool) {
        if let Ok(mut registry) = self.service.runs.lock()
            && let Some(progress) = &mut registry.progress
        {
            if !handler {
                progress.completed_steps += 1;
            }
            progress.output.clear();
        }
    }
    fn phase_started(&mut self, phase: &str) {
        if let Ok(mut registry) = self.service.runs.lock()
            && let Some(progress) = &mut registry.progress
        {
            progress.phase = phase.into();
            progress.output.clear();
            progress.truncated = false;
        }
    }

    fn execute(
        &mut self,
        script: &str,
        timeout_s: u32,
        cancel: &Cancellation,
    ) -> Result<ExecOutput> {
        // Re-read the revision before every remote/local effect. No active mutex
        // is held during process I/O. User edits cannot authorize stale source.
        self.service.run_document(self.request, Some(self.source))?;
        let mut captured = vec![];
        let mut truncated = false;
        self.inner.execute_streaming(
            &format!("export GHOSH_INPUT={}\n{}", quote(self.input), script),
            timeout_s,
            cancel,
            &mut |bytes| {
                let previous = (captured.len(), truncated);
                bounded_append(&mut captured, bytes, &mut truncated);
                // Publish the last available bytes even if the process becomes
                // silent; after the cap, continue draining without rebuilding.
                if previous == (captured.len(), truncated) {
                    return;
                }
                if let Ok(mut registry) = self.service.runs.lock()
                    && let Some(progress) = &mut registry.progress
                {
                    progress.output = live_redact(&captured, self.input);
                    progress.truncated = truncated;
                }
            },
        )
    }
}
impl Services {
    pub(super) fn run_active(&self) -> Result<bool> {
        Ok(self.runs.lock().map_err(poisoned)?.active.is_some())
    }
    pub(super) fn run_document(
        &self,
        request: &AutomationReadRequest,
        source: Option<&str>,
    ) -> Result<AutomationDocument> {
        let guard = self.active.lock().map_err(poisoned)?;
        let active = guard
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?;
        let snapshot = active.workspace.snapshot();
        if !snapshot.trusted || snapshot.read_only || snapshot.external_changed {
            return Err(AppError::new(ErrorCode::Denied));
        }
        let document = active
            .workspace
            .automation_read(Principal::LocalUi, request)?;
        if !validate(&document).valid || source.is_some_and(|s| s != document.source) {
            return Err(invalid());
        }
        Ok(document)
    }
    pub fn runs_list(&self, request: RevisionRequest) -> Result<Vec<RunReport>> {
        // Listing is inert and remains available when the workspace needs reload.
        let snapshot = self
            .snapshot()?
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?;
        if request.workspace_id != snapshot.id {
            return Err(AppError::new(ErrorCode::Denied));
        }
        let root = dir(self)?;
        let mut files: Vec<_> = fs::read_dir(root)?
            .filter_map(|entry| entry.ok())
            .filter(|entry| entry.path().extension().is_some_and(|s| s == "json"))
            .collect();
        files.sort_by_key(|entry| {
            std::cmp::Reverse(entry.metadata().and_then(|m| m.modified()).ok())
        });
        let mut reports = vec![];
        for entry in files.into_iter().take(1024) {
            let name = entry.file_name();
            let Some(id) = name.to_str().and_then(|s| s.strip_suffix(".json")) else {
                continue;
            };
            if let Ok(report) = load(self, id)
                && report.workspace_id == request.workspace_id
            {
                reports.push(report);
            }
            if reports.len() == 100 {
                break;
            }
        }
        reports.sort_by_key(|report| std::cmp::Reverse(report.started_at));
        Ok(reports)
    }
    pub fn runs_cleanup(&self, request: RunCleanupRequest) -> Result<usize> {
        let guard = self.active.lock().map_err(poisoned)?;
        let active = guard
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?;
        let snapshot = active.workspace.snapshot();
        if request.context.workspace_id != snapshot.id
            || request.context.revision != snapshot.revision
            || snapshot.read_only
        {
            return Err(AppError::new(ErrorCode::Denied));
        }
        let root = dir(self)?;
        let ids = if let Some(id) = &request.id {
            vec![id.clone()]
        } else {
            fs::read_dir(&root)?
                .filter_map(|e| e.ok())
                .filter_map(|e| {
                    e.file_name()
                        .to_str()?
                        .strip_suffix(".json")
                        .map(str::to_owned)
                })
                .collect()
        };
        let mut count = 0;
        for id in ids {
            let report = match load(self, &id) {
                Ok(r) => r,
                Err(e) if request.id.is_some() => return Err(e),
                Err(_) => continue,
            };
            if report.workspace_id != snapshot.id {
                if request.id.is_some() {
                    return Err(AppError::new(ErrorCode::Denied));
                } else {
                    continue;
                }
            }
            let registry = self.runs.lock().map_err(poisoned)?;
            if report.state == RunState::Running || registry.active.as_deref() == Some(&id) {
                if request.id.is_some() {
                    return Err(AppError::new(ErrorCode::Busy));
                } else {
                    continue;
                }
            }
            // UUIDs are validated by load; unlink only these two workspace-owned artifacts.
            match fs::remove_file(root.join(format!("{id}.jsonl"))) {
                Ok(()) => (),
                Err(e) if e.kind() == std::io::ErrorKind::NotFound => (),
                Err(e) => return Err(e.into()),
            }
            fs::remove_file(root.join(format!("{id}.json")))?;
            count += 1;
        }
        #[cfg(unix)]
        fs::File::open(root)?.sync_all()?;
        Ok(count)
    }
    pub fn run_stop(&self, id: String) -> Result<()> {
        let report = load(self, &id)?;
        if self
            .snapshot()?
            .is_none_or(|snapshot| snapshot.id != report.workspace_id)
        {
            return Err(AppError::new(ErrorCode::Denied));
        }
        self.jobs.cancel(&report.job_id)
    }
    pub fn run_start(self: &Arc<Self>, mut request: RunRequest) -> Result<RunReport> {
        if request.targets.is_empty() || request.targets.len() > 8 || !input_valid(&request.input) {
            return Err(invalid());
        }
        // Reserve while holding active, the same lock used by all mutations.
        let (document, mut report, job) = {
            let guard = self.active.lock().map_err(poisoned)?;
            let active = guard
                .as_ref()
                .ok_or_else(|| AppError::new(ErrorCode::Denied))?;
            let snapshot = active.workspace.snapshot();
            if !snapshot.trusted || snapshot.read_only || snapshot.external_changed {
                return Err(AppError::new(ErrorCode::Denied));
            }
            let document = active
                .workspace
                .automation_read(Principal::LocalUi, &request.automation)?;
            if !validate(&document).valid
                || document.kind.is_script() && request.mode != RunMode::Apply
            {
                return Err(invalid());
            }
            if document.kind == AutomationKind::Notebook {
                notebook_cells(&parse_notebook(&document.source)?, &request.cells)?;
            } else if !request.cells.is_empty() {
                return Err(invalid());
            }
            let mut seen = BTreeSet::new();
            for target in &request.targets {
                if !seen.insert(&target.node_id)
                    || target.node_id != "@local"
                        && !snapshot.nodes.iter().any(|node| node.id == target.node_id)
                    || (target.node_id == "@local") != target.connection.is_none()
                {
                    return Err(invalid());
                }
                if let Some(connection) = &target.connection {
                    self.validate_run_ticket(
                        connection,
                        &request.automation.context,
                        &target.node_id,
                    )?;
                }
            }
            if document.kind == AutomationKind::Playbook && !uses_ansible(&document) {
                let book = parse_playbook(&document.source)?;
                if !book.targets.is_empty()
                    && request
                        .targets
                        .iter()
                        .any(|target| !book.targets.contains(&target.node_id))
                {
                    return Err(invalid());
                }
            }
            if self
                .jobs
                .snapshot()?
                .jobs
                .iter()
                .any(|job| matches!(job.state, JobState::Running | JobState::Cancelling))
            {
                return Err(AppError::new(ErrorCode::Busy));
            }
            let mut registry = self.runs.lock().map_err(poisoned)?;
            if registry.active.is_some() {
                return Err(AppError::new(ErrorCode::Busy));
            }
            let job = self.jobs.begin(JobKind::AutomationRun, true)?;
            let report = RunReport {
                id: uuid::Uuid::new_v4().to_string(),
                job_id: job.id().into(),
                workspace_id: snapshot.id,
                revision: snapshot.revision,
                automation_id: document.id.clone(),
                kind: document.kind,
                mode: request.mode,
                targets: request.targets.iter().map(|t| t.node_id.clone()).collect(),
                state: RunState::Running,
                started_at: now(),
                finished_at: None,
                results: vec![],
                progress: Some(RunProgress {
                    total_steps: if uses_ansible(&document) {
                        None
                    } else {
                        Some(
                            if document.kind == AutomationKind::Playbook {
                                parse_playbook(&document.source)?.tasks.len()
                            } else if document.kind == AutomationKind::Notebook {
                                notebook_cells(&parse_notebook(&document.source)?, &request.cells)?
                                    .len()
                            } else {
                                1
                            } * request.targets.len(),
                        )
                    },
                    step: "Connecting targets".into(),
                    ..RunProgress::default()
                }),
                message: "Targets execute sequentially. Stop cannot roll back completed changes."
                    .into(),
            };
            save(self, &report, None)?;
            registry.active = Some(report.id.clone());
            registry.progress = report.progress.clone();
            (document, report, job)
        };
        let initial = report.clone();
        let service = self.clone();
        let spawned = std::thread::Builder::new().name("ghosh-run".into()).spawn(move || {
            let cancel = job.cancellation();
            let outcome: Result<()> = (|| {
                if uses_ansible(&document) {
                    return super::ansible::run(&service, &mut request, &document, &mut report, cancel);
                }
                let mut prepared = vec![];
                // Consume all short-lived handshake tickets before long tasks begin.
                for target in request.targets.iter_mut() {
                    cancel.check()?;
                    let executor: Box<dyn Executor> = if target.node_id == "@local" { Box::new(super::local_exec::LocalExecutor) } else {
                        match service.with_connection(target.connection.take().ok_or_else(invalid)?, cancel, |connection, _| Ok(connection)) {
                            Ok(connection) => Box::new(connection),
                            Err(error) => {
                                append(&service, &mut report, TaskResult { node_id: target.node_id.clone(), name: "SSH authentication".into(), state: TaskState::Failed, phase: "connect".into(), output: error.message.clone(), exit_code: None, truncated: false }, &request.input)?;
                                if error.code == ErrorCode::Cancelled { return Err(error); } else { continue; }
                            }
                        }
                    };
                    prepared.push((target.node_id.clone(), executor));
                }
                for (node_id, mut executor) in prepared {
                    cancel.check()?;
                    let mut verified = VerifiedExecutor { service: &service, request: &request.automation, source: &document.source, input: &request.input, node: &node_id, inner: executor.as_mut() };
                    let mut emit = |value| append(&service, &mut report, value, &request.input);
                    let result = if document.kind == AutomationKind::Playbook {
                        run_playbook(&mut verified, &parse_playbook(&document.source)?, &node_id, request.mode, cancel, &mut emit)
                    } else if document.kind == AutomationKind::Notebook {
                        run_notebook(&mut verified, &parse_notebook(&document.source)?, &request.cells, &request.input, &node_id, cancel, &mut emit)
                    } else { run_script(&mut verified, &document, &request.input, &node_id, cancel, &mut emit) };
                    if let Err(error) = result {
                        append(&service, &mut report, TaskResult { node_id: node_id.clone(), name: document.id.clone(), state: if error.code == ErrorCode::Cancelled { TaskState::Stopped } else { TaskState::Failed }, phase: "execution".into(), output: error.message.clone(), exit_code: None, truncated: false }, &request.input)?;
                        return Err(error);
                    }
                }
                Ok(())
            })();
            if let Err(error) = &outcome {
                let _ = append(&service, &mut report, TaskResult { node_id: "@controller".into(), name: "Execution error".into(), phase: "controller".into(), state: if error.code == ErrorCode::Cancelled { TaskState::Stopped } else { TaskState::Failed }, output: error.message.clone(), exit_code: None, truncated: false }, &request.input);
            }
            report.state = if report.results.iter().any(|r| r.state == TaskState::OutcomeUnknown) { RunState::OutcomeUnknown }
                else if outcome.as_ref().err().is_some_and(|e| e.code == ErrorCode::Cancelled) { RunState::Stopped }
                else if outcome.is_err() || report.results.iter().any(|r| r.state == TaskState::Failed) { RunState::Failed }
                else { RunState::Succeeded };
            if let Ok(registry) = service.runs.lock() { report.progress = registry.progress.clone(); }
            if let Some(progress) = &mut report.progress { progress.output.clear(); }
            report.finished_at = Some(now());
            report.message = match report.state {
                RunState::Succeeded if request.mode == RunMode::Check => if report.results.iter().any(|r| r.state == TaskState::Drift || r.state == TaskState::Changed) { "Check complete — changes needed. No Apply was requested. Choose Apply to make changes." } else { "Check complete — no changes reported. No Apply was requested." },
                RunState::Succeeded if uses_ansible(&document) => "Ansible finished. Check reports predicted changes; validation and handlers follow the original playbook.",
                RunState::Succeeded => "Finished. Check results may include drift; Apply verifies every changed task.",
                RunState::Stopped => "Stopped before the next invocation. Completed changes remain applied.",
                RunState::OutcomeUnknown => "A command was interrupted, timed out or lost its connection. Inspect affected targets before retrying. No rollback is implied.",
                _ => "Run failed. Inspect task results; completed changes remain applied.",
            }.into();
            let stored = save(&service, &report, None);
            let finish = if let Err(error) = stored { Err(error) } else if report.state == RunState::Succeeded { Ok(()) } else { Err(AppError::new(if report.state == RunState::Stopped { ErrorCode::Cancelled } else { ErrorCode::ConnectionFailed })) };
            let _ = job.finish(&finish);
            if let Ok(mut registry) = service.runs.lock() { registry.active = None; registry.progress = None; }
        });
        if spawned.is_err() {
            if let Ok(mut registry) = self.runs.lock() {
                registry.active = None;
                registry.progress = None;
            }
            let mut failed = initial.clone();
            failed.state = RunState::Failed;
            failed.finished_at = Some(now());
            save(self, &failed, None)?;
            return Err(AppError::new(ErrorCode::Internal));
        }
        Ok(initial)
    }
}
pub(super) fn append(
    service: &Services,
    report: &mut RunReport,
    mut value: TaskResult,
    input: &str,
) -> Result<()> {
    if report.results.len() >= 2048 {
        return Err(AppError::new(ErrorCode::Capacity));
    }
    value.output = if value.truncated {
        live_redact(value.output.as_bytes(), input)
    } else {
        redact(&value.output, input)
    };
    let used: usize = report.results.iter().map(|r| r.output.len()).sum();
    let available = (256 * 1024usize).saturating_sub(used).min(OUTPUT_LIMIT);
    if value.output.len() > available {
        let mut end = available;
        while !value.output.is_char_boundary(end) {
            end -= 1;
        }
        value.output.truncate(end);
        value.truncated = true;
    }
    if let Ok(registry) = service.runs.lock() {
        report.progress = registry.progress.clone();
    }
    if let Some(progress) = &mut report.progress {
        progress.output.clear();
    }
    report.results.push(value.clone());
    save(service, report, Some(&value))
}

#[cfg(test)]
mod tests {
    use super::*;
    use ghosh_core::workspace::automation::{AutomationCommit, AutomationMutation};
    use std::time::{Duration, Instant};
    fn fixture(
        source: &str,
        kind: AutomationKind,
    ) -> (tempfile::TempDir, Arc<Services>, RunRequest) {
        let tmp = tempfile::tempdir().unwrap();
        let root = tmp.path().canonicalize().unwrap();
        let service = Arc::new(
            Services::new(root.join("data"), Arc::new(DocumentOpenQueue::default())).unwrap(),
        );
        let mut workspace =
            Workspace::create(&root, "Runs", &root.join("locks"), &Cancellation::default())
                .unwrap();
        let snapshot = workspace.snapshot();
        let mutation = AutomationMutation {
            context: RevisionRequest {
                workspace_id: snapshot.id,
                revision: snapshot.revision,
            },
            document: AutomationDocument {
                id: "smoke".into(),
                kind,
                source: source.into(),
            },
            replace: false,
            remove: false,
        };
        let preview = workspace
            .automation_preview(Principal::LocalUi, &mutation)
            .unwrap();
        let snapshot = workspace
            .automation_commit(
                Principal::LocalUi,
                &AutomationCommit {
                    mutation,
                    digest: preview.digest,
                },
                &Cancellation::default(),
            )
            .unwrap();
        let context = RevisionRequest {
            workspace_id: snapshot.id.clone(),
            revision: snapshot.revision.clone(),
        };
        let watch = WorkspaceWatch::start(workspace.root(), || {}).unwrap();
        *service.active.lock().unwrap() = Some(Active {
            workspace,
            context: ApprovalContext::new(snapshot.id, snapshot.revision),
            _watch: watch,
        });
        (
            tmp,
            service,
            RunRequest {
                cells: vec![],
                automation: AutomationReadRequest {
                    context,
                    id: "smoke".into(),
                    kind,
                },
                mode: RunMode::Apply,
                targets: vec![RunTarget {
                    node_id: "@local".into(),
                    connection: None,
                }],
                input: "{}".into(),
                controller_python: None,
            },
        )
    }
    fn wait(service: &Services, id: &str) -> RunReport {
        let deadline = Instant::now() + Duration::from_secs(10);
        while service.run_active().unwrap() {
            assert!(Instant::now() < deadline);
            std::thread::sleep(Duration::from_millis(10));
        }
        load(service, id).unwrap()
    }
    #[test]
    fn running_snapshots_stream_both_outputs_before_exit_and_persist_step_counts() {
        let (_tmp, service, mut request) = fixture(
            "printf 'LIVE_STDOUT\\n'; printf 'LIVE_STDERR\\n' >&2; sleep 2; printf 'FINAL\\n'",
            AutomationKind::Shell,
        );
        request.input = "{}".into();
        let context = request.automation.context.clone();
        let first = service.run_start(request).unwrap();
        let deadline = Instant::now() + Duration::from_secs(5);
        loop {
            let reports = service.runs_list(context.clone()).unwrap();
            let report = &reports[0];
            let progress = report.progress.as_ref().unwrap();
            if progress.output.contains("LIVE_STDOUT") && progress.output.contains("LIVE_STDERR") {
                assert_eq!(report.state, RunState::Running);
                assert!(report.results.is_empty());
                assert_eq!(progress.total_steps, Some(1));
                assert_eq!(progress.completed_steps, 0);
                break;
            }
            assert!(Instant::now() < deadline, "no live output: {report:?}");
            std::thread::sleep(Duration::from_millis(30));
        }
        let report = wait(&service, &first.id);
        assert_eq!(report.state, RunState::Succeeded);
        assert_eq!(report.progress.as_ref().unwrap().completed_steps, 1);
        assert!(report.results[0].output.contains("FINAL"));
        assert!(report.progress.as_ref().unwrap().output.is_empty());
    }
    #[test]
    fn check_apply_verify_count_tasks_once_and_old_reports_still_load() {
        let files = tempfile::tempdir().unwrap();
        let path = quote(files.path().join("done").to_str().unwrap());
        let source = format!(
            "schema: 1\nname: Steps\ntasks:\n  - name: First\n    check:\n      command: test -f {path} || exit 10\n    shell:\n      command: touch {path}\n  - name: Second\n    check:\n      command: 'true'\n    shell:\n      command: 'true'\n"
        );
        let (_tmp, service, request) = fixture(&source, AutomationKind::Playbook);
        let first = service.run_start(request).unwrap();
        let report = wait(&service, &first.id);
        assert_eq!(report.state, RunState::Succeeded, "{report:?}");
        let progress = report.progress.as_ref().unwrap();
        assert_eq!(progress.total_steps, Some(2));
        assert_eq!(progress.completed_steps, 2);
        assert!(report.results.len() > 2);
        let mut old = serde_json::to_value(&report).unwrap();
        old.as_object_mut().unwrap().remove("progress");
        assert!(
            serde_json::from_value::<RunReport>(old)
                .unwrap()
                .progress
                .is_none()
        );
    }
    #[test]
    fn local_run_persists_masks_inputs_and_rejects_stale_source() {
        let (_tmp, service, mut request) = fixture(
            "printf '%s' \"$GHOSH_INPUT\"; printf 'stderr\\n' >&2",
            AutomationKind::Shell,
        );
        request.input = r#"{"value":"private-token"}"#.into();
        let initial = service.run_start(request).unwrap();
        let report = wait(&service, &initial.id);
        assert_eq!(report.state, RunState::Succeeded);
        assert!(report.results[0].output.contains("stderr"));
        let stored =
            fs::read_to_string(dir(&service).unwrap().join(format!("{}.json", report.id))).unwrap();
        assert!(!stored.contains("private-token"));
        assert!(stored.contains("[input]"));
        assert!(load(&service, "../../bad").is_err());
        let snapshot = service.snapshot().unwrap().unwrap();
        assert_eq!(
            service
                .runs_list(RevisionRequest {
                    workspace_id: snapshot.id.clone(),
                    revision: snapshot.revision
                })
                .unwrap()
                .len(),
            1
        );
        assert!(
            service
                .run_start(RunRequest {
                    cells: vec![],
                    automation: AutomationReadRequest {
                        context: RevisionRequest {
                            workspace_id: snapshot.id,
                            revision: "stale".into()
                        },
                        id: "smoke".into(),
                        kind: AutomationKind::Shell
                    },
                    mode: RunMode::Apply,
                    targets: vec![RunTarget {
                        node_id: "@local".into(),
                        connection: None
                    }],
                    input: "{}".into(),
                    controller_python: None,
                })
                .is_err()
        );
        let mut crashed = report;
        crashed.state = RunState::Running;
        save(&service, &crashed, None).unwrap();
        assert_eq!(
            load(&service, &crashed.id).unwrap().state,
            RunState::OutcomeUnknown
        );
    }
    #[test]
    #[ignore = "requires GHOSH_CONTROLLER_PYTHON pinned controller"]
    fn upstream_ansible_check_apply_include_no_log_failure_and_stop() {
        let python = std::env::var("GHOSH_CONTROLLER_PYTHON").unwrap();
        let files = tempfile::tempdir().unwrap();
        let target = files.path().join("target");
        let source = format!(
            "- name: Upstream\n  hosts: all\n  gather_facts: false\n  tasks:\n    - name: Ensure directory\n      ansible.builtin.file:\n        path: {}\n        state: directory\n        mode: '0700'\n    - name: Hidden\n      ansible.builtin.debug:\n        msg: NEVER_STORE_THIS\n      no_log: true\n    - ansible.builtin.include_tasks: included.yml\n",
            serde_json::to_string(&target).unwrap()
        );
        let (_tmp, service, request) = fixture(&source, AutomationKind::Playbook);
        let workspace_root = service
            .active
            .lock()
            .unwrap()
            .as_ref()
            .unwrap()
            .workspace
            .root()
            .to_owned();
        // Dependencies are kept beside the source and interpreted by upstream.
        fs::write(
            workspace_root.join("playbooks/included.yml"),
            "- name: Included task\n  ansible.builtin.debug:\n    msg: INCLUDED_OK\n",
        )
        .unwrap();
        // Include files participate in the workspace revision; reload before run.
        service.reload(request.automation.context.clone()).unwrap();
        let snapshot = service.snapshot().unwrap().unwrap();
        let mut automation = request.automation;
        automation.context.revision = snapshot.revision;
        let make = |mode| RunRequest {
            cells: vec![],
            automation: automation.clone(),
            mode,
            targets: vec![RunTarget {
                node_id: "@local".into(),
                connection: None,
            }],
            input: "{}".into(),
            controller_python: Some(python.clone()),
        };
        let first = service.run_start(make(RunMode::Check)).unwrap();
        let check = wait(&service, &first.id);
        assert_eq!(check.state, RunState::Succeeded, "{check:?}");
        assert!(check.results.iter().any(|r| r.state == TaskState::Drift));
        assert!(!target.exists());
        let first = service.run_start(make(RunMode::Apply)).unwrap();
        let apply = wait(&service, &first.id);
        assert_eq!(apply.state, RunState::Succeeded, "{apply:?}");
        assert!(target.is_dir());
        assert!(
            apply
                .results
                .iter()
                .any(|r| r.output.contains("INCLUDED_OK"))
        );
        assert!(
            !serde_json::to_string(&apply)
                .unwrap()
                .contains("NEVER_STORE_THIS")
        );
        let first = service.run_start(make(RunMode::Apply)).unwrap();
        let repeat = wait(&service, &first.id);
        assert_eq!(repeat.state, RunState::Succeeded);
        assert!(!repeat.results.iter().any(|r| r.state == TaskState::Changed));
        assert!(!fs::read_dir(&service.data_dir).unwrap().any(|p| {
            p.unwrap()
                .file_name()
                .to_string_lossy()
                .starts_with("runner-")
        }));
        let (_tmp2, failed_service, mut failed) = fixture(
            "- hosts: all\n  gather_facts: false\n  tasks:\n    - ansible.builtin.fail:\n        msg: EXPECTED_FAILURE\n",
            AutomationKind::Playbook,
        );
        failed.controller_python = Some(python.clone());
        let first = failed_service.run_start(failed).unwrap();
        assert_eq!(wait(&failed_service, &first.id).state, RunState::Failed);
        let (_tmp3, stopped_service, mut stop) = fixture(
            "- hosts: all\n  gather_facts: false\n  tasks:\n    - ansible.builtin.command: sleep 30\n",
            AutomationKind::Playbook,
        );
        stop.controller_python = Some(python.clone());
        let first = stopped_service.run_start(stop).unwrap();
        std::thread::sleep(Duration::from_secs(1));
        stopped_service.run_stop(first.id.clone()).unwrap();
        let report = wait(&stopped_service, &first.id);
        assert!(
            matches!(report.state, RunState::OutcomeUnknown | RunState::Stopped),
            "{report:?}"
        );
        // A real external project retains its inventory groups, group_vars,
        // roles and handlers. Nothing is translated into the legacy schema.
        let project = tempfile::tempdir().unwrap();
        fs::create_dir_all(project.path().join("group_vars")).unwrap();
        fs::create_dir_all(project.path().join("roles/example/tasks")).unwrap();
        fs::write(
            project.path().join("hosts.ini"),
            "[test_group]\nlocalhost\n",
        )
        .unwrap();
        fs::write(
            project.path().join("group_vars/test_group.yml"),
            "project_fact: EXTERNAL_PROJECT_OK\n",
        )
        .unwrap();
        fs::write(project.path().join("roles/example/tasks/main.yml"), "- name: Original role\n  ansible.builtin.debug:\n    msg: '{{ project_fact }}'\n  changed_when: true\n  notify: original handler\n").unwrap();
        fs::write(project.path().join("main.yml"), "- hosts: test_group\n  gather_facts: false\n  roles: [example]\n  handlers:\n    - name: original handler\n      ansible.builtin.debug:\n        msg: HANDLER_OK\n").unwrap();
        let link = format!(
            "schema: 1\nname: Linked project\nproject: {}\nplaybook: main.yml\ninventory: hosts.ini\n",
            serde_json::to_string(project.path()).unwrap()
        );
        let (_tmp4, linked_service, mut linked) = fixture(&link, AutomationKind::Ansible);
        linked.controller_python = Some(python);
        let first = linked_service.run_start(linked).unwrap();
        let report = wait(&linked_service, &first.id);
        assert_eq!(report.state, RunState::Succeeded, "{report:?}");
        assert!(
            report
                .results
                .iter()
                .any(|r| r.output.contains("EXTERNAL_PROJECT_OK"))
        );
        assert!(
            report
                .results
                .iter()
                .any(|r| r.output.contains("HANDLER_OK"))
        );
    }
    #[test]
    fn real_local_check_apply_verify_and_repeat_are_idempotent() {
        let files = tempfile::tempdir().unwrap();
        let target = files.path().join("target's file");
        let notice = files.path().join("notice");
        let task = |path: &Path| {
            let check = format!(
                "if test -e {}; then exit 0; else exit 10; fi",
                quote(path.to_str().unwrap())
            );
            let apply = format!("printf done > {}", quote(path.to_str().unwrap()));
            serde_json::json!({"name":"Ensure file", "check":{"command":check}, "shell":{"command":apply}, "timeout_s":5})
        };
        let mut main = task(&target);
        main["notify"] = serde_json::json!(["notice"]);
        let source = serde_json::json!({"schema":1,"name":"Real idempotence", "tasks":[main], "handlers":{"notice":task(&notice)}}).to_string();
        let (_tmp, service, request) = fixture(&source, AutomationKind::Playbook);
        let make_request = |mode| RunRequest {
            cells: vec![],
            automation: request.automation.clone(),
            mode,
            targets: vec![RunTarget {
                node_id: "@local".into(),
                connection: None,
            }],
            input: "{}".into(),
            controller_python: None,
        };
        let check = service.run_start(make_request(RunMode::Check)).unwrap();
        let check = wait(&service, &check.id);
        assert_eq!(check.state, RunState::Succeeded);
        assert_eq!(check.results[0].state, TaskState::Drift);
        assert!(!target.exists());
        assert!(check.message.contains("changes needed"));
        assert!(!notice.exists());
        let apply = service.run_start(make_request(RunMode::Apply)).unwrap();
        let apply = wait(&service, &apply.id);
        assert_eq!(apply.state, RunState::Succeeded);
        assert!(
            apply
                .results
                .iter()
                .any(|r| r.phase == "verify" && r.state == TaskState::Ok)
        );
        assert!(
            apply
                .results
                .iter()
                .any(|r| r.name.starts_with("Handler:") && r.state == TaskState::Changed)
        );
        assert_eq!(fs::read_to_string(&target).unwrap(), "done");
        assert_eq!(fs::read_to_string(&notice).unwrap(), "done");
        let repeat = service.run_start(make_request(RunMode::Apply)).unwrap();
        let repeat = wait(&service, &repeat.id);
        assert_eq!(repeat.state, RunState::Succeeded);
        assert_eq!(repeat.results.len(), 1);
        assert_eq!(repeat.results[0].state, TaskState::Ok);
    }
    #[test]
    fn notebook_saved_subset_order_verification_and_failure() {
        let files = tempfile::tempdir().unwrap();
        let target = files.path().join("cells");
        let cell = |id: &str, code: String, check: Option<String>| serde_json::json!({"id":id,"title":id,"runtime":"shell","code":code,"timeout_s":3,"success_check":check});
        let source = serde_json::json!({"schema":1,"name":"Cells","cells":[
            cell("first", format!("printf first >> {}", quote(target.to_str().unwrap())), None),
            cell("second", format!("printf second >> {}", quote(target.to_str().unwrap())), Some(format!("test -f {}",quote(target.to_str().unwrap())))),
            cell("fail", "exit 7".into(), None),
            cell("never", format!("printf never >> {}", quote(target.to_str().unwrap())), None)
        ]}).to_string();
        let (_tmp, service, mut request) = fixture(&source, AutomationKind::Notebook);
        request.cells = vec!["missing".into()];
        assert!(
            service
                .run_start(serde_json::from_value(serde_json::to_value(&request).unwrap()).unwrap())
                .is_err()
        );
        request.cells = vec!["second".into(), "first".into()];
        let initial = service
            .run_start(serde_json::from_value(serde_json::to_value(&request).unwrap()).unwrap())
            .unwrap();
        let report = wait(&service, &initial.id);
        assert_eq!(report.state, RunState::Succeeded, "{report:?}");
        assert_eq!(fs::read_to_string(&target).unwrap(), "firstsecond");
        assert_eq!(report.progress.unwrap().completed_steps, 2);
        assert_eq!(report.results[0].name, "first · first");
        assert_eq!(report.results[2].phase, "verify");
        request.cells = vec!["fail".into(), "never".into()];
        let initial = service.run_start(request).unwrap();
        let report = wait(&service, &initial.id);
        assert_eq!(report.state, RunState::Failed);
        assert_eq!(report.results.len(), 1);
        assert_eq!(fs::read_to_string(&target).unwrap(), "firstsecond");
    }
    #[test]
    fn bundled_playbooks_success_failure_timeout_and_unlimited() {
        let files = tempfile::tempdir().unwrap();
        let source = include_str!("../../../examples/playbooks/steps-success.steps.yml").replace(
            "/tmp/ghosh-playbook-example",
            files.path().to_str().unwrap(),
        );
        let (_tmp, service, request) = fixture(&source, AutomationKind::Notebook);
        let initial = service.run_start(request).unwrap();
        let report = wait(&service, &initial.id);
        assert_eq!(report.state, RunState::Succeeded, "{report:?}");
        assert_eq!(
            fs::read_to_string(files.path().join("output.txt")).unwrap(),
            "Hello from Ghosh\n"
        );
        assert_eq!(report.progress.unwrap().completed_steps, 3);
        for (source, state) in [
            (
                include_str!("../../../examples/playbooks/steps-failure.steps.yml"),
                RunState::Failed,
            ),
            (
                include_str!("../../../examples/playbooks/steps-timeout.steps.yml"),
                RunState::OutcomeUnknown,
            ),
        ] {
            let (_tmp, service, request) = fixture(source, AutomationKind::Notebook);
            let initial = service.run_start(request).unwrap();
            let report = wait(&service, &initial.id);
            assert_eq!(report.state, state, "{report:?}");
            assert!(
                !report
                    .results
                    .iter()
                    .any(|r| r.name.starts_with("skipped") || r.name.starts_with("unlimited"))
            );
        }
        let source = include_str!("../../../examples/playbooks/steps-timeout.steps.yml");
        let (_tmp, service, mut request) = fixture(source, AutomationKind::Notebook);
        request.cells = vec!["unlimited".into()];
        let initial = service.run_start(request).unwrap();
        let report = wait(&service, &initial.id);
        assert_eq!(report.state, RunState::Succeeded, "{report:?}");
        // Unlimited steps still honor Stop.
        let source = source.replace("sleep 5;", "sleep 30;");
        let (_tmp, service, mut request) = fixture(&source, AutomationKind::Notebook);
        request.cells = vec!["unlimited".into()];
        let initial = service.run_start(request).unwrap();
        std::thread::sleep(Duration::from_millis(100));
        service.run_stop(initial.id.clone()).unwrap();
        assert_eq!(wait(&service, &initial.id).state, RunState::OutcomeUnknown);
    }
    #[test]
    fn cleanup_is_workspace_scoped_rejects_active_and_removes_events() {
        let (_tmp, service, request) = fixture("echo ready", AutomationKind::Shell);
        let context = request.automation.context.clone();
        let initial = service
            .run_start(serde_json::from_value(serde_json::to_value(&request).unwrap()).unwrap())
            .unwrap();
        let report = wait(&service, &initial.id);
        let root = dir(&service).unwrap();
        assert!(root.join(format!("{}.jsonl", report.id)).exists());
        let mut foreign = report.clone();
        foreign.id = uuid::Uuid::new_v4().to_string();
        foreign.workspace_id = "other".into();
        save(&service, &foreign, None).unwrap();
        assert_eq!(
            service
                .runs_cleanup(RunCleanupRequest {
                    context: context.clone(),
                    id: Some(foreign.id.clone())
                })
                .unwrap_err()
                .code,
            ErrorCode::Denied
        );
        assert!(
            service
                .runs_cleanup(RunCleanupRequest {
                    context: context.clone(),
                    id: Some("../bad".into())
                })
                .is_err()
        );
        let mut stale = context.clone();
        stale.revision = "stale".into();
        assert!(
            service
                .runs_cleanup(RunCleanupRequest {
                    context: stale,
                    id: None
                })
                .is_err()
        );
        assert_eq!(
            service
                .runs_cleanup(RunCleanupRequest {
                    context: context.clone(),
                    id: None
                })
                .unwrap(),
            1
        );
        assert!(!root.join(format!("{}.json", report.id)).exists());
        assert!(!root.join(format!("{}.jsonl", report.id)).exists());
        assert!(root.join(format!("{}.json", foreign.id)).exists());
        // Registry ownership must protect a report even while final state is being saved.
        let mut active = report.clone();
        active.id = uuid::Uuid::new_v4().to_string();
        save(&service, &active, None).unwrap();
        service.runs.lock().unwrap().active = Some(active.id.clone());
        assert_eq!(
            service
                .runs_cleanup(RunCleanupRequest {
                    context: context.clone(),
                    id: Some(active.id.clone())
                })
                .unwrap_err()
                .code,
            ErrorCode::Busy
        );
        assert_eq!(
            service
                .runs_cleanup(RunCleanupRequest { context, id: None })
                .unwrap(),
            0
        );
        assert!(root.join(format!("{}.json", active.id)).exists());
        service.runs.lock().unwrap().active = None;
    }
    #[test]
    fn stop_reaps_local_group_and_blocks_workspace_changes_while_running() {
        let (_tmp, service, request) = fixture("sleep 30", AutomationKind::Shell);
        let initial = service.run_start(request).unwrap();
        assert!(service.run_active().unwrap());
        assert!(
            service
                .close(RevisionRequest {
                    workspace_id: initial.workspace_id.clone(),
                    revision: initial.revision.clone()
                })
                .is_err()
        );
        std::thread::sleep(Duration::from_millis(60));
        service.run_stop(initial.id.clone()).unwrap();
        let report = wait(&service, &initial.id);
        assert!(matches!(
            report.state,
            RunState::Stopped | RunState::OutcomeUnknown
        ));
    }
}
