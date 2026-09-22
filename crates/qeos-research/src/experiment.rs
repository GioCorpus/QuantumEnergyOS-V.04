//! P7.8-00/01 — Experiment model and lifecycle.
//!
//! An [`Experiment`] captures everything needed to reproduce a research run:
//! identity, timestamp, user/software/runtime versions, backend/device,
//! parameters/configuration, seed, input, output, telemetry, artifacts and
//! status. Lifecycle transitions are validated.

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

/// Unique id of an experiment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct ExperimentId(pub u64);

/// Experiment lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum ExperimentStatus {
    #[default]
    Created,
    Validated,
    Queued,
    Running,
    Measuring,
    Completed,
    Failed,
    Cancelled,
    Archived,
}

impl ExperimentStatus {
    pub fn can_transition_to(self, next: ExperimentStatus) -> bool {
        use ExperimentStatus::*;
        match self {
            Created => matches!(next, Validated | Failed | Cancelled),
            Validated => matches!(next, Queued | Failed | Cancelled),
            Queued => matches!(next, Running | Failed | Cancelled),
            Running => matches!(next, Measuring | Failed | Cancelled),
            Measuring => matches!(next, Completed | Failed | Cancelled),
            Completed => matches!(next, Archived),
            Failed | Cancelled => matches!(next, Archived),
            Archived => false,
        }
    }
}

/// A research experiment.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Experiment {
    pub id: ExperimentId,
    pub timestamp: String,
    pub user: String,
    pub software_version: String,
    pub runtime_version: String,
    pub backend: String,
    pub device: Option<String>,
    pub parameters: Json,
    pub configuration: Json,
    pub seed: u64,
    pub input: Option<String>,
    pub output: Option<String>,
    pub telemetry_summary: Option<String>,
    pub artifact_ids: Vec<String>,
    pub status: ExperimentStatus,
    pub dataset_version: Option<String>,
}

impl Experiment {
    pub fn new(id: u64, user: &str, software_version: &str) -> Self {
        Self {
            id: ExperimentId(id),
            timestamp: chrono::Utc::now().to_rfc3339(),
            user: user.to_string(),
            software_version: software_version.to_string(),
            runtime_version: String::new(),
            backend: "unknown".into(),
            device: None,
            parameters: Json::Null,
            configuration: Json::Null,
            seed: 0,
            input: None,
            output: None,
            telemetry_summary: None,
            artifact_ids: Vec::new(),
            status: ExperimentStatus::Created,
            dataset_version: None,
        }
    }

    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = seed;
        self
    }

    pub fn with_params(mut self, params: Json) -> Self {
        self.parameters = params;
        self
    }

    pub fn with_backend(mut self, backend: &str) -> Self {
        self.backend = backend.to_string();
        self
    }

    /// Attempt a validated status transition.
    pub fn transition(&mut self, next: ExperimentStatus) -> bool {
        if self.status.can_transition_to(next) {
            self.status = next;
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_runs_to_completion() {
        let mut e = Experiment::new(1, "alice", "0.5.0");
        assert!(e.transition(ExperimentStatus::Validated));
        assert!(e.transition(ExperimentStatus::Queued));
        assert!(e.transition(ExperimentStatus::Running));
        assert!(e.transition(ExperimentStatus::Measuring));
        assert!(e.transition(ExperimentStatus::Completed));
        assert!(e.transition(ExperimentStatus::Archived));
    }

    #[test]
    fn invalid_transition_rejected() {
        let mut e = Experiment::new(1, "alice", "0.5.0");
        // Cannot jump straight to Running.
        assert!(!e.transition(ExperimentStatus::Running));
        assert_eq!(e.status, ExperimentStatus::Created);
    }

    #[test]
    fn cancels_from_queue() {
        let mut e = Experiment::new(1, "alice", "0.5.0");
        e.transition(ExperimentStatus::Validated);
        e.transition(ExperimentStatus::Queued);
        assert!(e.transition(ExperimentStatus::Cancelled));
    }

    #[test]
    fn archived_is_terminal() {
        let mut e = Experiment::new(1, "alice", "0.5.0");
        e.transition(ExperimentStatus::Validated);
        e.transition(ExperimentStatus::Queued);
        e.transition(ExperimentStatus::Running);
        e.transition(ExperimentStatus::Completed);
        e.transition(ExperimentStatus::Archived);
        assert!(!e.transition(ExperimentStatus::Validated));
    }
}
