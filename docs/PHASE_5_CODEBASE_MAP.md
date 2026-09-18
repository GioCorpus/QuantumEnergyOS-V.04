# QEOS V.04 — Phase 5 Codebase Map & Subsystem Topology

**Document Version:** 5.0  
**Phase:** 5 (Quantum Execution Platform, User Space, Security & System Integration)  
**Date:** 2026-09-17  
**Status:** VERIFIED BASELINE  

---

## 1. Executive Summary

This document provides the definitive topological map of all crates, kernel modules, driver abstractions, user-space service frameworks, quantum simulation engines, and security subsystems across the QuantumEnergyOS V.04 workspace.

### Criticality Tiers:
- **CRITICAL**: Kernel core (`kernel`), memory management, scheduler, capability security, DMA/IOMMU, and atomic synchronization.
- **HIGH**: Service framework (`system-core`), Device Manager (`device-manager`), Quantum HAL (`quantum-hal`), Telemetry pipeline (`energy-telemetry`), and Identity Service (`identity-service`).
- **MEDIUM**: Quantum Runtime (`quantum-runtime`), Majorana Simulator, and GPU Compute backend (`qeos-gpu-compute`).
- **LOW / USER**: Developer CLI tools (`qeos-qpu`, `qeos-cli`).
- **EXPERIMENTAL**: 5D Quartz storage model (`quartz5d`).

---

## 2. Workspace Crates Map

| Crate Path | Criticality | Purpose | Direct Dependencies | Consumers | Unsafe Blocks | Test Coverage |
|---|---|---|---|---|---|---|
| **`kernel`** | **CRITICAL** | Host-testable OS kernel (Memory, Scheduler, IPC, Syscalls, VFS, Drivers, Capabilities, Telemetry) | `spin` (embedded) | OS Runtime / Host Harness | 4 blocks in `sync/spin.rs` | 48 tests |
| **`crates/system-core`** | **HIGH** | Microkernel service orchestration, service bus, gateway rate limiting, RBAC | `serde`, `tokio`, `thiserror`, `uuid`, `chrono`, `tracing` | OS daemons, apps | 0 (`#![forbid(unsafe_code)]`) | 70 tests |
| **`crates/quantum-runtime`** | **MEDIUM** | Quantum circuit compilation, topological Majorana simulator, decoders, noise | `quantum-hal`, `serde`, `rand`, `thiserror` | `qeos-qpu`, applications | 0 (`#![forbid(unsafe_code)]`) | 133 tests |
| **`crates/quantum-hal`** | **HIGH** | Quantum hardware abstraction: device traits, IR representation, accelerator dispatch | `thiserror`, `serde` | `quantum-runtime`, `qeos-qpu` | 0 (`#![forbid(unsafe_code)]`) | 24 tests |
| **`crates/identity-service`** | **HIGH** | Authentication, JWT sessions, Argon2id password hashing, RBAC, JWKS | `argon2`, `jsonwebtoken`, `sha2`, `tokio`, `serde` | `system-core`, dashboard | 0 (`#![forbid(unsafe_code)]`) | 22 tests |
| **`crates/energy-telemetry`** | **HIGH** | SPSC lock-free telemetry ring buffer, energy counters, provenance tagging | `serde`, `tokio`, `thiserror` | `system-core`, dashboard | 3 blocks in `ring_buffer.rs` | 19 tests |
| **`crates/hardware-abstraction`** | **HIGH** | Hardware telemetry schemas, device classification, facts structs | `serde`, `chrono` | `device-manager`, `energy-telemetry` | 0 | Schema tests |
| **`crates/device-manager`** | **HIGH** | PCI configuration decode, BAR allocation, capability masks, driver lifecycle | `hardware-abstraction`, `serde`, `thiserror`, `tracing` | CLI, OS services | 0 (`#![forbid(unsafe_code)]`) | 14 tests |
| **`crates/qeos-gpu-compute`** | **MEDIUM** | GPU compute abstraction, CPU fallback reference, compute queues | `thiserror`, `tracing` | `quantum-runtime` | 0 | 8 tests |
| **`crates/quartz5d`** | **EXPERIMENTAL** | 5D spatial-temporal coordinate model and projection engine | `serde`, `thiserror` | Storage engine | 0 | 48 tests |
| **`crates/qeos-qpu`** | **LOW** | CLI frontend for QPU jobs, benchmarking, and Majorana experiments | `quantum-runtime`, `quantum-hal`, `serde_json` | End user / admin | 0 | CLI Harness |

---

## 3. Kernel Subsystems Directory (`kernel/src/`)

```text
kernel/src/
├── arch/                CPU state flags, GDT, IDT, paging scaffolds (x86_64, aarch64, riscv64).
├── boot/                Deterministic 11-stage boot sequence.
├── core/                Kernel health status and runtime constants.
├── device/              In-kernel device descriptors.
├── dma/                 DmaRegion, DmaMapping, DmaRing (fixed descriptor buffer).
├── driver/              Device registry, bus kinds, top-half IRQ handlers, IOMMU domain isolation,
│                        9-state lifecycle state machine, bounded MMIO windows, PCI configuration.
├── elf/                 ELF header parsing and validation.
├── fs/                  Virtual File System (VFS), Inodes, FileHandle, OpenFlags.
├── hal/                 CpuHal, TimerHal, PcieHal, NullHal.
├── ipc/                 Channel, Message envelope, bounded queue transfer.
├── logging/             Structured Logger and log level filtering.
├── memory/              Physical page allocator, Virtual memory manager, Heap bump allocator, OOM handler.
├── panic.rs             Host-testable kernel panic handler.
├── process/             Process, Thread, Pid/Tid context management.
├── qpu/                 Kernel-level QPU device hook (UnsupportedDevice by default).
├── ring/                SpscRing (atomic head/tail buffer with overflow policies).
├── scheduler/           Priority runqueues, per-CPU scheduler, scheduling classes.
├── security/            CapSet, Capability enumeration, access permission validator.
├── sync/                SpinLock (UnsafeCell + AtomicBool), Mutex, Atomics.
├── syscall/             Syscall dispatcher, Syscall numbers, pointer/range validation.
├── telemetry/           In-kernel sample capture and energy telemetry counters.
├── time/                MonotonicClock, KernelTimer, HostTimer.
└── tracing/             TraceCtx context propagation (CPU, thread, PID, timestamp).
```

---

## 4. Unsafe Code Inventory & Formal Safety Invariants

| Module | Location | Primitive | Safety Invariant |
|---|---|---|---|
| `kernel::sync::spin` | `kernel/src/sync/spin.rs:53,59,65` | `UnsafeCell`, raw dereference | Guarded by `AtomicBool` test-and-set with Acquire/Release synchronization. Non-sleepable. |
| `energy_telemetry::ring_buffer` | `crates/energy-telemetry/src/ring_buffer.rs:55,86,111` | `UnsafeCell`, raw slot access | Lock-free SPSC circular buffer guarded by atomic write/read index Acquire/Release barriers. |

**Zero unsafe code** is used in any user-space crate (`device-manager`, `quantum-hal`, `quantum-runtime`, `system-core`, `identity-service`, `quartz5d`).
