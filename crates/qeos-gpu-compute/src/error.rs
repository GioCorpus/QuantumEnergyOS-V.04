//! Error types for QEOS GPU compute.
//!
//! The error set is explicit so failure paths (stale handles, allocation
//! failure, device removal, timeout) are never silently swallowed.

use thiserror::Error;

/// Result alias for GPU operations.
pub type Result<T> = std::result::Result<T, GpuError>;

/// Errors raised by the GPU compute runtime.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum GpuError {
    #[error("backend unavailable: {0}")]
    Unavailable(String),
    #[error("bad buffer: {0}")]
    BadBuffer(String),
    #[error("dispatch failed: {0}")]
    Dispatch(String),
    #[error("allocation failed: {0}")]
    AllocationFailed(String),
    #[error("stale or invalid device handle")]
    StaleHandle,
    #[error("device has been removed/lost")]
    DeviceRemoved,
    #[error("mapping conflict: {0}")]
    MappingConflict(String),
    #[error("operation timed out")]
    Timeout,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        assert!(GpuError::StaleHandle.to_string().contains("stale"));
        assert!(GpuError::DeviceRemoved.to_string().contains("removed"));
        assert!(GpuError::Timeout.to_string().contains("timed out"));
    }

    #[test]
    fn error_eq() {
        assert_eq!(GpuError::StaleHandle, GpuError::StaleHandle);
    }
}
