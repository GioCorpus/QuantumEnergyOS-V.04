# Quantum Architecture

## Scope

The quantum layer is a runtime and research environment for classical orchestration of quantum simulation, emulation, and future QPU adapters. It does not assume direct access to a physical quantum processor.

## Core concepts

- Qubit and logical qubit abstractions
- measurement values such as bit values, distribution, parity, syndrome, logical state
- circuit representation and compiler pipeline
- backend execution model with simulation and emulation first
- scheduler for queueing, priority, deadlines, cancellation, availability

## Backend model

```rust
pub enum QuantumBackendType {
    Simulation,
    Emulation,
    Remote,
    Physical,
}

pub struct SimulationMetadata {
    pub backend: QuantumBackendType,
    pub model: String,
    pub assumptions: Vec<String>,
    pub fidelity: Option<f64>,
}
```

## Topological guidance

The system models concepts compatible with Majorana and topological qubit design without claiming real device physics. Examples include parity, tetron-like logical structures, braiding adjacency, and error correction state tracking. Any simulation must make clear that it is a model and not direct hardware behavior.

## Supported backends

- `SimulatorBackend`
- `LocalEmulatorBackend`
- `AzureQuantumBackend`
- `MajoranaBackend` as a capability-gated adapter with a documented hardware interface requirement

## Execution pipeline

High-Level Circuit -> IR -> Backend Representation -> Scheduler -> Execution -> Measurement

## Hard boundary

Physical QPU access remains behind a documented adapter boundary and is never reverse-engineered or assumed from mathematical simulation output.
