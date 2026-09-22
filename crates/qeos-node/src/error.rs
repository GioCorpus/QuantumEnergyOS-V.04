//! Error types for the QEOS node operations platform.
//!
//! All errors in this crate flow through [`NodeError`]. New error sources must
//! be added as explicit variants so that failure handling stays exhaustive and
//! observable at the boundary.

use thiserror::Error;

/// Result alias for fallible node operations.
pub type Result<T> = std::result::Result<T, NodeError>;

/// Errors raised by the node operations platform.
#[derive(Debug, Error)]
pub enum NodeError {
    #[error("illegal lifecycle transition: {from} -> {to}")]
    IllegalTransition { from: String, to: String },

    #[error("node is not in the required state to perform operation (required: {required}, actual: {actual})")]
    InvalidNodeState { required: String, actual: String },

    #[error("device {0} not found")]
    DeviceNotFound(String),

    #[error("duplicate device registration: {0}")]
    DuplicateDevice(String),

    #[error("invalid inventory operation: {0}")]
    InvalidInventory(String),

    #[error("inventory persistence error: {0}")]
    Persistence(String),

    #[error("hardware source unavailable: {0}")]
    HardwareSourceUnavailable(String),

    #[error(transparent)]
    Hardware(#[from] hardware_abstraction::HardwareError),

    #[error("serde error: {0}")]
    Serde(#[from] serde_json::Error),

    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn illegal_transition_error_display() {
        let e = NodeError::IllegalTransition {
            from: "READY".into(),
            to: "BOOTING".into(),
        };
        assert!(e.to_string().contains("illegal lifecycle transition"));
    }

    #[test]
    fn hardware_error_conversion() {
        let _ = NodeError::Hardware(hardware_abstraction::HardwareError::UnsupportedDevice(
            "test".into(),
        ));
    }
}
