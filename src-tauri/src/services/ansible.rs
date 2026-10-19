//! Controller-side upstream Runner. No module interpretation lives in Ghosh.
use super::*;
use ghosh_core::{automation::*, runs::*, ssh::NodeConnectionRequest};
use std::process::Command;
use zeroize::{Zeroize, Zeroizing};

pub(super) fn run(
    service: &Services,
    request: &mut RunRequest,
    document: &AutomationDocument,
    report: &mut RunReport,
    cancel: &Cancellation,
) -> Result<()> {
    let python = request
        .controller_python
        .as_deref()
        .filter(|p| Path::new(p).is_absolute() && p.len() <= 4096 && !p.contains('\0'))
        .ok_or_else(|| AppError::new(ErrorCode::InvalidPath))?;
    let location = {
        let guard = service.active.lock().map_err(poisoned)?;
        guard
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .workspace
            .automation_location(&request.automation)?
    };
    let (project, playbook, inventory) = if document.kind == AutomationKind::Ansible {
        let link = parse_ansible(&document.source)?;
        let project = fs::canonicalize(link.project)?;
        let book = fs::canonicalize(project.join(link.playbook))?;
        let inventory = link
            .inventory
            .map(|p| fs::canonicalize(project.join(p)))
            .transpose()?;
        if !book.starts_with(&project)
            || inventory.as_ref().is_some_and(|p| !p.starts_with(&project))
        {
            return Err(AppError::new(ErrorCode::InvalidPath));
        }
        (project, book, inventory)
    } else {
        (
            location
                .parent()
                .ok_or_else(|| AppError::new(ErrorCode::InvalidPath))?
                .to_owned(),
            location,
            None,
        )
    };
    // Capture every short-lived reviewed identity before the first long play.
    // Secret JSON is zeroized after it is sent through the private stdin pipe.
    let mut prepared = vec![];
    for target in request.targets.iter_mut() {
        cancel.check()?;
        let mut cfg = serde_json::json!({"node": target.node_id, "host": "localhost", "user": "", "port": 22});
        if target.node_id != "@local" {
            let connection_request = NodeConnectionRequest {
                context: request.automation.context.clone(),
                node_id: target.node_id.clone(),
            };
            let connection = target
                .connection
                .take()
                .ok_or_else(|| AppError::new(ErrorCode::Denied))?;
            let mut identity =
                service.runner_identity(&connection_request, &connection.ticket, cancel)?;
            cfg["credential"] = serde_json::json!(connection.credential.as_deref());
            let details = service.with_connection(connection, cancel, |connection, node| {
                Ok(serde_json::json!({"host": node.host, "port": node.port, "user": node.user, "host_pin": connection.host.openssh_record(&node.host, node.port)}))
            });
            match details {
                Ok(details) => {
                    for (key, value) in details.as_object().unwrap() {
                        cfg[key] = value.clone();
                    }
                }
                Err(error) => {
                    wipe(&mut identity);
                    wipe(&mut cfg);
                    return Err(error);
                }
            }
            cfg["identity"] = identity;
        }
        cfg["project"] = serde_json::json!(project);
        cfg["playbook"] = serde_json::json!(playbook);
        cfg["inventory"] = serde_json::json!(inventory);
        cfg["check"] = serde_json::json!(request.mode == RunMode::Check);
        cfg["input"] = serde_json::from_str(&request.input)
            .map_err(|_| AppError::new(ErrorCode::InvalidDocument))?;
        let text = Zeroizing::new(
            serde_json::to_string(&cfg).map_err(|_| AppError::new(ErrorCode::Internal))?,
        );
        wipe(&mut cfg);
        prepared.push((target.node_id.clone(), text));
    }
    for (node_id, payload) in prepared {
        cancel.check()?;
        service.run_document(&request.automation, Some(&document.source))?;
        let private = tempfile::Builder::new()
            .prefix("runner-")
            .tempdir_in(&service.data_dir)?;
        let mut cfg: serde_json::Value =
            serde_json::from_str(&payload).map_err(|_| AppError::new(ErrorCode::Internal))?;
        cfg["private_dir"] = serde_json::json!(private.path());
        let stdin = Zeroizing::new(
            serde_json::to_string(&cfg).map_err(|_| AppError::new(ErrorCode::Internal))?,
        );
        wipe(&mut cfg);
        let mut command = Command::new(python);
        command
            .args(["-I", "-u", "-c", include_str!("ansible_bridge.py")])
            .current_dir(&project);
        let mut pending = vec![];
        let mut count = 0;
        let mut diagnostic = String::new();
        let output = super::local_exec::run_command(command, &stdin, 3650, cancel, &mut |bytes| {
            pending.extend_from_slice(bytes);
            if pending.len() > 256 * 1024 {
                return Err(AppError::new(ErrorCode::Capacity));
            }
            while let Some(end) = pending.iter().position(|b| *b == b'\n') {
                let line: Vec<_> = pending.drain(..=end).collect();
                if let Ok(value) = serde_json::from_slice::<TaskResult>(&line) {
                    count += 1;
                    super::runs::append(service, report, value, &request.input)?;
                } else if let Ok(value) = serde_json::from_slice::<serde_json::Value>(&line)
                    && let Some(error) = value.get("error").and_then(|v| v.as_str())
                {
                    diagnostic = error.chars().take(1000).collect();
                }
            }
            Ok(())
        })?;
        let success = output.end == ExecEnd::Exited(0);
        if !success || count == 0 {
            let state = match output.end {
                ExecEnd::Exited(_) => TaskState::Failed,
                _ => TaskState::OutcomeUnknown,
            };
            if diagnostic.is_empty() {
                diagnostic = "Ansible controller failed or was interrupted. Inspect tasks and controller dependencies; completed remote changes remain applied.".into();
            }
            super::runs::append(
                service,
                report,
                TaskResult {
                    node_id,
                    name: "Ansible controller".into(),
                    phase: "controller".into(),
                    state,
                    output: diagnostic,
                    exit_code: if let ExecEnd::Exited(code) = output.end {
                        Some(code)
                    } else {
                        None
                    },
                    truncated: false,
                },
                &request.input,
            )?;
        }
        cancel.check()?;
    }
    Ok(())
}
fn wipe(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::String(text) => text.zeroize(),
        serde_json::Value::Object(values) => {
            for value in values.values_mut() {
                wipe(value);
            }
        }
        serde_json::Value::Array(values) => {
            for value in values {
                wipe(value);
            }
        }
        _ => (),
    }
}
