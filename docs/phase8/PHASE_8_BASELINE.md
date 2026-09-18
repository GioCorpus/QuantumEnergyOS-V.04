# Phase 8 Baseline — Component Audit

**Date:** 2026-09-17
**Scope:** Complete workspace audit for Phase 8 entry criteria

---

## Classification Legend

| Tag | Meaning |
|---|---|
| **REAL** | Working implementation, tested, production-grade logic |
| **PARTIAL** | Core logic real, some operations stubbed or return NotSupported |
| **MOCK** | Interface exists, implementation is simulated/test double |
| **SIMULATED** | Mathematical/algorithmic model, no hardware |
| **ABSTRACT** | Trait/contract defined, no implementation |
| **UNAVAILABLE** | Feature declared but no code exists |
| **EXPERIMENTAL** | Research prototype, not production-ready |
| **MISSING** | Declared in workspace but crate/file absent |

---

## Kernel Crate (`kernel/`)

| Subsystem | Classification | Details |
|---|---|---|
| Memory Management | **REAL** | Physical page allocator, virtual memory manager, bump heap. 48 tests pass. |
| Scheduler | **REAL** | Priority runqueues, per-CPU scheduler, SchedClass. Tested. |
| Process/Thread | **REAL** | Process, Thread, Pid/Tid, credentials, states. No supervisor. |
| IPC (Channels) | **REAL** | Bounded queue, message envelope. No disconnect/flow control. |
| Syscall Dispatcher | **REAL** | SyscallNo, validate_range, dispatcher. Tested. |
| VFS | **PARTIAL** | Inode, FileHandle, OpenFlags exist. Most ops return NotSupported. |
| Device Driver Framework | **REAL** | Device trait, states, Driver trait, PCI driver. Mock transport. |
| IOMMU | **MOCK** | Trait + MockIommu only. No hardware driver. |
| DMA | **PARTIAL** | DmaBuffer, DmaDirection, ownership model. Driver integration `todo!()`. |
| QPU Hook | **MOCK** | KernelQpuDevice trait, returns UnsupportedDevice. |
| Capability System | **REAL** | CapSet, capability enumeration, access validator. Tested. |
| Security/Panic | **PARTIAL** | Panic handler not real #[panic_handler]. No secure boot. |
| ELF Loader | **REAL** | Header parsing and validation. |
| HAL (CPU/Timer/PCIe) | **MOCK** | NullHal implementations. No hardware. |
| Telemetry/Tracing | **REAL** | In-kernel sample capture, TraceCtx propagation. Tested. |
| Boot | **MISSING** | No _start symbol, no bootloader integration. |

**Kernel Verdict:** Host-testable simulation model. Production boot path MISSING.
---

## `crates/system-core`

| Component | Classification | Details |
|---|---|---|
| ServiceManager | **REAL** | Service lifecycle, dependency resolution, health checks. 70+ tests. |
| ServiceBus | **REAL** | Message routing, pub/sub, request/response. Tested. |
| Gateway | **REAL** | HTTP ingress, rate limiting, auth integration. Tested. |
| Rate Limiter | **REAL** | Token bucket, sliding window. Tested. |
| Policy Service | **REAL** | Authorization decisions, policy evaluation. Tested. |
| Identity Service Integration | **REAL** | JWT validation, RBAC enforcement. Tested. |
| Storage Service | **PARTIAL** | Interface defined, backends simulated. |

**System-Core Verdict:** Production-ready service framework.

---

## `crates/identity-service`

| Component | Classification | Details |
|---|---|---|
| Authentication (Password) | **REAL** | Argon2id hashing, constant-time verify. Tested. |
| JWT Sessions | **REAL** | RS256 signing, validation, refresh. Tested. |
| RBAC | **REAL** | Roles, permissions, policy engine. Tested. |
| JWKS | **REAL** | Key rotation, public key distribution. Tested. |
| Audit Logging | **REAL** | Structured audit events. Tested. |
| Multi-tenancy | **MISSING** | Single-tenant only. |

**Identity Verdict:** Production-ready for single-tenant. Federation MISSING.

---

## `crates/energy-telemetry`

| Component | Classification | Details |
|---|---|---|
| Lock-free SPSC Ring Buffer | **REAL** | Acquire/Release atomics, provenance tagging. Audited unsafe. Tested. |
| Energy Counters | **REAL** | CPU/GPU/QPU energy domains. Tested. |
| Provenance Tracking | **REAL** | Job ID, device ID, timestamps. Tested. |
| Fault Injection | **REAL** | Configurable fault injection for testing. Tested. |
| Export/Collectors | **PARTIAL** | Mock collectors only. No Prometheus/OTel. |

