//! P7.8-04 — Reproducibility.
//!
//! An [`EnvironmentRecord`] captures the full execution context (source commit,
//! toolchain, dependencies, runtime, hardware, seed, parameters, dataset/model
//! versions, configuration) so a run can be reproduced. Two records are
//! `compatible` when the reproduction-relevant fields match.

use serde::{Deserialize, Serialize};
use serde_json::Value as Json;

/// The environment a research run executed under.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvironmentRecord {
    pub source_commit: String,
    pub compiler: String,
    pub toolchain: String,
    pub dependencies: Vec<String>,
    pub runtime_version: String,
    pub hardware: String,
    pub backend: String,
    pub seed: u64,
    pub parameters: Json,
    pub configuration: Json,
    pub dataset_version: Option<String>,
    pub model_version: Option<String>,
}

impl EnvironmentRecord {
    /// Whether two records are reproduction-compatible (all reproduction-
    /// relevant fields equal).
    pub fn is_reproducible_with(&self, other: &EnvironmentRecord) -> bool {
        self.source_commit == other.source_commit
            && self.toolchain == other.toolchain
            && self.dependencies == other.dependencies
            && self.runtime_version == other.runtime_version
            && self.hardware == other.hardware
            && self.seed == other.seed
            && self.parameters == other.parameters
            && self.configuration == other.configuration
            && self.dataset_version == other.dataset_version
            && self.model_version == other.model_version
    }
}

/// A reproducibility verdict.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Reproducibility {
    Reproducible,
    Diverged(String),
}

/// Compare a reproduction record against the original.
pub fn verify_reproduction(
    original: &EnvironmentRecord,
    reproduction: &EnvironmentRecord,
) -> Reproducibility {
    if original.is_reproducible_with(reproduction) {
        Reproducibility::Reproducible
    } else {
        Reproducibility::Diverged("environment differs from original run".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn env(seed: u64, param: i64) -> EnvironmentRecord {
        EnvironmentRecord {
            source_commit: "abc123".into(),
            compiler: "rustc".into(),
            toolchain: "1.98".into(),
            dependencies: vec!["serde".into()],
            runtime_version: "0.5.0".into(),
            hardware: "host-x86_64".into(),
            backend: "qpu-sim".into(),
            seed,
            parameters: serde_json::json!({"p": param}),
            configuration: serde_json::json!({"c": 1}),
            dataset_version: Some("d1@1".into()),
            model_version: None,
        }
    }

    #[test]
    fn identical_environments_reproducible() {
        let a = env(42, 1);
        let b = env(42, 1);
        assert_eq!(verify_reproduction(&a, &b), Reproducibility::Reproducible);
    }

    #[test]
    fn different_seed_diverges() {
        let a = env(42, 1);
        let b = env(43, 1);
        assert!(matches!(
            verify_reproduction(&a, &b),
            Reproducibility::Diverged(_)
        ));
    }

    #[test]
    fn different_parameters_diverges() {
        let a = env(42, 1);
        let b = env(42, 2);
        assert!(matches!(
            verify_reproduction(&a, &b),
            Reproducibility::Diverged(_)
        ));
    }

    #[test]
    fn different_source_commit_diverges() {
        let mut b = env(42, 1);
        b.source_commit = "def456".into();
        assert!(matches!(
            verify_reproduction(&env(42, 1), &b),
            Reproducibility::Diverged(_)
        ));
    }
}
