//! P7.8-02 — Dataset infrastructure.
//!
//! Datasets are versioned and integrity-protected (SHA-256 checksum). The
//! registry **never silently overwrites** research data: registering an
//! existing `dataset_id` at the same version is rejected.

use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::ResearchError;

/// A versioned dataset.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub dataset_id: String,
    pub version: u32,
    pub checksum: String,
    pub schema: String,
    pub source: String,
    pub license: String,
    pub provenance: String,
}

/// Compute a SHA-256 hex digest for content.
pub fn digest(bytes: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(bytes);
    let out = h.finalize();
    let mut hex = String::with_capacity(64);
    for b in out {
        hex.push_str(&format!("{b:02x}"));
    }
    hex
}

/// Registry of datasets.
#[derive(Debug, Clone, Default)]
pub struct DatasetRegistry {
    datasets: Vec<Dataset>,
}

impl DatasetRegistry {
    pub fn new() -> Self {
        Self::default()
    }

    /// Register a new dataset version. Rejects overwriting an existing
    /// id+version (research data is never silently replaced).
    pub fn register(&mut self, dataset: Dataset) -> Result<(), ResearchError> {
        if self
            .datasets
            .iter()
            .any(|d| d.dataset_id == dataset.dataset_id && d.version == dataset.version)
        {
            return Err(ResearchError::DatasetExists(dataset.dataset_id));
        }
        self.datasets.push(dataset);
        Ok(())
    }

    pub fn get(&self, dataset_id: &str, version: u32) -> Option<&Dataset> {
        self.datasets
            .iter()
            .find(|d| d.dataset_id == dataset_id && d.version == version)
    }

    pub fn len(&self) -> usize {
        self.datasets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.datasets.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ds(id: &str, version: u32) -> Dataset {
        Dataset {
            dataset_id: id.to_string(),
            version,
            checksum: digest(b"content"),
            schema: "csv".into(),
            source: "test".into(),
            license: "MIT".into(),
            provenance: "generated".into(),
        }
    }

    #[test]
    fn register_new_version_ok() {
        let mut reg = DatasetRegistry::new();
        reg.register(ds("d1", 1)).unwrap();
        reg.register(ds("d1", 2)).unwrap();
        assert_eq!(reg.len(), 2);
    }

    #[test]
    fn silent_overwrite_rejected() {
        let mut reg = DatasetRegistry::new();
        reg.register(ds("d1", 1)).unwrap();
        assert!(matches!(
            reg.register(ds("d1", 1)),
            Err(ResearchError::DatasetExists(_))
        ));
        assert_eq!(reg.len(), 1);
    }

    #[test]
    fn get_by_id_version() {
        let mut reg = DatasetRegistry::new();
        reg.register(ds("d1", 1)).unwrap();
        assert!(reg.get("d1", 1).is_some());
        assert!(reg.get("d1", 2).is_none());
    }

    #[test]
    fn checksum_is_deterministic() {
        assert_eq!(digest(b"abc"), digest(b"abc"));
        assert_ne!(digest(b"abc"), digest(b"abd"));
    }
}
