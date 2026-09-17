use quantum_runtime::*;
use std::f64::consts::PI;

#[test]
fn test_quantum_module_integration() {
    // Test 1: Create simulator with 3 qubits
    let simulator = QuantumSimulator::new(3).unwrap();
    assert_eq!(simulator.num_qubits(), 3);

    // State should be |000⟩
    let state = simulator.state();
    assert_eq!(state[0], Complex::one());
}

#[test]
fn test_circuit_creation_and_gates() {
    // Create a Bell state circuit
    let mut circuit = QuantumCircuit::new("bell-state", 2).unwrap();

    // Add Hadamard on first qubit
    circuit.add_gate(QuantumGate::Hadamard).unwrap();
    assert_eq!(circuit.gate_count(), 1);

    // Add CNOT
    circuit
        .add_gate(QuantumGate::CNOT {
            control: 0,
            target: 1,
        })
        .unwrap();
    assert_eq!(circuit.gate_count(), 2);

    // Add measurements
    circuit
        .add_gate(QuantumGate::Measurement { qubit: 0 })
        .unwrap();
    circuit
        .add_gate(QuantumGate::Measurement { qubit: 1 })
        .unwrap();

    assert!(circuit.has_measurements());
    assert_eq!(circuit.metadata.measurement_count, 2);
}

#[test]
fn test_simulator_single_qubit_gates() {
    let mut sim = QuantumSimulator::new(1).unwrap();

    // Apply Hadamard: should create equal superposition
    sim.apply_single_qubit_gate(&QuantumGate::Hadamard, 0)
        .unwrap();

    let probs = sim.probabilities();
    assert_eq!(probs.len(), 2);
    assert!((probs[0] - 0.5).abs() < 1e-10);
    assert!((probs[1] - 0.5).abs() < 1e-10);
}

#[test]
fn test_pauli_x_gate() {
    let mut sim = QuantumSimulator::new(1).unwrap();

    // Apply Pauli-X (flip)
    sim.apply_single_qubit_gate(&QuantumGate::PauliX, 0)
        .unwrap();

    // Should be in |1⟩
    let probs = sim.probabilities();
    assert!((probs[1] - 1.0).abs() < 1e-10);
}

#[test]
fn test_cnot_gate() {
    let mut sim = QuantumSimulator::new(2).unwrap();

    // Put control in |1⟩
    sim.apply_single_qubit_gate(&QuantumGate::PauliX, 0)
        .unwrap();

    // Apply CNOT
    sim.apply_cnot(0, 1).unwrap();

    // Should be in |11⟩
    let probs = sim.probabilities();
    assert!((probs[3] - 1.0).abs() < 1e-10); // |11⟩ = index 3
}

#[test]
fn test_bell_state_preparation() {
    // Prepare Bell state |Φ+⟩ = (|00⟩ + |11⟩)/√2
    let mut sim = QuantumSimulator::new(2).unwrap();

    // Hadamard on first qubit
    sim.apply_single_qubit_gate(&QuantumGate::Hadamard, 0)
        .unwrap();

    // CNOT
    sim.apply_cnot(0, 1).unwrap();

    // Check probabilities
    let probs = sim.probabilities();
    assert!((probs[0] - 0.5).abs() < 1e-10); // |00⟩
    assert!((probs[3] - 0.5).abs() < 1e-10); // |11⟩
    assert!(probs[1].abs() < 1e-10); // |01⟩
    assert!(probs[2].abs() < 1e-10); // |10⟩
}

#[test]
fn test_simulator_backend() {
    let mut backend = SimulatorBackend::new();

    // Test capabilities
    let cap = backend.capabilities();
    assert_eq!(cap.qubits, 20);

    // Allocate 3 qubits
    let alloc = backend
        .allocate(crate::backend::AllocationRequest {
            qubits: 3,
            logical_qubits: 0,
        })
        .unwrap();
    assert_eq!(alloc.qubits, 3);

    // Execute circuit
    let result = backend
        .execute(crate::backend::QuantumCircuitInfo {
            name: "test".to_string(),
            qubits: 3,
            gates: 5,
        })
        .unwrap();
    assert_eq!(result.status, "completed");

    // Measure
    let measure_result = backend
        .measure(crate::backend::MeasurementRequest {
            register_id: "qr-0".to_string(),
            shots: 100,
        })
        .unwrap();
    assert_eq!(measure_result.bits.len(), 100);
}

#[test]
fn test_error_handling() {
    // Try to create circuit with too many qubits
    let result = QuantumCircuit::new("test", 25);
    assert!(result.is_err());

    // Try to apply gate to invalid qubit
    let mut sim = QuantumSimulator::new(2).unwrap();
    let result = sim.apply_single_qubit_gate(&QuantumGate::Hadamard, 5);
    assert!(result.is_err());
}

