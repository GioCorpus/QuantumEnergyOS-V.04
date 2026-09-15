//! Phase 4.3 tetron + logical qubit model (MODEL).
//!
//! CLASSIFICATION: MODEL
//!
//! A tetron groups four Majorana modes (gamma1..gamma4). The logical qubit is
//! encoded in the joint parity sector; total parity is fixed (even) and the
//! logical Z is the parity of a chosen mode pair. Braiding is a symbolic
//! exchange record with a unitary note, never a hardware claim.

use serde::{Deserialize, Serialize};

use crate::error::{QuantumError, Result};
use crate::majorana::{FermionParity, MajoranaMode};
use crate::resources::LogicalQubitId;

/// Four-mode tetron abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Tetron {
    pub label: String,
    pub modes: [MajoranaMode; 4],
    pub total_parity: FermionParity,
}

impl Tetron {
    pub fn new(label: impl Into<String>, base_id: u32) -> Self {
        let label_str: String = label.into();
        Self {
            modes: [
                MajoranaMode::new(base_id, format!("{label_str}-gamma1")),
                MajoranaMode::new(base_id + 1, format!("{label_str}-gamma2")),
                MajoranaMode::new(base_id + 2, format!("{label_str}-gamma3")),
                MajoranaMode::new(base_id + 3, format!("{label_str}-gamma4")),
            ],
            label: label_str,
            total_parity: FermionParity::Even,
        }
    }

    pub fn mode_ids(&self) -> [u32; 4] {
        [
            self.modes[0].id.0,
            self.modes[1].id.0,
            self.modes[2].id.0,
            self.modes[3].id.0,
        ]
    }

    pub fn validate(&self) -> Result<()> {
        let ids = self.mode_ids();
        for i in 0..4 {
            for j in (i + 1)..4 {
                if ids[i] == ids[j] {
                    return Err(QuantumError::InvalidGateParameters(
                        "tetron modes must be distinct".to_string(),
                    ));
                }
            }
        }
        Ok(())
    }
}

/// Logical qubit encoded in a tetron parity sector.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MajoranaLogicalQubit {
    pub id: LogicalQubitId,
    pub tetron_label: String,
    pub logical_z: FermionParity,
}

impl MajoranaLogicalQubit {
    pub fn new(id: u32, tetron: &Tetron) -> Self {
        Self {
            id: LogicalQubitId::new(id),
            tetron_label: tetron.label.clone(),
            logical_z: FermionParity::Even,
        }
    }

    pub fn set_from_parity(&mut self, parity: FermionParity) {
        self.logical_z = parity;
    }

    pub fn logical_bit(&self) -> u8 {
        self.logical_z.bit()
    }
}

/// Symbolic braiding exchange between two modes (simulator bookkeeping).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MajoranaExchange {
    pub mode_a: u32,
    pub mode_b: u32,
    pub note: String,
}

impl MajoranaExchange {
    pub fn new(mode_a: u32, mode_b: u32) -> Result<Self> {
        if mode_a == mode_b {
            return Err(QuantumError::InvalidGateParameters(
                "exchange requires two distinct modes".to_string(),
            ));
        }
        Ok(Self {
            mode_a,
            mode_b,
            note: "simulator symbolic exchange; unitary note only, no hardware braiding"
                .to_string(),
        })
    }
}

/// Measurement-based control primitive: parity -> classical condition.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalOperation {
    pub parity_bit: u8,
    pub apply_correction_when_odd: bool,
}

impl ConditionalOperation {
    pub fn new(parity_bit: u8, apply_correction_when_odd: bool) -> Self {
        Self {
            parity_bit: parity_bit & 1,
            apply_correction_when_odd,
        }
    }

    pub fn should_correct(&self) -> bool {
        self.apply_correction_when_odd && self.parity_bit == 1
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tetron_modes_distinct() {
        let t = Tetron::new("tetron-0", 0);
        assert!(t.validate().is_ok());
        assert_eq!(t.mode_ids(), [0, 1, 2, 3]);
    }

    #[test]
    fn test_logical_bit_mapping() {
        let t = Tetron::new("tetron-0", 10);
        let mut q = MajoranaLogicalQubit::new(0, &t);
        assert_eq!(q.logical_bit(), 0);
        q.set_from_parity(FermionParity::Odd);
        assert_eq!(q.logical_bit(), 1);
    }

    #[test]
    fn test_exchange_rejects_same_mode() {
        assert!(MajoranaExchange::new(3, 3).is_err());
    }

    #[test]
    fn test_conditional_feed_forward() {
        assert!(ConditionalOperation::new(1, true).should_correct());
        assert!(!ConditionalOperation::new(0, true).should_correct());
        assert!(!ConditionalOperation::new(1, false).should_correct());
    }
}
