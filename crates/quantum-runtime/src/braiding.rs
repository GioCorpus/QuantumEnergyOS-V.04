//! Phase 4.7 braiding model (MODEL, seeded, deterministic).
use serde::{Deserialize, Serialize};
use crate::error::{QuantumError, Result};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)] pub struct BraidStep { pub i: u32, pub j: u32, pub inverse: bool }
impl BraidStep { pub fn new(i: u32, j: u32) -> Result<Self> { if i == j { return Err(QuantumError::InvalidGateParameters("braid needs distinct modes".into())); } Ok(Self { i, j, inverse: false }) } pub fn inverse(mut self) -> Self { self.inverse = true; self } }
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)] pub struct BraidSequence { pub steps: Vec<BraidStep> }
impl BraidSequence { pub fn new() -> Self { Self::default() } pub fn push(&mut self, s: BraidStep) { self.steps.push(s); } pub fn len(&self) -> usize { self.steps.len() } pub fn is_empty(&self) -> bool { self.steps.is_empty() } }
/// Symbolic braid unitary ledger: records ordered exchanges (model only).
pub fn apply_braid_unitary(seq: &BraidSequence) -> Vec<String> { seq.steps.iter().map(|s| format!("B({},{}){}", s.i, s.j, if s.inverse { "^-1" } else { "" })).collect() }
#[cfg(test)] mod tests { use super::*; #[test] fn braid() { let mut s = BraidSequence::new(); s.push(BraidStep::new(0, 1).unwrap()); s.push(BraidStep::new(1, 2).unwrap().inverse()); assert_eq!(s.len(), 2); assert_eq!(apply_braid_unitary(&s).len(), 2); assert!(BraidStep::new(1, 1).is_err()); } }
