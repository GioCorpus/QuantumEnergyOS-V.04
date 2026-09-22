//! P7.5-05 — Cluster membership semantics.
//!
//! The **control plane** is the single authority that decides membership
//! (join, leave, revocation, stale handling). This module documents and
//! enforces that authority model rather than assuming every node is equal or
//! that membership is self-managed. The consistency model is
//! **eventually consistent single-authority**: the control plane is the sole
//! writer of membership state, which avoids split-brain assumptions.

use serde::{Deserialize, Serialize};

use crate::error::{ClusterError, Result};
use crate::identity::NodeIdentity;
use crate::registration::NodeRegistrationState;

/// A member node known to the control plane.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Member {
    pub identity: NodeIdentity,
    pub state: NodeRegistrationState,
    /// Unix milliseconds of the last accepted heartbeat.
    pub last_heartbeat_ms: u64,
    /// Whether the node is currently reachable (liveness).
    pub reachable: bool,
    /// Generation counter; bumped on reconnect to invalidate stale sessions.
    pub session_generation: u64,
}

impl Member {
    pub fn node_id(&self) -> &str {
        &self.identity.node_id.0
    }
}

/// Cluster membership authority.
#[derive(Debug, Clone)]
pub struct Membership {
    nodes: std::collections::HashMap<String, Member>,
}

impl Default for Membership {
    fn default() -> Self {
        Self::new()
    }
}

impl Membership {
    pub fn new() -> Self {
        Self {
            nodes: std::collections::HashMap::new(),
        }
    }

