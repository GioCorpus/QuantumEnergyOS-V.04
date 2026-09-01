use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuantumBackendType {
    Simulation,
    Emulation,
    Remote,
    Physical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationMetadata {
    pub backend: QuantumBackendType,
    pub model: String,
    pub assumptions: Vec<String>,
    pub fidelity: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuantumCapabilities {
    pub qubits: usize,
    pub logical_qubits: usize,
    pub supports_remote: bool,
    pub supports_majorana_adapter: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AllocationRequest {
    pub qubits: usize,
    pub logical_qubits: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QubitRegister {
    pub id: String,
    pub qubits: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuantumCircuit {
    pub name: String,
    pub operations: Vec<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuantumResult {
    pub job_id: String,
    pub status: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeasurementRequest {
    pub register_id: String,
    pub shots: usize,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MeasurementResult {
    pub bits: Vec<u8>,
    pub probabilities: Vec<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuantumHealth {
    pub available: bool,
    pub status: String,
}

pub trait QuantumProcessor {
    fn capabilities(&self) -> QuantumCapabilities;
    fn allocate(&mut self, request: AllocationRequest) -> Result<QubitRegister, String>;
    fn execute(&mut self, circuit: QuantumCircuit) -> Result<QuantumResult, String>;
    fn measure(&mut self, request: MeasurementRequest) -> Result<MeasurementResult, String>;
    fn reset(&mut self) -> Result<(), String>;
    fn health(&self) -> QuantumHealth;
}

#[derive(Debug, Clone, Default)]
pub struct SimulatorBackend {
    pub metadata: SimulationMetadata,
}

impl QuantumProcessor for SimulatorBackend {
    fn capabilities(&self) -> QuantumCapabilities {
        QuantumCapabilities {
            qubits: 64,
            logical_qubits: 16,
            supports_remote: false,
            supports_majorana_adapter: false,
        }
    }

    fn allocate(&mut self, request: AllocationRequest) -> Result<QubitRegister, String> {
        Ok(QubitRegister {
            id: format!("sim-{}", request.qubits),
            qubits: request.qubits,
        })
    }

    fn execute(&mut self, circuit: QuantumCircuit) -> Result<QuantumResult, String> {
        let _ = circuit;
        Ok(QuantumResult {
            job_id: "sim-job".to_string(),
            status: "SIMULATION_ONLY".to_string(),
        })
    }

    fn measure(&mut self, request: MeasurementRequest) -> Result<MeasurementResult, String> {
        let _ = request;
        Ok(MeasurementResult {
            bits: vec![0, 1],
            probabilities: vec![0.5, 0.5],
        })
    }

    fn reset(&mut self) -> Result<(), String> {
        Ok(())
    }

    fn health(&self) -> QuantumHealth {
        QuantumHealth {
            available: true,
            status: "SIMULATION".to_string(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MajoranaBackend {
    pub metadata: SimulationMetadata,
    pub enabled: bool,
}

impl QuantumProcessor for MajoranaBackend {
    fn capabilities(&self) -> QuantumCapabilities {
        QuantumCapabilities {
            qubits: 0,
            logical_qubits: 0,
            supports_remote: false,
            supports_majorana_adapter: false,
        }
    }

    fn allocate(&mut self, _request: AllocationRequest) -> Result<QubitRegister, String> {
        if !self.enabled {
            return Err("MajoranaBackend is disabled: documented hardware interface required".to_string());
        }
        Ok(QubitRegister::default())
    }

    fn execute(&mut self, _circuit: QuantumCircuit) -> Result<QuantumResult, String> {
        if !self.enabled {
            return Err("MajoranaBackend is disabled: documented hardware interface required".to_string());
        }
        Ok(QuantumResult::default())
    }

    fn measure(&mut self, _request: MeasurementRequest) -> Result<MeasurementResult, String> {
        if !self.enabled {
            return Err("MajoranaBackend is disabled: documented hardware interface required".to_string());
        }
        Ok(MeasurementResult::default())
    }

    fn reset(&mut self) -> Result<(), String> {
        if !self.enabled {
            return Err("MajoranaBackend is disabled: documented hardware interface required".to_string());
        }
        Ok(())
    }

    fn health(&self) -> QuantumHealth {
        QuantumHealth {
            available: self.enabled,
            status: if self.enabled { "READY".to_string() } else { "DISABLED".to_string() },
        }
    }
}
