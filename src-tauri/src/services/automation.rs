use super::*;
use ghosh_core::{automation::*, workspace::automation::*};

impl Services {
    pub fn automations(&self, request: RevisionRequest) -> Result<Vec<AutomationSummary>> {
        self.active
            .lock()
            .map_err(poisoned)?
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .workspace
            .automations(Principal::LocalUi, &request)
    }
    pub fn automation_read(&self, request: AutomationReadRequest) -> Result<AutomationDocument> {
        self.active
            .lock()
            .map_err(poisoned)?
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .workspace
            .automation_read(Principal::LocalUi, &request)
    }
    pub fn automation_preview(&self, request: AutomationMutation) -> Result<AutomationPreview> {
        self.active
            .lock()
            .map_err(poisoned)?
            .as_ref()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .workspace
            .automation_preview(Principal::LocalUi, &request)
    }
    pub fn automation_commit(&self, request: AutomationCommit) -> Result<WorkspaceSnapshot> {
        let mut guard = self.active.lock().map_err(poisoned)?;
        self.invalidate_reviews(&mut guard)?;
        guard
            .as_mut()
            .ok_or_else(|| AppError::new(ErrorCode::Denied))?
            .workspace
            .automation_commit(Principal::LocalUi, &request, &Cancellation::default())
    }
}

/// Import is inert; normal workspace preview/commit still controls publication.
pub(crate) fn import_playbook(path: &std::path::Path) -> Result<AutomationDocument> {
    use std::io::Read;
    let metadata = std::fs::symlink_metadata(path)?;
    if !metadata.is_file() || metadata.len() > MAX_SOURCE as u64 {
        return Err(AppError::new(ErrorCode::InvalidDocument));
    }
    let mut source = String::new();
    let mut options = std::fs::OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK);
    }
    let file = options.open(path)?;
    if !file.metadata()?.is_file() {
        return Err(AppError::new(ErrorCode::InvalidDocument));
    }
    file.take(MAX_SOURCE as u64 + 1)
        .read_to_string(&mut source)?;
    parse_notebook(&source)?;
    let stem = path
        .file_stem()
        .and_then(|s| s.to_str())
        .unwrap_or("imported-playbook");
    let id: String = stem
        .trim_end_matches(".steps")
        .to_ascii_lowercase()
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '_' || c == '-' {
                c
            } else {
                '-'
            }
        })
        .skip_while(|c| !c.is_ascii_alphanumeric())
        .take(64)
        .collect();
    Ok(AutomationDocument {
        id: if id.is_empty() {
            "imported-playbook".into()
        } else {
            id
        },
        kind: AutomationKind::Notebook,
        source,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn import_preserves_source_without_execution_and_rejects_unsafe_inputs() {
        let tmp = tempfile::tempdir().unwrap();
        let source = include_str!("../../../examples/playbooks/steps-success.steps.yml");
        let path = tmp.path().join("My Playbook.steps.yml");
        std::fs::write(&path, source).unwrap();
        let imported = import_playbook(&path).unwrap();
        assert_eq!(imported.id, "my-playbook");
        assert_eq!(imported.kind, AutomationKind::Notebook);
        assert_eq!(imported.source, source);
        std::fs::write(&path, "not a Playbook").unwrap();
        assert!(import_playbook(&path).is_err());
        std::fs::write(&path, " ".repeat(MAX_SOURCE + 1)).unwrap();
        assert!(import_playbook(&path).is_err());
        assert!(import_playbook(tmp.path()).is_err());
        #[cfg(unix)]
        {
            let link = tmp.path().join("link.steps.yml");
            std::os::unix::fs::symlink(&path, &link).unwrap();
            assert!(import_playbook(&link).is_err());
        }
    }
}
