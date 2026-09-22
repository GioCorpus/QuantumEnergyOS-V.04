//! Error types for the cluster control plane.

use thiserror::Error;

/// Result alias.
pub type Result<T> = std::result::Result<T, ClusterError>;

/// Errors raised by the cluster control plane.
#[derive(Debug, Error, PartialEq, Eq)]
pub enum ClusterError {
    #[error("invalid registration: {0}")]
    InvalidRegistration(String),
    #[error("node unknown: {0}")]
    NodeUnknown(String),
    #[error("credential verification failed for node {0}")]
    CredentialMismatch(String),
    #[error("stale heartbeat (session generation mismatch)")]
    StaleHeartbeat,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn error_display() {
        assert!(ClusterError::NodeUnknown("n".into())
            .to_string()
            .contains("unknown"));
        assert!(ClusterError::StaleHeartbeat.to_string().contains("stale"));
    }
}
