use crate::contracts::{AppError, ErrorCode, Result};

pub const REQUIRED_APPROVALS: &[&str] = &[
    "node.exec",
    "node.run",
    "profile.open_tab",
    "page.eval",
    "page.click",
    "page.type",
    "page.run",
    "cert.issue",
    "cert.sync",
    "recipe.apply",
    "script.run",
    "term.run",
];
pub fn validate_approvals(list: &[String]) -> Result<()> {
    if REQUIRED_APPROVALS
        .iter()
        .any(|required| !list.iter().any(|item| item == required))
    {
        return Err(AppError::new(ErrorCode::InvalidDocument));
    }
    Ok(())
}
pub fn needs_approval(tool: &str, check: bool) -> bool {
    REQUIRED_APPROVALS.contains(&tool) && !(tool == "node.run" && check)
}
#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Principal {
    LocalUi,
    NativeService,
    Script,
    Mcp,
}
pub fn authorize_edit(principal: Principal) -> Result<()> {
    if principal != Principal::LocalUi {
        return Err(AppError::new(ErrorCode::Denied));
    }
    Ok(())
}
/// Explicit native-service authorization. Workspace opening never constructs one.
pub struct SecretAccess {
    workspace_id: String,
}
impl SecretAccess {
    pub fn authorize(principal: Principal, workspace_id: &str, trusted: bool) -> Result<Self> {
        if principal != Principal::NativeService
            || !trusted
            || ulid::Ulid::from_string(workspace_id).is_err()
        {
            return Err(AppError::new(ErrorCode::Denied));
        }
        Ok(Self {
            workspace_id: workspace_id.into(),
        })
    }
    pub fn workspace_id(&self) -> &str {
        &self.workspace_id
    }
}
