# Quantum Circuit (V.04 §5)

Canonical implementation: `crates/quantum-runtime/src/circuit.rs`.

High-level circuit model validated on construction (1..=20 qubits).
Execution never direct: circuit -> compiler IR -> backend artifact -> scheduler -> backend.