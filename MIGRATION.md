# V.03 → V.04 Migration Classification

This document classifies every component from the V.03 codebase into migration categories for the V.04 rebuild.

## Classification Categories

- **KEEP** — Solid foundation, minimal changes needed
- **REFACTOR** — Good structure but needs architectural improvements
- **REWRITE** — Fundamental design issues, needs reimplementation
- **DEPRECATE** — Redundant or superseded by better design
- **REMOVE** — Dead code, unnecessary complexity, or violates V.04 principles

---

## system-core Crate

### service.rs — KEEP
- `QuantumService` trait is well-designed
- `ServiceStatus` and `HealthStatus` enums are correct
- Minor: Add `Send + Sync` bounds documentation

### manager.rs — KEEP
- `ServiceManager` is solid with good lifecycle management
- Health reporting and status aggregation work correctly
- Minor: Consider parallel startup (currently sequential)

### service_bus.rs — REFACTOR
- `Message` envelope is correct and versioned
- `ServiceRegistry` routing works
- `MessageBuffer` is a good testing utility
- Refactor: Add authentication/authorization hooks to message handling
- Refactor: Add rate limiting hooks

### services.rs — REFACTOR
- All 9 service stubs exist but are empty shells
- Refactor: Each service needs real implementation in its own crate
- Refactor: Move service implementations to dedicated crates (identity, energy, etc.)
- Keep: The stub pattern for registration/testing

### error.rs — KEEP
- Comprehensive error taxonomy
- Good `From` implementations for error conversion
- Covers service, IPC, quantum, database, auth domains

### lib.rs — REFACTOR
- Update exports as services move to dedicated crates

---

## quantum-runtime Crate

### gates.rs — KEEP
- `QuantumGate` enum is complete and correct
- `Complex` number implementation is solid
- Matrix representations are physically accurate
- Good serialization support

### circuit.rs — REFACTOR
- `QuantumCircuit` is well-designed with metadata
- Validation logic is thorough
- Refactor: Unify with `quantum_ir::Circuit` (duplicate concept)
- Refactor: Gate validation for single-qubit gates doesn't check qubit indices

### simulator.rs — REFACTOR
- State vector simulation is physically correct
- Gate applications are mathematically sound
- **CRITICAL**: Measurement uses `(value).sin().abs()` as randomness — violates V.04 principle against arbitrary mathematical values representing quantum behavior
- Refactor: Use proper RNG (rand crate) for measurement outcomes
- Refactor: Document simulation limitations clearly

### backend.rs — REWRITE
- `QuantumProcessor` trait is correct architecture
- `SimulatorBackend` works but is tightly coupled
- **CRITICAL**: Contains dead code after tests module (lines 342-418) — will not compile
- `MajoranaBackend` stub is correct approach (disabled by default)
- Rewrite: Remove dead code, clean up trait implementations
- Rewrite: Add `LocalEmulatorBackend` and `AzureQuantumBackend` stubs

### topology.rs — REWRITE
- Current implementation is just data structs, no behavior
- Missing: parity operations, braiding logic, lattice model, error model
- Rewrite: Full topological subsystem per master prompt

### measurement.rs — KEEP
- `MeasurementValue` enum covers all required types
- Clean and extensible

### error.rs — KEEP
- Comprehensive error types
- Good `From` implementations

### quantum_ir.rs — DEPRECATE
- `Circuit` here duplicates `circuit::Circuit`
- IR concept is good but implementation overlaps
- Deprecate: Merge into circuit.rs as the canonical circuit representation
- Keep: The `QuantumOperation` enum as a lower-level representation

### quantum_device.rs — REFACTOR
- `QuantumDevice` trait is well-designed
- `DeviceCapabilities`, `DeviceHealth`, `DeviceInfo` are comprehensive
- Refactor: This should be the primary hardware abstraction trait
- Refactor: Unify with `QuantumProcessor` from backend.rs

### job.rs — KEEP
- `QuantumJob` is comprehensive with full lifecycle
- `JobQueue` with priority scheduling works correctly
- Good validation and timeout handling

### hal_simulator.rs — REFACTOR
- Good reference implementation of `QuantumDevice`
- **ISSUE**: `simulate_circuit` generates synthetic results using arithmetic, not actual simulation
- Refactor: Connect to real `QuantumSimulator` for state-vector simulation
- Refactor: Document clearly as SIMULATION

### lib.rs — REFACTOR
- Update exports after module reorganization

---

## energy-telemetry Crate

### ring_buffer.rs — REWRITE
- Current implementation uses `VecDeque` (not lock-free)
- Missing: SPSC design, memory ordering, dropped-frame counters, overflow detection
- Rewrite: Implement lock-free SPSC ring buffer with `AtomicUsize` indices
- Rewrite: AddAcquire/Release memory ordering
- Rewrite: Add timestamps and backpressure

### lib.rs — REWRITE
- Only exports ring_buffer
- Rewrite: Add telemetry service, sensor abstractions, forecasting stubs

---

## Architecture Documents

### ARCHITECTURE.md — KEEP
- Solid foundation, minor updates for V.04

### ROADMAP.md — KEEP
- Phase structure is correct

### THREAT_MODEL.md — KEEP
- Comprehensive threat analysis

### QUANTUM_ARCHITECTURE.md — KEEP
- Good backend model and simulation guidance

### HARDWARE_ABSTRACTION.md — KEEP
- Clear abstraction principles

### docs/IPC_PROTOCOL.md — KEEP
- Thorough protocol specification

### docs/SERVICE_FRAMEWORK.md — KEEP
- Good service lifecycle documentation

---

## CI/CD

### .github/workflows/ci.yml — REFACTOR
- Basic structure is correct
- Refactor: Add security audit step (cargo audit or similar)
- Refactor: Add cargo fmt check as separate job
- Refactor: Add build matrix for both Linux targets

---

## New Components Required (Not in V.03)

1. **quartz5d** crate — Computational model (X,Y,Z,T,S)
2. **identity-service** crate — Auth, sessions, RBAC, JWKS
3. **hardware-abstraction** crate — CPU, GPU, PCI, NVMe, SPI/I2C, power, telemetry
4. **quantum/topology** — Full Majorana subsystem
5. **energy/forecasting** — Load prediction, anomaly detection
6. **browser-manager** — Profile management, certificates
7. **dashboard-service** — Unified observability views

---

## Critical Issues to Fix

1. **backend.rs dead code** — Compilation failure, must remove
2. **simulator.rs fake randomness** — Violates V.04 principles
3. **Duplicate circuit types** — circuit.rs vs quantum_ir.rs
4. **hal_simulator.rs synthetic results** — Not real simulation
5. **ring_buffer.rs not lock-free** — Violates architecture spec
6. **Empty service stubs** — Need real implementations
