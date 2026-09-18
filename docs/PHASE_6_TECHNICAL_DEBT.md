# QEOS V.04 — Phase 6 Technical Debt Catalogue

**Date:** 2026-09-17
**Source:** P6-00 Repository & Phase 5 Audit
**Status:** BASELINE ESTABLISHED

---

## 1. Purpose

This document catalogues all known technical debt, stubs, TODOs, experimental code, and architectural gaps identified during the P6-00 audit. Each item is traced to its source, severity, and recommended Phase 6 milestone for remediation.

---

## 2. Debt Classification

| Severity | Definition |
|---|---|
| **CRITICAL** | Blocks production readiness; security or memory safety risk; data loss potential |
| **HIGH** | Blocks key Phase 6 milestones; major functionality incomplete; significant maintenance burden |
| **MEDIUM** | Degrades reliability, observability, or operability; should be addressed in Phase 6 |
| **LOW** | Cosmetic, documentation, or minor duplication; can be deferred |

---

## 3. Kernel-Level Debt

| ID | Location | Description | Severity | Source | Recommended Milestone |
|---|---|---|---|---|---|
| K-01 | `kernel/src/sync/mutex.rs` | `KernelMutex` wraps `std::sync::Mutex` — sleepable, not IRQ-safe, prevents `no_std` | HIGH | KERNEL_AUDIT.md §6 | P6-03 (Concurrency) |
| K-02 | `kernel/src/memory/virtual_.rs` | Uses `usize` for physical addresses; no `PhysAddr`/`PhysPage` types; no canonical address validation | MEDIUM | KERNEL_AUDIT.md §8 | P6-04 (Memory) |
| K-03 | `kernel/src/memory/heap.rs` | `KernelHeap::alloc` rejects zero alignment but lacks `free` implementation | MEDIUM | KERNEL_AUDIT.md §9 | P6-04 (Memory) |
| K-04 | `kernel/src/panic.rs` | `panic_info` is not a real `#[panic_handler]`; placeholder only | HIGH | KERNEL_AUDIT.md §14 | P6-02 (Boot/Init) |
| K-05 | `kernel/boot/linker.ld` | `ENTRY(_start)` but no `_start` symbol in Rust code | HIGH | KERNEL_AUDIT.md §14 | P6-02 (Boot/Init) |
| K-06 | `kernel/src/core/config.rs` + `kernel/src/memory/mod.rs` | `PAGE_SIZE` duplicated in two locations | LOW | KERNEL_AUDIT.md §11 | P6-03 (Concurrency) |
| K-07 | `kernel/src/dma/` + `kernel/src/driver/dma.rs` | DMA module duplicated in two locations | LOW | KERNEL_AUDIT.md §11 | P6-03 (Concurrency) |
| K-08 | `kernel/src/device/` + `kernel/src/driver/device.rs` | `Device` re-exported via both `device/` and `driver/` | LOW | KERNEL_AUDIT.md §11 | P6-03 (Concurrency) |
| K-09 | `kernel/src/sync/spin.rs` | `SpinLock` documented as IRQ-unsafe unless caller disables interrupts; no compile-time enforcement | MEDIUM | KERNEL_AUDIT.md §10 | P6-03 (Concurrency) |
| K-10 | `kernel/src/sync/` | No formal lock ordering documentation beyond comment "Memory < Device < Process" | MEDIUM | KERNEL_AUDIT.md §10 | P6-03 (Concurrency) |
| K-11 | `kernel/src/driver/iommu.rs` | `MockIommu` is software-only; no hardware IOMMU driver | HIGH | KERNEL_AUDIT.md §12 | P6-05 (Drivers) |
| K-12 | `kernel/src/driver/dma.rs` | Driver DMA integration is a stub (`todo!()`) | HIGH | KERNEL_AUDIT.md §12 | P6-05 (Drivers) |
| K-13 | `kernel/src/qpu/` | `UnsupportedDevice` by default; no real QPU driver integration | MEDIUM | KERNEL_AUDIT.md §12 | P6-07 (Quantum) |
| K-14 | `kernel/src/fs/` | VFS inode ops return `Err(Error::NotSupported)` for most operations | HIGH | KERNEL_AUDIT.md §13 | P6-06 (Storage) |
| K-15 | `kernel/src/process/process.rs` | No process supervisor, restart policy, or health checks | HIGH | P6-00 finding | P6-08 (Services) |
| K-16 | `kernel/src/ipc/channel.rs` | No IPC disconnect handling, no flow control backpressure propagation | MEDIUM | P6-00 finding | P6-08 (Services) |
| K-17 | `kernel/src/security/capability.rs` | Capability system exists but no capability delegation or revocation | MEDIUM | P6-00 finding | P6-02 (Security) |
---

