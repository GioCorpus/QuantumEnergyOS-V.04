use thiserror::Error;

/// Result type for hardware abstraction operations.
pub type Result<T> = std::result::Result<T, HardwareError>;

/// Errors that can occur in the hardware abstraction layer.
#[derive(Debug, Error)]
pub enum HardwareError {
    #[error("device not found: {0}")]
    DeviceNotFound(String),

    #[error("device initialization failed: {0}")]
    InitializationFailed(String),

    #[error("device communication error: {0}")]
    CommunicationError(String),

    #[error("device health check failed: {0}")]
    HealthCheckFailed(String),

    #[error("unsupported device: {0}")]
    UnsupportedDevice(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("invalid configuration: {0}")]
    InvalidConfiguration(String),

    #[error("device timeout: {0}")]
    Timeout(String),

    #[error("internal error: {0}")]
    InternalError(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = HardwareError::DeviceNotFound("cpu0".to_string());
        assert!(err.to_string().contains("cpu0"));
    }

    #[test]
    fn test_error_from_io() {
        let io_err = std::io::Error::other("test");
        let err: HardwareError = io_err.into();
        assert!(matches!(err, HardwareError::IoError(_)));
    }
}
