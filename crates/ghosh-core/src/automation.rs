//! Inert automation documents. Validation never imports code or contacts a node.
use crate::contracts::{AppError, ErrorCode, Result};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use ts_rs::TS;

pub const MAX_SOURCE: usize = 256 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum AutomationKind {
    Playbook,
    Ansible,
    Shell,
    Python,
    Deno,
}
impl AutomationKind {
    pub fn extension(self) -> &'static str {
        match self {
            Self::Playbook => "yml",
            Self::Ansible => "ansible.yml",
            Self::Shell => "sh",
            Self::Python => "py",
            Self::Deno => "ts",
        }
    }
    pub fn is_script(self) -> bool {
        matches!(self, Self::Shell | Self::Python | Self::Deno)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct AutomationDocument {
    pub id: String,
    pub kind: AutomationKind,
    pub source: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct AutomationValidation {
    pub valid: bool,
    pub name: String,
    pub task_count: u32,
    pub issues: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct AutomationSummary {
    pub id: String,
    pub kind: AutomationKind,
    pub validation: AutomationValidation,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Command {
    pub command: String,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShellTask {
    pub name: String,
    pub check: Command,
    pub shell: Command,
    #[serde(default = "default_timeout")]
    pub timeout_s: u32,
    #[serde(default)]
    pub retries: u32,
    #[serde(default)]
    pub creates: Option<String>,
    #[serde(default)]
    pub unless: Option<String>,
    #[serde(default)]
    pub changed_when: Option<String>,
    #[serde(default)]
    pub notify: Vec<String>,
}
fn default_timeout() -> u32 {
    600
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Playbook {
    pub schema: u32,
    pub name: String,
    #[serde(default)]
    pub targets: Vec<String>,
    pub tasks: Vec<ShellTask>,
    #[serde(default)]
    pub handlers: BTreeMap<String, ShellTask>,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AnsibleProject {
    pub schema: u32,
    pub name: String,
    pub project: String,
    pub playbook: String,
    pub inventory: Option<String>,
}
fn invalid() -> AppError {
    AppError::new(ErrorCode::InvalidDocument)
}
fn command_valid(command: &str) -> bool {
    !command.trim().is_empty() && command.len() <= 64 * 1024 && !command.contains('\0')
}
pub fn parse_playbook(source: &str) -> Result<Playbook> {
    if source.len() > MAX_SOURCE {
        return Err(invalid());
    }
    let book: Playbook = serde_yaml_ng::from_str(source).map_err(|_| invalid())?;
    if book.schema != 1
        || book.name.trim().is_empty()
        || book.name.len() > 120
        || book.tasks.is_empty()
        || book.tasks.len() > 256
        || book.handlers.len() > 64
        || book.targets.len() > 256
        || book
            .targets
            .iter()
            .any(|s| s.len() > 256 || s.contains('\0'))
    {
        return Err(invalid());
    }
    for task in book.tasks.iter().chain(book.handlers.values()) {
        if task.name.trim().is_empty()
            || task.name.len() > 200
            || !command_valid(&task.check.command)
            || !command_valid(&task.shell.command)
            || !(1..=3600).contains(&task.timeout_s)
            || task.retries > 5
            || task
                .creates
                .as_ref()
                .is_some_and(|s| s.is_empty() || s.len() > 4096 || s.contains('\0'))
            || task.unless.as_ref().is_some_and(|s| !command_valid(s))
            || task
                .changed_when
                .as_deref()
                .is_some_and(|s| !["exit_code == 0", "false"].contains(&s))
            || task
                .notify
                .iter()
                .any(|name| !book.handlers.contains_key(name))
        {
            return Err(invalid());
        }
    }
    Ok(book)
}
pub fn parse_ansible(source: &str) -> Result<AnsibleProject> {
    if source.len() > MAX_SOURCE {
        return Err(invalid());
    }
    let project: AnsibleProject = serde_yaml_ng::from_str(source).map_err(|_| invalid())?;
    if project.schema != 1
        || project.name.trim().is_empty()
        || project.name.len() > 120
        || !std::path::Path::new(&project.project).is_absolute()
        || project.project.len() > 4096
        || project.project.contains('\0')
    {
        return Err(invalid());
    }
    super::workspace::model::relative_path(&project.playbook)?;
    if let Some(inventory) = &project.inventory {
        super::workspace::model::relative_path(inventory)?;
    }
    Ok(project)
}
pub fn validate(document: &AutomationDocument) -> AutomationValidation {
    let mut result = AutomationValidation {
        valid: true,
        name: document.id.clone(),
        task_count: 0,
        issues: vec![],
    };
    if !super::workspace::model::entity_id(&document.id)
        || document.source.len() > MAX_SOURCE
        || document.source.contains('\0')
        || document.source.trim().is_empty()
    {
        result
            .issues
            .push("Use a safe ID and a nonempty UTF-8 document of at most 256 KiB.".into());
    } else {
        match document.kind {
            AutomationKind::Playbook => match parse_playbook(&document.source) {
                Ok(book) => { result.name = book.name; result.task_count = book.tasks.len() as u32; }
                Err(_) => result.issues.push("Invalid native YAML. Each task needs name, check.command and shell.command; schema must be 1. Supported optional fields: timeout_s (1–3600), retries (0–5), creates, unless, notify and changed_when (exit_code == 0 or false). Handler names must resolve. Other native action types are not enabled yet.".into()),
            },
            AutomationKind::Ansible => match parse_ansible(&document.source) {
                Ok(project) => result.name = project.name,
                Err(_) => result.issues.push("Invalid Ansible project link. Use schema: 1, name, an absolute project folder, and relative playbook/inventory paths. Project files are not copied.".into()),
            },
            _ => result.task_count = 1,
        }
    }
    result.valid = result.issues.is_empty();
    result
}