## 4. User-Space Crate Debt

| ID | Location | Description | Severity | Source | Recommended Milestone |
|---|---|---|---|---|---|
| U-01 | `Cargo.toml` (workspace) | `quantum-service` crate declared but not implemented (DEBT) | HIGH | Cargo.toml comment | P6-07 (Quantum) |
| U-02 | `crates/device-manager/src/pci/simulated.rs` | PCI simulated backend returns `unimplemented!()` for BAR probing | MEDIUM | Search: `unimplemented` | P6-05 (Drivers) |
| U-03 | `crates/device-manager/src/pci/bus.rs` | PCI bus enumeration uses `unimplemented!()` for slot scanning | MEDIUM | Search: `unimplemented` | P6-05 (Drivers) |
| U-04 | `crates/quantum-hal/src/simulator.rs` | Simulator backend is the only implementation; no hardware adapter | MEDIUM | P6-00 finding | P6-07 (Quantum) |
| U-05 | `crates/quantum-hal/src/qpu.rs` | QPU trait has no hardware implementations | MEDIUM | P6-00 finding | P6-07 (Quantum) |
| U-06 | `crates/qeos-gpu-compute/src/lib.rs` | GPU compute is CPU fallback only; no real GPU driver integration | MEDIUM | P6-00 finding | P6-07 (Quantum/GPU) |
| U-07 | `crates/quartz5d/` | Entire crate marked EXPERIMENTAL; no production use case defined | LOW | P6-00 finding | P6-10 (Experimental) |
| U-08 | `crates/identity-service/src/lib.rs` | Key storage is in-memory only; no HSM/TPM integration | HIGH | P6-00 finding | P6-02 (Security) |
| U-09 | `crates/energy-telemetry/src/ring_buffer.rs` | Uses `zeroed()` with `UnsafeCell` — audited but should use `MaybeUninit` | LOW | KERNEL_AUDIT.md §15 | P6-04 (Memory) |
| U-10 | `crates/system-core/src/manager.rs` | ServiceManager has no restart policy, health checks, or backoff | HIGH | P6-00 finding | P6-08 (Services) |
| U-11 | `crates/system-core/src/service_gateway.rs` | Rate limiting uses in-memory `HashMap`; no distributed coordination | MEDIUM | P6-00 finding | P6-08 (Services) |
| U-12 | `crates/system-core/src/service_bus.rs` | No message persistence, dead letter queue, or replay | MEDIUM | P6-00 finding | P6-08 (Services) |

---

## 5. Build & CI Debt

| ID | Location | Description | Severity | Source | Recommended Milestone |
|---|---|---|---|---|---|
| B-01 | Repository root | No `rust-toolchain.toml` for toolchain pinning | HIGH | P6-00 finding | P6-01 (Reliability) |
| B-02 | `.github/workflows/ci.yml` | No kernel `no_std` build target test (x86_64-unknown-none, etc.) | HIGH | P6-00 finding | P6-01 (Reliability) |
| B-03 | `.github/workflows/ci.yml` | No kernel integration tests in CI (only user-space tests run) | HIGH | P6-00 finding | P6-01 (Reliability) |
| B-04 | `.github/workflows/ci.yml` | No fuzzing (cargo-fuzz) for syscall boundary validation | MEDIUM | P6-00 finding | P6-02 (Security) |
| B-05 | `.github/workflows/ci.yml` | No property-based testing (proptest) for kernel data structures | MEDIUM | P6-00 finding | P6-01 (Reliability) |
| B-06 | `.github/workflows/ci.yml` | No `cargo-deny` for license/checks | LOW | P6-00 finding | P6-01 (Reliability) |
| B-07 | `.github/workflows/ci.yml` | No SBOM generation (cyclonedx) | LOW | P6-00 finding | P6-01 (Reliability) |
| B-08 | Repository root | No hermetic build environment (Nix/Bazel) | MEDIUM | P6-00 finding | P6-22 (Reproducible) |
---

## 6. Security Debt

