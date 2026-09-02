use crate::error::{QuantumError, Result};
use crate::gates::QuantumGate;
use serde::{Deserialize, Serialize};

/// Represents a quantum circuit - a sequence of quantum gates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumCircuit {
    /// Name of the circuit
    pub name: String,

    /// Number of qubits
    pub num_qubits: usize,

    /// Number of classical bits for measurement results
    pub num_classical_bits: usize,

    /// Sequence of gates
    pub gates: Vec<QuantumGate>,

    /// Metadata
    pub metadata: CircuitMetadata,
}

/// Metadata about a quantum circuit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircuitMetadata {
    /// Circuit description
    pub description: Option<String>,

    /// Circuit depth (maximum path length through gates)
    pub depth: usize,

    /// Total number of single-qubit gates
    pub single_qubit_gate_count: usize,

    /// Total number of two-qubit gates
    pub two_qubit_gate_count: usize,

    /// Total number of measurements
    pub measurement_count: usize,
}

impl QuantumCircuit {
    /// Create a new empty circuit
    pub fn new(name: impl Into<String>, num_qubits: usize) -> Result<Self> {
        if num_qubits == 0 {
            return Err(QuantumError::InvalidQubitCount {
                expected: 1,
                got: 0,
            });
        }

        if num_qubits > 20 {
            return Err(QuantumError::InvalidQubitCount {
                expected: 20,
                got: num_qubits,
            });
        }

        Ok(Self {
            name: name.into(),
            num_qubits,
            num_classical_bits: 0,
            gates: Vec::new(),
            metadata: CircuitMetadata {
                description: None,
                depth: 0,
                single_qubit_gate_count: 0,
                two_qubit_gate_count: 0,
                measurement_count: 0,
            },
        })
    }

