//! Quantum job system (Phase 3, §11).
//!
//! Every quantum execution is a job. Jobs carry the backend-neutral device IR
//! ([`QuantumIR`]), scheduling metadata and simulation parameters so that a
//! result can always be reproduced and audited.
//!
//! CLASSIFICATION: REAL (job bookkeeping) / SIMULATED (execution by the
//! reference simulator device).

use std::collections::HashMap;

use chrono::Utc;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::ir::QuantumIR;

/// Opaque handle to a submitted job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord, Serialize, Deserialize)]
pub struct JobHandle(pub u64);

impl std::fmt::Display for JobHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "job-handle:{}", self.0)
    }
}

/// Lifecycle status of a job (§11).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// Accepted and waiting for a device.
    Queued,
    /// Currently executing.
    Running,
    /// Finished successfully; a result is available.
    Completed,
    /// Finished with an error; `read_result` reports the error.
    Failed,
    /// Cancelled before completion.
    Cancelled,
    /// The device cannot execute this job (missing hardware or unsupported op).
    Unsupported,
}

impl JobStatus {
    /// True when the job will not change status again.
    pub fn is_terminal(&self) -> bool {
        matches!(
            self,
            JobStatus::Completed
                | JobStatus::Failed
                | JobStatus::Cancelled
                | JobStatus::Unsupported
        )
    }

    /// True when the job completed successfully.
    pub fn is_success(&self) -> bool {
        matches!(self, JobStatus::Completed)
    }
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            JobStatus::Queued => "Queued",
            JobStatus::Running => "Running",
            JobStatus::Completed => "Completed",
            JobStatus::Failed => "Failed",
            JobStatus::Cancelled => "Cancelled",
            JobStatus::Unsupported => "Unsupported",
        };
        f.write_str(label)
    }
}

/// Scheduling priority of a job.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub enum JobPriority {
    /// Background work.
    Low = 0,
    /// Default priority.
    #[default]
    Normal = 1,
    /// Interactive work.
    High = 2,
    /// Reserved for system operations.
    Critical = 3,
}

impl std::fmt::Display for JobPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            JobPriority::Low => "Low",
            JobPriority::Normal => "Normal",
            JobPriority::High => "High",
            JobPriority::Critical => "Critical",
        };
        f.write_str(label)
    }
}

/// Classical, parameterised noise model for the reference simulator.
///
/// CLASSIFICATION: SIMULATED
///
/// This is a classical bit-flip/measurement-error model applied to sampled
/// outcomes. It is NOT physical device noise, it is not derived from
/// cryogenic measurements, and it must not be reported as hardware fidelity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NoiseModel {
    /// Per-shot bit-flip probability of sampled outcomes.
    pub bit_flip_probability: f64,
    /// Additional measurement error probability.
    pub measurement_error_probability: f64,
    /// Classical proxy for decoherence accumulated over the circuit.
    pub decoherence_rate: f64,
}

impl Default for NoiseModel {
    fn default() -> Self {
        Self::ideal()
    }
}

impl NoiseModel {
    /// Noise-free reference model.
    pub fn ideal() -> Self {
        Self {
            bit_flip_probability: 0.0,
            measurement_error_probability: 0.0,
            decoherence_rate: 0.0,
        }
    }

    /// Depolarising-style classical model with a single probability parameter.
    pub fn depolarizing(probability: f64) -> Self {
        let p = probability.clamp(0.0, 0.5);
        Self {
            bit_flip_probability: p,
            measurement_error_probability: p,
            decoherence_rate: p,
        }
    }

    /// Effective per-bit flip probability used by the simulator backend.
    pub fn total_bit_flip_probability(&self) -> f64 {
        (self.bit_flip_probability + self.measurement_error_probability + self.decoherence_rate)
            .clamp(0.0, 0.5)
    }

    /// True when no noise is configured.
    pub fn is_ideal(&self) -> bool {
        self.total_bit_flip_probability() == 0.0
    }
}