| ID | Location | Description | Severity | Source | Recommended Milestone |
|---|---|---|---|---|---|
| S-01 | `kernel/src/boot/mod.rs` | No secure boot implementation; ELF loaded without verification | CRITICAL | P6-00 finding | P6-02 (Security) |
| S-02 | `kernel/src/security/mod.rs` | `// TODO: Capability-based access control` — incomplete | HIGH | Search: `TODO` | P6-02 (Security) |
| S-03 | `kernel/src/syscall/validate.rs` | `// FIXME: Complete user pointer validation for all syscalls` | HIGH | Search: `FIXME` | P6-02 (Security) |
| S-04 | `crates/identity-service/src/lib.rs` | `// STUB: Key storage — replace with HSM/TPM` | HIGH | P6-00 finding | P6-02 (Security) |
| S-05 | `kernel/src/driver/iommu.rs` | IOMMU stub — no actual hardware programming | CRITICAL | P6-00 finding | P6-05 (Drivers) |
| S-06 | `kernel/src/dma/mod.rs` | DMA ownership not tracked; buffers can be freed while device owns them | CRITICAL | P6-00 finding | P6-04 (Memory) |
| S-07 | `kernel/src/driver/mmio.rs` | MMIO read/write — no bounds checking in some paths | HIGH | P6-00 finding | P6-05 (Drivers) |
| S-08 | `kernel/src/arch/mod.rs` | ~15 unsafe blocks (assembly, MSR, CR registers) — no safety comments | HIGH | P6-00 finding | P6-03 (Concurrency) |
| S-09 | `kernel/src/boot/mod.rs` | 5 unsafe blocks (memory map, framebuffer) — minimal docs | MEDIUM | P6-00 finding | P6-02 (Boot) |
| S-10 | `kernel/src/memory/physical.rs` | 8 unsafe blocks (page allocator) — no invariants documented | HIGH | P6-00 finding | P6-04 (Memory) |
| S-11 | `kernel/src/memory/virtual_.rs` | 12 unsafe blocks (page table manipulation) — critical for isolation | CRITICAL | P6-00 finding | P6-04 (Memory) |
| S-12 | `kernel/src/dma/mod.rs` | 6 unsafe blocks (DMA buffer mapping) — IOMMU not enforced | CRITICAL | P6-00 finding | P6-04 (Memory) |
| S-13 | `kernel/src/syscall/dispatcher.rs` | 3 unsafe blocks (user pointer validation) — incomplete | HIGH | P6-00 finding | P6-02 (Security) |
| S-14 | `crates/energy-telemetry/src/ring_buffer.rs` | 12 unsafe blocks — no formal verification of memory ordering | MEDIUM | P6-00 finding | P6-04 (Memory) |

---

## 7. Observability & Reliability Debt

| ID | Location | Description | Severity | Source | Recommended Milestone |
|---|---|---|---|---|---|
| O-01 | `crates/energy-telemetry/` | No tracing, correlation IDs, structured logging | MEDIUM | P6-00 finding | P6-01 (Reliability) |
| O-02 | `kernel/src/logging/` | Basic logger only; no structured fields, no sampling | MEDIUM | P6-00 finding | P6-01 (Reliability) |
| O-03 | `kernel/src/tracing/` | TraceCtx exists but not integrated with user-space tracing | LOW | P6-00 finding | P6-01 (Reliability) |
| O-04 | `crates/system-core/` | No health check endpoint, no readiness/liveness probes | MEDIUM | P6-00 finding | P6-08 (Services) |
| O-05 | `crates/device-manager/` | No device health monitoring, no predictive failure detection | MEDIUM | P6-00 finding | P6-05 (Drivers) |
| O-06 | All crates | No metrics export (Prometheus/OpenTelemetry) | MEDIUM | P6-00 finding | P6-01 (Reliability) |
| O-07 | `kernel/src/core/health.rs` | KernelHealth exists but no automated degradation response | MEDIUM | P6-00 finding | P6-01 (Reliability) |

---

## 8. Storage & Update Debt

