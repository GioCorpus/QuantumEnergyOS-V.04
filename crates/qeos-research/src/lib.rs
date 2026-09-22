//! # QEOS Research — P7.8 research platform & reproducibility
//!
//! Experiments, datasets, artifacts, reproducibility records and research
//! workflows. Research data is **never silently overwritten** (versioned +
//! checksum-protected), and environments are recorded for reproducible runs.
//!
//! ## Reality classification
//!
//! - Models and validation logic — **REAL** (host-testable).
//! - The workflow/execution is orchestrated over the simulator backends from
//!   other crates (CPU/GPU/QPU simulators); real hardware is UNAVAILABLE.

#![forbid(unsafe_code)]

pub mod artifact;
pub mod dataset;
pub mod error;
pub mod experiment;
pub mod reproducibility;
pub mod workflow;

pub use artifact::{Artifact, ArtifactKind, ArtifactStore};
pub use dataset::{digest, Dataset, DatasetRegistry};
pub use error::{ResearchError, Result};
pub use experiment::{Experiment, ExperimentId, ExperimentStatus};
pub use reproducibility::{verify_reproduction, EnvironmentRecord, Reproducibility};
pub use workflow::{RetryPolicy, Workflow, WorkflowStage};

/// QEOS research crate version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
