//! P7.5-03/04 — Cluster control plane.
//!
//! The control plane is the **single authority** for membership, heartbeats,
//! health, capabilities, configuration and jobs. Control-plane decisions are
//! observable and auditable. It does not assume nodes are always reachable:
//! heartbeat timeouts, duplicate registration, stale connections, node crashes
//! and credential mismatches are all handled explicitly.
//!
//! ## Consistency / membership model
//!
//! - **Membership authority**: the control plane (single writer, no split-brain).
//! - **Failure model**: nodes may crash or lose connectivity at any time;
//!   reachability is derived from heartbeats, never assumed.
//! - **Stale handling**: a reconnect bumps the session generation, which
//!   invalidates any older connection (stale heartbeats are rejected).

use crate::error::{ClusterError, Result};
use crate::identity::{NodeIdentity, RegistrationRequest};
use crate::membership::Membership;
use crate::registration::NodeRegistrationState;

/// The cluster control plane.
#[derive(Debug)]
pub struct ControlPlane {
    pub cluster_id: String,
    membership: Membership,
    heartbeat_timeout_ms: u64,
    now_ms: u64,
}

impl ControlPlane {
    pub fn new(cluster_id: &str, heartbeat_timeout_ms: u64) -> Self {
        Self {
            cluster_id: cluster_id.to_string(),
            membership: Membership::new(),
            heartbeat_timeout_ms,
            now_ms: 0,
        }
    }

    /// Advance the logical clock (usually called once per control-loop tick).
    /// Detects nodes that stopped sending heartbeats.
    pub fn tick(&mut self, now_ms: u64) {
        self.now_ms = now_ms;
        self.membership
            .expire_missing(self.now_ms, self.heartbeat_timeout_ms);
    }

    /// P7.5-02 register — verify the presented credential against the stored
    /// fingerprint, then admit the node to membership and mark it registered.
    pub fn register(&mut self, request: RegistrationRequest) -> Result<NodeIdentity> {
        let fingerprint = crate::identity::credential_fingerprint(&request.secret);
        if fingerprint != request.identity.credential_fingerprint {
            return Err(ClusterError::CredentialMismatch(
                request.identity.node_id.0.clone(),
            ));
        }
        self.membership.admit(request.identity.clone())?;
        self.membership
            .set_state(
                &request.identity.node_id.0,
                NodeRegistrationState::Registered,
            )
            .map_err(|_| ClusterError::InvalidRegistration("admission failed".into()))?;
        Ok(request.identity)
    }

    /// P7.5-02 authenticate + heartbeat — proves credential knowledge, checks
    /// the session generation (rejects stale/duplicate connections) and updates
    /// liveness. A registered node is promoted to Healthy.
    pub fn heartbeat(
        &mut self,
        request: &RegistrationRequest,
        claimed_generation: u64,
    ) -> Result<()> {
        let fingerprint = crate::identity::credential_fingerprint(&request.secret);
        if fingerprint != request.identity.credential_fingerprint {
            return Err(ClusterError::CredentialMismatch(
                request.identity.node_id.0.clone(),
            ));
        }
        let id = &request.identity.node_id.0;
        let state = self
            .membership
            .get(id)
            .map(|m| m.state)
            .ok_or_else(|| ClusterError::NodeUnknown(id.clone()))?;
        if state == NodeRegistrationState::Revoked {
            return Err(ClusterError::InvalidRegistration("node is revoked".into()));
        }

        // Generation check first: a heartbeat from an older session is stale.
        self.membership
            .heartbeat(id, self.now_ms, claimed_generation)?;

        // Promote to Healthy if the node is already registered/active.
        if matches!(
            state,
            NodeRegistrationState::Registered
                | NodeRegistrationState::Healthy
                | NodeRegistrationState::Degraded
        ) {
            self.membership
                .set_state(id, NodeRegistrationState::Healthy)?;
        }
        Ok(())
    }

    /// Begin a reconnect: bump session generation (invalidating older
    /// connections) and move the node to OFFLINE. It must re-register to
    /// become Healthy again.
    pub fn begin_reconnect(&mut self, node_id: &str) -> Result<u64> {
        self.membership.begin_reconnect(node_id)
    }