    pub fn len(&self) -> usize {
        self.nodes.len()
    }

    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }

    pub fn get(&self, node_id: &str) -> Option<&Member> {
        self.nodes.get(node_id)
    }

    /// Admit a node into membership (PENDING). Rejects duplicate active
    /// registrations of the same node id unless it is currently offline/revoked.
    pub fn admit(&mut self, identity: NodeIdentity) -> Result<()> {
        let id = identity.node_id.0.clone();
        if let Some(existing) = self.nodes.get(&id) {
            // Duplicate registration of an already-active node is rejected.
            if existing.state != NodeRegistrationState::Offline
                && existing.state != NodeRegistrationState::Revoked
                && existing.state != NodeRegistrationState::Discovered
            {
                return Err(ClusterError::InvalidRegistration(format!(
                    "duplicate active registration for node {id}"
                )));
            }
        }
        self.nodes.insert(
            id.clone(),
            Member {
                identity,
                state: NodeRegistrationState::Pending,
                last_heartbeat_ms: 0,
                reachable: false,
                session_generation: 0,
            },
        );
        Ok(())
    }

    /// Transition a member's registration state (validated).
    pub fn set_state(&mut self, node_id: &str, next: NodeRegistrationState) -> Result<()> {
        let member = self
            .nodes
            .get_mut(node_id)
            .ok_or_else(|| ClusterError::NodeUnknown(node_id.to_string()))?;
        member.state.can_transition_to(next)?;
        member.state = next;
        if next == NodeRegistrationState::Revoked {
            member.reachable = false;
        }
        Ok(())
    }

    /// Record a heartbeat. Returns the previous session generation (used to
    /// detect a stale/duplicate connection that should be rejected).
    pub fn heartbeat(
        &mut self,
        node_id: &str,
        now_ms: u64,
        claimed_generation: u64,
    ) -> Result<u64> {
        let member = self
            .nodes
            .get_mut(node_id)
            .ok_or_else(|| ClusterError::NodeUnknown(node_id.to_string()))?;
        // Reject a heartbeat from an older (stale) session.
        if claimed_generation != member.session_generation {
            return Err(ClusterError::StaleHeartbeat);
        }
        member.last_heartbeat_ms = now_ms;
        member.reachable = true;
        Ok(member.session_generation)
    }

    /// Mark nodes whose heartbeat is older than `timeout_ms` as offline.
    /// Returns the node ids that transitioned to offline.
    pub fn expire_missing(&mut self, now_ms: u64, timeout_ms: u64) -> Vec<String> {
        let mut expired = Vec::new();
        for member in self.nodes.values_mut() {
            if member.reachable
                && member.last_heartbeat_ms > 0
                && now_ms.saturating_sub(member.last_heartbeat_ms) > timeout_ms
            {
                member.reachable = false;
                // Move to offline unless revoked/discovered.
                if member.state != NodeRegistrationState::Revoked
                    && member.state != NodeRegistrationState::Discovered
                    && member.state != NodeRegistrationState::Pending
                {
                    let _ = member
                        .state
                        .can_transition_to(NodeRegistrationState::Offline);
                    member.state = NodeRegistrationState::Offline;
                }
                expired.push(member.node_id().to_string());
            }
        }
        expired
    }

    /// Reconnect: bump the session generation (invalidating any older
    /// connection) and move the node to OFFLINE. It must re-register to become
    /// Healthy again.
    pub fn begin_reconnect(&mut self, node_id: &str) -> Result<u64> {
        let member = self
            .nodes
            .get_mut(node_id)
            .ok_or_else(|| ClusterError::NodeUnknown(node_id.to_string()))?;
        member.session_generation += 1;
        if member.state != NodeRegistrationState::Revoked
            && member.state != NodeRegistrationState::Discovered
            && member
                .state
                .can_transition_to(NodeRegistrationState::Offline)
                .is_ok()
        {
            member.state = NodeRegistrationState::Offline;
        }
        member.reachable = false;
        Ok(member.session_generation)
    }

    /// Remove a member record entirely (used after revocation cleanup).
    pub fn remove_member(&mut self, node_id: &str) -> Option<Member> {
        self.nodes.remove(node_id)
    }

    /// Iterate members.
    pub fn members(&self) -> impl Iterator<Item = &Member> {
        self.nodes.values()
    }

    /// Number of nodes currently able to accept workloads.
    pub fn healthy_count(&self) -> usize {
        self.nodes
            .values()
            .filter(|m| m.state.can_accept_workloads())
            .count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::RegistrationRequest;

    fn req(id: &str) -> RegistrationRequest {
        RegistrationRequest::new(id, b"token")
    }

    /// Admit and promote through the valid Pending -> Registered -> Healthy path.
    fn activate(m: &mut Membership, id: &str) {
        m.set_state(id, NodeRegistrationState::Registered).unwrap();
        m.set_state(id, NodeRegistrationState::Healthy).unwrap();
    }

    #[test]
    fn admit_and_heartbeat() {
        let mut m = Membership::new();
        m.admit(req("n1").identity).unwrap();
        activate(&mut m, "n1");
        m.heartbeat("n1", 1000, 0).unwrap();
        assert!(m.get("n1").unwrap().reachable);
        assert_eq!(m.healthy_count(), 1);
    }

    #[test]
    fn duplicate_active_registration_rejected() {
        let mut m = Membership::new();
        m.admit(req("n1").identity.clone()).unwrap();
        activate(&mut m, "n1");
        assert!(m.admit(req("n1").identity).is_err());
    }

    #[test]
    fn offline_duplicate_allowed() {
        let mut m = Membership::new();
        m.admit(req("n1").identity).unwrap();
        activate(&mut m, "n1");
        // Force offline then re-admit (re-register).
        m.heartbeat("n1", 100_000, 0).unwrap();
        m.expire_missing(200_000, 1000);
        assert_eq!(m.get("n1").unwrap().state, NodeRegistrationState::Offline);
        // Re-admitting an offline node is allowed (fresh session).
        m.admit(req("n1").identity).unwrap();
        assert_eq!(m.get("n1").unwrap().state, NodeRegistrationState::Pending);
    }

    #[test]
    fn heartbeat_timeout_detects_failure() {
        let mut m = Membership::new();
        m.admit(req("n1").identity).unwrap();
        activate(&mut m, "n1");
        m.heartbeat("n1", 1000, 0).unwrap();
        let expired = m.expire_missing(5000, 2000);
        assert_eq!(expired, vec!["n1".to_string()]);
        assert_eq!(m.get("n1").unwrap().state, NodeRegistrationState::Offline);
    }

    #[test]
    fn stale_heartbeat_rejected() {
        let mut m = Membership::new();
        m.admit(req("n1").identity).unwrap();
        m.begin_reconnect("n1").unwrap(); // generation -> 1
                                          // A heartbeat claiming the old generation (0) is stale.
        assert!(matches!(
            m.heartbeat("n1", 1000, 0),
            Err(ClusterError::StaleHeartbeat)
        ));
        // Correct generation accepted.
        m.heartbeat("n1", 1000, 1).unwrap();
    }

    #[test]
    fn revoked_not_healthy() {
        let mut m = Membership::new();
        m.admit(req("n1").identity).unwrap();
        activate(&mut m, "n1");
        m.set_state("n1", NodeRegistrationState::Revoked).unwrap();
        assert_eq!(m.healthy_count(), 0);
        assert!(!m.get("n1").unwrap().reachable);
    }
}
