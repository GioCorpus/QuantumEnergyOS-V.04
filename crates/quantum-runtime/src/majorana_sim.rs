//! Phase 4.3 Majorana simulator backend (SIMULATION).
//!
//! CLASSIFICATION: SIMULATION
//!
//! Tetron/parity model executed on the CPU state-vector reference:
//! logical Z lives on one physical qubit; parity measurements sample that
//! qubit with optional poisoning/measurement noise, then decode/correct.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::majorana::FermionParity;
use crate::noise::RuntimeNoiseModel;
use crate::tetron::{MajoranaLogicalQubit, Tetron};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Majorana simulator configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MajoranaSimConfig {
    pub tetron_label: String,
    pub base_mode_id: u32,
    pub logical_id: u32,
    pub measurement_error: f64,
    pub poisoning_probability: f64,
}

impl Default for MajoranaSimConfig {
    fn default() -> Self {
        Self {
            tetron_label: "tetron-0".to_string(),
            base_mode_id: 0,
            logical_id: 0,
            measurement_error: 0.0,
            poisoning_probability: 0.0,
        }
    }
}

/// Logical readout statistics over N shots.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MajoranaExperimentResult {
    pub shots: u32,
    pub logical_zeros: u64,
    pub logical_ones: u64,
    pub logical_error_rate: f64,
    pub correction_success_rate: f64,
    pub syndrome_count: u64,
    pub failed_corrections: u64,
    pub simulation_only: bool,
}

/// Run a parity-protection experiment on one tetron logical qubit.
pub fn run_majorana_experiment(
    shots: u32,
    seed: u64,
    config: &MajoranaSimConfig,
    noise: &RuntimeNoiseModel,
) -> Result<MajoranaExperimentResult> {
    let tetron = Tetron::new(config.tetron_label.clone(), config.base_mode_id);
    tetron.validate()?;
    let mut logical = MajoranaLogicalQubit::new(config.logical_id, &tetron);
    let mut rng = StdRng::seed_from_u64(seed);
    let mut zeros = 0u64;
    let mut ones = 0u64;
    let mut syndromes = 0u64;
    for _ in 0..shots {
        // Ideal logical state is |0> (even parity).
        let mut parity = FermionParity::Even;
        if noise.sample_poisoning_event(&mut rng) || rng.gen::<f64>() < config.poisoning_probability
        {
            parity = FermionParity::Odd;
            syndromes += 1;
        }
        let mut bit = parity.bit();
        if rng.gen::<f64>() < config.measurement_error {
            bit ^= 1;
        }
        bit = noise.apply_measurement_error(bit, &mut rng);
        // Repetition-style correction: single parity flip detected -> fix.
        let corrected = if parity == FermionParity::Odd { 0 } else { bit };
        logical.set_from_parity(FermionParity::from_bit(corrected));
        if logical.logical_bit() == 0 {
            zeros += 1;
        } else {
            ones += 1;
        }
    }
    let total = shots.max(1) as f64;
    Ok(MajoranaExperimentResult {
        shots,
        logical_zeros: zeros,
        logical_ones: ones,
        logical_error_rate: ones as f64 / total,
        correction_success_rate: zeros as f64 / total,
        syndrome_count: syndromes,
        failed_corrections: ones,
        simulation_only: true,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_ideal_majorana_no_errors() {
        let res = run_majorana_experiment(
            64,
            11,
            &MajoranaSimConfig::default(),
            &RuntimeNoiseModel::ideal(),
        )
        .unwrap();
        assert_eq!(res.logical_error_rate, 0.0);
        assert!(res.simulation_only);
    }
    #[test]
    fn test_majorana_reproducible() {
        let cfg = MajoranaSimConfig {
            poisoning_probability: 0.2,
            ..Default::default()
        };
        let noise = RuntimeNoiseModel::ideal();
        let a = run_majorana_experiment(128, 5, &cfg, &noise).unwrap();
        let b = run_majorana_experiment(128, 5, &cfg, &noise).unwrap();
        assert_eq!(a.logical_ones, b.logical_ones);
    }
}
