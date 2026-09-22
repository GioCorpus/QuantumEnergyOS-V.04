//! P7.5-01 — Node identity.
//!
//! Every node carries a stable [`NodeIdentity`]: a unique `node_id`, a
//! capability summary, software version and hardware metadata, plus a
//! **cryptographically bound credential**: we store only a SHA-256 hash of the
//! node's secret token (never the secret), so a control plane can prove a node
//! knows its credential without holding a plaintext secret at rest.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

/// Stable unique identifier of a node.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub String);

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Summary of a node's capabilities (from the node platform).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub supports_cpu: bool,
    pub supports_gpu: bool,
    pub supports_qpu: bool,
    pub supports_storage: bool,
    pub supports_network: bool,
}

/// Metadata describing the node's hardware (from the node inventory).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeHardware {
    pub cpu_cores: u32,
    pub memory_bytes: u64,
    pub storage_bytes: u64,
    pub gpu_count: u32,
    pub qpu_count: u32,
}

/// Full identity of a QEOS node.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeIdentity {
    pub node_id: NodeId,
    /// SHA-256 hex digest of the node's secret token. Used for proof-of-knowledge.
    pub credential_fingerprint: String,
    pub capabilities: NodeCapabilities,
    pub software_version: String,
    pub hardware: NodeHardware,
}

/// Compute the SHA-256 hex digest of a secret token.
pub fn credential_fingerprint(secret: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(secret);
    let digest = hasher.finalize();
    let mut hex = String::with_capacity(64);
    for byte in digest {
        hex.push_str(&format!("{byte:02x}"));
    }
    hex
}

/// A node that is being presented to a control plane for registration.
#[derive(Debug, Clone)]
pub struct RegistrationRequest {
    pub identity: NodeIdentity,
    /// The caller-held secret (sent only over a protected channel; never stored).
    pub secret: Vec<u8>,
}

impl RegistrationRequest {
    /// Create a registration request from a node identity.
    pub fn new(node_id: &str, secret: &[u8]) -> Self {
        Self {
            identity: NodeIdentity {
                node_id: NodeId(node_id.to_string()),
                credential_fingerprint: credential_fingerprint(secret),
                capabilities: NodeCapabilities::default(),
                software_version: "0.5.0".into(),
                hardware: NodeHardware::default(),
            },
            secret: secret.to_vec(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fingerprint_is_deterministic() {
        assert_eq!(
            credential_fingerprint(b"secret"),
            credential_fingerprint(b"secret")
        );
        // Different secrets produce different fingerprints (with overwhelming
        // probability).
        assert_ne!(
            credential_fingerprint(b"secret"),
            credential_fingerprint(b"secret2")
        );
    }

    #[test]
    fn fingerprint_does_not_leak_secret() {
        let fp = credential_fingerprint(b"my-secret-token");
        assert!(!fp.contains("my-secret-token"));
        assert_eq!(fp.len(), 64); // 256-bit hex
    }

    #[test]
    fn request_binds_identity_to_secret() {
        let req = RegistrationRequest::new("node-1", b"token");
        assert_eq!(req.identity.node_id, NodeId("node-1".into()));
        assert_eq!(
            req.identity.credential_fingerprint,
            credential_fingerprint(b"token")
        );
    }
}
