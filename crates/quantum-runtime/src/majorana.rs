//! Phase 4.3 Majorana operator algebra (MODEL).
//!
//! CLASSIFICATION: MODEL
//!
//! Computational model of Majorana zero modes satisfying
//! `{gamma_i, gamma_j} = 2 delta_ij`. Modes are tracked symbolically plus an
//! explicit 4x4 Pauli representation used ONLY for algebraic verification
//! tests (two-mode sector). This is not device physics.

use serde::{Deserialize, Serialize};

use crate::error::{QuantumError, Result};
use crate::resources::PhysicalModeId;

/// A Majorana mode identity (stable label + numeric index).
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MajoranaMode {
    pub id: PhysicalModeId,
    pub label: String,
}

impl MajoranaMode {
    pub fn new(id: u32, label: impl Into<String>) -> Self {
        Self {
            id: PhysicalModeId::new(id),
            label: label.into(),
        }
    }

    pub fn index(self) -> usize {
        self.id.0 as usize
    }
}

/// Fermion parity eigenvalue: +1 (even) or -1 (odd).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FermionParity {
    Even,
    Odd,
}

impl FermionParity {
    pub fn eigenvalue(self) -> i8 {
        match self {
            FermionParity::Even => 1,
            FermionParity::Odd => -1,
        }
    }

    pub fn bit(self) -> u8 {
        match self {
            FermionParity::Even => 0,
            FermionParity::Odd => 1,
        }
    }

    pub fn from_bit(bit: u8) -> Self {
        if bit & 1 == 0 {
            FermionParity::Even
        } else {
            FermionParity::Odd
        }
    }

    pub fn from_eigenvalue(v: i8) -> Result<Self> {
        match v {
            1 => Ok(FermionParity::Even),
            -1 => Ok(FermionParity::Odd),
            _ => Err(QuantumError::InvalidGateParameters(format!(
                "parity eigenvalue must be +1/-1, got {v}"
            ))),
        }
    }
}

/// Parity operator P_ij = i gamma_i gamma_j (symbolic handle).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ParityOperator {
    pub mode_i: u32,
    pub mode_j: u32,
}

impl ParityOperator {
    pub fn new(mode_i: u32, mode_j: u32) -> Result<Self> {
        if mode_i == mode_j {
            return Err(QuantumError::InvalidGateParameters(
                "parity operator requires two distinct modes".to_string(),
            ));
        }
        Ok(Self { mode_i, mode_j })
    }

    /// Canonical ordering so (j,i) and (i,j) compare equal up to sign note.
    pub fn canonical(&self) -> (u32, u32) {
        if self.mode_i < self.mode_j {
            (self.mode_i, self.mode_j)
        } else {
            (self.mode_j, self.mode_i)
        }
    }
}

/// Explicit 2x2 complex matrix helper for algebra tests.
pub type Mat2 = [[(f64, f64); 2]; 2];

fn mat_mul(a: &Mat2, b: &Mat2) -> Mat2 {
    let mut out = [[(0.0, 0.0); 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            let mut re = 0.0;
            let mut im = 0.0;
            for k in 0..2 {
                re += a[i][k].0 * b[k][j].0 - a[i][k].1 * b[k][j].1;
                im += a[i][k].0 * b[k][j].1 + a[i][k].1 * b[k][j].0;
            }
            out[i][j] = (re, im);
        }
    }
    out
}

fn mat_add(a: &Mat2, b: &Mat2) -> Mat2 {
    let mut out = [[(0.0, 0.0); 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            out[i][j] = (a[i][j].0 + b[i][j].0, a[i][j].1 + b[i][j].1);
        }
    }
    out
}

/// Gamma matrices for a single two-mode sector: gamma_0 = X, gamma_1 = Y.
/// They satisfy {gamma_i, gamma_j} = 2 delta_ij.
pub fn two_mode_gamma_matrices() -> [Mat2; 2] {
    let x: Mat2 = [[(0.0, 0.0), (1.0, 0.0)], [(1.0, 0.0), (0.0, 0.0)]];
    let y: Mat2 = [[(0.0, 0.0), (0.0, -1.0)], [(0.0, 1.0), (0.0, 0.0)]];
    [x, y]
}

/// Verify the anticommutation relation numerically for the 2-mode matrices.
pub fn verify_anticommutation() -> bool {
    let g = two_mode_gamma_matrices();
    let identity: Mat2 = [[(1.0, 0.0), (0.0, 0.0)], [(0.0, 0.0), (1.0, 0.0)]];
    let zero = [[(0.0, 0.0); 2]; 2];
    for i in 0..2 {
        for j in 0..2 {
            let anti = mat_add(&mat_mul(&g[i], &g[j]), &mat_mul(&g[j], &g[i]));
            let expected = if i == j {
                let mut e = zero;
                for a in 0..2 {
                    for b in 0..2 {
                        e[a][b] = (2.0 * identity[a][b].0, 0.0);
                    }
                }
                e
            } else {
                zero
            };
            for a in 0..2 {
                for b in 0..2 {
                    if (anti[a][b].0 - expected[a][b].0).abs() > 1e-12
                        || (anti[a][b].1 - expected[a][b].1).abs() > 1e-12
                    {
                        return false;
                    }
                }
            }
        }
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_anticommutation_holds() {
        assert!(verify_anticommutation());
    }

    #[test]
    fn test_parity_bit_roundtrip() {
        assert_eq!(FermionParity::Even.eigenvalue(), 1);
        assert_eq!(FermionParity::Odd.bit(), 1);
        assert_eq!(FermionParity::from_bit(0), FermionParity::Even);
        assert_eq!(
            FermionParity::from_eigenvalue(-1).unwrap(),
            FermionParity::Odd
        );
        assert!(FermionParity::from_eigenvalue(0).is_err());
    }

    #[test]
    fn test_parity_operator_rejects_same_mode() {
        assert!(ParityOperator::new(1, 1).is_err());
        let op = ParityOperator::new(2, 1).unwrap();
        assert_eq!(op.canonical(), (1, 2));
    }
}
