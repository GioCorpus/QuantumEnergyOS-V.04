//! Phase 4.3 classical noise engine (SIMULATION).
//!
//! CLASSIFICATION: SIMULATION
//!
//! Effective classical error model for simulator studies only. It is NOT a
//! microscopic device model and must never be reported as hardware noise.

use rand::Rng;
use serde::{Deserialize, Serialize};

use crate::error::{QuantumError, Result};

/// Effective simulator noise configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeNoiseModel {
    pub enabled: bool,
    pub seed: Option<u64>,
    pub measurement_error_probability: f64,
    pub dephasing_rate: f64,
    pub operation_error_probability: f64,
    pub quasiparticle_poisoning_probability: f64,
}

impl Default for RuntimeNoiseModel {
    fn default() -> Self {
        Self::ideal()
    }
}

impl RuntimeNoiseModel {
    pub fn ideal() -> Self {
        Self {
            enabled: false,
            seed: None,
            measurement_error_probability: 0.0,
            dephasing_rate: 0.0,
            operation_error_probability: 0.0,
            quasiparticle_poisoning_probability: 0.0,
        }
    }

    pub fn validate(&self) -> Result<()> {
        for (name, v) in [
            (
                "measurement_error_probability",
                self.measurement_error_probability,
            ),
            ("dephasing_rate", self.dephasing_rate),
            (
                "operation_error_probability",
                self.operation_error_probability,
            ),
            (
                "quasiparticle_poisoning_probability",
                self.quasiparticle_poisoning_probability,
            ),
        ] {
            if !(0.0..=1.0).contains(&v) || !v.is_finite() {
                return Err(QuantumError::InvalidGateParameters(format!(
                    "{name} must be in [0,1], got {v}"
                )));
            }
        }
        Ok(())
    }

    pub fn is_ideal(&self) -> bool {
        !self.enabled
            || (self.measurement_error_probability == 0.0
                && self.dephasing_rate == 0.0
                && self.operation_error_probability == 0.0
                && self.quasiparticle_poisoning_probability == 0.0)
    }

    /// Flip a classical outcome bit with the measurement-error probability.
    pub fn apply_measurement_error<R: Rng + ?Sized>(&self, bit: u8, rng: &mut R) -> u8 {
        if !self.enabled || self.measurement_error_probability <= 0.0 {
            return bit & 1;
        }
        if rng.gen::<f64>() < self.measurement_error_probability {
            (bit ^ 1) & 1
        } else {
            bit & 1
        }
    }

    /// Effective quasiparticle-poisoning event sampler (classical bit flip).
    pub fn sample_poisoning_event<R: Rng + ?Sized>(&self, rng: &mut R) -> bool {
        if !self.enabled || self.quasiparticle_poisoning_probability <= 0.0 {
            return false;
        }
        rng.gen::<f64>() < self.quasiparticle_poisoning_probability
    }

    /// Sample whether a Clifford-scale operation suffers an effective error.
    pub fn sample_operation_error<R: Rng + ?Sized>(&self, rng: &mut R) -> bool {
        if !self.enabled || self.operation_error_probability <= 0.0 {
            return false;
        }
        rng.gen::<f64>() < self.operation_error_probability
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn test_ideal_is_noop() {
        let model = RuntimeNoiseModel::ideal();
        assert!(model.is_ideal());
        let mut rng = StdRng::seed_from_u64(1);
        assert_eq!(model.apply_measurement_error(1, &mut rng), 1);
        assert!(!model.sample_poisoning_event(&mut rng));
    }

    #[test]
    fn test_bounds_rejected() {
        let mut model = RuntimeNoiseModel::ideal();
        model.measurement_error_probability = 2.0;
        assert!(model.validate().is_err());
    }

    #[test]
    fn test_seeded_flip_reproducible() {
        let model = RuntimeNoiseModel {
            enabled: true,
            seed: Some(42),
            measurement_error_probability: 0.5,
            dephasing_rate: 0.0,
            operation_error_probability: 0.0,
            quasiparticle_poisoning_probability: 0.0,
        };
        let mut a = StdRng::seed_from_u64(42);
        let mut b = StdRng::seed_from_u64(42);
        let ra: Vec<u8> = (0..32)
            .map(|_| model.apply_measurement_error(0, &mut a))
            .collect();
        let rb: Vec<u8> = (0..32)
            .map(|_| model.apply_measurement_error(0, &mut b))
            .collect();
        assert_eq!(ra, rb);
    }
}
