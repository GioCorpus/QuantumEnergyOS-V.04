//! Device-level intermediate representation (Phase 3, §10).
//!
//! Pipeline implemented across the stack:
//!
//! ```text
//! QuantumCircuit                 (crates/quantum-runtime)
//!      -> IntermediateRepresentation   (runtime IR, carries qubit indices)
//!      -> QuantumIR / QuantumOperation  (device-level IR, this module)
//!      -> backend lowering              (simulator / accelerator / experimental QPU)
//! ```
//!
//! CLASSIFICATION: REAL (data model + lowering) / backend execution is
//! classified by the device that consumes it.
//!
//! The device-level IR is deliberately small and explicit. Rich gates that are
//! not part of it lower to [`QuantumOperation::Custom`] instead of being
//! silently rewritten, so a backend can honestly reject them.

use serde::{Deserialize, Serialize};

use quantum_runtime::{
    CompilerConfig, IntermediateRepresentation, IrOperation, QuantumCircuit, QuantumCompiler,
    QuantumGate,
};

use crate::error::{QuantumError, Result};

/// Identifier of a qubit inside a device-level circuit.
pub type QubitId = u32;

/// Device-level quantum operation (§10).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum QuantumOperation {
    /// Pauli-X.
    X(QubitId),
    /// Pauli-Y.
    Y(QubitId),
    /// Pauli-Z.
    Z(QubitId),
    /// Hadamard.
    H(QubitId),
    /// Controlled-NOT (control, target).
    CNOT(QubitId, QubitId),
    /// Measurement in the computational basis.
    Measure(QubitId),
    /// Reset a qubit to |0>.
    Reset(QubitId),
    /// Operation outside the minimal device op set (for example a rotation).
    /// Backends must reject unknown labels rather than approximate them.
    Custom(String),
}

impl QuantumOperation {
    /// Stable operation name used in telemetry and error messages.
    pub fn name(&self) -> String {
        match self {
            QuantumOperation::X(_) => "x".to_string(),
            QuantumOperation::Y(_) => "y".to_string(),
            QuantumOperation::Z(_) => "z".to_string(),
            QuantumOperation::H(_) => "h".to_string(),
            QuantumOperation::CNOT(_, _) => "cx".to_string(),
            QuantumOperation::Measure(_) => "measure".to_string(),
            QuantumOperation::Reset(_) => "reset".to_string(),
            QuantumOperation::Custom(label) => label.clone(),
        }
    }

    /// Qubits referenced by the operation, in argument order.
    pub fn qubits(&self) -> Vec<QubitId> {
        match self {
            QuantumOperation::X(q)
            | QuantumOperation::Y(q)
            | QuantumOperation::Z(q)
            | QuantumOperation::H(q)
            | QuantumOperation::Measure(q)
            | QuantumOperation::Reset(q) => vec![*q],
            QuantumOperation::CNOT(control, target) => vec![*control, *target],
            QuantumOperation::Custom(_) => Vec::new(),
        }
    }

    /// True when the operation samples a measurement outcome.
    pub fn is_measurement(&self) -> bool {
        matches!(self, QuantumOperation::Measure(_))
    }

    /// True when the operation is part of the minimal device op set.
    pub fn is_native(&self) -> bool {
        !matches!(self, QuantumOperation::Custom(_))
    }
}

/// Backend-neutral, device-level circuit.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct QuantumIR {
    /// Circuit name (propagated from the source circuit).
    pub name: String,
    /// Number of qubits in the register.
    pub num_qubits: usize,
    /// Ordered operations.
    pub operations: Vec<QuantumOperation>,
    /// Lowering warnings and limitations recorded during translation.
    pub warnings: Vec<String>,
}

impl QuantumIR {
    /// Create an empty IR container.
    pub fn new(name: impl Into<String>, num_qubits: usize) -> Self {
        Self {
            name: name.into(),
            num_qubits,
            operations: Vec::new(),
            warnings: Vec::new(),
        }
    }

    /// Append an operation.
    pub fn push(&mut self, operation: QuantumOperation) -> &mut Self {
        self.operations.push(operation);
        self
    }

    /// Builder-style append.
    pub fn with_operation(mut self, operation: QuantumOperation) -> Self {
        self.operations.push(operation);
        self
    }

    /// Record a lowering warning.
    pub fn warn(&mut self, warning: impl Into<String>) -> &mut Self {
        self.warnings.push(warning.into());
        self
    }

    /// Number of operations.
    pub fn op_count(&self) -> usize {
        self.operations.len()
    }

    /// Number of measurement operations.
    pub fn measurement_count(&self) -> usize {
        self.operations
            .iter()
            .filter(|op| op.is_measurement())
            .count()
    }

    /// True when at least one measurement operation is present.
    pub fn has_measurements(&self) -> bool {
        self.measurement_count() > 0
    }

