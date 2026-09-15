use crate::error::{QuantumError, Result};
use crate::simulator::QuantumSimulator;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Classification of quantum backend types.
/// Every simulation must explicitly identify its category.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantumBackendType {
    Simulation,
    Emulation,
    Remote,
    Physical,
}

/// Metadata describing a simulation or emulation backend.
/// Every simulation module must expose this metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetadata {
    pub backend: QuantumBackendType,
    pub model: String,
    pub assumptions: Vec<String>,
    pub fidelity: Option<f64>,
}

impl Default for SimulationMetadata {
    fn default() -> Self {
        Self {
            backend: QuantumBackendType::Simulation,
            model: "unspecified-simulation-model".to_string(),
            assumptions: Vec::new(),
            fidelity: None,
        }
    }
}

/// Capabilities reported by a quantum backend.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuantumCapabilities {
    pub qubits: usize,
    pub logical_qubits: usize,
    pub supports_remote: bool,
    pub supports_majorana_adapter: bool,
}

/// Request to allocate qubits on a backend.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AllocationRequest {
    pub qubits: usize,
    pub logical_qubits: usize,
}

/// Handle to an allocated qubit register.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QubitRegister {
    pub id: String,
    pub qubits: usize,
}

/// Information about a circuit to be executed.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuantumCircuitInfo {
    pub name: String,
    pub qubits: usize,
    pub gates: usize,
}

/// Result of a quantum execution.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuantumResult {
    pub job_id: String,
    pub status: String,
}

/// Request to measure a qubit register.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeasurementRequest {
    pub register_id: String,
    pub shots: usize,
}

/// Result of a measurement operation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeasurementResult {
    pub bits: Vec<u8>,
    pub probabilities: Vec<f64>,
}

/// Health status of a quantum backend.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuantumHealth {
    pub available: bool,
    pub status: String,
}

/// Core trait for quantum processor backends.
///
/// This trait defines the interface between QuantumEnergyOS and any quantum
/// backend, whether simulator, emulator, remote service, or physical hardware.
///
/// Implementations must clearly distinguish between:
/// - SIMULATION: Classical simulation of quantum behavior
/// - EMULATION: Hardware-emulated quantum operations
/// - REMOTE: Cloud or network-accessible quantum service
/// - PHYSICAL: Direct hardware control (requires documented interface)
pub trait QuantumProcessor: Send + Sync {
    fn capabilities(&self) -> QuantumCapabilities;
    fn allocate(&mut self, request: AllocationRequest) -> Result<QubitRegister>;
    fn execute(&mut self, circuit_info: QuantumCircuitInfo) -> Result<QuantumResult>;
    fn measure(&mut self, request: MeasurementRequest) -> Result<MeasurementResult>;
    fn reset(&mut self) -> Result<()>;
    fn health(&self) -> QuantumHealth;
}

/// State-vector quantum simulator backend.
///
/// CLASSIFICATION: SIMULATION
/// This backend performs classical simulation of quantum circuits using
/// state-vector evolution. It does not interact with physical hardware.
#[derive(Debug, Clone)]
pub struct SimulatorBackend {
    pub metadata: SimulationMetadata,
    simulator: Option<QuantumSimulator>,
    allocated_qubits: usize,
    job_counter: usize,
}

impl Default for SimulatorBackend {
    fn default() -> Self {
        Self {
            metadata: SimulationMetadata {
                backend: QuantumBackendType::Simulation,
                model: "state-vector".to_string(),
                assumptions: vec![
                    "Ideal qubits (no decoherence)".to_string(),
                    "Perfect gates (no errors)".to_string(),
                    "Classical bit recording".to_string(),
                ],
                fidelity: Some(1.0),
            },
            simulator: None,
            allocated_qubits: 0,
            job_counter: 0,
        }
    }
}

impl SimulatorBackend {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_fidelity(mut self, fidelity: f64) -> Self {
        self.metadata.fidelity = Some(fidelity);
        self
    }
}

impl QuantumProcessor for SimulatorBackend {
    fn capabilities(&self) -> QuantumCapabilities {
        QuantumCapabilities {
            qubits: 20,
            logical_qubits: 16,
            supports_remote: false,
            supports_majorana_adapter: false,
        }
    }

