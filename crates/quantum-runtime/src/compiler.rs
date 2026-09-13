/// Quantum Compiler: High-Level Circuit -> IR -> Backend representation -> Execution
///
/// Pipeline:
/// ```text
/// High-Level Circuit
///         ↓
/// Intermediate Representation (IR)
///         ↓
/// Backend-specific representation
///         ↓
/// Execution
/// ```
///
/// The compiler never claims hardware execution. Every output records the
/// intended `QuantumBackendType` and whether lowering is simulation-only.

use crate::backend::QuantumBackendType;
use crate::circuit::QuantumCircuit;
use crate::error::{QuantumError, Result};
use crate::gates::QuantumGate;
use serde::{Deserialize, Serialize};

/// Maximum qubits supported by the reference lowering (matches simulator).
pub const MAX_SUPPORTED_QUBITS: usize = 20;

/// A single IR operation: backend-neutral form of a gate/measurement.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct IrOperation {
    /// Stable op name, e.g. "h", "x", "cx", "measure".
    pub op: String,
    /// Target qubits.
    pub qubits: Vec<usize>,
    /// Optional rotation angle (for rx/ry/rz/phase).
    pub angle: Option<f64>,
    /// Human-readable lowering note.
    pub note: Option<String>,
}

impl IrOperation {
    pub fn new(op: impl Into<String>, qubits: Vec<usize>) -> Self {
        Self {
            op: op.into(),
            qubits,
            angle: None,
            note: None,
        }
    }

    pub fn with_angle(mut self, angle: f64) -> Self {
        self.angle = Some(angle);
        self
    }
}

/// Backend-neutral intermediate representation of a circuit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntermediateRepresentation {
    /// Source circuit name.
    pub source: String,
    /// Qubit count.
    pub num_qubits: usize,
    /// Ordered IR ops.
    pub ops: Vec<IrOperation>,
    /// Warnings produced during lowering.
    pub warnings: Vec<String>,
}

impl IntermediateRepresentation {
    pub fn op_count(&self) -> usize {
        self.ops.len()
    }

    pub fn has_measurements(&self) -> bool {
        self.ops.iter().any(|o| o.op == "measure")
    }
}

/// Backend-specific compiled artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompiledCircuit {
    /// Source circuit name.
    pub source: String,
    /// Target backend.
    pub backend: QuantumBackendType,
    /// Backend-neutral IR used for this artifact.
    pub ir: IntermediateRepresentation,
    /// Backend-specific payload (JSON string; opaque per backend).
    pub payload: String,
    /// True when this artifact is simulation/emulation only.
    pub simulation_only: bool,
}

/// Compiler configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompilerConfig {
    /// Target backend for lowering.
    pub target: QuantumBackendType,
    /// Fail on warnings instead of passing them through.
    pub strict: bool,
    /// Allow circuits without measurements (state-vector jobs).
    pub allow_no_measurements: bool,
}

impl Default for CompilerConfig {
    fn default() -> Self {
        Self {
            target: QuantumBackendType::Simulation,
            strict: false,
            allow_no_measurements: true,
        }
    }
}

/// Stateless quantum compiler.
#[derive(Debug, Clone, Default)]
pub struct QuantumCompiler {
    config: CompilerConfig,
}

impl QuantumCompiler {
    pub fn new(config: CompilerConfig) -> Self {
        Self { config }
    }

    pub fn config(&self) -> &CompilerConfig {
        &self.config
    }

    /// Lower a high-level circuit to backend-neutral IR.
    pub fn to_ir(&self, circuit: &QuantumCircuit) -> Result<IntermediateRepresentation> {
        if circuit.num_qubits == 0 || circuit.num_qubits > MAX_SUPPORTED_QUBITS {
            return Err(QuantumError::InvalidQubitCount {
                expected: MAX_SUPPORTED_QUBITS,
                got: circuit.num_qubits,
            });
        }
        let mut ops = Vec::with_capacity(circuit.gates.len());
        let mut warnings = Vec::new();

        for gate in &circuit.gates {
            match gate {
                QuantumGate::Hadamard => {
                    // Single-qubit gates in this circuit model are applied
                    // without embedded targets; record a generic single-qubit op.
                    // Target resolution happens at schedule/execution time.
                    ops.push(IrOperation::new("h", vec![0]));
                }
                QuantumGate::PauliX => ops.push(IrOperation::new("x", vec![0])),
                QuantumGate::PauliY => ops.push(IrOperation::new("y", vec![0])),
                QuantumGate::PauliZ => ops.push(IrOperation::new("z", vec![0])),
                QuantumGate::Phase { theta } => {
                    ops.push(IrOperation::new("s", vec![0]).with_angle(*theta));
                }
                QuantumGate::RotationX { theta } => {
                    ops.push(IrOperation::new("rx", vec![0]).with_angle(*theta));
                }
                QuantumGate::RotationY { theta } => {
                    ops.push(IrOperation::new("ry", vec![0]).with_angle(*theta));
                }
                QuantumGate::RotationZ { theta } => {
                    ops.push(IrOperation::new("rz", vec![0]).with_angle(*theta));
                }
                QuantumGate::CNOT { control, target } => {
                    ops.push(IrOperation::new("cx", vec![*control, *target]));
                }
                QuantumGate::ControlledZ { control, target } => {
                    ops.push(IrOperation::new("cz", vec![*control, *target]));
                }
                QuantumGate::Swap { qubit1, qubit2 } => {
                    ops.push(IrOperation::new("swap", vec![*qubit1, *qubit2]));
                }
                QuantumGate::Toffoli {
                    control1,
                    control2,
                    target,
                } => {
                    ops.push(IrOperation {
                        op: "ccx".to_string(),
                        qubits: vec![*control1, *control2, *target],
                        angle: None,
                        note: Some("toffoli; decompose on backends without ccx".to_string()),
                    });
                    if self.config.target == QuantumBackendType::Physical {
                        warnings.push(
                            "ccx requires physical-backend decomposition; no hardware assumed"
                                .to_string(),
                        );
                    }
                }
                QuantumGate::Measurement { qubit } => {
                    let mut op = IrOperation::new("measure", vec![*qubit]);
                    op.note = Some(format!("c[{qubit}]"));
                    ops.push(op);
                }
            }
        }

        // Validate qubit indices for ops that carry explicit indices.
        for op in &ops {
            for q in &op.qubits {
                if *q >= circuit.num_qubits {
                    return Err(QuantumError::InvalidQubitIndex {
                        index: *q,
                        max: circuit.num_qubits,
                    });
                }
            }
        }

        Ok(IntermediateRepresentation {
            source: circuit.name.clone(),
            num_qubits: circuit.num_qubits,
            ops,
            warnings,
        })
    }