    /// Sorted, de-duplicated qubits that are measured.
    pub fn measured_qubits(&self) -> Vec<QubitId> {
        let mut qubits: Vec<QubitId> = self
            .operations
            .iter()
            .filter_map(|op| match op {
                QuantumOperation::Measure(q) => Some(*q),
                _ => None,
            })
            .collect();
        qubits.sort_unstable();
        qubits.dedup();
        qubits
    }

    /// True when a measurement appears before further operations, which the
    /// reference device cannot model (no classical feed-forward).
    pub fn has_mid_circuit_measurement(&self) -> bool {
        let mut seen_measurement = false;
        for operation in &self.operations {
            if operation.is_measurement() {
                seen_measurement = true;
            } else if seen_measurement {
                return true;
            }
        }
        false
    }

    /// Validate qubit bounds and two-qubit argument constraints.
    pub fn validate(&self) -> Result<()> {
        if self.num_qubits == 0 {
            return Err(QuantumError::InvalidQubitCount {
                expected: 1,
                got: 0,
            });
        }

        let max = self.num_qubits - 1;
        for operation in &self.operations {
            for qubit in operation.qubits() {
                if qubit as usize > max {
                    return Err(QuantumError::InvalidQubitIndex {
                        index: qubit as usize,
                        max,
                    });
                }
            }
            if let QuantumOperation::CNOT(control, target) = operation {
                if control == target {
                    return Err(QuantumError::InvalidGateParameters(
                        "control and target must be different".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

// ---------------------------------------------------------------------------
// Lowering: runtime representations -> device-level IR
// ---------------------------------------------------------------------------

/// Qubit argument at `index`, defaulting to 0 when absent (see notes below).
fn qubit_at(operation: &IrOperation, index: usize) -> QubitId {
    operation.qubits.get(index).copied().unwrap_or(0) as QubitId
}

/// First qubit argument of an operation.
fn first_qubit(operation: &IrOperation) -> QubitId {
    qubit_at(operation, 0)
}

fn qubits_text(operation: &IrOperation) -> String {
    operation
        .qubits
        .iter()
        .map(|q| q.to_string())
        .collect::<Vec<_>>()
        .join(",")
}

fn describe(operation: &IrOperation) -> String {
    match operation.angle {
        Some(angle) => format!(
            "{}({}; angle={:.6})",
            operation.op,
            qubits_text(operation),
            angle
        ),
        None => format!("{}({})", operation.op, qubits_text(operation)),
    }
}

/// Map a runtime IR operation onto the minimal device-level op set.
///
/// Operations with parameters that the device IR does not model (phase and
/// rotation gates, CZ, SWAP, Toffoli) become [`QuantumOperation::Custom`] so
/// that the consuming backend rejects them explicitly instead of approximating.
fn lower_operation(operation: &IrOperation) -> QuantumOperation {
    match operation.op.as_str() {
        "h" | "hadamard" => QuantumOperation::H(first_qubit(operation)),
        "x" => QuantumOperation::X(first_qubit(operation)),
        "y" => QuantumOperation::Y(first_qubit(operation)),
        "z" => QuantumOperation::Z(first_qubit(operation)),
        "cx" | "cnot" => QuantumOperation::CNOT(qubit_at(operation, 0), qubit_at(operation, 1)),
        "measure" => QuantumOperation::Measure(first_qubit(operation)),
        "reset" => QuantumOperation::Reset(first_qubit(operation)),
        _ => QuantumOperation::Custom(describe(operation)),
    }
}

/// Lower runtime IR (crates/quantum-runtime) into device-level IR.
///
/// The runtime compiler resolves targets for gates that carry them (CX, CZ,
/// SWAP, CCX, measurement). Single-qubit gates in the runtime circuit model are
/// target-less and are recorded against qubit 0 by the runtime compiler; this
/// limitation is propagated into `warnings` so it is never hidden.
pub fn lower_runtime_ir(ir: &IntermediateRepresentation) -> Result<QuantumIR> {
    let mut device_ir = QuantumIR::new(ir.source.clone(), ir.num_qubits);

    for warning in &ir.warnings {
        device_ir.warn(format!("runtime lowering warning: {}", warning));
    }

    let mut target_less_single_qubit_ops = 0usize;
    for operation in &ir.ops {
        if operation.qubits.len() == 1
            && matches!(
                operation.op.as_str(),
                "h" | "x" | "y" | "z" | "s" | "rx" | "ry" | "rz"
            )
        {
            target_less_single_qubit_ops += 1;
        }
        device_ir.push(lower_operation(operation));
    }

    if target_less_single_qubit_ops > 0 && ir.num_qubits > 1 {
        device_ir.warn(format!(
            "{} single-qubit operation(s) carry a default target (qubit 0): the runtime circuit model records single-qubit gates without an explicit target",
            target_less_single_qubit_ops
        ));
    }

    device_ir.validate()?;
    Ok(device_ir)
}

/// Lower a high-level circuit directly, using the runtime compiler defaults.
pub fn lower_circuit_to_ir(circuit: &QuantumCircuit) -> Result<QuantumIR> {
    let compiler = QuantumCompiler::new(CompilerConfig::default());
    let runtime_ir = compiler.to_ir(circuit)?;
    lower_runtime_ir(&runtime_ir)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn bell_circuit() -> QuantumCircuit {
        let mut circuit = QuantumCircuit::new("bell", 2).unwrap();
        circuit.add_gate(QuantumGate::Hadamard).unwrap();
        circuit
            .add_gate(QuantumGate::CNOT {
                control: 0,
                target: 1,
            })
            .unwrap();
        circuit
            .add_gate(QuantumGate::Measurement { qubit: 0 })
            .unwrap();
        circuit
            .add_gate(QuantumGate::Measurement { qubit: 1 })
            .unwrap();
        circuit
    }

    #[test]
    fn test_operation_names_and_qubits() {
        assert_eq!(QuantumOperation::H(3).name(), "h");
        assert_eq!(QuantumOperation::CNOT(0, 1).name(), "cx");
        assert_eq!(QuantumOperation::CNOT(0, 1).qubits(), vec![0, 1]);
        assert_eq!(QuantumOperation::Measure(2).qubits(), vec![2]);
        assert!(QuantumOperation::Measure(0).is_measurement());
        assert!(QuantumOperation::Custom("rx(0; angle=1.0)".to_string())
            .qubits()
            .is_empty());
        assert!(!QuantumOperation::Custom("rx".to_string()).is_native());
        assert!(QuantumOperation::X(0).is_native());
    }

    #[test]
    fn test_ir_container_helpers() {
        let mut ir = QuantumIR::new("demo", 2);
        ir.push(QuantumOperation::H(0))
            .push(QuantumOperation::Measure(0))
            .warn("note");

        assert_eq!(ir.op_count(), 2);
        assert_eq!(ir.measurement_count(), 1);
        assert!(ir.has_measurements());
        assert_eq!(ir.measured_qubits(), vec![0]);
        assert_eq!(ir.warnings, vec!["note".to_string()]);

        let built = QuantumIR::new("built", 1).with_operation(QuantumOperation::X(0));
        assert_eq!(built.op_count(), 1);
    }

    #[test]
    fn test_validate_rejects_bad_ir() {
        let empty = QuantumIR::new("empty", 0);
        assert!(matches!(
            empty.validate(),
            Err(QuantumError::InvalidQubitCount { .. })
        ));

        let mut out_of_range = QuantumIR::new("bad", 2);
        out_of_range.push(QuantumOperation::X(5));
        assert!(matches!(
            out_of_range.validate(),
            Err(QuantumError::InvalidQubitIndex { .. })
        ));

        let mut self_cnot = QuantumIR::new("bad", 2);
        self_cnot.push(QuantumOperation::CNOT(1, 1));
        assert!(matches!(
            self_cnot.validate(),
            Err(QuantumError::InvalidGateParameters(_))
        ));
    }

    #[test]
    fn test_mid_circuit_measurement_detection() {
        let mut early = QuantumIR::new("early", 2);
        early.push(QuantumOperation::Measure(0));
        early.push(QuantumOperation::X(1));
        assert!(early.has_mid_circuit_measurement());

        let mut late = QuantumIR::new("late", 2);
        late.push(QuantumOperation::X(1));
        late.push(QuantumOperation::Measure(0));
        assert!(!late.has_mid_circuit_measurement());
    }

    #[test]
    fn test_lower_bell_circuit_to_device_ir() {
        let ir = lower_circuit_to_ir(&bell_circuit()).unwrap();
        assert_eq!(ir.name, "bell");
        assert_eq!(ir.num_qubits, 2);
        assert_eq!(
            ir.operations,
            vec![
                QuantumOperation::H(0),
                QuantumOperation::CNOT(0, 1),
                QuantumOperation::Measure(0),
                QuantumOperation::Measure(1),
            ]
        );
        assert_eq!(ir.measured_qubits(), vec![0, 1]);
        assert!(!ir.warnings.is_empty());
    }

    #[test]
    fn test_parameterised_gate_lowers_to_custom() {
        let mut circuit = QuantumCircuit::new("rot", 1).unwrap();
        circuit
            .add_gate(QuantumGate::RotationX {
                theta: std::f64::consts::FRAC_PI_2,
            })
            .unwrap();

        let ir = lower_circuit_to_ir(&circuit).unwrap();
        assert_eq!(ir.operations.len(), 1);
        match &ir.operations[0] {
            QuantumOperation::Custom(label) => {
                assert!(label.starts_with("rx("));
                assert!(label.contains("angle="));
            }
            other => panic!("expected Custom operation, got {:?}", other),
        }
    }

    #[test]
    fn test_lowering_rejects_invalid_qubit_index() {
        let ir = IntermediateRepresentation {
            source: "broken".to_string(),
            num_qubits: 2,
            ops: vec![IrOperation::new("cx", vec![0, 9])],
            warnings: Vec::new(),
        };
        assert!(lower_runtime_ir(&ir).is_err());
    }
}