| ID | Location | Description | Severity | Source | Recommended Milestone |
|---|---|---|---|---|---|
| ST-01 | `kernel/src/fs/` | No filesystem abstraction — VFS operations mostly NotSupported | HIGH | KERNEL_AUDIT.md §13 | P6-06 (Storage) |
| ST-02 | `kernel/src/fs/` | No atomic writes, checksums, corruption detection | HIGH | P6-00 finding | P6-06 (Storage) |
| ST-03 | `crates/identity-service/` | Configuration stored as raw files — silent corruption possible | MEDIUM | P6-00 finding | P6-06 (Storage) |
| ST-04 | Repository | No update architecture — no transactional install, rollback, health check | HIGH | P6-00 finding | P6-09 (Updates) |
| ST-05 | Repository | No bootloader integration (systemd-boot, GRUB, UEFI) | HIGH | P6-00 finding | P6-09 (Updates) |

---

## 9. Virtualization Debt

| ID | Location | Description | Severity | Source | Recommended Milestone |
|---|---|---|---|---|---|
| V-01 | `kernel/src/` | No KVM/QEMU integration | MEDIUM | P6-00 finding | P6-11 (Virtualization) |
| V-02 | `kernel/src/` | No VM lifecycle management (create, start, stop, snapshot) | MEDIUM | P6-00 finding | P6-11 (Virtualization) |
| V-03 | `kernel/src/arch/` | No virtio device support in kernel | MEDIUM | P6-00 finding | P6-11 (Virtualization) |

---

## 10. Experimental & Unmarked Code

| ID | Location | Description | Severity | Source | Recommended Milestone |
|---|---|---|---|---|---|
| E-01 | `crates/quantum-runtime/src/scheduler.rs` | `SchedClass::Quantum` not behind `#[cfg(feature="experimental")]` | MEDIUM | KERNEL_AUDIT.md §11 | P6-10 (Experimental) |
| E-02 | `crates/qeos-qpu/` | CLI tool has no feature gate; always built | LOW | P6-00 finding | P6-10 (Experimental) |
| E-03 | `crates/quartz5d/` | Entire crate experimental; no feature gate | LOW | P6-00 finding | P6-10 (Experimental) |
---

## 11. Testing Gaps

| ID | Area | Gap | Severity | Recommended Milestone |
|---|---|---|---|---|
| T-01 | Kernel | No fuzzing of syscall boundary validation | HIGH | P6-02 (Security) |
| T-02 | Kernel | No fault injection tests (memory pressure, IRQ storms) | HIGH | P6-01 (Reliability) |
| T-03 | Kernel | No lock contention / deadlock detection tests | MEDIUM | P6-03 (Concurrency) |
| T-04 | Drivers | No IOMMU fault injection tests | HIGH | P6-05 (Drivers) |
| T-05 | IPC | No message ordering / delivery guarantee tests under load | MEDIUM | P6-08 (Services) |
| T-06 | Quantum | No hardware-in-the-loop tests (simulator only) | MEDIUM | P6-07 (Quantum) |
| T-07 | GPU | No equivalence testing between CPU fallback and real GPU | MEDIUM | P6-07 (Quantum/GPU) |
| T-08 | All | No chaos engineering / resilience testing | MEDIUM | P6-01 (Reliability) |
| T-09 | All | No performance regression benchmarks in CI | MEDIUM | P6-01 (Reliability) |

---

## 12. Documentation Debt

| ID | Area | Missing Documentation | Severity | Recommended Milestone |
|---|---|---|---|---|
| D-01 | Kernel | `architecture/boot/memory/scheduler/interrupts/smp/ipc/security/drivers/dma/qpu-interface/telemetry` not covered | MEDIUM | KERNEL_AUDIT.md §13 | P6-01 (Reliability) |
| D-02 | Repository | No ADRs (`docs/adr/`) for architectural decisions | LOW | KERNEL_AUDIT.md §13 | P6-01 (Reliability) |
| D-03 | All crates | Inconsistent module-level documentation coverage | LOW | P6-00 finding | P6-01 (Reliability) |

---

## 13. Debt Summary by Milestone