    fn allocate(&mut self, request: AllocationRequest) -> Result<QubitRegister> {
        if request.qubits == 0 {
            return Err(QuantumError::AllocationFailed(
                "Cannot allocate 0 qubits".to_string(),
            ));
        }

        if request.qubits > 20 {
            return Err(QuantumError::AllocationFailed(format!(
                "Requested {} qubits, but maximum is 20",
                request.qubits
            )));
        }

        self.allocated_qubits = request.qubits;
        self.simulator = Some(QuantumSimulator::new(request.qubits)?);

        Ok(QubitRegister {
            id: format!("qr-{}", self.job_counter),
            qubits: request.qubits,
        })
    }

    fn execute(&mut self, circuit_info: QuantumCircuitInfo) -> Result<QuantumResult> {
        if self.simulator.is_none() {
            return Err(QuantumError::BackendNotAvailable(
                "Simulator not allocated. Call allocate() first.".to_string(),
            ));
        }

        if circuit_info.qubits != self.allocated_qubits {
            return Err(QuantumError::InvalidQubitCount {
                expected: self.allocated_qubits,
                got: circuit_info.qubits,
            });
        }

        self.job_counter += 1;
        Ok(QuantumResult {
            job_id: format!("job-{}", self.job_counter),
            status: "completed".to_string(),
        })
    }

    fn measure(&mut self, request: MeasurementRequest) -> Result<MeasurementResult> {
        if let Some(simulator) = &mut self.simulator {
            let shots = request.shots;
            let dim = 1 << self.allocated_qubits;

            // Compute probabilities once from the state vector
            let probs = simulator.probabilities();

            // Sample from the probability distribution without cloning the simulator
            let mut rng = rand::thread_rng();
            let mut counts: std::collections::HashMap<u64, usize> =
                std::collections::HashMap::new();
            let mut bits = vec![];

            for _ in 0..shots {
                let random_val: f64 = rng.gen();
                let mut cumulative = 0.0;
                let mut measured_state = 0u64;

                for (state, &prob) in probs.iter().enumerate() {
                    cumulative += prob;
                    if random_val < cumulative {
                        measured_state = state as u64;
                        break;
                    }
                }

                bits.push((measured_state & 0xFF) as u8);
                *counts.entry(measured_state).or_insert(0) += 1;
            }

            // Convert counts to probabilities
            let mut probabilities = vec![0.0; dim];
            for (state, count) in counts {
                if (state as usize) < probabilities.len() {
                    probabilities[state as usize] = count as f64 / shots as f64;
                }
            }

            Ok(MeasurementResult {
                bits,
                probabilities,
            })
        } else {
            Err(QuantumError::BackendNotAvailable(
                "Simulator not allocated".to_string(),
            ))
        }
    }

    fn reset(&mut self) -> Result<()> {
        if let Some(simulator) = &mut self.simulator {
            simulator.reset();
            Ok(())
        } else {
            Err(QuantumError::BackendNotAvailable(
                "Simulator not allocated".to_string(),
            ))
        }
    }

    fn health(&self) -> QuantumHealth {
        QuantumHealth {
            available: true,
            status: "operational".to_string(),
        }
    }
}

/// A capability-gated backend adapter that is disabled by default.
///
/// This struct provides common functionality for backends that require
/// documented hardware interfaces or external configuration before they
/// can be enabled. The `ensure_enabled` method checks the enabled state
/// and returns an appropriate error if the backend is disabled.
#[derive(Debug, Clone, Default)]
pub struct DisabledBackend {
    pub metadata: SimulationMetadata,
    pub enabled: bool,
}

impl DisabledBackend {
    /// Create a new disabled backend with the specified metadata.
    pub fn new(metadata: SimulationMetadata) -> Self {
        Self {
            metadata,
            enabled: false,
        }
    }

    /// Enable the backend.
    pub fn enable(&mut self) {
        self.enabled = true;
    }

    /// Disable the backend.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if the backend is enabled, returning an error if not.
    pub fn ensure_enabled(&self, error_msg: &str) -> Result<()> {
        if !self.enabled {
            return Err(QuantumError::BackendNotAvailable(error_msg.to_string()));
        }
        Ok(())
    }
}

/// Majorana topological quantum processor backend.
///
/// CLASSIFICATION: PHYSICAL (capability-gated)
/// This backend is disabled by default. It requires a documented hardware
/// interface to Majorana-class topological quantum processors.
///
/// No hardware access is assumed. No undocumented registers or control
/// signals are used. This is a placeholder adapter that becomes functional
/// only when a verified hardware API is available.
#[derive(Debug, Clone, Default)]
pub struct MajoranaBackend {
    inner: DisabledBackend,
}

