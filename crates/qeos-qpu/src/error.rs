//! Error types for the QPU interface.

use thiserror::Error;

/// Result alias for QPU operations.
pub type Result<T> = std::result::Result<T, QpuError>;

/// Errors raised by the QPU interface layer.
#[derive(Debug, Error)]
pub enum QpuError {
    #[error("unsupported hardware")]
    UnsupportedHardware,
    #[error("resource limit exceeded: {0}")]
    ResourceLimit(String),
    #[error("invalid input: {0}")]
    InvalidInput(String),
    #[error("backend error: {0}")]
    Backend(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        assert!(QpuError::UnsupportedHardware
            .to_string()
            .contains("unsupported"));
        assert!(QpuError::ResourceLimit("r".into())
            .to_string()
            .contains("limit"));
    }
}
