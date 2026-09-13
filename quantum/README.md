# Quantum Runtime — OS Integration (V.04 §5–§6)

Canonical implementation: `crates/quantum-runtime`
(`core`/`circuit`/`compiler`/`simulator`/`topology`/`backends`/`scheduler`).

- `core/`: qubits, registers, circuits (see `circuit.rs`, `gates.rs`).
- `circuit/`: high-level circuit model.
- `compiler/`: `compiler.rs` — Circuit → IR → backend artifact.
- `simulator/`: `simulator.rs` state-vector SIMULATION backend.
- `topology/`: `topology.rs` — MajoranaZeroMode, parity, tetron-like, braiding, lattice, error model.
- `backends/`: `backend.rs` — `QuantumProcessor` trait, Simulator/Emulator/Remote/Majorana adapters.
- `scheduler/`: `scheduler.rs` + `job.rs` — queue, priority, deadlines, cancellation, availability.

`MajoranaBackend` is capability-gated (disabled without a documented hardware
interface). All simulation outputs carry `SimulationMetadata` with backend,
model, assumptions, and optional fidelity.