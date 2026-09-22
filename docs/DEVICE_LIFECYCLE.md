# QEOS V.04 — Device Lifecycle

**Status:** STABLE (as of Phase 7.3)

This document describes the QEOS device lifecycle architecture: the validated
state machine, safe device handles, the hardware-abstraction layer (HAL), and
the GPU device runtime.

---

## 1. Lifecycle State Machine

Device lifecycle states (validated transitions in
`crates/qeos-node/src/device_lifecycle.rs`):

```text
DISCOVERED
   │  initialize (once)
   ▼
INITIALIZING ──fail──▶ FAILED ──recover──▶ INITIALIZING / RESETTING / REMOVING
   │  ready
   ▼
READY ──▶ ACTIVE ──▶ QUIESCING ──▶ STOPPED ──▶ READY (restart)
   │         │
   │         └──▶ DEGRADED ──▶ READY / ACTIVE / FAILED / RESETTING
   │         └──▶ FAILED
   └──▶ QUESCING / DEGRADED / FAILED / RESETTING
   ...
FAILED ──▶ INITIALIZING | RESETTING | REMOVING   (recovery only)
REMOVING ──▶ REMOVED                              (terminal; deterministic cleanup)
REMOVED ──▶ (none)                                (no use-after-remove)
```

Guarantees:
- `REMOVED` is terminal — no operation may target it.
- No double initialization.
- No operation on `FAILED` without a recovery path.
- Deterministic cleanup `REMOVING -> REMOVED`.

## 2. Safe Device Handles

`crates/qeos-node/src/handles.rs`:

- Handles carry `{ handle_id, device_id, generation, capabilities }`.
- Capabilities are deny-by-default; a handle only grants what it was given.
- Generation is bumped on device reset/removal, invalidating all outstanding
  handles → stale-handle detection at the boundary.

## 3. HAL (Hardware Abstraction Layer)

`crates/hardware-abstraction`: CPU, Memory, PCI, Storage (NVMe), Network, GPU,
QPU, Telemetry, Power data models + `HardwareDevice` trait. Vendor logic stays
behind adapters; this crate is schema/interface only.

## 4. GPU Device Runtime

`crates/qeos-gpu-compute`:

- `GpuRuntime`: discover, capabilities, allocate/free/read/write/map, submit,
  synchronize, reset, telemetry.
- Memory safety: generation-bound `GpuMemoryHandle`, bounded allocation
  (deterministic OOM), single-mapping, device-removal semantics.
- CPU reference (`CpuReferenceCompute`) is the correctness oracle;
  `verify_against_reference` rejects divergence.

### GPU reality classification

| Layer | Status |
|---|---|
| GPU abstraction / runtime | REAL |
| CPU reference backend | REAL |
| Mock backend | SIMULATED |
| Vendor driver/hardware (Vulkan/CUDA/ROCm) | UNAVAILABLE |

A working abstraction does **not** imply hardware acceleration.
