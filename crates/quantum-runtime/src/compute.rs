//! Phase 4.3 compute backend (CPU reference + optional GPU path).
//!
//! CLASSIFICATION: SIMULATION
//!
//! CPU is the correctness reference. GPU entries are capability-gated
//! accelerators: when no vendor backend is compiled in, execution falls back
//! to the CPU reference and reports `gpu_used=false` honestly.

use serde::{Deserialize, Serialize};

use crate::circuit::QuantumCircuit;
use crate::error::{QuantumError, Result};
use crate::experiment::{run_experiment, ExperimentLimits, QuantumExperiment};

/// Compute backend selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ComputeBackendKind {
    Cpu,
    Wgpu,
    Cuda,
    Rocm,
}

impl ComputeBackendKind {
    pub fn label(self) -> &'static str {
        match self {
            ComputeBackendKind::Cpu => "cpu",
            ComputeBackendKind::Wgpu => "wgpu",
            ComputeBackendKind::Cuda => "cuda",
            ComputeBackendKind::Rocm => "rocm",
        }
    }
}

/// Compute-backend execution report (honest about fallback).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComputeReport {
    pub requested: ComputeBackendKind,
    pub executed_on: ComputeBackendKind,
    pub gpu_used: bool,
    pub note: String,
}

/// Availability probe: only CPU is guaranteed in this build.
pub fn probe_backend(kind: ComputeBackendKind) -> (bool, String) {
    match kind {
        ComputeBackendKind::Cpu => (true, "cpu reference available".to_string()),
        other => (
            false,
            format!(
                "{} vendor backend not compiled in; cpu fallback",
                other.label()
            ),
        ),
    }
}
/// Execute an experiment on the requested backend (fallback to CPU).
pub fn execute_on_backend(
    experiment: &QuantumExperiment,
    limits: &ExperimentLimits,
    requested: ComputeBackendKind,
) -> Result<(crate::experiment::ExperimentResult, ComputeReport)> {
    let (available, note) = probe_backend(requested);
    let executed_on = if available {
        requested
    } else {
        ComputeBackendKind::Cpu
    };
    let mut result = run_experiment(experiment, limits)?;
    result.backend = format!("{}-via-{}", experiment.backend, executed_on.label());
    Ok((
        result,
        ComputeReport {
            requested,
            executed_on,
            gpu_used: false,
            note,
        },
    ))
}
/// Validate that a circuit fits the compute budget before dispatch.
pub fn check_circuit_resources(circuit: &QuantumCircuit, limits: &ExperimentLimits) -> Result<()> {
    if circuit.num_qubits > limits.max_qubits {
        return Err(QuantumError::SimulationResource(format!(
            "qubits {} exceed {}",
            circuit.num_qubits, limits.max_qubits
        )));
    }
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::gates::QuantumGate;
    use crate::noise::RuntimeNoiseModel;
    #[test]
    fn test_cpu_probe_available() {
        assert!(probe_backend(ComputeBackendKind::Cpu).0);
        assert!(!probe_backend(ComputeBackendKind::Cuda).0);
    }
    #[test]
    fn test_gpu_falls_back_honestly() {
        let mut c = QuantumCircuit::new("x", 1).unwrap();
        c.add_gate(QuantumGate::PauliX).unwrap();
        let exp = QuantumExperiment::new("e", c, "sim", 8, RuntimeNoiseModel::ideal(), 3);
        let (res, report) =
            execute_on_backend(&exp, &ExperimentLimits::default(), ComputeBackendKind::Cuda)
                .unwrap();
        assert!(!report.gpu_used);
        assert_eq!(report.executed_on, ComputeBackendKind::Cpu);
        assert!(res.simulation_only);
    }
}
