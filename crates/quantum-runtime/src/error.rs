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
}
