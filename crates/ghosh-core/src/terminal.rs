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
