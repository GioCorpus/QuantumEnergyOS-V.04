//! P7.4-02 — QPU backend architecture.
//!
//! Three backend families are supported with explicit classification:
//!
//! - **SimulatorBackend** — **SIMULATED**. Delegates parity measurement to the
//!   `quantum-runtime` Majorana simulator (deterministic, seeded).
//! - **MockBackend** — **MOCK**. A test double over the simulator.
//! - **VendorBackend** — **UNAVAILABLE**. Requires a documented, authenticated,
//!   validated vendor adapter (`quantum_hal::qpu::QpuVendorAdapter`). No such
//!   adapter exists in this build, and none is fabricated.

use serde::{Deserialize, Serialize};

use crate::job::{QpuJob, QpuResult};
use crate::{QpuError, QpuResultType};

/// Backend family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QpuBackendKind {
    Simulator,
    Mock,
    Vendor,
}

/// Static capabilities of a QPU backend.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QpuCapabilities {
    pub kind: QpuBackendKind,
    /// Whether a physical/vendor device is actually reachable.
    pub vendor_available: bool,
    /// Maximum addressable qubits for simulation.
    pub qubit_limit: usize,
    /// Maximum shots per job.
    pub max_shots: u64,
    /// Whether measurement noise can be modeled.
    pub supports_noise: bool,
}

impl Default for QpuCapabilities {
    fn default() -> Self {
        Self {
            kind: QpuBackendKind::Simulator,
            vendor_available: false,
            qubit_limit: 16,
            max_shots: 1_000_000,
            supports_noise: true,
        }
    }
}

/// A backend that executes a [`QpuJob`] and returns a [`QpuResult`].
pub trait QpuBackend: Send {
    fn kind(&self) -> QpuBackendKind;
    fn capabilities(&self) -> &QpuCapabilities;
    fn execute(&mut self, job: &QpuJob) -> QpuResultType<QpuResult>;
}

/// SIMULATED backend that runs the deterministic Majorana simulator.
pub struct SimulatorBackend {
    caps: QpuCapabilities,
}

impl SimulatorBackend {
    pub fn new() -> Self {
        Self {
            caps: QpuCapabilities::default(),
        }
    }

    pub fn with_limits(qubit_limit: usize, max_shots: u64) -> Self {
        Self {
            caps: QpuCapabilities {
                qubit_limit,
                max_shots,
                ..Default::default()
            },
        }
    }
}

impl Default for SimulatorBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl QpuBackend for SimulatorBackend {
    fn kind(&self) -> QpuBackendKind {
        QpuBackendKind::Simulator
    }

    fn capabilities(&self) -> &QpuCapabilities {
        &self.caps
    }

    fn execute(&mut self, job: &QpuJob) -> QpuResultType<QpuResult> {
        if job.qubits > self.caps.qubit_limit {
            return Err(QpuError::ResourceLimit(format!(
                "qubits {} exceed backend limit {}",
                job.qubits, self.caps.qubit_limit
            )));
        }
        if job.shots > self.caps.max_shots {
            return Err(QpuError::ResourceLimit(format!(
                "shots {} exceed backend limit {}",
                job.shots, self.caps.max_shots
            )));
        }

        // Deterministic parity measurement via the quantum-runtime Majorana
        // simulator. Tagged SIMULATED.
        let cfg = quantum_runtime::MajoranaSimConfig {
            tetron_label: format!("tetron-{}", job.qubits),
            measurement_error: job.measurement_error,
            poisoning_probability: job.poisoning_probability,
            ..Default::default()
        };
        let res = quantum_runtime::run_majorana_experiment(
            job.shots as u32,
            job.seed,
            &cfg,
            &quantum_runtime::RuntimeNoiseModel::ideal(),
        )
        .map_err(|e| QpuError::Backend(e.to_string()))?;

        Ok(QpuResult {
            job_id: job.id,
            shots: job.shots,
            zeros: res.logical_zeros,
            ones: res.logical_ones,
            logical_error_rate: res.logical_error_rate,
            simulation_only: res.simulation_only,
            backend: QpuBackendKind::Simulator,
        })
    }
}