    /// P7.5-03 drain a node so it stops receiving workloads.
    pub fn drain(&mut self, node_id: &str) -> Result<()> {
        self.membership
            .set_state(node_id, NodeRegistrationState::Offline)
    }

    /// P7.5-03/05 revoke a node (terminal).
    pub fn revoke(&mut self, node_id: &str) -> Result<()> {
        self.membership
            .set_state(node_id, NodeRegistrationState::Revoked)
    }

    /// Remove a stale node record.
    pub fn remove(&mut self, node_id: &str) {
        let _ = self.membership.remove_member(node_id);
    }

    /// Snapshot of current members.
    pub fn members(&self) -> Vec<crate::membership::Member> {
        self.membership.members().cloned().collect()
    }

    pub fn member(&self, node_id: &str) -> Option<&crate::membership::Member> {
        self.membership.get(node_id)
    }

    pub fn healthy_count(&self) -> usize {
        self.membership.healthy_count()
    }

    pub fn membership_len(&self) -> usize {
        self.membership.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::RegistrationRequest;

    #[test]
    fn register_and_heartbeat() {
        let mut cp = ControlPlane::new("c1", 2000);
        let req = RegistrationRequest::new("n1", b"token");
        cp.register(req.clone()).unwrap();
        cp.tick(1000);
        cp.heartbeat(&req, 0).unwrap();
        assert_eq!(cp.healthy_count(), 1);
    }

    #[test]
    fn wrong_credential_rejected_at_register() {
        let mut cp = ControlPlane::new("c1", 2000);
        let mut req = RegistrationRequest::new("n1", b"token");
        // Attacker presents a correct identity but a wrong secret.
        req.secret = b"wrong".to_vec();
        assert!(matches!(
            cp.register(req),
            Err(ClusterError::CredentialMismatch(_))
        ));
    }

    #[test]
    fn heartbeat_loss_detected() {
        let mut cp = ControlPlane::new("c1", 2000);
        let req = RegistrationRequest::new("n1", b"token");
        cp.register(req.clone()).unwrap();
        cp.tick(1000);
        cp.heartbeat(&req, 0).unwrap();
        assert_eq!(cp.healthy_count(), 1);
        // No heartbeat for > 2000ms.
        cp.tick(5000);
        assert_eq!(cp.healthy_count(), 0);
        assert_eq!(
            cp.member("n1").unwrap().state,
            NodeRegistrationState::Offline
        );
    }

    #[test]
    fn duplicate_registration_rejected() {
        let mut cp = ControlPlane::new("c1", 2000);
        let req = RegistrationRequest::new("n1", b"token");
        cp.register(req.clone()).unwrap();
        // Second registration while active is rejected.
        assert!(cp.register(req).is_err());
    }

    #[test]
    fn revocation_is_terminal() {
        let mut cp = ControlPlane::new("c1", 2000);
        let req = RegistrationRequest::new("n1", b"token");
        cp.register(req.clone()).unwrap();
        cp.revoke("n1").unwrap();
        assert_eq!(cp.healthy_count(), 0);
        assert_eq!(
            cp.member("n1").unwrap().state,
            NodeRegistrationState::Revoked
        );
    }

    #[test]
    fn reconnect_invalidates_stale_generation() {
        let mut cp = ControlPlane::new("c1", 2000);
        let req = RegistrationRequest::new("n1", b"token");
        cp.register(req.clone()).unwrap();
        let gen = cp.begin_reconnect("n1").unwrap();
        assert_eq!(gen, 1);
        cp.tick(100);
        // The node moved OFFLINE and is not accepting workloads.
        assert_eq!(cp.healthy_count(), 0);
        // Old generation (0) heartbeat rejected as stale (older session).
        assert!(matches!(
            cp.heartbeat(&req, 0),
            Err(ClusterError::StaleHeartbeat)
        ));
        // A correct-generation heartbeat updates liveness but the node must
        // re-register to become Healthy again.
        assert!(cp.heartbeat(&req, 1).is_ok());
        assert_eq!(cp.healthy_count(), 0);
    }
}
