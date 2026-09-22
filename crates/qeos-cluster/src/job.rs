//! P7.6-02 — Distributed job model.
//!
//! A [`DistributedJob`] tracks the full lifecycle of a job executed across the
//! cluster: states, node assignment, backend, timestamps, result/error and
//! telemetry. State transitions are validated.

use serde::{Deserialize, Serialize};

use crate::identity::NodeId;
use crate::resources::ResourceRequirements;

/// Id of a distributed job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct DistributedJobId(pub u64);

/// Job lifecycle state (P7.6-02).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobState {
    Created,
    Queued,
    Assigned,
    Running,
    Completed,
    Failed,
    Cancelled,
    Timeout,
    Retrying,
}

impl JobState {
    pub fn can_transition_to(self, next: JobState) -> bool {
        use JobState::*;
        match self {
            Created => matches!(next, Queued | Failed | Cancelled),
            Queued => matches!(next, Assigned | Cancelled | Failed | Timeout | Retrying),
            Assigned => matches!(next, Running | Cancelled | Failed | Timeout | Retrying),
            Running => matches!(next, Completed | Failed | Cancelled | Timeout | Retrying),
            // A failed job may be retried (rescheduled), or remain ended.
            Failed => matches!(next, Retrying),
            Retrying => matches!(next, Queued | Assigned | Failed | Cancelled),
            Completed | Cancelled | Timeout => false,
        }
    }

    /// Whether the job is finished (terminal state).
    pub fn is_terminal(self) -> bool {
        matches!(
            self,
            JobState::Completed | JobState::Failed | JobState::Cancelled | JobState::Timeout
        )
    }
}

/// A distributed job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistributedJob {
    pub job_id: DistributedJobId,
    pub owner_id: String,
    pub state: JobState,
    pub requirements: ResourceRequirements,
    /// Node to which the job is assigned.
    pub node_id: Option<NodeId>,
    /// Backend selected (e.g. "gpu-sim", "qpu-sim", "cpu").
    pub backend: Option<String>,
    pub created_at_ms: u64,
    pub started_at_ms: Option<u64>,
    pub finished_at_ms: Option<u64>,
    pub result: Option<String>,
    pub error: Option<String>,
    /// Number of retries attempted.
    pub retries: u32,
    /// Priority (higher = scheduled first).
    pub priority: u32,
}

impl DistributedJob {
    pub fn new(job_id: u64, owner_id: &str, requirements: ResourceRequirements) -> Self {
        Self {
            job_id: DistributedJobId(job_id),
            owner_id: owner_id.to_string(),
            state: JobState::Created,
            requirements,
            node_id: None,
            backend: None,
            created_at_ms: 0,
            started_at_ms: None,
            finished_at_ms: None,
            result: None,
            error: None,
            retries: 0,
            priority: 0,
        }
    }

    /// Attempt a validated state transition.
    pub fn transition(&mut self, next: JobState) {
        if self.state.can_transition_to(next) {
            self.state = next;
        }
    }

    /// Assign the job to a node + backend and record start time.
    pub fn assign(&mut self, node: &NodeId, backend: &str, now_ms: u64) {
        self.node_id = Some(node.clone());
        self.backend = Some(backend.to_string());
        self.started_at_ms = Some(now_ms);
        self.transition(JobState::Running);
    }

    /// Mark completed with a result.
    pub fn complete(&mut self, result: String, now_ms: u64) {
        self.finished_at_ms = Some(now_ms);
        self.result = Some(result);
        self.transition(JobState::Completed);
    }

    /// Mark failed with an error; allow one retry transition.
    pub fn fail(&mut self, error: String, retry: bool) {
        self.transition(JobState::Failed);
        if self.state == JobState::Failed {
            self.error = Some(error);
        }
        if retry && self.state == JobState::Failed {
            self.retries += 1;
            self.transition(JobState::Retrying);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn req() -> ResourceRequirements {
        ResourceRequirements {
            cpu_cores: 1,
            ..Default::default()
        }
    }

    #[test]
    fn lifecycle_passes_through_states() {
        let mut j = DistributedJob::new(1, "alice", req());
        j.transition(JobState::Queued);
        j.transition(JobState::Assigned);
        let node = NodeId("n1".into());
        j.assign(&node, "cpu", 1000);
        assert_eq!(j.state, JobState::Running);
        j.complete("ok".into(), 2000);
        assert_eq!(j.state, JobState::Completed);
        assert_eq!(j.node_id, Some(NodeId("n1".into())));
        assert!(j.state.is_terminal());
    }

    #[test]
    fn terminal_states_are_final() {
        let mut j = DistributedJob::new(1, "alice", req());
        j.transition(JobState::Cancelled);
        assert!(j.state.is_terminal());
        // Cannot leave a terminal state.
        j.transition(JobState::Running);
        assert_eq!(j.state, JobState::Cancelled);
    }

    #[test]
    fn failure_with_retry() {
        let mut j = DistributedJob::new(1, "alice", req());
        j.transition(JobState::Queued);
        j.transition(JobState::Assigned);
        j.transition(JobState::Running);
        j.fail("bad".into(), true);
        assert_eq!(j.state, JobState::Retrying);
        assert_eq!(j.retries, 1);
    }

    #[test]
    fn timeout_state() {
        let mut j = DistributedJob::new(1, "alice", req());
        j.transition(JobState::Queued);
        j.transition(JobState::Timeout);
        assert_eq!(j.state, JobState::Timeout);
        assert!(j.state.is_terminal());
    }

    #[test]
    fn cannot_jump_to_running_from_queued() {
        let mut j = DistributedJob::new(1, "alice", req());
        j.transition(JobState::Queued);
        // Queued -> Running is not allowed (must be Assigned).
        j.transition(JobState::Running);
        assert_eq!(j.state, JobState::Queued);
    }
}