impl MajoranaBackend {
    pub fn new() -> Self {
        Self {
            inner: DisabledBackend::new(SimulationMetadata {
                backend: QuantumBackendType::Physical,
                model: "majorana-topological".to_string(),
                assumptions: vec![
                    "Requires documented hardware interface".to_string(),
                    "Measurement-based control via parity measurements".to_string(),
                    "Topological protection of logical qubits".to_string(),
                ],
                fidelity: None,
            }),
        }
    }

    /// Enable the backend. This should only be called when a documented
    /// hardware interface is verified to be available.
    pub fn enable(&mut self) {
        self.inner.enable();
    }

    pub fn disable(&mut self) {
        self.inner.disable();
    }
}

impl QuantumProcessor for MajoranaBackend {
    fn capabilities(&self) -> QuantumCapabilities {
        QuantumCapabilities {
            qubits: 0,
            logical_qubits: 0,
            supports_remote: false,
            supports_majorana_adapter: self.inner.enabled,
        }
    }

    fn allocate(&mut self, _request: AllocationRequest) -> Result<QubitRegister> {
        self.inner.ensure_enabled(
            "MajoranaBackend is disabled: documented hardware interface required",
        )?;
        Ok(QubitRegister::default())
    }

    fn execute(&mut self, _circuit_info: QuantumCircuitInfo) -> Result<QuantumResult> {
        self.inner.ensure_enabled(
            "MajoranaBackend is disabled: documented hardware interface required",
        )?;
        Ok(QuantumResult::default())
    }

    fn measure(&mut self, _request: MeasurementRequest) -> Result<MeasurementResult> {
        self.inner.ensure_enabled(
            "MajoranaBackend is disabled: documented hardware interface required",
        )?;
        Ok(MeasurementResult::default())
    }

    fn reset(&mut self) -> Result<()> {
        self.inner.ensure_enabled(
            "MajoranaBackend is disabled: documented hardware interface required",
        )?;
        Ok(())
    }

    fn health(&self) -> QuantumHealth {
        QuantumHealth {
            available: self.inner.enabled,
            status: if self.inner.enabled {
                "READY".to_string()
            } else {
                "DISABLED".to_string()
            },
        }
    }
}

/// Azure Quantum remote backend adapter.
///
/// CLASSIFICATION: REMOTE
/// This backend provides an interface to Azure Quantum cloud services.
/// It is a stub until Azure Quantum SDK integration is implemented.
#[derive(Debug, Clone, Default)]
pub struct AzureQuantumBackend {
    inner: DisabledBackend,
}

impl AzureQuantumBackend {
    pub fn new() -> Self {
        Self {
            inner: DisabledBackend::new(SimulationMetadata {
                backend: QuantumBackendType::Remote,
                model: "azure-quantum".to_string(),
                assumptions: vec![
                    "Requires Azure Quantum workspace configuration".to_string(),
                    "Network connectivity to Azure cloud services".to_string(),
                    "Authentication via Azure credentials".to_string(),
                ],
                fidelity: None,
            }),
        }
    }

    pub fn enable(&mut self) {
        self.inner.enable();
    }

    pub fn disable(&mut self) {
        self.inner.disable();
    }
}

impl QuantumProcessor for AzureQuantumBackend {
    fn capabilities(&self) -> QuantumCapabilities {
        QuantumCapabilities {
            qubits: 0,
            logical_qubits: 0,
            supports_remote: self.inner.enabled,
            supports_majorana_adapter: false,
        }
    }

    fn allocate(&mut self, _request: AllocationRequest) -> Result<QubitRegister> {
        self.inner
            .ensure_enabled("AzureQuantumBackend is disabled: configure Azure workspace")?;
        Ok(QubitRegister::default())
    }

    fn execute(&mut self, _circuit_info: QuantumCircuitInfo) -> Result<QuantumResult> {
        self.inner
            .ensure_enabled("AzureQuantumBackend is disabled: configure Azure workspace")?;
        Ok(QuantumResult::default())
    }

    fn measure(&mut self, _request: MeasurementRequest) -> Result<MeasurementResult> {
        self.inner
            .ensure_enabled("AzureQuantumBackend is disabled: configure Azure workspace")?;
        Ok(MeasurementResult::default())
    }

