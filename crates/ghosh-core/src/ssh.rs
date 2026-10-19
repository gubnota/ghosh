//! Public metadata for explicit, authentication-only node tests.
use crate::{contracts::RevisionRequest, workspace::identity::IdentityPreview};
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct NodeConnectionRequest {
    pub context: RevisionRequest,
    pub node_id: String,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum HostTrust {
    Known,
    Unknown,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct HostKeyPreview {
    pub algorithm: String,
    pub fingerprint: String,
    pub trust: HostTrust,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct NodeConnectionPreview {
    pub ticket: String,
    pub node_id: String,
    pub host: String,
    pub port: u16,
    pub user: String,
    pub identity: IdentityPreview,
    pub host_key: HostKeyPreview,
    pub credential_prompt: Option<String>,
}
#[derive(Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct NodeConnectionCommit {
    pub ticket: String,
    pub allow_unknown: bool,
    pub credential: Option<String>,
}
#[derive(Serialize, Deserialize, TS)]
#[serde(deny_unknown_fields)]
pub struct NodeTerminalRequest {
    pub connection: NodeConnectionCommit,
    pub dimensions: crate::terminal::TerminalSize,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct NodeConnectionResult {
    pub node_id: String,
    pub host_key: HostKeyPreview,
    pub elapsed_ms: u32,
}

impl Drop for NodeConnectionCommit {
    fn drop(&mut self) {
        use zeroize::Zeroize;
        if let Some(value) = &mut self.credential {
            value.zeroize();
        }
    }
}
