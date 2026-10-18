//! Native-created exact-context approvals. There is no executor in this module.
use crate::{
    contracts::{AppError, ErrorCode, Result},
    operations::now_ms,
    policy::{Principal, REQUIRED_APPROVALS},
};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::{
    collections::VecDeque,
    time::{Duration, Instant},
};
use ts_rs::TS;

const CAPACITY: usize = 128;
const AUDIT_CAPACITY: usize = 256;
const TTL: Duration = Duration::from_secs(300);

pub fn content_digest(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}
fn digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}
fn identifier(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b"-_.:".contains(&b))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum Requester {
    LocalUi,
    Script,
    Mcp,
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ApprovalContext {
    pub workspace_id: String,
    pub revision: String,
    pub session_id: String,
}
impl ApprovalContext {
    pub fn new(workspace_id: String, revision: String) -> Self {
        Self {
            workspace_id,
            revision,
            session_id: uuid::Uuid::new_v4().to_string(),
        }
    }
}
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, TS)]
pub struct ApprovalBinding {
    pub context: ApprovalContext,
    pub requester: Requester,
    pub tool: String,
    pub source_digest: String,
    pub targets: Vec<String>,
    pub route_digest: String,
    pub identity_digest: String,
}
impl ApprovalBinding {
    fn validate(&self) -> Result<()> {
        if ulid::Ulid::from_string(&self.context.workspace_id).is_err()
            || !digest(&self.context.revision)
            || uuid::Uuid::parse_str(&self.context.session_id).is_err()
            || !REQUIRED_APPROVALS.contains(&self.tool.as_str())
            || !digest(&self.source_digest)
            || !digest(&self.route_digest)
            || !digest(&self.identity_digest)
            || self.targets.is_empty()
            || self.targets.len() > 1024
            || !self.targets.iter().all(|t| identifier(t))
            || !self.targets.windows(2).all(|pair| pair[0] < pair[1])
        {
            return Err(AppError::new(ErrorCode::InvalidDocument));
        }
        Ok(())
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalState {
    Pending,
    AllowedOnce,
    AllowedSession,
    Denied,
    Expired,
    Invalidated,
    Consumed,
}
impl ApprovalState {
    fn live(self) -> bool {
        matches!(
            self,
            Self::Pending | Self::AllowedOnce | Self::AllowedSession
        )
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalDecision {
    AllowOnce,
    AllowSession,
    Deny,
    Revoke,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ApprovalDecisionRequest {
    pub id: String,
    pub decision: ApprovalDecision,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ApprovalSnapshot {
    pub id: String,
    pub binding: ApprovalBinding,
    pub state: ApprovalState,
    #[ts(type = "number")]
    pub expires_at: u64,
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalAuditKind {
    Requested,
    AllowedOnce,
    AllowedSession,
    Denied,
    Revoked,
    Expired,
    Invalidated,
    Consumed,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ApprovalAudit {
    pub request_id: String,
    pub kind: ApprovalAuditKind,
    #[ts(type = "number")]
    pub at: u64,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct ApprovalsSnapshot {
    pub sequence: u32,
    pub requests: Vec<ApprovalSnapshot>,
    pub audit: Vec<ApprovalAudit>,
}
struct Entry {
    snapshot: ApprovalSnapshot,
    deadline: Instant,
}
#[derive(Default)]
pub struct Approvals {
    context: Option<ApprovalContext>,
    sequence: u32,
    entries: VecDeque<Entry>,
    audit: VecDeque<ApprovalAudit>,
}
impl Approvals {
    fn record(&mut self, id: &str, kind: ApprovalAuditKind) {
        self.sequence = self.sequence.saturating_add(1);
        if self.audit.len() == AUDIT_CAPACITY {
            self.audit.pop_front();
        }
        self.audit.push_back(ApprovalAudit {
            request_id: id.into(),
            kind,
            at: now_ms(),
        });
    }
    fn expire(&mut self) {
        let mut expired = Vec::new();
        for entry in &mut self.entries {
            if entry.snapshot.state.live() && Instant::now() >= entry.deadline {
                entry.snapshot.state = ApprovalState::Expired;
                expired.push(entry.snapshot.id.clone());
            }
        }
        for id in expired {
            self.record(&id, ApprovalAuditKind::Expired);
        }
    }
    /// Native composition supplies None when untrusted/changed/read-only. It must
    /// call this again after re-reading disk before decisions and consumption.
    pub fn synchronize(&mut self, context: Option<ApprovalContext>) {
        self.expire();
        if self.context == context {
            return;
        }
        self.context = context;
        let mut invalidated = Vec::new();
        for entry in &mut self.entries {
            if entry.snapshot.state.live() {
                entry.snapshot.state = ApprovalState::Invalidated;
                invalidated.push(entry.snapshot.id.clone());
            }
        }
        for id in invalidated {
            self.record(&id, ApprovalAuditKind::Invalidated);
        }
    }
    /// Native service API only; deliberately no request-creation desktop command.
    pub fn request(&mut self, principal: Principal, binding: ApprovalBinding) -> Result<String> {
        self.expire();
        binding.validate()?;
        if principal != Principal::NativeService || self.context.as_ref() != Some(&binding.context)
        {
            return Err(AppError::new(ErrorCode::Denied));
        }
        if self.entries.len() == CAPACITY {
            let index = self
                .entries
                .iter()
                .position(|e| !e.snapshot.state.live())
                .ok_or_else(|| AppError::new(ErrorCode::Capacity))?;
            self.entries.remove(index);
        }
        let id = uuid::Uuid::new_v4().to_string();
        self.entries.push_back(Entry {
            snapshot: ApprovalSnapshot {
                id: id.clone(),
                binding,
                state: ApprovalState::Pending,
                expires_at: now_ms() + TTL.as_millis() as u64,
            },
            deadline: Instant::now() + TTL,
        });
        self.record(&id, ApprovalAuditKind::Requested);
        Ok(id)
    }
    pub fn decide(&mut self, principal: Principal, request: ApprovalDecisionRequest) -> Result<()> {
        self.expire();
        if principal != Principal::LocalUi {
            return Err(AppError::new(ErrorCode::Denied));
        }
        let entry = self
            .entries
            .iter_mut()
            .find(|e| e.snapshot.id == request.id)
            .ok_or_else(|| AppError::new(ErrorCode::MissingReference))?;
        if self.context.as_ref() != Some(&entry.snapshot.binding.context)
            || (request.decision != ApprovalDecision::Revoke
                && entry.snapshot.state != ApprovalState::Pending)
            || (request.decision == ApprovalDecision::Revoke && !entry.snapshot.state.live())
        {
            return Err(AppError::new(ErrorCode::Denied));
        }
        let (state, audit) = match request.decision {
            ApprovalDecision::AllowOnce => {
                (ApprovalState::AllowedOnce, ApprovalAuditKind::AllowedOnce)
            }
            ApprovalDecision::AllowSession => (
                ApprovalState::AllowedSession,
                ApprovalAuditKind::AllowedSession,
            ),
            ApprovalDecision::Deny => (ApprovalState::Denied, ApprovalAuditKind::Denied),
            ApprovalDecision::Revoke => (ApprovalState::Invalidated, ApprovalAuditKind::Revoked),
        };
        entry.snapshot.state = state;
        self.record(&request.id, audit);
        Ok(())
    }
    /// Recomputed binding, never a UI-supplied digest. A worker must also validate
    /// its execution lease/source immediately before effects. No waiting here.
    pub fn consume(
        &mut self,
        principal: Principal,
        id: &str,
        binding: &ApprovalBinding,
    ) -> Result<()> {
        self.expire();
        if principal != Principal::NativeService || self.context.as_ref() != Some(&binding.context)
        {
            return Err(AppError::new(ErrorCode::Denied));
        }
        let entry = self
            .entries
            .iter_mut()
            .find(|e| e.snapshot.id == id)
            .ok_or_else(|| AppError::new(ErrorCode::MissingReference))?;
        if &entry.snapshot.binding != binding
            || !matches!(
                entry.snapshot.state,
                ApprovalState::AllowedOnce | ApprovalState::AllowedSession
            )
        {
            return Err(AppError::new(ErrorCode::Denied));
        }
        if entry.snapshot.state == ApprovalState::AllowedOnce {
            entry.snapshot.state = ApprovalState::Consumed;
        }
        self.record(id, ApprovalAuditKind::Consumed);
        Ok(())
    }
    pub fn snapshot(&mut self) -> ApprovalsSnapshot {
        self.expire();
        ApprovalsSnapshot {
            sequence: self.sequence,
            requests: self
                .entries
                .iter()
                .rev()
                .map(|e| e.snapshot.clone())
                .collect(),
            audit: self.audit.iter().rev().cloned().collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn monotonic_deadline_expires_pending_and_granted_requests() {
        let context = ApprovalContext {
            workspace_id: ulid::Ulid::new().to_string(),
            revision: content_digest(b"config"),
            session_id: uuid::Uuid::new_v4().to_string(),
        };
        for decision in [
            None,
            Some(ApprovalDecision::AllowOnce),
            Some(ApprovalDecision::AllowSession),
        ] {
            let mut service = Approvals::default();
            service.synchronize(Some(context.clone()));
            let binding = ApprovalBinding {
                context: context.clone(),
                requester: Requester::LocalUi,
                tool: "term.run".into(),
                source_digest: content_digest(b"source"),
                targets: vec!["local".into()],
                route_digest: content_digest(b"local"),
                identity_digest: content_digest(b"user"),
            };
            let id = service
                .request(Principal::NativeService, binding.clone())
                .unwrap();
            if let Some(decision) = decision {
                service
                    .decide(
                        Principal::LocalUi,
                        ApprovalDecisionRequest {
                            id: id.clone(),
                            decision,
                        },
                    )
                    .unwrap();
            }
            // Wall-clock display timestamps cannot extend authorization.
            service.entries[0].snapshot.expires_at = u64::MAX;
            service.entries[0].deadline = Instant::now() - Duration::from_secs(1);
            assert!(
                service
                    .consume(Principal::NativeService, &id, &binding)
                    .is_err()
            );
            assert_eq!(service.snapshot().requests[0].state, ApprovalState::Expired);
        }
    }
}
