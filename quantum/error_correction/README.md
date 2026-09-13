# Quantum Error Correction (V.04 §5-§6, MODEL)

Canonical implementation: `crates/quantum-runtime/src/error_correction.rs` (`Syndrome`, `LogicalQubit`, `CorrectionStrategy`) plus `topology.rs::TopologicalErrorModel`.

Models only: repetition-code majority vote, parity-check syndromes, detection-only mode. No physical fidelity claimed; simulation metadata required on all outputs.