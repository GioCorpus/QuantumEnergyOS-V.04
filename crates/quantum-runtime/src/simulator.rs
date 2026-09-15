use crate::error::{QuantumError, Result};
use crate::gates::{Complex, QuantumGate};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::f64::consts::PI;

/// State vector quantum simulator.
///
/// CLASSIFICATION: SIMULATION
/// This is a classical state-vector simulator. It does not simulate physical
/// hardware. Measurement outcomes are generated using a pseudorandom number
/// generator, not derived from arbitrary mathematical formulas.
///
/// Represents the state of n qubits using a 2^n dimensional state vector.
/// Each element is a complex amplitude corresponding to a computational basis state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumSimulator {
    state: Vec<Complex>,
    num_qubits: usize,
    /// Reusable scratch buffer to avoid per-gate allocations.
    scratch: Vec<Complex>,
}

impl QuantumSimulator {
    /// Create a new simulator with n qubits, initialized to |0⟩^n.
    pub fn new(num_qubits: usize) -> Result<Self> {
        if num_qubits > 20 {
            return Err(QuantumError::InvalidQubitCount {
                expected: 20,
                got: num_qubits,
            });
        }

        let dim = 1 << num_qubits;
        let mut state = vec![Complex::zero(); dim];
        state[0] = Complex::one();
        let scratch = vec![Complex::zero(); dim];

        Ok(Self { state, num_qubits, scratch })
    }

    /// Get the number of qubits.
    pub fn num_qubits(&self) -> usize {
        self.num_qubits
    }

    /// Get the current state vector.
    pub fn state(&self) -> &[Complex] {
        &self.state
    }

    /// Apply a single-qubit gate to a specific qubit.
    pub fn apply_single_qubit_gate(&mut self, gate: &QuantumGate, target: usize) -> Result<()> {
        if target >= self.num_qubits {
            return Err(QuantumError::InvalidQubitIndex {
                index: target,
                max: self.num_qubits - 1,
            });
        }

        if !gate.is_single_qubit() {
            return Err(QuantumError::UnsupportedGate(
                "Expected single-qubit gate".to_string(),
            ));
        }

        match gate {
            QuantumGate::Measurement { qubit } => {
                self.measure_qubit(*qubit)?;
            }
            _ => {
                if let Some(matrix) = gate.single_qubit_matrix() {
                    self.apply_single_qubit_matrix(&matrix, target)?;
                }
            }
        }

        Ok(())
    }

    fn apply_single_qubit_matrix(
        &mut self,
        matrix: &[[Complex; 2]; 2],
        target: usize,
    ) -> Result<()> {
        let dim = 1 << self.num_qubits;

        // Reuse the scratch buffer instead of allocating
        self.scratch.fill(Complex::zero());

        for i in 0..dim {
            let target_bit = (i >> target) & 1;

            if target_bit == 0 {
                let idx_1 = i | (1 << target);
                self.scratch[i] = matrix[0][0] * self.state[i] + matrix[0][1] * self.state[idx_1];
            } else {
                let idx_0 = i & !(1 << target);
                self.scratch[i] = matrix[1][0] * self.state[idx_0] + matrix[1][1] * self.state[i];
            }
        }

        std::mem::swap(&mut self.state, &mut self.scratch);
        Ok(())
    }

    /// Apply a CNOT gate (two-qubit gate).
    pub fn apply_cnot(&mut self, control: usize, target: usize) -> Result<()> {
        if control >= self.num_qubits || target >= self.num_qubits {
            return Err(QuantumError::InvalidQubitIndex {
                index: control.max(target),
                max: self.num_qubits - 1,
            });
        }

        if control == target {
            return Err(QuantumError::InvalidGateParameters(
                "Control and target must be different".to_string(),
            ));
        }

        let dim = 1 << self.num_qubits;
        for i in 0..dim {
            if ((i >> control) & 1) == 1 {
                let flipped = i ^ (1 << target);
                self.state.swap(i, flipped);
            }
        }

        Ok(())
    }

