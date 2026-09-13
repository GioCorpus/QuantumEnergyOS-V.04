# QuantumEnergyOS V.04 Architecture Baseline

**Classification labels**
- `REAL`: implemented and runnable on conventional hardware today.
- `SIMULATED`: classical software models with explicit simulation semantics.
- `ABSTRACT`: documented interfaces without physical implementation.
- `FUTURE`: planned capability, no working hardware assumed.
- `EXPERIMENTAL`: research interfaces not ready for production.

---

## Target tree

```text
system/
└── quantum/
    ├── qhal/                # ABSTRACT: future quantum hardware abstraction
    ├── runtime/             # REAL/SIMULATED: quantum runtime and job system
    ├── scheduler/           # REAL/SIMULATED: job scheduling
    ├── devices/             # ABSTRACT: device discovery capability
    ├── simulator/           # SIMULATED: classical simulator backend
    ├── telemetry/           # REAL: telemetry and ring buffers
    └── protocols/           # ABSTRACT/FUTURE: quantum protocols
```

---

## Workspace crates (REAL)

- `crates/system-core` — service framework, IPC registry, gateway, orchestration.
- `crates/quantum-runtime` — quantum simulator, circuit model, jobs, backends.
- `crates/energy-telemetry` — energy/telemetry ring buffers.
- `crates/quartz5d` — Quartz5D data model and storage simulation.
- `crates/identity-service` — authentication, JWT, RBAC.
- `crates/hardware-abstraction` — CPU/GPU/NPU and quantum backend abstraction interfaces.

---

## Design principles

1. Simulation is always labeled `SIMULATED`.
2. No Majorana 2 or topological hardware is claimed to exist.
3. Future quantum backends are `FUTURE` interfaces only.
4. Security model: Identity → Policy → Quantum Service → HAL.
5. Build validation is reproducible with Rust and Arch Linux tools.
6. No passwords in plaintext; no secrets in Git.