/// MOCK backend — a test double that delegates exactly to the simulator.
pub struct MockBackend {
    inner: SimulatorBackend,
}

impl MockBackend {
    pub fn new() -> Self {
        Self {
            inner: SimulatorBackend::new(),
        }
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl QpuBackend for MockBackend {
    fn kind(&self) -> QpuBackendKind {
        QpuBackendKind::Mock
    }

    fn capabilities(&self) -> &QpuCapabilities {
        self.inner.capabilities()
    }

    fn execute(&mut self, job: &QpuJob) -> QpuResultType<QpuResult> {
        self.inner.execute(job)
    }
}

/// Vendor backend — **UNAVAILABLE**. Construction succeeds but capabilities
/// always report `vendor_available == false` and execution always fails with
/// `UnsupportedHardware`. A real integration requires a validated
/// `quantum_hal::qpu::QpuVendorAdapter`.
pub struct VendorBackend;

impl QpuBackend for VendorBackend {
    fn kind(&self) -> QpuBackendKind {
        QpuBackendKind::Vendor
    }

    fn capabilities(&self) -> &QpuCapabilities {
        // A shared static, always-unavailable cap set.
        static CAPS: std::sync::OnceLock<QpuCapabilities> = std::sync::OnceLock::new();
        CAPS.get_or_init(|| QpuCapabilities {
            kind: QpuBackendKind::Vendor,
            vendor_available: false,
            qubit_limit: 0,
            max_shots: 0,
            supports_noise: false,
        })
    }

    fn execute(&mut self, _job: &QpuJob) -> QpuResultType<QpuResult> {
        Err(QpuError::UnsupportedHardware)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::QpuJob;

    fn job(qubits: usize, shots: u64) -> QpuJob {
        QpuJob::new(1, qubits, shots, 42)
    }

    #[test]
    fn simulator_executes_deterministically() {
        let mut b = SimulatorBackend::new();
        let r = b.execute(&job(1, 64)).unwrap();
        assert!(r.simulation_only);
        assert_eq!(r.shots, 64);
    }

    #[test]
    fn measurement_is_repeatable_for_same_seed() {
        // P7.4-05: deterministic simulation must reproduce identical outcomes.
        let mut b1 = SimulatorBackend::with_limits(16, 100000);
        let mut b2 = SimulatorBackend::with_limits(16, 100000);
        let r1 = b1.execute(&job(1, 4096)).unwrap();
        let r2 = b2.execute(&job(1, 4096)).unwrap();
        assert_eq!(r1.zeros, r2.zeros);
        assert_eq!(r1.ones, r2.ones);
        assert_eq!(r1.logical_error_rate, r2.logical_error_rate);
    }

    #[test]
    fn simulator_rejects_excess_qubits() {
        let mut b = SimulatorBackend::with_limits(2, 1000);
        assert!(matches!(
            b.execute(&job(8, 64)),
            Err(QpuError::ResourceLimit(_))
        ));
    }

    #[test]
    fn simulator_rejects_excess_shots() {
        let mut b = SimulatorBackend::with_limits(16, 10);
        assert!(matches!(
            b.execute(&job(1, 100)),
            Err(QpuError::ResourceLimit(_))
        ));
    }

    #[test]
    fn mock_kind_is_mock() {
        let b = MockBackend::new();
        assert_eq!(b.kind(), QpuBackendKind::Mock);
    }

    #[test]
    fn vendor_is_unavailable() {
        let mut b = VendorBackend;
        assert!(!b.capabilities().vendor_available);
        assert!(matches!(
            b.execute(&job(1, 10)),
            Err(QpuError::UnsupportedHardware)
        ));
    }

    #[test]
    fn capabilities_defaults_honest() {
        let caps = QpuCapabilities::default();
        assert!(!caps.vendor_available);
        assert!(caps.supports_noise);
    }
}
