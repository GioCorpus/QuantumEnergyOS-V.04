//! Phase 4.3 golden + scientific validation (SIMULATION/MODEL).
//! Bell/H/X/parity/tetron/unitarity/normalization checks.

use quantum_runtime::{verify_anticommutation, Complex, QuantumGate, QuantumSimulator, Tetron};

#[test]
fn golden_h_creates_balanced_superposition() {
    let mut sim = QuantumSimulator::new(1).unwrap();
    sim.apply_single_qubit_gate(&QuantumGate::Hadamard, 0)
        .unwrap();
    let p = sim.probabilities();
    assert!((p[0] - 0.5).abs() < 1e-10);
    assert!((p[1] - 0.5).abs() < 1e-10);
    assert!(sim.is_normalized());
}

#[test]
fn golden_x_flips_to_one() {
    let mut sim = QuantumSimulator::new(1).unwrap();
    sim.apply_single_qubit_gate(&QuantumGate::PauliX, 0)
        .unwrap();
    assert_eq!(sim.state()[1], Complex::one());
}

#[test]
fn golden_hh_returns_to_zero() {
    let mut sim = QuantumSimulator::new(1).unwrap();
    sim.apply_single_qubit_gate(&QuantumGate::Hadamard, 0)
        .unwrap();
    sim.apply_single_qubit_gate(&QuantumGate::Hadamard, 0)
        .unwrap();
    assert!((sim.state()[0].real - 1.0).abs() < 1e-10);
    assert!(sim.state()[1].magnitude() < 1e-10);
}

#[test]
fn golden_bell_state_correlations() {
    let mut sim = QuantumSimulator::new(2).unwrap();
    sim.apply_single_qubit_gate(&QuantumGate::Hadamard, 0)
        .unwrap();
    sim.apply_cnot(0, 1).unwrap();
    let p = sim.probabilities();
    assert!((p[0] - 0.5).abs() < 1e-10);
    assert!((p[3] - 0.5).abs() < 1e-10);
    assert!(p[1].abs() < 1e-10 && p[2].abs() < 1e-10);
}

#[test]
fn golden_single_qubit_unitaries_preserve_norm() {
    for gate in [
        QuantumGate::Hadamard,
        QuantumGate::PauliX,
        QuantumGate::PauliY,
        QuantumGate::PauliZ,
        QuantumGate::SGate,
        QuantumGate::TGate,
    ] {
        let m = gate.single_qubit_matrix().unwrap();
        // U^dagger U == I check.
        for i in 0..2 {
            for j in 0..2 {
                let mut re = 0.0;
                let mut im = 0.0;
                for (k, gate_row) in m.iter().enumerate() {
                    let a_re = gate_row[i].real;
                    let a_im = -gate_row[i].imag;
                    re += a_re * m[k][j].real - a_im * m[k][j].imag;
                    im += a_re * m[k][j].imag + a_im * m[k][j].real;
                }
                let (e_re, e_im) = if i == j { (1.0, 0.0) } else { (0.0, 0.0) };
                assert!((re - e_re).abs() < 1e-12, "{gate:?} unitarity");
                assert!((im - e_im).abs() < 1e-12, "{gate:?} unitarity");
            }
        }
    }
}

#[test]
fn golden_majorana_algebra_and_tetron() {
    assert!(verify_anticommutation());
    let t = Tetron::new("golden", 0);
    assert!(t.validate().is_ok());
    assert_eq!(t.mode_ids(), [0, 1, 2, 3]);
}