    /// Measure a qubit, returning 0 or 1 with appropriate probabilities.
    ///
    /// Uses a pseudorandom number generator seeded from system entropy to
    /// determine the outcome.
    /// The outcome probabilities are determined by the state-vector amplitudes.
    pub fn measure_qubit(&mut self, qubit: usize) -> Result<u8> {
        let mut rng = rand::thread_rng();
        self.measure_qubit_with_rng(qubit, &mut rng)
    }

    /// Measure a qubit using a caller-supplied random number generator.
    ///
    /// CLASSIFICATION: SIMULATION
    ///
    /// Passing a deterministically seeded generator (for example
    /// `StdRng::seed_from_u64(42)`) makes shot outcomes reproducible, which is
    /// required for reproducible simulation runs and for CI validation of the
    /// Quantum HAL simulator backend.
    pub fn measure_qubit_with_rng<R: rand::Rng + ?Sized>(
        &mut self,
        qubit: usize,
        rng: &mut R,
    ) -> Result<u8> {
        if qubit >= self.num_qubits {
            return Err(QuantumError::InvalidQubitIndex {
                index: qubit,
                max: self.num_qubits - 1,
            });
        }

        let mut prob_zero = 0.0;
        let mut prob_one = 0.0;

        let dim = 1 << self.num_qubits;
        for i in 0..dim {
            let amplitude_squared = self.state[i].magnitude_squared();
            if ((i >> qubit) & 1) == 0 {
                prob_zero += amplitude_squared;
            } else {
                prob_one += amplitude_squared;
            }
        }

        let random_val: f64 = rng.gen();
        let measurement = if random_val < prob_zero { 0 } else { 1 };

        let norm = if measurement == 0 {
            prob_zero.sqrt()
        } else {
            prob_one.sqrt()
        };

        if norm < 1e-15 {
            return Ok(measurement);
        }

        let mut new_state = vec![Complex::zero(); dim];
        for i in 0..dim {
            if ((i >> qubit) & 1) == measurement as usize {
                new_state[i] = self.state[i] * (1.0 / norm);
            }
        }
        self.state = new_state;

        Ok(measurement)
    }

    /// Measure all qubits.
    pub fn measure_all(&mut self) -> Result<u64> {
        let mut rng = rand::thread_rng();
        self.measure_all_with_rng(&mut rng)
    }

    /// Measure all qubits using a caller-supplied random number generator.
    pub fn measure_all_with_rng<R: rand::Rng + ?Sized>(&mut self, rng: &mut R) -> Result<u64> {
        let mut result = 0u64;
        for qubit in 0..self.num_qubits {
            let bit = self.measure_qubit_with_rng(qubit, rng)? as u64;
            result |= bit << qubit;
        }
        Ok(result)
    }

    /// Get probability of measuring a specific basis state.
    pub fn probability(&self, state_index: usize) -> Result<f64> {
        if state_index >= (1 << self.num_qubits) {
            return Err(QuantumError::InvalidState(
                "State index out of range".to_string(),
            ));
        }
        Ok(self.state[state_index].magnitude_squared())
    }

    /// Get probabilities for all basis states.
    pub fn probabilities(&self) -> Vec<f64> {
        self.state.iter().map(|c| c.magnitude_squared()).collect()
    }

    /// Reset the simulator to |0⟩^n.
    pub fn reset(&mut self) {
        let dim = 1 << self.num_qubits;
        self.state = vec![Complex::zero(); dim];
        self.state[0] = Complex::one();
    }

