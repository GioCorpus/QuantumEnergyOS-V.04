# QEOS V.04 — Experiment Reproducibility

**Status:** STABLE (as of Phase 7.8)

## 1. Reproducibility model

Every run records an `EnvironmentRecord`:

- source commit
- compiler / toolchain
- dependencies
- runtime version
- hardware / backend
- seed
- parameters / configuration
- dataset version / model version

`verify_reproduction(original, reproduction)` returns `Reproducible` when all
reproduction-relevant fields match, else `Diverged`.

## 2. Deterministic replay

Simulation backends are seeded (`seed`), so running the same job with the same
seed produces identical outcomes (verified by the `qeos-qpu` repeatability test
and the `qeos experiment run` workflow).

## 3. Example

`qeos experiment run --seed 42`:
1. Validates the Dataset→QuantumSimulation→Measurement→Analysis→Artifact
   workflow.
2. Runs a deterministic seeded QPU measurement.
3. Advances the experiment lifecycle to Completed.
4. Records the environment for later verification.

## 4. Honesty

Simulation is always tagged `SIMULATED`; real hardware (GPU/QPU) is UNAVAILABLE
and never claimed as reproducible hardware results.
