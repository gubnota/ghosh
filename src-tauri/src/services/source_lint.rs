//! Syntax-only local diagnostics. Sources are stdin data, never executed or
//! imported. No project configuration, dependencies or remote node is accessed.
use ghosh_core::{automation::*, contracts::*, runs::ExecEnd};
use std::process::Command;

pub(crate) fn lint_source(document: AutomationDocument) -> Result<SourceDiagnostics> {
    if document.source.len() > MAX_SOURCE || document.source.contains('\0') {
        return Err(AppError::new(ErrorCode::InvalidDocument));
    }
    let mut diagnostics = vec![];
    let engine = match document.kind {
        AutomationKind::Playbook | AutomationKind::Ansible => {
            let validation = validate(&document);
            diagnostics.extend(
                validation
                    .issues
                    .into_iter()
                    .map(|message| SourceDiagnostic {
                        line: 1,
                        column: 1,
                        message,
                    }),
            );
            "Ghosh YAML/schema validation (Ansible module analysis requires its controller)".into()
        }
        AutomationKind::Deno => "Bundled TypeScript language service".into(),
        kind => {
            let mut command = if kind == AutomationKind::Shell {
                let mut c = Command::new("/bin/sh");
                c.arg("-n");
                c
            } else {
                let mut c = Command::new("python3");
                c.args(["-I", "-S", "-c", "import ast,json,sys\ntry:\n ast.parse(sys.stdin.read())\nexcept SyntaxError as e:\n print(json.dumps({'line':e.lineno or 1,'column':e.offset or 1,'message':e.msg}))\n sys.exit(1)"]);
                c
            };
            command.current_dir(std::env::temp_dir());
            match super::local_exec::run_command(command, &document.source, 3, &Cancellation::default(), &mut |_| Ok(())) {
                Ok(out) if out.end == ExecEnd::Exited(0) => (),
                Ok(out) => {
                    let text = String::from_utf8_lossy(&out.output);
                    if let Ok(diagnostic) = serde_json::from_str::<SourceDiagnostic>(&text) {
                        diagnostics.push(diagnostic);
                    } else {
                        diagnostics.push(SourceDiagnostic { line: 1, column: 1, message: if text.trim().is_empty() { "Syntax checker did not finish. Check the local interpreter installation.".into() } else { text.chars().take(2000).collect() } });
                    }
                }
                Err(_) => return Ok(SourceDiagnostics { engine: "Syntax checker unavailable: install the local interpreter to enable diagnostics. Source was not executed.".into(), diagnostics }),
            }
            if kind == AutomationKind::Shell {
                "POSIX sh -n · syntax only; no ShellCheck/style analysis".into()
            } else {
                "Python ast.parse · syntax only; no imports or execution".into()
            }
        }
    };
    Ok(SourceDiagnostics {
        engine,
        diagnostics,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn syntax_checks_do_not_execute_source() {
        let temp = tempfile::tempdir().unwrap();
        let path = temp.path().join("must-not-exist");
        let source = format!("touch '{}'\n", path.display());
        let result = lint_source(AutomationDocument {
            id: "test".into(),
            kind: AutomationKind::Shell,
            source,
        })
        .unwrap();
        assert!(result.diagnostics.is_empty());
        assert!(!path.exists());
        let result = lint_source(AutomationDocument {
            id: "test".into(),
            kind: AutomationKind::Shell,
            source: "if then\n".into(),
        })
        .unwrap();
        assert!(!result.diagnostics.is_empty());
    }
}
