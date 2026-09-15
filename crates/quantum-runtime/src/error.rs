use thiserror::Error;

/// Result type for quantum operations
pub type Result<T> = std::result::Result<T, QuantumError>;

/// Errors that can occur during quantum operations
#[derive(Debug, Error)]
pub enum QuantumError {
    #[error("invalid qubit index: {index} (max: {max})")]
    InvalidQubitIndex { index: usize, max: usize },

    #[error("invalid number of qubits: expected {expected}, got {got}")]
    InvalidQubitCount { expected: usize, got: usize },

    #[error("circuit compilation failed: {0}")]
    CompilationFailed(String),

    #[error("simulator error: {0}")]
    SimulatorError(String),

    #[error("measurement failed: {0}")]
    MeasurementFailed(String),

    #[error("invalid quantum state: {0}")]
    InvalidState(String),

    #[error("backend not available: {0}")]
    BackendNotAvailable(String),

    #[error("qubit allocation failed: {0}")]
    AllocationFailed(String),

    #[error("unsupported gate: {0}")]
    UnsupportedGate(String),

    /// The requested hardware capability is not present.
    ///
    /// Used by the Quantum HAL (Phase 3) whenever a device, backend or
    /// accelerator is not physically available. Returning this variant instead
    /// of a fabricated result is mandatory: no backend may pretend to have
    /// connectivity to hardware that does not exist.
    #[error("unsupported hardware: {0}")]
    UnsupportedHardware(String),

    #[error("device not initialized: {0}")]
    DeviceNotInitialized(String),

    #[error("device calibration failed: {0}")]
    CalibrationFailed(String),

    #[error("job not found: {0}")]
    JobNotFound(String),

    #[error("job timed out: {0}")]
    JobTimeout(String),

    #[error("invalid gate parameters: {0}")]
    InvalidGateParameters(String),

    #[error("state vector dimension mismatch: expected {expected}, got {got}")]
    StateDimensionMismatch { expected: usize, got: usize },

    #[error("io error: {0}")]
    IoError(String),

    #[error("serialization error: {0}")]
    SerializationError(String),
}

impl From<std::io::Error> for QuantumError {
    fn from(err: std::io::Error) -> Self {
        QuantumError::IoError(err.to_string())
    }
}

impl From<serde_json::Error> for QuantumError {
    fn from(err: serde_json::Error) -> Self {
        QuantumError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = QuantumError::InvalidQubitIndex { index: 10, max: 8 };
        assert!(err.to_string().contains("10"));
        assert!(err.to_string().contains("8"));
    }

    #[test]
    fn test_error_conversion_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let q_err: QuantumError = io_err.into();
        assert!(matches!(q_err, QuantumError::IoError(_)));
    }

    #[test]
    fn test_result_type() {
        let result: Result<i32> = Ok(42);
        assert!(result.is_ok());
        
        let error_result: Result<i32> = Err(QuantumError::InvalidState("test".to_string()));
        assert!(error_result.is_err());
    }

    #[test]
    fn test_unsupported_hardware_display() {
        let err = QuantumError::UnsupportedHardware("no physical QPU present".to_string());
        assert!(err.to_string().contains("unsupported hardware"));
        assert!(err.to_string().contains("no physical QPU present"));
    }

    #[test]
    fn test_hal_error_variants_display() {
        let cases = vec![
            QuantumError::DeviceNotInitialized("qpu0".to_string()),
            QuantumError::CalibrationFailed("visibility too low".to_string()),
            QuantumError::JobNotFound("job-1".to_string()),
            QuantumError::JobTimeout("job-2".to_string()),
        ];
        for err in cases {
            assert!(!err.to_string().is_empty());
        }
    }
}