    fn reset(&mut self) -> Result<()> {
        self.inner
            .ensure_enabled("AzureQuantumBackend is disabled: configure Azure workspace")?;
        Ok(())
    }

    fn health(&self) -> QuantumHealth {
        QuantumHealth {
            available: self.inner.enabled,
            status: if self.inner.enabled {
                "CONNECTED".to_string()
            } else {
                "DISABLED".to_string()
            },
        }
    }
}

/// Local emulator backend for testing.
///
/// CLASSIFICATION: EMULATION
/// Provides simplified quantum operation emulation for development
/// and testing without full state-vector simulation overhead.
#[derive(Debug, Clone, Default)]
pub struct LocalEmulatorBackend {
    pub metadata: SimulationMetadata,
    pub allocated: bool,
    job_counter: usize,
}

impl LocalEmulatorBackend {
    pub fn new() -> Self {
        Self {
            metadata: SimulationMetadata {
                backend: QuantumBackendType::Emulation,
                model: "simplified-emulator".to_string(),
                assumptions: vec![
                    "Simplified gate model".to_string(),
                    "No decoherence simulation".to_string(),
                    "Instant execution".to_string(),
                ],
                fidelity: Some(0.95),
            },
            allocated: false,
            job_counter: 0,
        }
    }
}

impl QuantumProcessor for LocalEmulatorBackend {
    fn capabilities(&self) -> QuantumCapabilities {
        QuantumCapabilities {
            qubits: 32,
            logical_qubits: 32,
            supports_remote: false,
            supports_majorana_adapter: false,
        }
    }

    fn allocate(&mut self, request: AllocationRequest) -> Result<QubitRegister> {
        if request.qubits == 0 || request.qubits > 32 {
            return Err(QuantumError::AllocationFailed(
                "Emulator supports 1-32 qubits".to_string(),
            ));
        }
        self.allocated = true;
        Ok(QubitRegister {
            id: "emulated-register".to_string(),
            qubits: request.qubits,
        })
    }

    fn execute(&mut self, _circuit_info: QuantumCircuitInfo) -> Result<QuantumResult> {
        if !self.allocated {
            return Err(QuantumError::BackendNotAvailable(
                "No qubits allocated".to_string(),
            ));
        }
        self.job_counter += 1;
        Ok(QuantumResult {
            job_id: format!("emu-job-{}", self.job_counter),
            status: "EMULATION_ONLY".to_string(),
        })
    }

    fn measure(&mut self, request: MeasurementRequest) -> Result<MeasurementResult> {
        if !self.allocated {
            return Err(QuantumError::BackendNotAvailable(
                "No qubits allocated".to_string(),
            ));
        }
        Ok(MeasurementResult {
            bits: vec![0; request.shots],
            probabilities: vec![1.0, 0.0],
        })
    }

    fn reset(&mut self) -> Result<()> {
        self.allocated = false;
        Ok(())
    }

