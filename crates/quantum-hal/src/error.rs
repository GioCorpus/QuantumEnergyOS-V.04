//! Error vocabulary of the Quantum Hardware Abstraction Layer.
//!
//! CLASSIFICATION: REAL (interface) / ABSTRACT (hardware paths)
//!
//! The HAL intentionally re-uses the single canonical `QuantumError` enum owned
//! by `quantum-runtime` rather than introducing a second, divergent error type.
//! One vocabulary across the quantum stack guarantees that
//! `QuantumError::UnsupportedHardware` always means the same thing:
//!
//! > this capability requires hardware that is not present.
//!
//! Backends MUST return that variant (instead of a fabricated result) whenever
//! a device, accelerator or QPU is unavailable.

pub use quantum_runtime::error::{QuantumError, Result};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unsupported_hardware_is_constructible() {
        let err = QuantumError::UnsupportedHardware("no physical QPU present".to_string());
        assert!(matches!(err, QuantumError::UnsupportedHardware(_)));
        assert!(err.to_string().contains("no physical QPU present"));
    }

    #[test]
    fn test_result_alias_roundtrip() {
        fn ok() -> Result<u32> {
            Ok(7)
        }
        fn fail() -> Result<u32> {
            Err(QuantumError::JobNotFound("job-1".to_string()))
        }

        assert_eq!(ok().unwrap(), 7);
        assert!(matches!(fail(), Err(QuantumError::JobNotFound(_))));
    }
}