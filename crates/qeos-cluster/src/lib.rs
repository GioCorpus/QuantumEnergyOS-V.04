//! # QEOS Cluster — P7.5 distributed node architecture & cluster control plane
//!
//! Node identity (cryptographically bound), registration lifecycle, cluster
//! membership and the control plane authority model. The control plane is the
//! single authority for membership; the consistency model is
//! single-authority (no split-brain assumption).
//!
//! ## Reality classification
//!
//! - Node identity / credential fingerprinting — **REAL** (SHA-256 binding).
//! - Registration / membership / control plane logic — **REAL** (host-testable).
//! - Physical node-to-node networking transport — **NOT IMPLEMENTED** in this
//!   phase (the control plane models reachability via heartbeats; a network
//!   transport is a later integration point).

#![forbid(unsafe_code)]

pub mod control_plane;
pub mod error;
pub mod identity;
pub mod membership;
pub mod registration;

pub use control_plane::ControlPlane;
pub use error::{ClusterError, Result};
pub use identity::{
    credential_fingerprint, NodeCapabilities, NodeHardware, NodeId, NodeIdentity,
    RegistrationRequest,
};
pub use membership::{Member, Membership};
pub use registration::NodeRegistrationState;

/// QEOS cluster crate version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