#[test]
fn test_complex_number_operations() {
    let z1 = Complex::new(1.0, 2.0);
    let z2 = Complex::new(3.0, 4.0);

    // Addition
    let sum = z1 + z2;
    assert_eq!(sum.real, 4.0);
    assert_eq!(sum.imag, 6.0);

    // Multiplication
    let prod = z1 * z2;
    assert!((prod.real - (-5.0)).abs() < 1e-10);
    assert!((prod.imag - 10.0).abs() < 1e-10);

    // Magnitude
    let mag = z1.magnitude();
    assert!((mag - (5.0_f64.sqrt())).abs() < 1e-10);
}

#[test]
fn test_circuit_validation() {
    let mut circuit = QuantumCircuit::new("test", 2).unwrap();

    // Try to add gate to non-existent qubit
    let result = circuit.add_gate(QuantumGate::Measurement { qubit: 5 });
    assert!(result.is_err());

    // Try to add CNOT with same control and target
    let result = circuit.add_gate(QuantumGate::CNOT {
        control: 0,
        target: 0,
    });
    assert!(result.is_err());
}

#[test]
fn test_rotation_gates() {
    let mut sim = QuantumSimulator::new(1).unwrap();

    // Rotation by π around X should flip the qubit
    let gate = QuantumGate::RotationX { theta: PI };
    sim.apply_single_qubit_gate(&gate, 0).unwrap();

    // Should be close to |1⟩
    assert!(sim.state()[1].magnitude() > 0.99);
}

#[test]
fn test_circuit_depth_calculation() {
    let mut circuit = QuantumCircuit::new("test", 3).unwrap();

    circuit.add_gate(QuantumGate::Hadamard).unwrap();
    let depth_1 = circuit.metadata.depth;

    circuit.add_gate(QuantumGate::Hadamard).unwrap();
    circuit.add_gate(QuantumGate::Hadamard).unwrap();
    let depth_n = circuit.metadata.depth;

    assert!(depth_n > depth_1);
}

#[test]
fn test_state_vector_normalization() {
    let mut sim = QuantumSimulator::new(2).unwrap();
    assert!(sim.is_normalized());

    // Apply gates
    sim.apply_single_qubit_gate(&QuantumGate::Hadamard, 0)
        .unwrap();
    sim.apply_cnot(0, 1).unwrap();

    // State should remain normalized
    assert!(sim.is_normalized());
}

#[test]
fn test_simulator_reset() {
    let mut sim = QuantumSimulator::new(2).unwrap();

    // Modify state
    sim.apply_single_qubit_gate(&QuantumGate::PauliX, 0)
        .unwrap();
    sim.apply_single_qubit_gate(&QuantumGate::PauliX, 1)
        .unwrap();

    // Reset
    sim.reset();

    // Should be back in |00⟩
    assert_eq!(sim.state()[0], Complex::one());
}

#[test]
fn test_circuit_summary() {
    let mut circuit = QuantumCircuit::new("grover", 3)
        .unwrap()
        .with_description("Grover's algorithm circuit");

    circuit.add_gate(QuantumGate::Hadamard).unwrap();
    circuit
        .add_gate(QuantumGate::CNOT {
            control: 0,
            target: 1,
        })
        .unwrap();

    let summary = circuit.summary();
    assert!(summary.contains("grover"));
    assert!(summary.contains("3 qubits"));
}

#[test]
fn test_backend_health() {
    let backend = SimulatorBackend::new();
    let health = backend.health();
    assert!(health.available);
    assert_eq!(health.status, "operational");
}

#[test]
fn test_hadamard_properties() {
    let gate = QuantumGate::Hadamard;
    assert!(gate.is_single_qubit());
    assert!(!gate.is_two_qubit());
    assert_eq!(gate.name(), "H");

    // H matrix should be:
    // [[1/√2, 1/√2],
    //  [1/√2, -1/√2]]
    if let Some(matrix) = gate.single_qubit_matrix() {
        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        assert!((matrix[0][0].real - inv_sqrt2).abs() < 1e-10);
        assert!((matrix[0][1].real - inv_sqrt2).abs() < 1e-10);
        assert!((matrix[1][0].real - inv_sqrt2).abs() < 1e-10);
        assert!((matrix[1][1].real + inv_sqrt2).abs() < 1e-10);
    }
}

#[test]
fn test_toffoli_three_qubit_gate() {
    let mut circuit = QuantumCircuit::new("ccx-test", 3).unwrap();

    let toffoli = QuantumGate::Toffoli {
        control1: 0,
        control2: 1,
        target: 2,
    };

    assert!(toffoli.is_three_qubit());
    assert_eq!(toffoli.name(), "CCX");
    circuit.add_gate(toffoli).unwrap();
}

#[test]
fn test_swap_gate() {
    let gate = QuantumGate::Swap {
        qubit1: 0,
        qubit2: 1,
    };
    assert!(gate.is_two_qubit());
    assert_eq!(gate.name(), "SWAP");
}

#[test]
fn test_phase_gate() {
    let gate = QuantumGate::Phase { theta: PI / 2.0 };
    assert!(gate.is_single_qubit());
    if let Some(matrix) = gate.single_qubit_matrix() {
        // Phase gate: [[1, 0], [0, e^(iθ)]]
        assert_eq!(matrix[0][0], Complex::one());
        assert_eq!(matrix[0][1], Complex::zero());
    }
}
