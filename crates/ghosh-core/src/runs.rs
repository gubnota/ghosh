//! Portable execution contract and sequential check/apply/verify engine.
//! Adapters own processes/transports; the engine never resolves credentials.
use crate::{
    automation::*, contracts::*, ssh::NodeConnectionCommit,
    workspace::automation::AutomationReadRequest,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeSet;
use ts_rs::TS;

pub const OUTPUT_LIMIT: usize = 32 * 1024;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RunMode {
    Check,
    Apply,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum RunState {
    Running,
    Succeeded,
    Failed,
    Stopped,
    OutcomeUnknown,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum TaskState {
    Ok,
    Changed,
    Drift,
    Retrying,
    Skipped,
    Failed,
    Stopped,
    OutcomeUnknown,
}
#[derive(Serialize, Deserialize, TS)]
pub struct RunTarget {
    pub node_id: String,
    pub connection: Option<NodeConnectionCommit>,
}
#[derive(Serialize, Deserialize, TS)]
pub struct RunRequest {
    pub automation: AutomationReadRequest,
    pub mode: RunMode,
    pub targets: Vec<RunTarget>,
    // Non-secret JSON inputs, passed as GHOSH_INPUT. Never persisted in the report.
    pub input: String,
    // Absolute Python from the user's Ansible controller environment.
    #[serde(default)]
    pub controller_python: Option<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct TaskResult {
    pub node_id: String,
    pub name: String,
    pub state: TaskState,
    pub phase: String,
    pub output: String,
    pub exit_code: Option<i32>,
    pub truncated: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct RunReport {
    pub id: String,
    pub job_id: String,
    pub workspace_id: String,
    pub revision: String,
    pub automation_id: String,
    pub kind: AutomationKind,
    pub mode: RunMode,
    pub targets: Vec<String>,
    pub state: RunState,
    #[ts(type = "number")]
    pub started_at: u64,
    #[ts(type = "number | null")]
    pub finished_at: Option<u64>,
    pub results: Vec<TaskResult>,
    pub message: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecEnd {
    Exited(i32),
    Cancelled,
    TimedOut,
    Disconnected,
}
pub struct ExecOutput {
    pub end: ExecEnd,
    pub output: Vec<u8>,
    pub truncated: bool,
}
/// One bounded invocation. Adapters must drain both streams and observe cancellation.
pub trait Executor {
    fn execute(
        &mut self,
        script: &str,
        timeout_s: u32,
        cancel: &Cancellation,
    ) -> Result<ExecOutput>;
}
pub fn quote(value: &str) -> String {
    format!("'{}'", value.replace('\'', "'\\''"))
}
pub fn bounded_append(output: &mut Vec<u8>, bytes: &[u8], truncated: &mut bool) {
    let remaining = OUTPUT_LIMIT.saturating_sub(output.len());
    output.extend_from_slice(&bytes[..bytes.len().min(remaining)]);
    *truncated |= bytes.len() > remaining;
}
pub fn input_valid(input: &str) -> bool {
    input.len() <= 32 * 1024
        && !input.contains('\0')
        && serde_json::from_str::<serde_json::Value>(input).is_ok_and(|value| value.is_object())
}
pub fn script_payload(kind: AutomationKind, source: &str, input: &str) -> Result<String> {
    if !input_valid(input) {
        return Err(AppError::new(ErrorCode::InvalidDocument));
    }
    let command = match kind {
        AutomationKind::Shell => format!("exec sh -c {}", quote(source)),
        AutomationKind::Python => format!(
            "command -v python3 >/dev/null || {{ printf 'Python 3 is not installed\\n'; exit 127; }}\nexec python3 -u -c {}",
            quote(source)
        ),
        // No prompts, project config discovery, dependency downloads or lockfile writes.
        AutomationKind::Deno => format!(
            "command -v deno >/dev/null || {{ printf 'Deno is not installed\\n'; exit 127; }}\ndeno --version || exit $?\nprintf %s {} | deno run --no-prompt --no-config --no-lock --cached-only --allow-env=GHOSH_INPUT --allow-sys=hostname --ext=ts -",
            quote(source)
        ),
        _ => return Err(AppError::new(ErrorCode::InvalidDocument)),
    };
    Ok(format!("export GHOSH_INPUT={}\n{command}\n", quote(input)))
}
fn result(node: &str, name: &str, phase: &str, output: ExecOutput, changed: bool) -> TaskResult {
    let (state, exit_code) = match output.end {
        ExecEnd::Exited(0) => (
            if changed {
                TaskState::Changed
            } else {
                TaskState::Ok
            },
            Some(0),
        ),
        ExecEnd::Exited(code) => (TaskState::Failed, Some(code)),
        // Closing a remote channel is not proof that a process stopped.
        ExecEnd::Cancelled | ExecEnd::TimedOut | ExecEnd::Disconnected => {
            (TaskState::OutcomeUnknown, None)
        }
    };
    TaskResult {
        node_id: node.into(),
        name: name.into(),
        state,
        phase: phase.into(),
        output: String::from_utf8_lossy(&output.output).into_owned(),
        exit_code,
        truncated: output.truncated,
    }
}
fn invoke(
    executor: &mut dyn Executor,
    script: &str,
    timeout: u32,
    cancel: &Cancellation,
) -> Result<ExecOutput> {
    cancel.check()?;
    executor.execute(script, timeout, cancel)
}
fn failed(state: TaskState) -> bool {
    matches!(
        state,
        TaskState::Failed | TaskState::Stopped | TaskState::OutcomeUnknown
    )
}
fn run_task(
    executor: &mut dyn Executor,
    task: &ShellTask,
    node: &str,
    mode: RunMode,
    cancel: &Cancellation,
    emit: &mut dyn FnMut(TaskResult) -> Result<()>,
) -> Result<TaskState> {
    // A separate invocation is a fresh check immediately before each apply attempt.
    let mut changed = false;
    for attempt in 0..=task.retries {
        if let Some(path) = &task.creates {
            let out = invoke(
                executor,
                &format!("test -e {}", quote(path)),
                task.timeout_s,
                cancel,
            )?;
            match out.end {
                ExecEnd::Exited(0) => {
                    let mut value = result(node, &task.name, "creates", out, false);
                    value.state = TaskState::Skipped;
                    emit(value)?;
                    return Ok(TaskState::Skipped);
                }
                ExecEnd::Exited(1) => (),
                _ => {
                    let value = result(node, &task.name, "creates", out, false);
                    let state = value.state;
                    emit(value)?;
                    return Ok(state);
                }
            }
        }
        if let Some(unless) = &task.unless {
            let out = invoke(executor, unless, task.timeout_s, cancel)?;
            match out.end {
                ExecEnd::Exited(0) => {
                    let mut value = result(node, &task.name, "unless", out, false);
                    value.state = TaskState::Skipped;
                    emit(value)?;
                    return Ok(TaskState::Skipped);
                }
                ExecEnd::Exited(1) => (),
                _ => {
                    let value = result(node, &task.name, "unless", out, false);
                    let state = value.state;
                    emit(value)?;
                    return Ok(state);
                }
            }
        }
        let out = invoke(executor, &task.check.command, task.timeout_s, cancel)?;
        let drift = out.end == ExecEnd::Exited(10);
        let mut value = result(node, &task.name, "check", out, false);
        if drift {
            value.state = TaskState::Drift;
        }
        let state = value.state;
        emit(value)?;
        if !drift || mode == RunMode::Check {
            return Ok(if state == TaskState::Ok && changed {
                TaskState::Changed
            } else {
                state
            });
        }
        changed |= task.changed_when.as_deref() != Some("false");
        let out = invoke(executor, &task.shell.command, task.timeout_s, cancel)?;
        let mut value = result(
            node,
            &task.name,
            "apply",
            out,
            task.changed_when.as_deref() != Some("false"),
        );
        let apply_state = value.state;
        if apply_state == TaskState::Failed && attempt < task.retries {
            value.state = TaskState::Retrying;
        }
        emit(value)?;
        if failed(apply_state) {
            if apply_state == TaskState::Failed && attempt < task.retries {
                continue;
            }
            return Ok(apply_state);
        }
        let out = invoke(executor, &task.check.command, task.timeout_s, cancel)?;
        let mut value = result(node, &task.name, "verify", out, false);
        let verify_state = value.state;
        if verify_state == TaskState::Failed && attempt < task.retries {
            value.state = TaskState::Retrying;
        }
        emit(value)?;
        if verify_state == TaskState::Ok {
            return Ok(apply_state);
        }
        if verify_state != TaskState::Failed || attempt == task.retries {
            return Ok(verify_state);
        }
    }
    unreachable!()
}
pub fn run_playbook(
    executor: &mut dyn Executor,
    book: &Playbook,
    node: &str,
    mode: RunMode,
    cancel: &Cancellation,
    emit: &mut dyn FnMut(TaskResult) -> Result<()>,
) -> Result<()> {
    let mut notified = BTreeSet::new();
    for task in &book.tasks {
        let state = run_task(executor, task, node, mode, cancel, emit)?;
        if failed(state) {
            return Ok(());
        }
        if state == TaskState::Changed {
            notified.extend(task.notify.iter().cloned());
        }
    }
    // Only successfully completed nodes reach handlers. Deduplicate in stable order.
    for name in notified {
        let mut handler = book.handlers[&name].clone();
        handler.name = format!("Handler: {}", handler.name);
        if failed(run_task(executor, &handler, node, mode, cancel, emit)?) {
            break;
        }
    }
    Ok(())
}
pub fn run_script(
    executor: &mut dyn Executor,
    document: &AutomationDocument,
    input: &str,
    node: &str,
    cancel: &Cancellation,
    emit: &mut dyn FnMut(TaskResult) -> Result<()>,
) -> Result<()> {
    let out = invoke(
        executor,
        &script_payload(document.kind, &document.source, input)?,
        600,
        cancel,
    )?;
    emit(result(node, &document.id, "script", out, true))
}
/// Literal JSON string inputs are masked. Arbitrary program output is sensitive;
/// this is not a general secret detector and never permits passing SSH credentials.
pub fn redact(output: &str, input: &str) -> String {
    fn strings(value: &serde_json::Value, values: &mut Vec<String>) {
        match value {
            serde_json::Value::String(s) if !s.is_empty() => values.push(s.clone()),
            serde_json::Value::Array(items) => items.iter().for_each(|v| strings(v, values)),
            serde_json::Value::Object(items) => items.values().for_each(|v| strings(v, values)),
            _ => (),
        }
    }
    let mut values = vec![];
    if let Ok(value) = serde_json::from_str(input) {
        strings(&value, &mut values);
    }
    values.sort_by_key(|s| std::cmp::Reverse(s.len()));
    let mut text = output.to_owned();
    for value in values {
        text = text.replace(&value, "[input]");
    }
    // Strip control sequences rather than rendering remote terminal instructions in reports.
    text.chars()
        .filter(|c| !c.is_control() || matches!(c, '\n' | '\t' | '\r'))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::VecDeque;
    struct Fake {
        ends: VecDeque<ExecEnd>,
        commands: Vec<String>,
    }
    impl Executor for Fake {
        fn execute(&mut self, script: &str, _: u32, _: &Cancellation) -> Result<ExecOutput> {
            self.commands.push(script.into());
            Ok(ExecOutput {
                end: self.ends.pop_front().expect("unexpected invocation"),
                output: vec![],
                truncated: false,
            })
        }
    }
    fn task() -> ShellTask {
        ShellTask {
            name: "Ensure".into(),
            check: Command {
                command: "check".into(),
            },
            shell: Command {
                command: "apply".into(),
            },
            timeout_s: 5,
            retries: 0,
            creates: None,
            unless: None,
            changed_when: None,
            notify: vec![],
        }
    }
    fn execute(
        tasks: Vec<ShellTask>,
        handlers: std::collections::BTreeMap<String, ShellTask>,
        mode: RunMode,
        codes: Vec<ExecEnd>,
    ) -> (Fake, Vec<TaskResult>) {
        let mut fake = Fake {
            ends: codes.into(),
            commands: vec![],
        };
        let mut results = vec![];
        run_playbook(
            &mut fake,
            &Playbook {
                schema: 1,
                name: "test".into(),
                targets: vec![],
                tasks,
                handlers,
            },
            "node",
            mode,
            &Cancellation::default(),
            &mut |value| {
                results.push(value);
                Ok(())
            },
        )
        .unwrap();
        assert!(fake.ends.is_empty());
        (fake, results)
    }
    #[test]
    fn check_never_applies_and_failed_checks_stop_the_node() {
        let (fake, results) = execute(
            vec![task()],
            Default::default(),
            RunMode::Check,
            vec![ExecEnd::Exited(10)],
        );
        assert_eq!(fake.commands, ["check"]);
        assert_eq!(results[0].state, TaskState::Drift);
        let (fake, results) = execute(
            vec![task(), task()],
            Default::default(),
            RunMode::Apply,
            vec![ExecEnd::Exited(7)],
        );
        assert_eq!(fake.commands, ["check"]);
        assert_eq!(results[0].state, TaskState::Failed);
    }
    #[test]
    fn apply_verifies_deduplicates_handlers_and_reports_retry_recovery() {
        let mut first = task();
        first.notify = vec!["restart".into()];
        first.retries = 1;
        let mut second = task();
        second.notify = first.notify.clone();
        let mut handlers = std::collections::BTreeMap::new();
        handlers.insert("restart".into(), task());
        let (fake, results) = execute(
            vec![first, second],
            handlers,
            RunMode::Apply,
            [10, 1, 10, 0, 0, 10, 0, 0, 10, 0, 0]
                .into_iter()
                .map(ExecEnd::Exited)
                .collect(),
        );
        assert_eq!(fake.commands.iter().filter(|s| *s == "apply").count(), 4);
        assert!(results.iter().any(|r| r.state == TaskState::Retrying));
        assert!(!results.iter().any(|r| r.state == TaskState::Failed));
        assert_eq!(
            results
                .iter()
                .filter(|r| r.name.starts_with("Handler:") && r.phase == "apply")
                .count(),
            1
        );
    }
    #[test]
    fn guards_skip_and_interruption_never_retries_or_runs_handlers() {
        let mut guarded = task();
        guarded.creates = Some("/tmp/a'b".into());
        let (fake, results) = execute(
            vec![guarded],
            Default::default(),
            RunMode::Apply,
            vec![ExecEnd::Exited(0)],
        );
        assert_eq!(fake.commands, ["test -e '/tmp/a'\\''b'"]);
        assert_eq!(results[0].state, TaskState::Skipped);
        for end in [ExecEnd::Cancelled, ExecEnd::TimedOut, ExecEnd::Disconnected] {
            let mut retry = task();
            retry.retries = 3;
            let (fake, results) = execute(
                vec![retry, task()],
                Default::default(),
                RunMode::Apply,
                vec![ExecEnd::Exited(10), end],
            );
            assert_eq!(fake.commands, ["check", "apply"]);
            assert_eq!(results.last().unwrap().state, TaskState::OutcomeUnknown);
        }
    }
    #[test]
    fn inputs_are_objects_quoted_and_masked_capture_stays_bounded() {
        assert!(!input_valid("[]"));
        assert!(!input_valid("null"));
        let input = r#"{"value":"a'b $(touch /tmp/nope)","nested":["private"]}"#;
        let payload =
            script_payload(AutomationKind::Shell, "printf '%s' \"$GHOSH_INPUT\"", input).unwrap();
        assert!(payload.contains("a'\\''b $(touch /tmp/nope)"));
        assert_eq!(
            redact("private\u{0} a'b $(touch /tmp/nope)", input),
            "[input] [input]"
        );
        let mut bytes = vec![];
        let mut truncated = false;
        bounded_append(&mut bytes, &vec![1; OUTPUT_LIMIT + 500], &mut truncated);
        assert!(truncated);
        assert_eq!(bytes.len(), OUTPUT_LIMIT);
    }
}
