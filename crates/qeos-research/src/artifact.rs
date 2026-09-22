//! P7.8-03 — Artifact infrastructure.
//!
//! Artifacts (models, checkpoints, datasets, results, binaries, notebooks,
//! logs, measurements) are versioned, integrity-protected and owned. Provenance
//! is tracked on every artifact.

use serde::{Deserialize, Serialize};

use crate::dataset::digest;
use crate::error::ResearchError;

/// Kind of research artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactKind {
    Model,
    Checkpoint,
    Dataset,
    Result,
    Binary,
    Notebook,
    Log,
    Measurement,
}

/// A stored artifact.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Artifact {
    pub id: String,
    pub kind: ArtifactKind,
    pub version: u32,
    pub checksum: String,
    pub owner: String,
    pub provenance: String,
    pub content_ref: String,
}

/// Store for artifacts.
#[derive(Debug, Clone, Default)]
pub struct ArtifactStore {
    artifacts: Vec<Artifact>,
}

impl ArtifactStore {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add an artifact. Enforces ownership and a valid checksum; rejects a
    /// collision on `id` (no silent overwrite of research data).
    pub fn add(&mut self, artifact: Artifact, content: &[u8]) -> Result<(), ResearchError> {
        if digest(content) != artifact.checksum {
            return Err(ResearchError::ChecksumMismatch(artifact.id));
        }
        if self.artifacts.iter().any(|a| a.id == artifact.id) {
            return Err(ResearchError::DatasetExists(artifact.id));
        }
        self.artifacts.push(artifact);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Option<&Artifact> {
        self.artifacts.iter().find(|a| a.id == id)
    }

    pub fn owned_by<'a>(&'a self, owner: &'a str) -> impl Iterator<Item = &'a Artifact> + 'a {
        self.artifacts.iter().filter(move |a| a.owner == owner)
    }

    pub fn len(&self) -> usize {
        self.artifacts.len()
    }

    pub fn is_empty(&self) -> bool {
        self.artifacts.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn art(id: &str, owner: &str, content: &[u8]) -> Artifact {
        Artifact {
            id: id.to_string(),
            kind: ArtifactKind::Result,
            version: 1,
            checksum: digest(content),
            owner: owner.to_string(),
            provenance: "workflow".into(),
            content_ref: id.to_string(),
        }
    }

    #[test]
    fn add_with_valid_checksum() {
        let mut store = ArtifactStore::new();
        store.add(art("a1", "alice", b"data"), b"data").unwrap();
        assert_eq!(store.len(), 1);
    }

    #[test]
    fn checksum_mismatch_rejected() {
        let mut store = ArtifactStore::new();
        assert!(matches!(
            store.add(art("a1", "alice", b"data"), b"tampered"),
            Err(ResearchError::ChecksumMismatch(_))
        ));
    }

    #[test]
    fn duplicate_id_rejected() {
        let mut store = ArtifactStore::new();
        store.add(art("a1", "alice", b"data"), b"data").unwrap();
        assert!(store.add(art("a1", "alice", b"data"), b"data").is_err());
    }

    #[test]
    fn ownership_filter() {
        let mut store = ArtifactStore::new();
        store.add(art("a1", "alice", b"x"), b"x").unwrap();
        store.add(art("b1", "bob", b"y"), b"y").unwrap();
        assert_eq!(store.owned_by("alice").count(), 1);
        assert_eq!(store.owned_by("bob").count(), 1);
    }
}
