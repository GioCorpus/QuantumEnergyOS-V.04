//! Error types for the research platform.

use thiserror::Error;

/// Result alias.
pub type Result<T> = std::result::Result<T, ResearchError>;

/// Errors raised by the research platform.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ResearchError {
    #[error("dataset already exists: {0}")]
    DatasetExists(String),
    #[error("checksum mismatch for artifact: {0}")]
    ChecksumMismatch(String),
    #[error("invalid workflow: {0}")]
    WorkflowInvalid(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        assert!(ResearchError::WorkflowInvalid("x".into())
            .to_string()
            .contains("workflow"));
        assert!(ResearchError::DatasetExists("d".into())
            .to_string()
            .contains("exists"));
    }
}
