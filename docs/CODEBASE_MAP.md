# QEOS V.04 — Codebase Map & Component Topology
**Date:** 2026-09-16
**Auditor:** Senior Rust / Systems Architecture & Hardening Engine

---

## 1. Executive Overview

This document provides a comprehensive map of all crates, kernel modules, device interfaces, services, and subsystems across the QuantumEnergyOS V.04 repository. Each module is evaluated by purpose, dependency graph, criticality, unsafe surface, test coverage, and known debt.

### Criticality Tiers:
- **CRITICAL**: Kernel core, memory management, scheduler, security boundaries, DMA/IOMMU, and atomic synchronization.
- **HIGH**: Device manager, HAL interfaces, IPC protocol, telemetry pipeline, and identity services.
- **MEDIUM**: Quantum runtime simulation, error correction algorithms, and GPU compute backend.
- **LOW**: User CLI tools (`qeos-qpu`, `qeos-devices`) and documentation fixtures.
- **EXPERIMENTAL**: 5D quartz storage projection model and Majorana braiding simulator.
- **DEPRECATED**: None currently marked deprecated.

---

## 2. Workspace Crates Map

| Crate / Module | Criticality | Purpose | Direct Dependencies | Consumers | Unsafe Surface | Test Coverage | Known Issues & Debt |
|---|---|---|---|---|---|---|---|
| **`kernel`** | **CRITICAL** | Host-testable kernel core: scheduler, memory, IPC, DMA, HAL, VFS, security | `spin` | Root / OS runtime | 4 blocks in `sync/spin.rs` (audited SpinLock) | High (47 unit + integration tests) | `std` dependency in host mode; `PciAddr` vs `PciAddress` mismatch; MMIO needs typed registers. |
| **`crates/device-manager`** | **HIGH** | PCI configuration decode, BAR allocation, capability masks, driver lifecycle | `hardware-abstraction`, `serde`, `thiserror`, `tracing` | CLI, OS services | `#![forbid(unsafe_code)]` (0 unsafe) | High (14 tests) | 6-state lifecycle vs kernel 9-state lifecycle; simulated transport only. |
| **`crates/hardware-abstraction`** | **HIGH** | Hardware telemetry schemas, device classification, facts structs | `serde`, `chrono` | `device-manager`, `energy-telemetry` | 0 unsafe | Schema only | Pure data models; needs harmonization with kernel HAL traits. |
| **`crates/quantum-hal`** | **HIGH** | Quantum hardware abstraction: device traits, IR representation, accelerator dispatch | `thiserror`, `serde` | `quantum-runtime`, `qeos-qpu` | 0 unsafe | Medium (24 tests) | Linter warnings (unused imports/fields in `accelerator.rs`, `clippy::approx_constant`). |
| **`crates/quantum-runtime`** | **MEDIUM** | Quantum circuit compilation, topological Majorana simulator, decoders, noise | `quantum-hal`, `serde`, `rand` | `qeos-qpu`, applications | 0 unsafe | Very High (133 tests) | Clippy warnings in scheduler/simulator; braiding simulation needs seeded API hardening. |
| **`crates/energy-telemetry`** | **HIGH** | SPSC lock-free telemetry ring buffer, energy counters, provenance tagging | `serde`, `tokio`, `thiserror` | `system-core`, dashboard | 3 blocks in `ring_buffer.rs` (audited SPSC) | High (19 tests) | Dead code warning on `TelemetryService.config`; SPSC ring needs comprehensive concurrency tests. |
| **`crates/qeos-gpu-compute`** | **MEDIUM** | GPU compute abstraction, CPU fallback reference, compute queues | `thiserror`, `tracing` | `quantum-runtime` | 0 unsafe | Basic (8 tests) | Initial skeleton; needs formal memory allocation and queue fences. |
| **`crates/system-core`** | **HIGH** | Microkernel service orchestration, service bus, gateway rate limiting, RBAC | `serde`, `tokio`, `thiserror` | OS daemons, apps | 0 unsafe | High (70 tests) | Unused field in test mock; needs formal connection with kernel IPC. |
| **`crates/identity-service`** | **HIGH** | Authentication, JWT sessions, Argon2 password hashing, RBAC | `argon2`, `jsonwebtoken`, `sha2`, `tokio` | `system-core`, dashboard | 0 unsafe | High (22 tests) | Standard auth crate; functional and well-isolated. |
| **`crates/quartz5d`** | **EXPERIMENTAL** | 5D spatial-temporal coordinate model and projection engine | `serde`, `thiserror` | Storage engine | 0 unsafe | High (48 tests) | Experimental storage model. |
| **`crates/qeos-qpu`** | **LOW** | CLI frontend for QPU jobs, benchmarking, and Majorana experiments | `quantum-runtime`, `quantum-hal`, `serde_json` | End user / admin | 0 unsafe | Manual / CLI harness | Uses `println!`/`eprintln!` for CLI output. |