/// Simulation parameters attached to a job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    /// Number of measurement shots.
    pub shots: u32,
    /// Optional seed. When set, results are reproducible.
    pub seed: Option<u64>,
    /// Classical noise model (SIMULATED).
    pub noise: NoiseModel,
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            shots: 1024,
            seed: None,
            noise: NoiseModel::ideal(),
        }
    }
}

impl SimulationConfig {
    /// Configuration with an explicit shot count.
    pub fn new(shots: u32) -> Self {
        Self {
            shots,
            ..Self::default()
        }
    }

    /// Enable deterministic, reproducible sampling.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Attach a classical noise model.
    pub fn with_noise(mut self, noise: NoiseModel) -> Self {
        self.noise = noise;
        self
    }
}

/// A quantum job: the unit of work submitted to a device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumJob {
    /// Unique job identifier (UUID v4).
    pub job_id: String,
    /// Requested backend/device identifier.
    pub backend: String,
    /// Scheduling priority.
    pub priority: JobPriority,
    /// Creation timestamp (RFC 3339).
    pub created_at: String,
    /// Timeout in milliseconds; `0` means no timeout.
    pub timeout_ms: u64,
    /// Device-level circuit representation.
    pub circuit: QuantumIR,
    /// Free-form audit metadata (caller, project, material, experiment...).
    pub metadata: HashMap<String, String>,
    /// Simulation parameters.
    pub simulation: SimulationConfig,
}

impl QuantumJob {
    /// Create a job for a backend with a generated identifier.
    pub fn new(backend: impl Into<String>, circuit: QuantumIR) -> Self {
        Self {
            job_id: Uuid::new_v4().to_string(),
            backend: backend.into(),
            priority: JobPriority::default(),
            created_at: Utc::now().to_rfc3339(),
            timeout_ms: 0,
            circuit,
            metadata: HashMap::new(),
            simulation: SimulationConfig::default(),
        }
    }

    /// Override the generated job identifier (useful for reproducible tests).
    pub fn with_job_id(mut self, job_id: impl Into<String>) -> Self {
        self.job_id = job_id.into();
        self
    }

    /// Set the scheduling priority.
    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set a timeout in milliseconds.
    pub fn with_timeout_ms(mut self, timeout_ms: u64) -> Self {
        self.timeout_ms = timeout_ms;
        self
    }

    /// Set the number of shots.
    pub fn with_shots(mut self, shots: u32) -> Self {
        self.simulation.shots = shots;
        self
    }

    /// Set the sampling seed for reproducible results.
    pub fn with_seed(mut self, seed: u64) -> Self {
        self.simulation.seed = Some(seed);
        self
    }

    /// Attach a classical noise model.
    pub fn with_noise(mut self, noise: NoiseModel) -> Self {
        self.simulation.noise = noise;
        self
    }

    /// Add a single metadata entry.
    pub fn with_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Qubit count of the job's circuit.
    pub fn qubits(&self) -> usize {
        self.circuit.num_qubits
    }

    /// True when the job declares a timeout.
    pub fn has_timeout(&self) -> bool {
        self.timeout_ms > 0
    }
}

/// Result of a completed job.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumResult {
    /// Job this result belongs to.
    pub job_id: String,
    /// Device/backend that produced the result.
    pub backend: String,
    /// Terminal status.
    pub status: JobStatus,
    /// Number of shots requested.
    pub shots: u32,
    /// Sampled outcomes, one string per shot (leftmost character = highest qubit).
    pub bitstrings: Vec<String>,
    /// Observed outcome histogram (after any simulated noise).
    pub counts: HashMap<String, u64>,
    /// Ideal state-vector probabilities in computational-basis order.
    pub probabilities: Vec<f64>,
    /// True when the result came from a classical simulation.
    pub simulation_only: bool,
    /// Classification and limitation notes (mandatory for simulated results).
    pub notes: Vec<String>,
}

impl QuantumResult {
    /// Empty result skeleton for a job.
    pub fn new(job_id: impl Into<String>, backend: impl Into<String>, shots: u32) -> Self {
        Self {
            job_id: job_id.into(),
            backend: backend.into(),
            status: JobStatus::Queued,
            shots,
            bitstrings: Vec::new(),
            counts: HashMap::new(),
            probabilities: Vec::new(),
            simulation_only: true,
            notes: Vec::new(),
        }
    }

