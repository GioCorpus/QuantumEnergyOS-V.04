# Quantum Core (V.04 §5)

Canonical implementation: `crates/quantum-runtime/src/{circuit,gates,job,measurement}.rs`.

Represents: qubits, logical qubits, measurements, operations, registers, circuits.

- `QuantumCircuit { name, num_qubits, num_classical_bits, gates, metadata }`
- `QuantumGate` (Hadamard, Pauli X/Y/Z, Phase, RX/RY/RZ, CNOT, ControlledZ, Swap, Toffoli, Measurement)
- `QuantumJob`, `JobPriority`, `JobStatus`, `JobQueue`
- `Measurement`, `MeasurementValue` (Bit0/Bit1, ProbabilityDistribution, Parity, Syndrome, LogicalState)

No hardware claimed. All execution goes through compiler → scheduler → backend.