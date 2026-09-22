//! P7.7-00/01 — Fleet identity and trust boundaries.
//!
//! Trust in a QEOS fleet is **explicit and directional**. This module documents
//! and models the trust relationships between principals (nodes, the control
//! plane, services, users, devices). Identities are enforced by the
//! `identity-service` (authentication, authorization, rotation, revocation,
//! audit) at integration time; here we model the boundary policy so that a
//! caller can programmatically query which relationships are trusted.

use serde::{Deserialize, Serialize};

/// A trust relationship between two principals.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct TrustEdge {
    pub from: PrincipalKind,
    pub to: PrincipalKind,
}

/// Kinds of principals in a fleet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PrincipalKind {
    User,
    Node,
    ControlPlane,
    Service,
    Device,
    RemoteCluster,
}

/// The fleet trust model.
#[derive(Debug, Clone)]
pub struct TrustModel {
    /// Allowed (from, to) trust edges.
    allowed: Vec<TrustEdge>,
}

impl Default for TrustModel {
    /// The default, least-privilege trust model (also [`TrustModel::least_privilege`]).
    fn default() -> Self {
        Self::least_privilege()
    }
}

impl TrustModel {
    /// The least-privilege trust model.
    ///
    /// - **Node → ControlPlane**: nodes register and report to the control plane.
    /// - **ControlPlane → Node**: control plane directs nodes.
    /// - **Node → Node**: only via explicit operator authorization (not default).
    /// - **Service → Service**: only within an authorized scope.
    /// - **User → ControlPlane**: authenticated users may issue authorized ops.
    /// - **User → Node**: only through the control plane (not direct).
    pub fn least_privilege() -> Self {
        Self {
            allowed: vec![
                TrustEdge {
                    from: PrincipalKind::Node,
                    to: PrincipalKind::ControlPlane,
                },
                TrustEdge {
                    from: PrincipalKind::ControlPlane,
                    to: PrincipalKind::Node,
                },
                TrustEdge {
                    from: PrincipalKind::User,
                    to: PrincipalKind::ControlPlane,
                },
                TrustEdge {
                    from: PrincipalKind::RemoteCluster,
                    to: PrincipalKind::ControlPlane,
                },
            ],
        }
    }

    /// Whether the model explicitly trusts `from -> to`.
    pub fn trusts(&self, from: PrincipalKind, to: PrincipalKind) -> bool {
        self.allowed.contains(&TrustEdge { from, to })
    }

    /// Add an explicit trust edge.
    pub fn add_edge(&mut self, edge: TrustEdge) {
        if !self.allowed.contains(&edge) {
            self.allowed.push(edge);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn node_trusts_control_plane_by_default() {
        let m = TrustModel::default();
        assert!(m.trusts(PrincipalKind::Node, PrincipalKind::ControlPlane));
        assert!(m.trusts(PrincipalKind::ControlPlane, PrincipalKind::Node));
        assert!(m.trusts(PrincipalKind::User, PrincipalKind::ControlPlane));
    }

    #[test]
    fn node_to_node_not_trusted_by_default() {
        let m = TrustModel::default();
        assert!(!m.trusts(PrincipalKind::Node, PrincipalKind::Node));
        assert!(!m.trusts(PrincipalKind::User, PrincipalKind::Node));
    }

    #[test]
    fn explicit_edge_can_be_granted() {
        let mut m = TrustModel::default();
        m.add_edge(TrustEdge {
            from: PrincipalKind::Service,
            to: PrincipalKind::Service,
        });
        assert!(m.trusts(PrincipalKind::Service, PrincipalKind::Service));
    }

    #[test]
    fn non_explicit_denied() {
        let m = TrustModel::default();
        assert!(!m.trusts(PrincipalKind::Device, PrincipalKind::Node));
    }
}
