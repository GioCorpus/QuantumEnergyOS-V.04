//! P7.7-06 — Checkpointing.
//!
//! Checkpoints are versioned, integrity-protected snapshots of a job's state.
//! They support checkpoint/resume/retry/partial-recovery. Integrity is enforced
//! with a SHA-256 checksum; version compatibility is checked on load so a
//! checkpoint from an incompatible version is rejected rather than silently
//! mis-loaded.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{ClusterError, Result};

/// A versioned, integrity-protected checkpoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Checkpoint {
    pub checkpoint_format_version: u32,
    pub job_id: String,
    pub data: String,
    pub checksum: String,
}

impl Checkpoint {
    /// Create a checkpoint and immediately compute its integrity checksum.
    pub fn new(version: u32, job_id: &str, data: &str) -> Self {
        let ck = Self {
            checkpoint_format_version: version,
            job_id: job_id.to_string(),
            data: data.to_string(),
            checksum: String::new(),
        };
        ck.with_checksum()
    }

    /// Recompute the checksum over the canonical payload.
    fn with_checksum(mut self) -> Self {
        self.checksum = Self::digest(&self.canonical());
        self
    }

    fn canonical(&self) -> String {
        format!(
            "v{}:{job}:{data}",
            self.checkpoint_format_version,
            job = self.job_id,
            data = self.data
        )
    }

    fn digest(canonical: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.update(canonical.as_bytes());
        let out = hasher.finalize();
        let mut hex = String::with_capacity(64);
        for b in out {
            hex.push_str(&format!("{b:02x}"));
        }
        hex
    }

    /// Verify that the checkpoint's payload matches its checksum (integrity).
    pub fn verify_integrity(&self) -> bool {
        !self.checksum.is_empty() && self.checksum == Self::digest(&self.canonical())
    }

    /// Load a checkpoint and validate it against the current format version.
    pub fn load(&self, supported_version: u32) -> Result<()> {
        if !self.verify_integrity() {
            return Err(ClusterError::InvalidRegistration(
                "checkpoint checksum mismatch".into(),
            ));
        }
        if self.checkpoint_format_version != supported_version {
            return Err(ClusterError::InvalidRegistration(format!(
                "incompatible checkpoint format v{} (expected v{})",
                self.checkpoint_format_version, supported_version
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn integrity_holds() {
        let ck = Checkpoint::new(1, "job-1", "{}");
        assert!(ck.verify_integrity());
    }

    #[test]
    fn tampering_detected() {
        let mut ck = Checkpoint::new(1, "job-1", "{}");
        ck.data = "{\"altered\":true}".to_string();
        assert!(!ck.verify_integrity());
        assert!(ck.load(1).is_err());
    }

    #[test]
    fn version_mismatch_rejected() {
        let ck = Checkpoint::new(1, "job-1", "{}");
        assert!(ck.load(1).is_ok());
        assert!(ck.load(2).is_err());
    }

    #[test]
    fn checksum_deterministic() {
        let a = Checkpoint::new(1, "job-1", "{}");
        let b = Checkpoint::new(1, "job-1", "{}");
        assert_eq!(a.checksum, b.checksum);
    }
}
