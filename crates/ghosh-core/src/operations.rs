//! Bounded, secret-free live operation state. Durable run history is separate.
use crate::contracts::{AppError, Cancellation, ErrorCode, Result};
use serde::{Deserialize, Serialize};
use std::{
    collections::VecDeque,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use ts_rs::TS;

const CAPACITY: usize = 128;
const LOG_CAPACITY: usize = 32;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum JobKind {
    WorkspaceOpen,
    WorkspaceCreate,
    WorkspaceSave,
    WorkspaceReload,
    WorkspaceTrust,
    InventorySave,
    NodeConnect,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum JobState {
    Running,
    Cancelling,
    Succeeded,
    Failed,
    Cancelled,
}
impl JobState {
    fn terminal(self) -> bool {
        !matches!(self, Self::Running | Self::Cancelling)
    }
}

/// A closed vocabulary prevents paths, commands, credentials and remote output
/// from accidentally entering a shared diagnostic log.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, TS)]
#[serde(rename_all = "snake_case")]
pub enum JobStage {
    Started,
    CancellationRequested,
    Completed,
    Failed,
    Cancelled,
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct JobSnapshot {
    pub id: String,
    pub kind: JobKind,
    pub state: JobState,
    pub cancellable: bool,
    #[ts(type = "number")]
    pub started_at: u64,
    #[ts(type = "number | null")]
    pub finished_at: Option<u64>,
    pub error: Option<ErrorCode>,
    pub stages: Vec<JobStage>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct JobsSnapshot {
    pub sequence: u32,
    pub jobs: Vec<JobSnapshot>,
}
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
pub struct OperationsSnapshot {
    pub jobs: JobsSnapshot,
    pub approvals: crate::approvals::ApprovalsSnapshot,
}
struct Entry {
    snapshot: JobSnapshot,
    cancellation: Cancellation,
}
#[derive(Default)]
struct State {
    sequence: u32,
    entries: VecDeque<Entry>,
}
#[derive(Clone, Default)]
pub struct Jobs(Arc<Mutex<State>>);

pub(crate) fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u64::MAX as u128) as u64
}
fn poisoned<T>(_: std::sync::PoisonError<T>) -> AppError {
    AppError::new(ErrorCode::Internal)
}
impl State {
    fn changed(&mut self) {
        self.sequence = self.sequence.saturating_add(1);
    }
}
impl Jobs {
    pub fn begin(&self, kind: JobKind, cancellable: bool) -> Result<Job> {
        let mut state = self.0.lock().map_err(poisoned)?;
        if state.entries.len() == CAPACITY {
            let index = state
                .entries
                .iter()
                .position(|e| e.snapshot.state.terminal())
                .ok_or_else(|| AppError::new(ErrorCode::Capacity))?;
            state.entries.remove(index);
        }
        let id = uuid::Uuid::new_v4().to_string();
        let cancellation = Cancellation::default();
        state.entries.push_back(Entry {
            snapshot: JobSnapshot {
                id: id.clone(),
                kind,
                state: JobState::Running,
                cancellable,
                started_at: now_ms(),
                finished_at: None,
                error: None,
                stages: vec![JobStage::Started],
            },
            cancellation: cancellation.clone(),
        });
        state.changed();
        Ok(Job {
            jobs: self.clone(),
            id,
            cancellation,
            finished: false,
        })
    }
    pub fn snapshot(&self) -> Result<JobsSnapshot> {
        let state = self.0.lock().map_err(poisoned)?;
        Ok(JobsSnapshot {
            sequence: state.sequence,
            jobs: state
                .entries
                .iter()
                .rev()
                .map(|e| e.snapshot.clone())
                .collect(),
        })
    }
    pub fn cancel(&self, id: &str) -> Result<()> {
        let mut state = self.0.lock().map_err(poisoned)?;
        let entry = state
            .entries
            .iter_mut()
            .find(|e| e.snapshot.id == id)
            .ok_or_else(|| AppError::new(ErrorCode::MissingReference))?;
        if !entry.snapshot.cancellable || entry.snapshot.state.terminal() {
            return Err(AppError::new(ErrorCode::Denied));
        }
        if entry.snapshot.state == JobState::Cancelling {
            return Ok(());
        }
        entry.cancellation.cancel();
        entry.snapshot.state = JobState::Cancelling;
        entry.snapshot.stages.push(JobStage::CancellationRequested);
        entry.snapshot.stages.truncate(LOG_CAPACITY);
        state.changed();
        Ok(())
    }
    fn finish(&self, id: &str, error: Option<ErrorCode>) -> Result<()> {
        let mut state = self.0.lock().map_err(poisoned)?;
        let entry = state
            .entries
            .iter_mut()
            .find(|e| e.snapshot.id == id)
            .ok_or_else(|| AppError::new(ErrorCode::MissingReference))?;
        if entry.snapshot.state.terminal() {
            return Err(AppError::new(ErrorCode::Denied));
        }
        // A late cancellation cannot turn a committed successful operation into
        // a fabricated rollback. Only the worker reports Cancelled.
        entry.snapshot.state = match error {
            None => JobState::Succeeded,
            Some(ErrorCode::Cancelled) => JobState::Cancelled,
            _ => JobState::Failed,
        };
        entry.snapshot.error = error;
        entry.snapshot.finished_at = Some(now_ms());
        entry.snapshot.stages.push(match entry.snapshot.state {
            JobState::Succeeded => JobStage::Completed,
            JobState::Cancelled => JobStage::Cancelled,
            _ => JobStage::Failed,
        });
        entry.snapshot.stages.truncate(LOG_CAPACITY);
        state.changed();
        Ok(())
    }
}
/// Dropping an unfinished worker records failure instead of leaving a phantom
/// running job. Cancellation is cooperative and is never a rollback promise.
pub struct Job {
    jobs: Jobs,
    id: String,
    cancellation: Cancellation,
    finished: bool,
}
impl Job {
    pub fn id(&self) -> &str {
        &self.id
    }
    pub fn cancellation(&self) -> &Cancellation {
        &self.cancellation
    }
    pub fn finish<T>(mut self, result: &Result<T>) -> Result<()> {
        self.jobs
            .finish(&self.id, result.as_ref().err().map(|e| e.code))?;
        self.finished = true;
        Ok(())
    }
}
impl Drop for Job {
    fn drop(&mut self) {
        if !self.finished {
            let _ = self.jobs.finish(&self.id, Some(ErrorCode::Internal));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cancellation_is_a_request_and_never_claims_rollback() {
        let jobs = Jobs::default();
        let job = jobs.begin(JobKind::WorkspaceSave, true).unwrap();
        jobs.cancel(job.id()).unwrap();
        assert_eq!(
            job.cancellation().check().unwrap_err().code,
            ErrorCode::Cancelled
        );
        job.finish(&Ok(())).unwrap();
        assert_eq!(jobs.snapshot().unwrap().jobs[0].state, JobState::Succeeded);
        let job = jobs.begin(JobKind::WorkspaceOpen, true).unwrap();
        jobs.cancel(job.id()).unwrap();
        job.finish::<()>(&Err(AppError::new(ErrorCode::Cancelled)))
            .unwrap();
        assert_eq!(jobs.snapshot().unwrap().jobs[0].state, JobState::Cancelled);
    }
    #[test]
    fn bounds_do_not_evict_running_work_and_dropped_workers_fail() {
        let jobs = Jobs::default();
        let active: Vec<_> = (0..CAPACITY)
            .map(|_| jobs.begin(JobKind::WorkspaceOpen, false).unwrap())
            .collect();
        assert!(jobs.begin(JobKind::WorkspaceOpen, false).is_err());
        assert!(jobs.cancel(active[0].id()).is_err());
        let before = jobs.snapshot().unwrap().sequence;
        drop(active);
        assert!(
            jobs.snapshot()
                .unwrap()
                .jobs
                .iter()
                .all(|j| j.state == JobState::Failed)
        );
        let job = jobs.begin(JobKind::WorkspaceOpen, false).unwrap();
        job.finish(&Ok(())).unwrap();
        let snapshot = jobs.snapshot().unwrap();
        assert_eq!(snapshot.jobs.len(), CAPACITY);
        assert!(snapshot.sequence > before);
    }
}
