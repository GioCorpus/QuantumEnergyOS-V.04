# Quantum Simulator (V.04 §5)

Canonical implementation: `crates/quantum-runtime/src/simulator.rs` + `backend.rs::SimulatorBackend`.

Classification: SIMULATION. State-vector evolution, no hardware. Exposes `SimulationMetadata { backend: Simulation, model, assumptions, fidelity }`.