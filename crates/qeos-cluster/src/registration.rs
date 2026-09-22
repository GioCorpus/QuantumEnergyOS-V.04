//! P7.5-02 — Node registration lifecycle.
//!
//! A node transitions through a validated registration state machine:
//!
//! ```text
//! DISCOVERED -> PENDING -> REGISTERED -> HEALTHY <-> DEGRADED
//!                                ^           |
//!                                |___________|   (re-healthy)
//! REGISTERED/HEALTHY/DEGRADED -> OFFLINE <-> (reconnect -> re-register)
//! any active -> REVOKED (terminal)
//! ```

use serde::{Deserialize, Serialize};

use crate::error::ClusterError;

/// Registration lifecycle of a node.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeRegistrationState {
    /// Known but not yet registered.
    #[default]
    Discovered,
    /// Registration is being validated (challenge/credential check).
    Pending,
    /// Credentials verified and membership granted.
    Registered,
    /// Registered and healthy (heartbeats current).
    Healthy,
    /// Registered but degraded (partial capability or elevated error rate).
    Degraded,
    /// Lost heartbeat / unreachable.
    Offline,
    /// Revoked — terminal; cannot be re-added without explicit re-issue.
    Revoked,
}

impl NodeRegistrationState {
    pub fn can_transition_to(self, next: NodeRegistrationState) -> Result<(), ClusterError> {
        let allowed: &[NodeRegistrationState] = match self {
            NodeRegistrationState::Discovered => &[
                NodeRegistrationState::Pending,
                NodeRegistrationState::Revoked,
            ],
            NodeRegistrationState::Pending => &[
                NodeRegistrationState::Registered,
                NodeRegistrationState::Revoked,
            ],
            NodeRegistrationState::Registered => &[
                NodeRegistrationState::Healthy,
                NodeRegistrationState::Offline,
                NodeRegistrationState::Revoked,
            ],
            NodeRegistrationState::Healthy => &[
                NodeRegistrationState::Degraded,
                NodeRegistrationState::Offline,
                NodeRegistrationState::Revoked,
            ],
            NodeRegistrationState::Degraded => &[
                NodeRegistrationState::Healthy,
                NodeRegistrationState::Offline,
                NodeRegistrationState::Revoked,
            ],
            NodeRegistrationState::Offline => &[
                NodeRegistrationState::Pending, // re-authenticate on reconnect
                NodeRegistrationState::Revoked,
            ],
            // Revoked is terminal.
            NodeRegistrationState::Revoked => &[],
        };

        if allowed.contains(&next) {
            Ok(())
        } else {
            Err(ClusterError::InvalidRegistration(format!(
                "illegal registration transition {self:?} -> {next:?}"
            )))
        }
    }

    /// Whether the node may currently receive workloads.
    pub fn can_accept_workloads(self) -> bool {
        matches!(
            self,
            NodeRegistrationState::Healthy | NodeRegistrationState::Registered
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn happy_path() {
        let mut s = NodeRegistrationState::Discovered;
        s = after(s, NodeRegistrationState::Pending);
        s = after(s, NodeRegistrationState::Registered);
        s = after(s, NodeRegistrationState::Healthy);
        s = after(s, NodeRegistrationState::Degraded);
        s = after(s, NodeRegistrationState::Healthy);
        assert!(s.can_accept_workloads());
    }

    #[test]
    fn revocation_is_terminal() {
        let mut s = NodeRegistrationState::Healthy;
        s = after(s, NodeRegistrationState::Revoked);
        assert!(!s.can_accept_workloads());
        assert!(s.can_transition_to(NodeRegistrationState::Healthy).is_err());
    }

    #[test]
    fn offline_reconnect_requires_reauth() {
        let mut s = NodeRegistrationState::Healthy;
        s = after(s, NodeRegistrationState::Offline);
        // Cannot jump straight back to Healthy without re-authenticating.
        assert!(s.can_transition_to(NodeRegistrationState::Healthy).is_err());
        s = after(s, NodeRegistrationState::Pending);
        s = after(s, NodeRegistrationState::Registered);
        s = after(s, NodeRegistrationState::Healthy);
        assert!(s.can_accept_workloads());
    }

    #[test]
    fn cannot_skip_to_healthy_from_discovered() {
        assert!(NodeRegistrationState::Discovered
            .can_transition_to(NodeRegistrationState::Healthy)
            .is_err());
    }

    fn after(s: NodeRegistrationState, next: NodeRegistrationState) -> NodeRegistrationState {
        s.can_transition_to(next).unwrap();
        next
    }
}
