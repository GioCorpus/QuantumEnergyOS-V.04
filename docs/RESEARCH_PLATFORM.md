# QEOS V.04 — Research Platform

**Status:** STABLE (as of Phase 7.8)

## 1. Overview

The research platform (`crates/qeos-research`) provides experiments, versioned
datasets, integrity-protected artifacts, reproducibility records and validated
workflows. It is executed and driven through the `qeos` CLI and the Rust crates.

## 2. Data model

- **Experiment** — identity, timestamp, user/versions, backend/device,
  parameters/configuration, seed, input/output, telemetry, artifact ids, status.
- **Dataset** — dataset_id, version, checksum, schema, source, license,
  provenance. Never silently overwritten (id+version is unique).
- **Artifact** — Model/Checkpoint/Dataset/Result/Binary/Notebook/Log/Measurement,
  with version, checksum, owner, provenance.
- **Workflow** — ordered Dataset→…→Artifact pipeline with timeout/retry/
  checkpointing and validation.

## 3. Guarantees

- Research data is **never silently overwritten**.
- Artifacts are **integrity-protected** (SHA-256) and owned.
- Environments are **recorded** for reproducibility.

## 4. Reality

Models and validation logic are REAL. Execution backends (CPU/GPU/QPU) are
SIMULATED. Notebook/Python runtime is not bundled (documented limitation).
