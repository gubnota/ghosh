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
