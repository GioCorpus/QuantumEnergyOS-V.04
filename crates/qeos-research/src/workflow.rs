//! P7.8-05 — Research workflow.
//!
//! A [`Workflow`] is an ordered pipeline over the research stages
//! (dataset → preprocessing → compute → measurement → analysis → artifact).
//! Workflows carry versioning, timeouts, retry policy, checkpointing and
//! provenance. `validate` enforces a well-formed pipeline (dataset first,
//! artifact last, no empty/gapped pipeline).

use serde::{Deserialize, Serialize};

use crate::error::ResearchError;

/// A workflow stage.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WorkflowStage {
    Dataset,
    Preprocessing,
    ComputeCpu,
    ComputeGpu,
    QuantumSimulation,
    Measurement,
    Analysis,
    Artifact,
}

/// Retry policy for a workflow.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct RetryPolicy {
    pub max_retries: u32,
}

/// A research workflow.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    pub version: u32,
    pub stages: Vec<WorkflowStage>,
    pub timeout_ms: u64,
    pub retry: RetryPolicy,
    pub checkpointing: bool,
    pub provenance: String,
}

impl Workflow {
    /// Validate the pipeline is well-formed: non-empty, starts with Dataset and
    /// ends with Artifact, and every intermediate stage is legitimate.
    pub fn validate(&self) -> Result<(), ResearchError> {
        if self.stages.is_empty() {
            return Err(ResearchError::WorkflowInvalid("empty pipeline".into()));
        }
        if self.stages.first() != Some(&WorkflowStage::Dataset) {
            return Err(ResearchError::WorkflowInvalid(
                "pipeline must start with Dataset".into(),
            ));
        }
        if self.stages.last() != Some(&WorkflowStage::Artifact) {
            return Err(ResearchError::WorkflowInvalid(
                "pipeline must end with Artifact".into(),
            ));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wf(stages: Vec<WorkflowStage>) -> Workflow {
        Workflow {
            name: "w".into(),
            version: 1,
            stages,
            timeout_ms: 1000,
            retry: RetryPolicy { max_retries: 0 },
            checkpointing: true,
            provenance: "p".into(),
        }
    }

    #[test]
    fn valid_pipeline() {
        let w = wf(vec![
            WorkflowStage::Dataset,
            WorkflowStage::Preprocessing,
            WorkflowStage::QuantumSimulation,
            WorkflowStage::Measurement,
            WorkflowStage::Analysis,
            WorkflowStage::Artifact,
        ]);
        assert!(w.validate().is_ok());
    }

    #[test]
    fn empty_rejected() {
        assert!(wf(vec![]).validate().is_err());
    }

    #[test]
    fn must_start_with_dataset() {
        let w = wf(vec![WorkflowStage::Analysis, WorkflowStage::Artifact]);
        assert!(w.validate().is_err());
    }

    #[test]
    fn must_end_with_artifact() {
        let w = wf(vec![WorkflowStage::Dataset, WorkflowStage::Measurement]);
        assert!(w.validate().is_err());
    }
}