    fn health(&self) -> QuantumHealth {
        QuantumHealth {
            available: true,
            status: "EMULATION".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simulator_backend_creation() {
        let backend = SimulatorBackend::new();
        assert_eq!(backend.metadata.backend, QuantumBackendType::Simulation);
        assert_eq!(backend.allocated_qubits, 0);
    }

    #[test]
    fn test_capabilities() {
        let backend = SimulatorBackend::new();
        let cap = backend.capabilities();
        assert_eq!(cap.qubits, 20);
        assert_eq!(cap.logical_qubits, 16);
        assert!(!cap.supports_remote);
        assert!(!cap.supports_majorana_adapter);
    }

    #[test]
    fn test_allocate() {
        let mut backend = SimulatorBackend::new();
        let request = AllocationRequest {
            qubits: 5,
            logical_qubits: 0,
        };
        let result = backend.allocate(request).unwrap();
        assert_eq!(result.qubits, 5);
        assert_eq!(backend.allocated_qubits, 5);
    }

    #[test]
    fn test_allocate_zero_qubits() {
        let mut backend = SimulatorBackend::new();
        let request = AllocationRequest {
            qubits: 0,
            logical_qubits: 0,
        };
        let result = backend.allocate(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_allocate_too_many_qubits() {
        let mut backend = SimulatorBackend::new();
        let request = AllocationRequest {
            qubits: 25,
            logical_qubits: 0,
        };
        let result = backend.allocate(request);
        assert!(result.is_err());
    }

    #[test]
    fn test_health() {
        let backend = SimulatorBackend::new();
        let health = backend.health();
        assert!(health.available);
    }

    #[test]
    fn test_reset() {
        let mut backend = SimulatorBackend::new();
        let request = AllocationRequest {
            qubits: 3,
            logical_qubits: 0,
        };
        backend.allocate(request).unwrap();
        let result = backend.reset();
        assert!(result.is_ok());
    }

    #[test]
    fn test_execute() {
        let mut backend = SimulatorBackend::new();
        let alloc_req = AllocationRequest {
            qubits: 3,
            logical_qubits: 0,
        };
        backend.allocate(alloc_req).unwrap();

        let circuit = QuantumCircuitInfo {
            name: "test".to_string(),
            qubits: 3,
            gates: 5,
        };
        let result = backend.execute(circuit).unwrap();
        assert_eq!(result.status, "completed");
    }

    #[test]
    fn test_measure() {
        let mut backend = SimulatorBackend::new();
        let alloc_req = AllocationRequest {
            qubits: 2,
            logical_qubits: 0,
        };
        backend.allocate(alloc_req).unwrap();

        let measure_req = MeasurementRequest {
            register_id: "qr-0".to_string(),
            shots: 100,
        };
        let result = backend.measure(measure_req).unwrap();
        assert_eq!(result.bits.len(), 100);
        assert_eq!(result.probabilities.len(), 4);
    }

    #[test]
    fn test_fidelity() {
        let backend = SimulatorBackend::new().with_fidelity(0.99);
        assert!(backend.metadata.fidelity.is_some());
        assert!((backend.metadata.fidelity.unwrap() - 0.99).abs() < 1e-10);
    }

    #[test]
    fn test_majorana_backend_disabled() {
        let mut backend = MajoranaBackend::new();
        assert!(!backend.inner.enabled);
        assert_eq!(backend.health().status, "DISABLED");

        let result = backend.allocate(AllocationRequest::default());
        assert!(result.is_err());
    }

    #[test]
    fn test_majorana_backend_enable() {
        let mut backend = MajoranaBackend::new();
        backend.enable();
        assert!(backend.inner.enabled);
        assert_eq!(backend.health().status, "READY");
        assert!(backend.capabilities().supports_majorana_adapter);
    }

    #[test]
    fn test_majorana_backend_disable() {
        let mut backend = MajoranaBackend::new();
        backend.enable();
        backend.disable();
        assert!(!backend.inner.enabled);
    }

    #[test]
    fn test_azure_backend_disabled() {
        let backend = AzureQuantumBackend::new();
        assert!(!backend.inner.enabled);
        assert_eq!(backend.health().status, "DISABLED");
    }

    #[test]
    fn test_azure_backend_enable() {
        let mut backend = AzureQuantumBackend::new();
        backend.enable();
        assert!(backend.inner.enabled);
        assert!(backend.capabilities().supports_remote);
    }

    #[test]
    fn test_local_emulator() {
        let mut backend = LocalEmulatorBackend::new();
        assert_eq!(backend.health().status, "EMULATION");

        let alloc = backend
            .allocate(AllocationRequest {
                qubits: 8,
                logical_qubits: 0,
            })
            .unwrap();
        assert_eq!(alloc.qubits, 8);

        let result = backend
            .execute(QuantumCircuitInfo {
                name: "test".to_string(),
                qubits: 8,
                gates: 10,
            })
            .unwrap();
        assert_eq!(result.status, "EMULATION_ONLY");
    }

    #[test]
    fn test_emulator_bounds() {
        let mut backend = LocalEmulatorBackend::new();
        assert!(backend
            .allocate(AllocationRequest {
                qubits: 0,
                logical_qubits: 0
            })
            .is_err());
        assert!(backend
            .allocate(AllocationRequest {
                qubits: 33,
                logical_qubits: 0
            })
            .is_err());
    }

    #[test]
    fn test_backend_type_serde() {
        let backend_type = QuantumBackendType::Simulation;
        let json = serde_json::to_string(&backend_type).unwrap();
        let deserialized: QuantumBackendType = serde_json::from_str(&json).unwrap();
        assert_eq!(backend_type, deserialized);
    }
}