    /// Set circuit description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.metadata.description = Some(description.into());
        self
    }

    /// Add a gate to the circuit
    pub fn add_gate(&mut self, gate: QuantumGate) -> Result<()> {
        // Validate gate applies to valid qubits
        self.validate_gate(&gate)?;

        match gate {
            QuantumGate::Measurement { qubit } => {
                self.num_classical_bits = self.num_classical_bits.max(qubit + 1);
                self.metadata.measurement_count += 1;
            }
            _ => {
                if gate.is_single_qubit() {
                    self.metadata.single_qubit_gate_count += 1;
                } else if gate.is_two_qubit() {
                    self.metadata.two_qubit_gate_count += 1;
                } else if gate.is_three_qubit() {
                    // Three-qubit gates count as complex operations
                    self.metadata.two_qubit_gate_count += 1;
                }
            }
        }

        self.gates.push(gate);
        self.update_depth();
        Ok(())
    }

    /// Validate that a gate can be applied to this circuit
    fn validate_gate(&self, gate: &QuantumGate) -> Result<()> {
        match gate {
            QuantumGate::Measurement { qubit } => {
                if *qubit >= self.num_qubits {
                    return Err(QuantumError::InvalidQubitIndex {
                        index: *qubit,
                        max: self.num_qubits - 1,
                    });
                }
            }
            QuantumGate::CNOT { control, target } => {
                if *control >= self.num_qubits || *target >= self.num_qubits {
                    return Err(QuantumError::InvalidQubitIndex {
                        index: (*control).max(*target),
                        max: self.num_qubits - 1,
                    });
                }
                if control == target {
                    return Err(QuantumError::InvalidGateParameters(
                        "Control and target must be different".to_string(),
                    ));
                }
            }
            QuantumGate::ControlledZ { control, target } => {
                if *control >= self.num_qubits || *target >= self.num_qubits {
                    return Err(QuantumError::InvalidQubitIndex {
                        index: (*control).max(*target),
                        max: self.num_qubits - 1,
                    });
                }
                if control == target {
                    return Err(QuantumError::InvalidGateParameters(
                        "Control and target must be different".to_string(),
                    ));
                }
            }
            QuantumGate::Swap { qubit1, qubit2 } => {
                if *qubit1 >= self.num_qubits || *qubit2 >= self.num_qubits {
                    return Err(QuantumError::InvalidQubitIndex {
                        index: (*qubit1).max(*qubit2),
                        max: self.num_qubits - 1,
                    });
                }
                if qubit1 == qubit2 {
                    return Err(QuantumError::InvalidGateParameters(
                        "Qubits must be different".to_string(),
                    ));
                }
            }
            QuantumGate::Toffoli {
                control1,
                control2,
                target,
            } => {
                if *control1 >= self.num_qubits
                    || *control2 >= self.num_qubits
                    || *target >= self.num_qubits
                {
                    return Err(QuantumError::InvalidQubitIndex {
                        index: (*control1).max(*control2).max(*target),
                        max: self.num_qubits - 1,
                    });
                }
                if control1 == control2 || control1 == target || control2 == target {
                    return Err(QuantumError::InvalidGateParameters(
                        "All qubits must be different".to_string(),
                    ));
                }
            }
            _ => {
                // Single-qubit gates
                if let QuantumGate::Hadamard
                | QuantumGate::PauliX
                | QuantumGate::PauliY
                | QuantumGate::PauliZ
                | QuantumGate::Phase { .. }
                | QuantumGate::RotationX { .. }
                | QuantumGate::RotationY { .. }
                | QuantumGate::RotationZ { .. } = gate
                {
                    // These gates will be applied to some qubit, but we can't validate
                    // qubit index without more information
                }
            }
        }

        Ok(())
    }

    /// Update circuit depth calculation
    fn update_depth(&mut self) {
        // Simple depth calculation: count gate "layers"
        // This is a simplified version; real depth analysis is more complex
        self.metadata.depth = self.gates.len();
    }

    /// Get the total number of gates
    pub fn gate_count(&self) -> usize {
        self.gates.len()
    }

    /// Get the total number of operations
    pub fn operation_count(&self) -> usize {
        self.metadata.single_qubit_gate_count + self.metadata.two_qubit_gate_count + self.metadata.measurement_count
    }

    /// Check if circuit has measurements
    pub fn has_measurements(&self) -> bool {
        self.metadata.measurement_count > 0
    }

    /// Get circuit summary
    pub fn summary(&self) -> String {
        format!(
            "Circuit: {} ({} qubits, {} gates, depth={})\n  Single-qubit gates: {}\n  Two-qubit gates: {}\n  Measurements: {}",
            self.name,
            self.num_qubits,
            self.gate_count(),
            self.metadata.depth,
            self.metadata.single_qubit_gate_count,
            self.metadata.two_qubit_gate_count,
            self.metadata.measurement_count
        )
    }

    /// Validate circuit constraints.
    pub fn validate(&self) -> Result<(), String> {
        if self.num_qubits == 0 {
            return Err("Circuit must have at least 1 qubit".to_string());
        }
        if self.gates.is_empty() {
            return Err("Circuit has no gates".to_string());
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_circuit_creation() {
        let circuit = QuantumCircuit::new("test", 3).unwrap();
        assert_eq!(circuit.num_qubits, 3);
        assert_eq!(circuit.gate_count(), 0);
        assert!(!circuit.has_measurements());
    }

    #[test]
    fn test_circuit_too_many_qubits() {
        let result = QuantumCircuit::new("test", 25);
        assert!(result.is_err());
    }

    #[test]
    fn test_circuit_zero_qubits() {
        let result = QuantumCircuit::new("test", 0);
        assert!(result.is_err());
    }

    #[test]
    fn test_add_single_qubit_gate() {
        let mut circuit = QuantumCircuit::new("test", 2).unwrap();
        circuit.add_gate(QuantumGate::Hadamard).unwrap();
        
        assert_eq!(circuit.gate_count(), 1);
        assert_eq!(circuit.metadata.single_qubit_gate_count, 1);
    }

    #[test]
    fn test_add_two_qubit_gate() {
        let mut circuit = QuantumCircuit::new("test", 2).unwrap();
        circuit.add_gate(QuantumGate::CNOT { control: 0, target: 1 }).unwrap();
        
        assert_eq!(circuit.gate_count(), 1);
        assert_eq!(circuit.metadata.two_qubit_gate_count, 1);
    }

    #[test]
    fn test_add_measurement() {
        let mut circuit = QuantumCircuit::new("test", 2).unwrap();
        circuit.add_gate(QuantumGate::Measurement { qubit: 0 }).unwrap();
        circuit.add_gate(QuantumGate::Measurement { qubit: 1 }).unwrap();
        
        assert_eq!(circuit.gate_count(), 2);
        assert_eq!(circuit.metadata.measurement_count, 2);
        assert_eq!(circuit.num_classical_bits, 2);
        assert!(circuit.has_measurements());
    }

    #[test]
    fn test_invalid_qubit_index() {
        let mut circuit = QuantumCircuit::new("test", 2).unwrap();
        let result = circuit.add_gate(QuantumGate::Measurement { qubit: 5 });
        assert!(result.is_err());
    }

    #[test]
    fn test_cnot_control_target_different() {
        let mut circuit = QuantumCircuit::new("test", 2).unwrap();
        let result = circuit.add_gate(QuantumGate::CNOT { control: 0, target: 0 });
        assert!(result.is_err());
    }

    #[test]
    fn test_circuit_with_description() {
        let circuit = QuantumCircuit::new("test", 2)
            .unwrap()
            .with_description("A test circuit");
        
        assert!(circuit.metadata.description.is_some());
        assert_eq!(circuit.metadata.description.unwrap(), "A test circuit");
    }

    #[test]
    fn test_circuit_summary() {
        let mut circuit = QuantumCircuit::new("bell", 2).unwrap();
        circuit.add_gate(QuantumGate::Hadamard).unwrap();
        circuit.add_gate(QuantumGate::CNOT { control: 0, target: 1 }).unwrap();
        circuit.add_gate(QuantumGate::Measurement { qubit: 0 }).unwrap();
        circuit.add_gate(QuantumGate::Measurement { qubit: 1 }).unwrap();
        
        let summary = circuit.summary();
        assert!(summary.contains("bell"));
        assert!(summary.contains("2 qubits"));
        assert!(summary.contains("4 gates"));
    }

    #[test]
    fn test_toffoli_gate() {
        let mut circuit = QuantumCircuit::new("test", 3).unwrap();
        let result = circuit.add_gate(QuantumGate::Toffoli {
            control1: 0,
            control2: 1,
            target: 2,
        });
        assert!(result.is_ok());
    }

    #[test]
    fn test_toffoli_same_qubits() {
        let mut circuit = QuantumCircuit::new("test", 3).unwrap();
        let result = circuit.add_gate(QuantumGate::Toffoli {
            control1: 0,
            control2: 0,
            target: 2,
        });
        assert!(result.is_err());
    }

    #[test]
    fn test_operation_count() {
        let mut circuit = QuantumCircuit::new("test", 2).unwrap();
        circuit.add_gate(QuantumGate::Hadamard).unwrap();
        circuit.add_gate(QuantumGate::CNOT { control: 0, target: 1 }).unwrap();
        
        assert_eq!(circuit.operation_count(), 2);
    }
}