    /// Set the terminal status.
    pub fn with_status(mut self, status: JobStatus) -> Self {
        self.status = status;
        self
    }

    /// Append a classification/limitation note.
    pub fn push_note(&mut self, note: impl Into<String>) {
        self.notes.push(note.into());
    }

    /// Total number of recorded shots.
    pub fn total_counts(&self) -> u64 {
        self.counts.values().sum()
    }

    /// Most frequent observed outcome.
    ///
    /// Ordering is deterministic (highest count first, then lexicographic key)
    /// so that reproducible runs compare equal.
    pub fn most_frequent(&self) -> Option<(String, u64)> {
        let mut entries: Vec<(&String, &u64)> = self.counts.iter().collect();
        entries.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));
        entries
            .first()
            .map(|(key, count)| ((*key).clone(), **count))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{QuantumIR, QuantumOperation};

    fn sample_ir() -> QuantumIR {
        let mut ir = QuantumIR::new("sample", 2);
        ir.push(QuantumOperation::H(0));
        ir.push(QuantumOperation::Measure(0));
        ir
    }

    #[test]
    fn test_job_defaults() {
        let job = QuantumJob::new("sim0", sample_ir());
        assert_eq!(job.backend, "sim0");
        assert_eq!(job.priority, JobPriority::Normal);
        assert_eq!(job.simulation.shots, 1024);
        assert_eq!(job.qubits(), 2);
        assert!(!job.has_timeout());
        assert!(!job.job_id.is_empty());
    }

    #[test]
    fn test_job_builder_chain() {
        let job = QuantumJob::new("sim0", sample_ir())
            .with_job_id("job-42")
            .with_priority(JobPriority::High)
            .with_timeout_ms(1500)
            .with_shots(64)
            .with_seed(7)
            .with_noise(NoiseModel::depolarizing(0.02))
            .with_metadata("caller", "researcher@lab");

        assert_eq!(job.job_id, "job-42");
        assert_eq!(job.priority, JobPriority::High);
        assert!(job.has_timeout());
        assert_eq!(job.simulation.shots, 64);
        assert_eq!(job.simulation.seed, Some(7));
        assert!(!job.simulation.noise.is_ideal());
        assert_eq!(
            job.metadata.get("caller").map(String::as_str),
            Some("researcher@lab")
        );
    }

    #[test]
    fn test_noise_model_bounds() {
        let noise = NoiseModel::depolarizing(5.0);
        assert!((noise.total_bit_flip_probability() - 0.5).abs() < 1e-12);
        assert!(NoiseModel::ideal().is_ideal());
        assert_eq!(NoiseModel::default().total_bit_flip_probability(), 0.0);
    }

    #[test]
    fn test_job_status_terminality() {
        assert!(JobStatus::Completed.is_terminal());
        assert!(JobStatus::Unsupported.is_terminal());
        assert!(!JobStatus::Running.is_terminal());
        assert!(JobStatus::Completed.is_success());
        assert_eq!(JobStatus::Queued.to_string(), "Queued");
    }

    #[test]
    fn test_result_most_frequent_is_deterministic() {
        let mut result = QuantumResult::new("job-1", "sim0", 10);
        result.counts.insert("01".to_string(), 4);
        result.counts.insert("10".to_string(), 4);
        result.counts.insert("00".to_string(), 2);
        result.push_note("SIMULATED");

        assert_eq!(result.total_counts(), 10);
        assert_eq!(result.most_frequent(), Some(("01".to_string(), 4)));
        assert_eq!(result.notes, vec!["SIMULATED".to_string()]);
    }

    #[test]
    fn test_simulation_config_builder() {
        let config = SimulationConfig::new(8).with_seed(99);
        assert_eq!(config.shots, 8);
        assert_eq!(config.seed, Some(99));
        assert!(config.noise.is_ideal());
    }

    #[test]
    fn test_job_handle_display_and_ordering() {
        assert_eq!(JobHandle(3).to_string(), "job-handle:3");
        assert!(JobHandle(1) < JobHandle(2));
    }
}