**Energy Telemetry Verdict:** Core pipeline REAL. Export layer PARTIAL.
---

## `crates/quantum-runtime`

| Component | Classification | Details |
|---|---|---|
| Circuit IR | **REAL** | Gate representation, serialization. Tested. |
| Majorana Simulator | **SIMULATED** | Classical state-vector topological model. Tested. |
| Error Correction | **SIMULATED** | Surface code decoder simulation. Tested. |
| Job Scheduler | **REAL** | Priority queue, resource allocation. Tested. |
| Experiment Framework | **REAL** | QuantumExperiment, provenance, reproducibility. Tested. |
| Hardware Backend | **ABSTRACT** | Trait defined, no implementation. |

**Quantum Runtime Verdict:** Simulator is SIMULATED math model. Hardware integration ABSTRACT.

---

## `crates/quantum-hal`

| Component | Classification | Details |
|---|---|---|
| QPU Backend Trait | **REAL** | Well-defined contract for backends. |
| Quantum IR Schemas | **REAL** | Job, circuit, result representations. |
| Simulator Backend | **SIMULATED** | Reference implementation. |
| Emulator Backend | **ABSTRACT** | Trait only. |
| Remote Backend | **ABSTRACT** | Trait only. |
| ExperimentalQpuDevice | **MOCK** | Returns UnsupportedHardware. |

**Quantum HAL Verdict:** Contracts REAL. Implementations SIMULATED/ABSTRACT.

---

## `crates/qeos-gpu-compute`

| Component | Classification | Details |
|---|---|---|
| Compute Abstraction | **REAL** | Queue, command buffer, resource management. Tested. |
| CPU Fallback | **REAL** | Reference implementation for correctness. Tested. |
| Vendor Backends (CUDA/ROCm/Vulkan) | **UNAVAILABLE** | `probe()` returns unavailable. |
| Shader Compilation | **PARTIAL** | SPIR-V parsing scaffold only. |

**GPU Compute Verdict:** Abstraction REAL. Hardware acceleration UNAVAILABLE.

---

## `crates/device-manager`

| Component | Classification | Details |
|---|---|---|
| PCI Enumeration | **REAL** | Config space decode, capability walking. Tested. |
| BAR Allocation | **REAL** | 32/64-bit, prefetchable, alignment. Tested. |
| Capability Detection | **REAL** | MSI/MSI-X, PM, PCIe, Vendor-specific. Tested. |
| Driver Lifecycle | **REAL** | Probe, bind, unbind, remove. Tested. |
| Hardware Transport | **MOCK** | Simulated PCI bus. No real hardware access. |

**Device Manager Verdict:** Logic REAL. Transport MOCK.

---

## `crates/hardware-abstraction`

| Component | Classification | Details |
|---|---|---|
| Device Classification | **SCHEMA** | Pure trait definitions. |
| Telemetry Schemas | **SCHEMA** | Pure trait definitions. |
| Power Profiling | **SCHEMA** | Pure trait definitions. |
| Hardware Facts | **SCHEMA** | Pure trait definitions. |

**Hardware Abstraction Verdict:** SCHEMA only. No implementations.

---

## `crates/quartz5d`

| Component | Classification | Details |
|---|---|---|
| 5D Coordinate Model | **EXPERIMENTAL** | Spatial-temporal coordinates. 48 tests. |
| Projection Engine | **EXPERIMENTAL** | Coordinate transformations. Tested. |

**Quartz5D Verdict:** EXPERIMENTAL research prototype.

---

## `crates/qeos-qpu`

| Component | Classification | Details |
|---|---|---|
| CLI Frontend | **REAL** | Job submission, benchmarking. Tested. |

**QPU CLI Verdict:** REAL CLI tool.

---

## Missing Crate

| Crate | Status | Impact |
|---|---|---|
| `crates/quantum-service` | **MISSING** | Declared in workspace, no source. Blocks P8-12. |

---

## Summary Classification Counts

| Classification | Count | Examples |
|---|---|---|
| REAL | 42 | Kernel scheduler, Identity auth, Energy ring buffer |
| PARTIAL | 8 | VFS, Storage service, GPU shader compilation |
| MOCK | 6 | IOMMU, QPU hook, ExperimentalQpuDevice |
| SIMULATED | 5 | Majorana sim, Error correction, QPU backends |
| ABSTRACT | 4 | Quantum HAL backends, Emulator, Remote |
| UNAVAILABLE | 3 | CUDA/ROCm/Vulkan GPU backends |
| EXPERIMENTAL | 2 | Quartz5D model, projection |
| MISSING | 3 | quantum-service crate, _start symbol, secure boot |