    /// Check if state is normalized (for validation).
    pub fn is_normalized(&self) -> bool {
        let norm_squared: f64 = self.state.iter().map(|c| c.magnitude_squared()).sum();
        (norm_squared - 1.0).abs() < 1e-10
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::gates::QuantumGate;

    #[test]
    fn test_simulator_creation() {
        let sim = QuantumSimulator::new(3).unwrap();
        assert_eq!(sim.num_qubits(), 3);
        assert_eq!(sim.state().len(), 8);

        assert_eq!(sim.state()[0], Complex::one());
        for i in 1..8 {
            assert_eq!(sim.state()[i], Complex::zero());
        }
    }

    #[test]
    fn test_simulator_too_many_qubits() {
        let result = QuantumSimulator::new(25);
        assert!(result.is_err());
    }

    #[test]
    fn test_hadamard_creates_superposition() {
        let mut sim = QuantumSimulator::new(1).unwrap();
        let gate = QuantumGate::Hadamard;
        sim.apply_single_qubit_gate(&gate, 0).unwrap();

        let inv_sqrt2 = 1.0 / 2.0_f64.sqrt();
        assert!((sim.state()[0].real - inv_sqrt2).abs() < 1e-10);
        assert!((sim.state()[1].real - inv_sqrt2).abs() < 1e-10);
    }

    #[test]
    fn test_pauli_x_flip() {
        let mut sim = QuantumSimulator::new(1).unwrap();
        let gate = QuantumGate::PauliX;
        sim.apply_single_qubit_gate(&gate, 0).unwrap();

        assert_eq!(sim.state()[0], Complex::zero());
        assert_eq!(sim.state()[1], Complex::one());
    }

    #[test]
    fn test_cnot_gate() {
        let mut sim = QuantumSimulator::new(2).unwrap();

        sim.apply_single_qubit_gate(&QuantumGate::PauliX, 0).unwrap();
        sim.apply_cnot(0, 1).unwrap();

        assert_eq!(sim.state()[3], Complex::one());
    }

    #[test]
    fn test_measurement_collapses_state() {
        let mut sim = QuantumSimulator::new(1).unwrap();

        sim.apply_single_qubit_gate(&QuantumGate::Hadamard, 0).unwrap();

        let result = sim.measure_qubit(0).unwrap();
        assert!(result == 0 || result == 1);

        assert!(sim.is_normalized());
    }

    #[test]
    fn test_probabilities() {
        let mut sim = QuantumSimulator::new(1).unwrap();

        sim.apply_single_qubit_gate(&QuantumGate::Hadamard, 0).unwrap();

        let probs = sim.probabilities();
        assert_eq!(probs.len(), 2);

        assert!((probs[0] - 0.5).abs() < 1e-10);
        assert!((probs[1] - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_reset() {
        let mut sim = QuantumSimulator::new(2).unwrap();

        sim.apply_single_qubit_gate(&QuantumGate::Hadamard, 0).unwrap();
        sim.apply_single_qubit_gate(&QuantumGate::PauliX, 1).unwrap();

        sim.reset();

        assert_eq!(sim.state()[0], Complex::one());
        for i in 1..4 {
            assert_eq!(sim.state()[i], Complex::zero());
        }
    }

    #[test]
    fn test_rotation_gates() {
        let mut sim = QuantumSimulator::new(1).unwrap();

        let gate = QuantumGate::RotationX { theta: PI };
        sim.apply_single_qubit_gate(&gate, 0).unwrap();

        assert!(sim.state()[1].magnitude() > 0.99);
    }

    #[test]
    fn test_invalid_qubit_index() {
        let mut sim = QuantumSimulator::new(2).unwrap();
        let gate = QuantumGate::Hadamard;
        let result = sim.apply_single_qubit_gate(&gate, 5);
        assert!(result.is_err());
    }

    #[test]
    fn test_measure_all_deterministic() {
        let mut sim = QuantumSimulator::new(3).unwrap();

        sim.apply_single_qubit_gate(&QuantumGate::PauliX, 0).unwrap();
        sim.apply_single_qubit_gate(&QuantumGate::PauliX, 2).unwrap();

        let result = sim.measure_all().unwrap();
        assert_eq!(result, 0b101);
    }

    #[test]
    fn test_seeded_measurement_is_reproducible() {
        use rand::rngs::StdRng;
        use rand::SeedableRng;

        let mut first = QuantumSimulator::new(2).unwrap();
        first.apply_single_qubit_gate(&QuantumGate::Hadamard, 0).unwrap();
        let mut rng_a = StdRng::seed_from_u64(42);
        let a = first.measure_all_with_rng(&mut rng_a).unwrap();

        let mut second = QuantumSimulator::new(2).unwrap();
        second.apply_single_qubit_gate(&QuantumGate::Hadamard, 0).unwrap();
        let mut rng_b = StdRng::seed_from_u64(42);
        let b = second.measure_all_with_rng(&mut rng_b).unwrap();

        assert_eq!(a, b);
    }
}
