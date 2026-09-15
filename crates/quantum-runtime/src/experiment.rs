//! Shot engine (SIMULATION): N-shot aggregation without state retention.
use crate::circuit::QuantumCircuit;
use crate::error::{QuantumError, Result};
use crate::gates::QuantumGate;
use crate::noise::RuntimeNoiseModel;
use crate::simulator::QuantumSimulator;
use rand::rngs::StdRng;
use rand::SeedableRng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentLimits {
    pub max_qubits: usize,
    pub max_shots: u32,
    pub max_depth: usize,
}
impl Default for ExperimentLimits {
    fn default() -> Self {
        Self {
            max_qubits: 20,
            max_shots: 100_000,
            max_depth: 10_000,
        }
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumExperiment {
    pub experiment_id: String,
    pub circuit: QuantumCircuit,
    pub backend: String,
    pub shots: u32,
    pub noise: RuntimeNoiseModel,
    pub seed: u64,
    pub metadata: HashMap<String, String>,
    pub backend_version: String,
    pub compiler_version: String,
    pub runtime_version: String,
}
impl QuantumExperiment {
    pub fn new(
        experiment_id: impl Into<String>,
        circuit: QuantumCircuit,
        backend: impl Into<String>,
        shots: u32,
        noise: RuntimeNoiseModel,
        seed: u64,
    ) -> Self {
        Self {
            experiment_id: experiment_id.into(),
            circuit,
            backend: backend.into(),
            shots,
            noise,
            seed,
            metadata: HashMap::new(),
            backend_version: "cpu-simulator-4.3.0".to_string(),
            compiler_version: "qir-4.3.0".to_string(),
            runtime_version: env!("CARGO_PKG_VERSION").to_string(),
        }
    }
    pub fn validate(&self, limits: &ExperimentLimits) -> Result<()> {
        if self.circuit.num_qubits > limits.max_qubits {
            return Err(QuantumError::SimulationResource(format!(
                "qubits {} exceed limit {}",
                self.circuit.num_qubits, limits.max_qubits
            )));
        }
        if self.shots == 0 || self.shots > limits.max_shots {
            return Err(QuantumError::ResourceLimit(format!(
                "shots {} outside [1, {}]",
                self.shots, limits.max_shots
            )));
        }
        if self.circuit.gates.len() > limits.max_depth {
            return Err(QuantumError::ResourceLimit(format!(
                "depth {} exceeds {}",
                self.circuit.gates.len(),
                limits.max_depth
            )));
        }
        self.noise.validate()?;
        self.circuit.validate()?;
        Ok(())
    }
    pub fn circuit_hash(&self) -> u64 {
        let mut h: u64 = 0xcbf29ce484222325;
        for b in self.circuit.name.bytes() {
            h ^= b as u64;
            h = h.wrapping_mul(0x100000001b3);
        }
        h ^= self.circuit.num_qubits as u64;
        h = h.wrapping_mul(0x100000001b3);
        for g in &self.circuit.gates {
            for b in format!("{g:?}").bytes() {
                h ^= b as u64;
                h = h.wrapping_mul(0x100000001b3);
            }
        }
        h
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentResult {
    pub experiment_id: String,
    pub backend: String,
    pub shots: u32,
    pub seed: u64,
    pub counts: HashMap<String, u64>,
    pub probabilities: HashMap<String, f64>,
    pub circuit_hash: u64,
    pub noise_enabled: bool,
    pub simulation_only: bool,
}
fn bitstring(value: u64, width: usize) -> String {
    if width == 0 {
        return value.to_string();
    }
    let mut s = String::with_capacity(width);
    for i in (0..width).rev() {
        s.push(if ((value >> i) & 1) == 1 { '1' } else { '0' });
    }
    s
}
fn run_single_shot(
    circuit: &QuantumCircuit,
    noise: &RuntimeNoiseModel,
    rng: &mut StdRng,
) -> Result<u64> {
    QuantumSimulator::check_resources(circuit.num_qubits)?;
    let mut sim = QuantumSimulator::new(circuit.num_qubits)?;
    let mut measured: u64 = 0;
    let mut measured_any = false;
    for gate in &circuit.gates {
        match gate {
            QuantumGate::Measurement { qubit } => {
                let mut bit = sim.measure_qubit_with_rng(*qubit, rng)?;
                bit = noise.apply_measurement_error(bit, rng);
                if bit == 1 {
                    measured |= 1 << qubit;
                } else {
                    measured &= !(1 << qubit);
                }
                measured_any = true;
            }
            QuantumGate::CNOT { control, target } => sim.apply_cnot(*control, *target)?,
            QuantumGate::ControlledZ { control, target } => sim.apply_cz(*control, *target)?,
            QuantumGate::Swap { qubit1, qubit2 } => sim.apply_swap(*qubit1, *qubit2)?,
            QuantumGate::Toffoli {
                control1,
                control2,
                target,
            } => sim.apply_toffoli(*control1, *control2, *target)?,
            single if single.is_single_qubit() => {
                if noise.sample_operation_error(rng) {
                    continue;
                }
                sim.apply_single_qubit_gate(single, 0)?;
            }
            other => return Err(QuantumError::UnsupportedGate(format!("{other:?}"))),
        }
    }
    if !measured_any {
        measured = sim.measure_all_with_rng(rng)?;
    }
    Ok(measured)
}
pub fn run_experiment(
    experiment: &QuantumExperiment,
    limits: &ExperimentLimits,
) -> Result<ExperimentResult> {
    experiment.validate(limits)?;
    let mut rng = StdRng::seed_from_u64(experiment.seed);
    let width = experiment
        .circuit
        .num_classical_bits
        .max(experiment.circuit.num_qubits);
    let mut counts: HashMap<String, u64> = HashMap::new();
    for _ in 0..experiment.shots {
        let sample = run_single_shot(&experiment.circuit, &experiment.noise, &mut rng)?;
        let key = bitstring(sample, width);
        *counts.entry(key).or_insert(0) += 1;
    }
    let total = experiment.shots as f64;
    let probabilities = counts
        .iter()
        .map(|(k, v)| (k.clone(), *v as f64 / total))
        .collect();
    Ok(ExperimentResult {
        experiment_id: experiment.experiment_id.clone(),
        backend: experiment.backend.clone(),
        shots: experiment.shots,
        seed: experiment.seed,
        counts,
        probabilities,
        circuit_hash: experiment.circuit_hash(),
        noise_enabled: experiment.noise.enabled,
        simulation_only: true,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn x_circuit() -> QuantumCircuit {
        let mut c = QuantumCircuit::new("x", 1).unwrap();
        c.add_gate(QuantumGate::PauliX).unwrap();
        c.add_gate(QuantumGate::Measurement { qubit: 0 }).unwrap();
        c
    }
    #[test]
    fn test_x_experiment_deterministic() {
        let exp = QuantumExperiment::new(
            "exp-1",
            x_circuit(),
            "cpu-simulator",
            64,
            RuntimeNoiseModel::ideal(),
            42,
        );
        let res = run_experiment(&exp, &ExperimentLimits::default()).unwrap();
        assert_eq!(res.counts.get("1"), Some(&64));
        assert!(res.simulation_only);
    }
    #[test]
    fn test_reproducible_with_seed() {
        let mk = |id: &str| {
            QuantumExperiment::new(
                id,
                x_circuit(),
                "cpu-simulator",
                32,
                RuntimeNoiseModel::ideal(),
                7,
            )
        };
        let a = run_experiment(&mk("a"), &ExperimentLimits::default()).unwrap();
        let b = run_experiment(&mk("b"), &ExperimentLimits::default()).unwrap();
        assert_eq!(a.counts, b.counts);
    }
    #[test]
    fn test_limits_reject_huge_shots() {
        let exp = QuantumExperiment::new(
            "big",
            x_circuit(),
            "cpu-simulator",
            1_000_000,
            RuntimeNoiseModel::ideal(),
            1,
        );
        let limits = ExperimentLimits {
            max_shots: 10,
            ..Default::default()
        };
        assert!(run_experiment(&exp, &limits).is_err());
    }
}