    /// Lower IR to a backend-specific artifact.
    pub fn to_backend(&self, ir: &IntermediateRepresentation) -> Result<CompiledCircuit> {
        // Physical targets are capability-gated: refuse to fabricate payloads.
        if self.config.target == QuantumBackendType::Physical {
            return Err(QuantumError::BackendNotAvailable(
                "Physical QPU lowering requires a documented hardware interface; adapter is disabled"
                    .to_string(),
            ));
        }
        if self.config.target == QuantumBackendType::Remote {
            return Err(QuantumError::BackendNotAvailable(
                "Remote QPU lowering requires configured endpoint/credentials; no default remote"
                    .to_string(),
            ));
        }

        let payload = serde_json::json!({
            "backend": format!("{:?}", self.config.target),
            "source": ir.source,
            "num_qubits": ir.num_qubits,
            "ops": ir.ops,
            "warnings": ir.warnings,
        });
        let payload_str = serde_json::to_string(&payload).map_err(QuantumError::from)?;

        Ok(CompiledCircuit {
            source: ir.source.clone(),
            backend: self.config.target,
            ir: ir.clone(),
            payload: payload_str,
            simulation_only: matches!(
                self.config.target,
                QuantumBackendType::Simulation | QuantumBackendType::Emulation
            ),
        })
    }

    /// Full pipeline: circuit -> IR -> backend artifact.
    pub fn compile(&self, circuit: &QuantumCircuit) -> Result<CompiledCircuit> {
        let ir = self.to_ir(circuit)?;
        if self.config.strict && !ir.warnings.is_empty() {
            return Err(QuantumError::UnsupportedGate(ir.warnings.join("; ")));
        }
        self.to_backend(&ir)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bell_circuit() -> QuantumCircuit {
        let mut c = QuantumCircuit::new("bell", 2).unwrap();
        c.add_gate(QuantumGate::Hadamard).unwrap();
        c.add_gate(QuantumGate::CNOT {
            control: 0,
            target: 1,
        })
        .unwrap();
        c.add_gate(QuantumGate::Measurement { qubit: 0 }).unwrap();
        c
    }

    #[test]
    fn test_to_ir_bell() {
        let compiler = QuantumCompiler::new(CompilerConfig::default());
        let ir = compiler.to_ir(&bell_circuit()).unwrap();
        assert_eq!(ir.num_qubits, 2);
        assert_eq!(ir.ops.len(), 3);
        assert_eq!(ir.ops[0].op, "h");
        assert_eq!(ir.ops[1].op, "cx");
        assert!(ir.has_measurements());
    }

    #[test]
    fn test_compile_simulation() {
        let compiler = QuantumCompiler::new(CompilerConfig::default());
        let compiled = compiler.compile(&bell_circuit()).unwrap();
        assert_eq!(compiled.backend, QuantumBackendType::Simulation);
        assert!(compiled.simulation_only);
        assert!(compiled.payload.contains("bell"));
    }

    #[test]
    fn test_compile_physical_gated() {
        let compiler = QuantumCompiler::new(CompilerConfig {
            target: QuantumBackendType::Physical,
            strict: false,
            allow_no_measurements: true,
        });
        let err = compiler.compile(&bell_circuit()).unwrap_err();
        assert!(matches!(err, QuantumError::BackendNotAvailable(_)));
    }

    #[test]
    fn test_rejects_oversized_circuit() {
        let compiler = QuantumCompiler::new(CompilerConfig::default());
        let c = QuantumCircuit {
            name: "big".to_string(),
            num_qubits: 99,
            num_classical_bits: 99,
            gates: vec![],
            metadata: crate::circuit::CircuitMetadata {
                description: None,
                depth: 0,
                single_qubit_gate_count: 0,
                two_qubit_gate_count: 0,
                measurement_count: 0,
            },
        };
        assert!(compiler.to_ir(&c).is_err());
    }
}