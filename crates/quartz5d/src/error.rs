use thiserror::Error;

/// Result type for Quartz5D operations.
pub type Result<T> = std::result::Result<T, Quartz5DError>;

/// Errors that can occur in the Quartz5D model.
#[derive(Debug, Error)]
pub enum Quartz5DError {
    #[error("invalid coordinate: {0}")]
    InvalidCoordinate(String),

    #[error("projection error: {0}")]
    ProjectionError(String),

    #[error("storage error: {0}")]
    StorageError(String),

    #[error("prediction error: {0}")]
    PredictionError(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("dimension out of bounds: {0}")]
    DimensionOutOfBounds(String),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

impl From<serde_json::Error> for Quartz5DError {
    fn from(err: serde_json::Error) -> Self {
        Quartz5DError::SerializationError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Quartz5DError::InvalidCoordinate("test".to_string());
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::new(std::io::ErrorKind::Other, "test");
        let err: Quartz5DError = io_err.into();
        assert!(matches!(err, Quartz5DError::IoError(_)));
    }
}
