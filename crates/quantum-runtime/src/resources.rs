//! Phase 4.3 quantum resource identifiers.
//!
//! CLASSIFICATION: MODEL
//!
//! Explicit separation between physical Majorana modes, logical qubits,
//! registers and measurement handles. IDs are never mixed implicitly.

use serde::{Deserialize, Serialize};

/// Physical qubit / state-vector qubit index.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QubitId(pub u32);

/// Error-corrected logical qubit identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LogicalQubitId(pub u32);

/// Physical Majorana mode identifier (gamma index space).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PhysicalModeId(pub u32);

/// Classical/quantum register identifier.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct RegisterId(pub u32);

/// Stable measurement handle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MeasurementId(pub u64);

impl QubitId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
    pub fn index(self) -> usize {
        self.0 as usize
    }
}

impl LogicalQubitId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl PhysicalModeId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl RegisterId {
    pub fn new(id: u32) -> Self {
        Self(id)
    }
}

impl MeasurementId {
    pub fn new(id: u64) -> Self {
        Self(id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ids_are_distinct_types() {
        let q = QubitId::new(1);
        let l = LogicalQubitId::new(1);
        let m = PhysicalModeId::new(1);
        assert_eq!(q.index(), 1);
        assert_eq!(l.0, 1);
        assert_eq!(m.0, 1);
        // Distinct types: this would not compile if mixed.
        let _r = RegisterId::new(2);
        let _meas = MeasurementId::new(7);
    }
}
