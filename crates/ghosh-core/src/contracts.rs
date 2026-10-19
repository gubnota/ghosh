use serde::{Deserialize, Serialize};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use ts_rs::TS;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ErrorCode {
    InvalidDocument,
    InvalidPath,
    MissingReference,
    UnsupportedSchema,
    Conflict,
    Busy,
    Capacity,
    Io,
    Denied,
    Cancelled,
    SecretUnavailable,
    KeyUnavailable,
    ConnectionFailed,
    HostKeyChanged,
    AuthenticationFailed,
    UnsupportedConnection,
    Internal,
}

/// Messages are static: parsing errors and OS errors can contain document secrets.
#[derive(Debug, Clone, Serialize, Deserialize, TS, thiserror::Error)]
#[error("{message}")]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
}
pub type Result<T> = std::result::Result<T, AppError>;
impl AppError {
    pub fn new(code: ErrorCode) -> Self {
        let message = match code {
            ErrorCode::InvalidDocument => {
                "The workspace contains an invalid or unsupported field. Check its configuration locally."
            }
            ErrorCode::InvalidPath => {
                "Choose a local .ghosh package with safe relative configuration paths. Symlinks are not supported."
            }
            ErrorCode::MissingReference => {
                "A configured reference is missing, duplicated, or cyclic."
            }
            ErrorCode::UnsupportedSchema => "This workspace uses a newer schema and is read-only.",
            ErrorCode::Conflict => {
                "Files changed on disk. Reload before saving; your edits have not been written."
            }
            ErrorCode::Busy => {
                "This workspace is already open for writing in another app instance."
            }
            ErrorCode::Capacity => {
                "Too many active operations or approval requests. Wait for one to finish."
            }
            ErrorCode::Io => {
                "The filesystem operation failed. Check access, available space, and that the package still exists."
            }
            ErrorCode::Denied => "This caller or workspace is not authorized for the operation.",
            ErrorCode::Cancelled => "The operation was cancelled before commit.",
            ErrorCode::SecretUnavailable => {
                "The Keychain item is unavailable or access was denied."
            }
            ErrorCode::KeyUnavailable => {
                "The SSH key is missing, changed, unsafe, or unsupported. Select a regular private key file owned by you with permissions 0600 or 0400."
            }
            ErrorCode::Internal => "An internal operation failed. Reopen the workspace.",
            ErrorCode::ConnectionFailed => {
                "The SSH connection failed or timed out. Check the address, port and network route."
            }
            ErrorCode::HostKeyChanged => {
                "The server key differs from a known or reviewed fingerprint. Verify it independently before changing host trust."
            }
            ErrorCode::AuthenticationFailed => {
                "SSH authentication failed for the selected identity. No fallback identity was attempted."
            }
            ErrorCode::UnsupportedConnection => {
                "This connection requires a direct route. Legacy secret references and jump routes are not supported yet; select a file key, SSH agent, or password entered at connection time."
            }
        };
        Self {
            code,
            message: message.into(),
        }
    }
}
impl From<std::io::Error> for AppError {
    fn from(_: std::io::Error) -> Self {
        Self::new(ErrorCode::Io)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct NodeSummary {
    pub id: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub labels: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct WorkspaceSnapshot {
    pub id: String,
    pub name: String,
    pub path: String,
    pub revision: String,
    pub schema: u32,
    pub read_only: bool,
    pub trusted: bool,
    pub external_changed: bool,
    pub nodes: Vec<NodeSummary>,
    pub notices: Vec<String>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct RevisionRequest {
    pub workspace_id: String,
    pub revision: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct RenameRequest {
    pub workspace_id: String,
    pub revision: String,
    pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct TrustRequest {
    pub workspace_id: String,
    pub revision: String,
    pub trusted: bool,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct CreateRequest {
    pub parent: String,
    pub name: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct RecentWorkspace {
    pub name: String,
    pub path: String,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct WorkspaceEvent {
    pub workspace_id: String,
    pub revision: String,
}

#[derive(Clone, Default)]
pub struct Cancellation(Arc<AtomicBool>);
impl Cancellation {
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
    pub fn check(&self) -> Result<()> {
        if self.0.load(Ordering::Acquire) {
            Err(AppError::new(ErrorCode::Cancelled))
        } else {
            Ok(())
        }
    }
}
