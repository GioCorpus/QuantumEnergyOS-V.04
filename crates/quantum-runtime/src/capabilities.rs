//! Phase 4.3 backend capabilities (explicit, no assumptions).
//!
//! CLASSIFICATION: MODEL
//!
//! Every backend declares what it supports. The scheduler consults these
//! flags instead of assuming gate/noise/GPU/Majorana support.

use serde::{Deserialize, Serialize};

/// Explicit capability set for a quantum backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendCapabilities {
    pub backend_name: String,
    pub qubit_count: usize,
    pub supports_measurement: bool,
    pub supports_reset: bool,
    pub supports_conditional: bool,
    pub supports_noise: bool,
    pub supports_gpu: bool,
    pub supports_majorana: bool,
    pub supports_parity_measurement: bool,
    pub supports_logical_qubits: bool,
    pub supports_measurement_feed_forward: bool,
    pub supports_async: bool,
    pub simulation_only: bool,
}

impl BackendCapabilities {
    pub fn cpu_simulator(qubit_count: usize) -> Self {
        Self {
            backend_name: "cpu-simulator".to_string(),
            qubit_count,
            supports_measurement: true,
            supports_reset: true,
            supports_conditional: true,
            supports_noise: true,
            supports_gpu: false,
            supports_majorana: false,
            supports_parity_measurement: false,
            supports_logical_qubits: false,
            supports_measurement_feed_forward: true,
            supports_async: true,
            simulation_only: true,
        }
    }

    pub fn majorana_simulator(qubit_count: usize) -> Self {
        Self {
            backend_name: "majorana-simulator".to_string(),
            qubit_count,
            supports_measurement: true,
            supports_reset: true,
            supports_conditional: true,
            supports_noise: true,
            supports_gpu: false,
            supports_majorana: true,
            supports_parity_measurement: true,
            supports_logical_qubits: true,
            supports_measurement_feed_forward: true,
            supports_async: true,
            simulation_only: true,
        }
    }

    pub fn gpu_simulator(qubit_count: usize, available: bool) -> Self {
        let mut caps = Self::cpu_simulator(qubit_count);
        caps.backend_name = "gpu-simulator".to_string();
        caps.supports_gpu = available;
        caps
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cpu_caps_do_not_claim_majorana() {
        let caps = BackendCapabilities::cpu_simulator(8);
        assert!(!caps.supports_majorana);
        assert!(caps.simulation_only);
    }

    #[test]
    fn test_majorana_caps() {
        let caps = BackendCapabilities::majorana_simulator(4);
        assert!(caps.supports_majorana);
        assert!(caps.supports_parity_measurement);
        assert!(caps.supports_logical_qubits);
        assert!(caps.simulation_only);
    }
}