| Milestone | Debt Items | CRITICAL | HIGH | MEDIUM | LOW |
|---|---|---|---|---|---|
| P6-01 Reliability Baseline | B-01, B-02, B-03, B-05, B-06, B-07, O-01, O-02, O-03, O-06, O-07, T-02, T-08, T-09, D-01, D-02, D-03 | 0 | 3 | 11 | 3 |
| P6-02 Security Hardening | S-01, S-02, S-03, S-04, S-13, U-08, B-04, T-01 | 1 | 7 | 0 | 0 |
| P6-03 Concurrency & Locking | K-01, K-06, K-07, K-08, K-09, K-10, S-08 | 0 | 1 | 6 | 0 |
| P6-04 Memory & DMA | K-02, K-03, K-12, S-06, S-10, S-11, S-12, U-09, T-04 | 3 | 2 | 4 | 0 |
| P6-05 Drivers & IOMMU | K-11, K-12, S-05, S-07, U-02, U-03, O-05, T-03 | 1 | 4 | 3 | 0 |
| P6-06 Storage & Filesystem | K-14, ST-01, ST-02, ST-03 | 0 | 4 | 0 | 0 |
| P6-07 Quantum Runtime | K-13, U-01, U-04, U-05, U-06, T-06, T-07 | 0 | 1 | 6 | 0 |
| P6-08 Service Framework | K-15, K-16, K-17, U-10, U-11, U-12, O-04, T-05 | 0 | 4 | 4 | 0 |
| P6-09 Update/Rollback | ST-04, ST-05 | 0 | 2 | 0 | 0 |
| P6-10 Experimental Gates | E-01, E-02, E-03, U-07 | 0 | 0 | 1 | 3 |
| P6-11 Virtualization | V-01, V-02, V-03 | 0 | 0 | 3 | 0 |
| P6-22 Reproducible Builds | B-08 | 0 | 0 | 1 | 0 |

**Total:** 5 CRITICAL, 28 HIGH, 38 MEDIUM, 6 LOW = **77 debt items**

---

## 14. Remediation Priority Matrix

### Immediate (P6-01, P6-02, P6-04)
- **B-01, B-02, B-03**: Toolchain pinning, kernel target build, kernel tests in CI
- **S-01, S-05, S-06, S-11, S-12**: Secure boot, IOMMU, DMA ownership, page table safety
- **K-04, K-05**: Real panic handler, boot entry point

### Short-term (P6-03, P6-05, P6-06)
- **K-01, K-09, K-10**: IRQ-safe locking, lock ordering
- **K-11, K-12, S-07**: Hardware IOMMU, driver DMA, MMIO bounds
- **K-14, ST-01, ST-02**: VFS implementation, atomic writes

### Medium-term (P6-07, P6-08, P6-09)
- **U-01, U-04, U-05, U-06**: Quantum service, hardware adapters, GPU driver
- **K-15, K-16, U-10, U-11, U-12**: Process supervisor, IPC hardening, service recovery
- **ST-04, ST-05**: Update architecture, bootloader integration

### Long-term (P6-10, P6-11, P6-22)
- **E-01, E-02, E-03, U-07**: Feature-gate experimental code
- **V-01, V-02, V-03**: Virtualization support
- **B-08**: Hermetic builds

---

## 15. Acceptance Criteria for Debt Closure

Each debt item is considered **CLOSED** when:
1. Implementation complete and tested
2. Documentation updated
3. CI passes with new validation (where applicable)
4. Code review approved by kernel/security owner
5. No regression in existing test suite

---

## 16. Traceability

| Debt ID | Origin Audit | Phase 5 Milestone | Related Specs |
|---|---|---|---|
| K-01 through K-17 | KERNEL_AUDIT.md | 5.2–5.9 | SYSCALL_ABI.md, IPC_ARCHITECTURE.md |
| U-01 through U-12 | P6-00 Audit | 5.1–5.9 | SERVICE_FRAMEWORK.md, DEVICE_MANAGER.md |
| B-01 through B-08 | P6-00 Audit | — | CI.yml, Cargo.toml |
| S-01 through S-14 | P6-00 Audit + KERNEL_AUDIT.md | 5.2, 5.4, 5.5 | SECURITY_MODEL.md, THREAT_MODEL.md |
| O-01 through O-07 | P6-00 Audit | 5.9 | TELEMETRY.md |
| ST-01 through ST-05 | P6-00 Audit | — | — |
| V-01 through V-03 | P6-00 Audit | — | — |
| E-01 through E-03 | KERNEL_AUDIT.md §11 | 5.7 | QPU_RUNTIME.md |
| T-01 through T-09 | P6-00 Audit | — | — |
| D-01 through D-03 | KERNEL_AUDIT.md §13 | — | — |

---

## 17. Next Steps

1. **Prioritize** CRITICAL and HIGH items for P6-01 through P6-04
2. **Assign** ownership for each debt item to a responsible engineer
3. **Track** progress in issue tracker with milestone labels
4. **Review** at each Phase 6 gate (P6-01 through P6-22)
5. **Close** this document when all items resolved or explicitly deferred with rationale