---

## 3. Kernel Subsystems Breakdown (`kernel/src/`)

```text
kernel/src/
├── arch/
│   ├── x86_64/          [CRITICAL] CPU flag state (AtomicBool), GDT, IDT, paging scaffolds, context.
│   └── mod.rs           [CRITICAL] Architecture abstraction selector.
├── boot/                [HIGH]     Deterministic boot phase sequencer.
├── core/                [HIGH]     Kernel health state, config parameters.
├── device/              [HIGH]     In-kernel device descriptors.
├── dma/                 [CRITICAL] DmaRegion, DmaMapping, DmaRing (fixed descriptor buffer).
├── driver/
│   ├── bus.rs           [HIGH]     BusKind enumeration (Pcie, I2c, Spi, Nvme, Qpu).
│   ├── device.rs        [HIGH]     Device, DriverRegistry, Driver trait.
│   ├── interrupt.rs     [CRITICAL] Irq struct, IrqHandler trait (top-half acknowledge).
│   ├── iommu.rs         [CRITICAL] Iommu trait, DomainId, IommuPerm, MockIommu.
│   ├── lifecycle.rs     [HIGH]     DeviceLifecycle (9-state machine with validation).
│   ├── mmio.rs          [CRITICAL] MmioWindow (32-bit bounded slice access).
│   └── pci.rs           [HIGH]     PciAddr, PciIdentity, BarDescriptor, StubConfigSource.
├── elf/                 [MEDIUM]   ELF header parsing and validation.
├── fs/                  [MEDIUM]   Virtual File System (VFS), InodeKind, file handles.
├── hal/                 [HIGH]     CpuHal, TimerHal, PcieHal, NullHal.
├── ipc/                 [CRITICAL] Channel, Message envelope, bounded ring transfer.
├── logging/             [MEDIUM]   Structured Logger and log Level filters.
├── memory/              [CRITICAL] Physical page allocator, Virtual memory manager, Heap bump allocator, OOM handler.
├── panic.rs             [CRITICAL] Kernel panic handler entry point.
├── process/             [CRITICAL] Process and Thread lifecycle, context, TID/PID management.
├── qpu/                 [MEDIUM]   Kernel-level QPU device hook.
├── ring/                [CRITICAL] SpscRing (atomic head/tail buffer with overflow policies).
├── scheduler/           [CRITICAL] Priority runqueues, per-CPU scheduler, scheduling classes.
├── security/            [CRITICAL] Capability masks, Access permissions, capability checker.
├── sync/                [CRITICAL] SpinLock (UnsafeCell + AtomicBool), Mutex, Atomic helpers.
├── syscall/             [CRITICAL] Syscall dispatcher, Syscall numbers, pointer/range validation.
├── telemetry/           [HIGH]     Lightweight sample capture and energy telemetry counters.
├── time/                [HIGH]     MonotonicClock, KernelTimer, HostTimer.
└── tracing/             [HIGH]     TraceCtx context propagation (CPU, thread, PID, timestamp).
```

---

## 4. Unsafe Code Inventory & Boundaries

| Module | Location | Primitive | Purpose | Safety Invariant |
|---|---|---|---|---|
| `kernel::sync::spin` | `kernel/src/sync/spin.rs:53,59,65` | `UnsafeCell`, raw dereference | SpinLock interior mutability | Guarded by `AtomicBool` test-and-set. Non-sleepable, caller must ensure interrupt safety. |
| `energy_telemetry::ring_buffer` | `crates/energy-telemetry/src/ring_buffer.rs:54,84,109` | `UnsafeCell`, raw slot access | Lock-free SPSC buffer | Guarded by Acquire/Release atomic index synchronization. Single-producer, single-consumer. |

All user-space crates (`device-manager`, `quantum-hal`, `quantum-runtime`, `system-core`, `identity-service`, `quartz5d`) maintain **zero unsafe code** (`#![forbid(unsafe_code)]`).
