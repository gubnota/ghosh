//! Local-human terminal transport; output is untrusted bytes, never commands.
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Clone, Copy, Debug, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct TerminalSize {
    pub rows: u16,
    pub cols: u16,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct TerminalSession {
    pub id: String,
    pub target: String,
    pub shell: String,
    pub cwd: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, TS)]
pub struct TerminalPacket {
    // Bounded monotonic sequence; repeated polls replay until acknowledged.
    pub sequence: u32,
    pub bytes: Vec<u8>,
    pub exited: bool,
    pub exit_code: Option<u32>,
    pub failed: bool,
}

/// Native adapter port. Poll/input/resize must be bounded and never wait for
/// remote output. Dropping the adapter closes its owned transport.
pub trait Pty: Send {
    fn input(&mut self, bytes: Vec<u8>) -> crate::contracts::Result<()>;
    fn resize(&mut self, size: TerminalSize) -> crate::contracts::Result<()>;
    fn poll(&mut self) -> crate::contracts::Result<PtyRead>;
}
pub struct PtyRead {
    pub bytes: Vec<u8>,
    pub exited: bool,
    pub exit_code: Option<u32>,
    pub failed: bool,
}
impl TerminalSize {
    pub fn validate(self) -> crate::contracts::Result<Self> {
        if !(2..=500).contains(&self.rows) || !(2..=1000).contains(&self.cols) {
            return Err(crate::contracts::AppError::new(
                crate::contracts::ErrorCode::InvalidDocument,
            ));
        }
        Ok(self)
    }
}
