# Quantum Topology (V.04 §6)

Canonical implementation: `crates/quantum-runtime/src/topology.rs`
(MODEL, never HARDWARE).

Mapping to spec files:

- `majorana.rs` → `MajoranaZeroMode` (id, position, parity)
- `parity.rs` → `ParityMeasurement`, `compute_combined_parity`
- `tetron.rs` → `TetronLikeLogicalQubit` (4-mode logical structure, protection flag)
- `braiding.rs` → `BraidingOperation`, `BraidingSequence`
- `lattice.rs` → `LatticeConfig`, `LatticePosition`
- `error_model.rs` → `ErrorModelConfig`, `TopologicalErrorModel`, plus
  `crates/quantum-runtime/src/error_correction.rs` (`Syndrome`, `LogicalQubit`)

Every simulation exposes `SimulationMetadata { backend, model, assumptions, fidelity }`.
No sine/cosine Majorana physics is claimed as real hardware behavior.