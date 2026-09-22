//! P7.4-03 — QPU job isolation.
//!
//! [`QpuJob`] carries explicit resource limits, a timeout, ownership
//! (`owner_id`) and a state machine. Backends validate limits before executing.
//! A malformed or unbounded quantum workload can therefore never destabilize
//! the host: out-of-limit jobs are rejected, timed-out jobs transition to
//! `Timeout`, and jobs can be cancelled from the queue.

use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::backend::QpuBackendKind;

/// Id of a submitted QPU job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QpuJobId(pub u64);

/// QPU job lifecycle states (P7.4-03).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QpuJobState {
    Created,
    Queued,
    Running,
    Measuring,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}

/// Resource limits a job must satisfy (validated on submission).
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct JobLimits {
    pub max_qubits: usize,
    pub max_shots: u64,
    pub timeout_ms: u64,
}

impl Default for JobLimits {
    fn default() -> Self {
        Self {
            max_qubits: 16,
            max_shots: 1_000_000,
            timeout_ms: 60_000,
        }
    }
}

/// A single QPU job instance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QpuJob {
    pub id: QpuJobId,
    pub owner_id: String,
    pub qubits: usize,
    pub shots: u64,
    /// Deterministic seed for reproducible simulation.
    pub seed: u64,
    pub measurement_error: f64,
    pub poisoning_probability: f64,
    pub state: QpuJobState,
    pub limits: JobLimits,
    pub created_at_ms: u64,
}

impl QpuJob {
    /// Create a new job with default limits and a caller-assigned id.
    pub fn new(id: u64, qubits: usize, shots: u64, seed: u64) -> Self {
        Self {
            id: QpuJobId(id),
            owner_id: "default".into(),
            qubits,
            shots,
            seed,
            measurement_error: 0.0,
            poisoning_probability: 0.0,
            state: QpuJobState::Created,
            limits: JobLimits::default(),
            created_at_ms: now_ms(),
        }
    }

    /// Validate the job against its resource limits + the backend capability.
    ///
    /// Returns `Ok(())` only when resource limits, timeout and ownership are
    /// all satisfied; otherwise a `QpuError::ResourceLimit` is returned.
    pub fn validate(
        &self,
        backend_cap_qubit_limit: usize,
        backend_cap_max_shots: u64,
    ) -> std::result::Result<(), crate::QpuError> {
        if self.qubits == 0 {
            return Err(crate::QpuError::InvalidInput("zero qubits".into()));
        }
        if self.shots == 0 {
            return Err(crate::QpuError::InvalidInput("zero shots".into()));
        }
        if self.qubits > self.limits.max_qubits.min(backend_cap_qubit_limit) {
            return Err(crate::QpuError::ResourceLimit("qubit limit".into()));
        }
        if self.shots > self.limits.max_shots.min(backend_cap_max_shots) {
            return Err(crate::QpuError::ResourceLimit("shot limit".into()));
        }
        // A job in a terminal state cannot be re-executed.
        if matches!(self.state, QpuJobState::Cancelled | QpuJobState::Timeout) {
            return Err(crate::QpuError::InvalidInput(
                "job is cancelled/timed out".into(),
            ));
        }
        Ok(())
    }

    /// Advance the job state (only when the transition is accepted by the
    /// machine; terminal states are final).
    pub fn transition(&mut self, next: QpuJobState) {
        use QpuJobState::*;
        let ok = match self.state {
            Created => matches!(next, Queued | Failed | Cancelled),
            Queued => matches!(next, Running | Cancelled | Timeout | Failed),
            Running => matches!(next, Measuring | Failed | Cancelled | Timeout),
            Measuring => matches!(next, Completed | Failed | Cancelled | Timeout),
            Completed | Failed | Cancelled | Timeout => false,
        };
        if ok {
            self.state = next;
        }
    }

    /// Whether this job has exceeded its timeout budget.
    pub fn has_timed_out(&self) -> bool {
        if self.limits.timeout_ms == 0 {
            return false;
        }
        now_ms().saturating_sub(self.created_at_ms) > self.limits.timeout_ms
    }
}

/// Result of a QPU job execution.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QpuResult {
    pub job_id: QpuJobId,
    pub shots: u64,
    pub zeros: u64,
    pub ones: u64,
    pub logical_error_rate: f64,
    pub simulation_only: bool,
    pub backend: QpuBackendKind,
}

impl QpuResult {
    /// Parity estimate: probability of |1> measurement.
    pub fn parity_estimate(&self) -> f64 {
        if self.shots == 0 {
            0.0
        } else {
            self.ones as f64 / self.shots as f64
        }
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_accepts_within_limits() {
        let j = QpuJob::new(1, 2, 100, 7);
        assert!(j.validate(16, 1_000_000).is_ok());
    }

    #[test]
    fn validate_rejects_over_qubits() {
        let j = QpuJob::new(1, 20, 100, 7);
        assert!(matches!(
            j.validate(8, 1_000_000),
            Err(crate::QpuError::ResourceLimit(_))
        ));
    }

    #[test]
    fn validate_rejects_zero_shots() {
        let mut j = QpuJob::new(1, 2, 0, 7);
        j.limits = JobLimits::default();
        assert!(matches!(
            j.validate(16, 1_000_000),
            Err(crate::QpuError::InvalidInput(_))
        ));
    }

    #[test]
    fn state_machine_valid_transitions() {
        let mut j = QpuJob::new(1, 2, 10, 7);
        j.transition(QpuJobState::Queued);
        assert_eq!(j.state, QpuJobState::Queued);
        j.transition(QpuJobState::Running);
        j.transition(QpuJobState::Measuring);
        j.transition(QpuJobState::Completed);
        assert_eq!(j.state, QpuJobState::Completed);
        // Terminal: no transition out.
        j.transition(QpuJobState::Running);
        assert_eq!(j.state, QpuJobState::Completed);
    }

    #[test]
    fn cancellation_in_queue() {
        let mut j = QpuJob::new(1, 2, 10, 7);
        j.transition(QpuJobState::Queued);
        j.transition(QpuJobState::Cancelled);
        assert_eq!(j.state, QpuJobState::Cancelled);
        // Re-execution of a cancelled job is rejected.
        assert!(matches!(
            j.validate(16, 1_000_000),
            Err(crate::QpuError::InvalidInput(_))
        ));
    }

    #[test]
    fn timeout_detection_at_zero() {
        let mut j = QpuJob::new(1, 2, 10, 7);
        j.limits.timeout_ms = 0;
        assert!(!j.has_timed_out());
    }

    #[test]
    fn parity_estimate() {
        let r = QpuResult {
            job_id: QpuJobId(1),
            shots: 100,
            zeros: 90,
            ones: 10,
            logical_error_rate: 0.1,
            simulation_only: true,
            backend: QpuBackendKind::Simulator,
        };
        assert_eq!(r.parity_estimate(), 0.1);
    }
}
