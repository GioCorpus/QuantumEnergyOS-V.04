# QEOS V.04 — Phase 7 Technical Debt Register

**Date:** 2026-09-17
**Source:** Phase 7 Baseline Audit (P7-00)
**Status:** ACTIVE — Items tracked for resolution during Phase 7

---

## 1. Debt Classification Scheme

| Severity | Definition | SLA |
|---|---|---|
| **CRITICAL** | Blocks production deployment, security vulnerability, or data loss risk | Must fix before P7-24 |
| **HIGH** | Blocks a Phase 7 milestone, significant correctness gap | Must fix before dependent milestone |
| **MEDIUM** | Degrades maintainability, performance, or observability | Fix during Phase 7 |
| **LOW** | Cosmetic, documentation, or minor ergonomics | Fix opportunistically |

| Category | Prefix |
|---|---|
| Kernel | K- |
| User-space Runtime | U- |
| Build/CI/Tooling | B- |
| Security/Compliance | S- |
| Architecture/Design | A- |

---

## 2. Critical Debt Items (Must Fix Before P7-24)

| ID | Title | Location | Description | Impact | Resolution Strategy | Target Milestone |
|---|---|---|---|---|---|---|
| B-01 | Toolchain not pinned | `rust-toolchain.toml` missing | No pinned Rust version in repo; CI uses default. Non-reproducible builds. | All milestones | Add `rust-toolchain.toml` with MSRV 1.82.0; pin in CI | P7-01 |
| B-02 | Kernel not built for bare-metal in CI | `.github/workflows/ci.yml` | Kernel only built for host target (`x86_64-unknown-linux-gnu`). No `x86_64-unknown-none` or `aarch64-unknown-none` validation. | P7-24 (Production Deployment) | Add cross-compilation CI job with `cargo build --target x86_64-unknown-none`; verify no `std` leakage | P7-01 |
| S-01 | No secure/measured boot | `kernel/src/boot/` | No bootloader integration, no TPM measurement, no signed kernel verification. | P7-15, P7-24 | Implement UEFI boot stub with PE/COFF; add TPM2 event log; integrate with `kernel/src/security/` | P7-02 |
| S-05 | No hardware IOMMU driver | `kernel/src/driver/iommu.rs` | Only `MockIommu` exists. No VT-d, AMD-Vi, ARM SMMU drivers. DMA isolation unenforced on real hardware. | P7-03, P7-15 | Implement `IntelVtdIommu`, `AmdViIommu`, `ArmSmmuIommu` behind `IommuDriver` trait; gate with capability | P7-03 |

---

## 3. High Severity Debt Items (Block Phase 7 Milestones)

| ID | Title | Location | Description | Impact | Resolution Strategy | Target Milestone |
|---|---|---|---|---|---|---|
| K-04 | Panic handler not real | `kernel/src/panic.rs:1` | `#[panic_handler]` missing; uses `panic_info` function instead. Host tests work but not a real kernel panic handler. | P7-01 (Production Ops) | Add proper `#[panic_handler]` with serial/framebuffer output; keep host test shim | P7-01 |
| K-05 | No `_start` symbol | `kernel/boot/linker.ld` | `ENTRY(_start)` declared but no `_start` in any crate. Kernel cannot be linked as bootable image. | P7-01 | Add `kernel/src/arch/x86_64/start.rs` with `_start` → `kernel_main`; minimal boot assembly | P7-01 |
| K-11 | MockIommu only | `kernel/src/driver/iommu.rs` | `MockIommu` is software stub. No hardware IOMMU binding. | P7-03, P7-04 | See S-05; add `IommuDriver` trait and platform implementations | P7-03 |
| K-12 | Driver DMA integration stub | `kernel/src/driver/dma.rs` | `Driver::configure_dma` is `todo!()`. DMA buffer ownership not wired to driver lifecycle. | P7-03, P7-05 | Implement `Driver::configure_dma` with IOMMU domain allocation; wire to `DmaBuffer` | P7-03 |
| K-14 | VFS operations unsupported | `kernel/src/fs/` | Most `Inode` ops return `Error::NotSupported`. No persistent filesystem. | P7-01, P7-24 | Implement `RamFs` as minimal VFS backend; add `Ext4`/`Fat32` stubs behind trait | P7-01 |
| K-15 | No process supervisor | `kernel/src/process/process.rs` | Processes have no restart policy, health checks, or supervisor tree. | P7-01 | Add `ProcessSupervisor` with `RestartPolicy` (Always, OnFailure, Never); integrate with `ServiceManager` | P7-01 |
| K-16 | No IPC disconnect/flow control | `kernel/src/ipc/channel.rs` | `Channel` has no `close` notification, no backpressure propagation, no credit-based flow control. | P7-09, P7-10 | Add `Channel::close`, `Sender::is_closed`, `Receiver::is_closed`; implement credit-based flow control | P7-09 |
| U-01 | Missing `quantum-service` crate | `crates/system-core/Cargo.toml` | Workspace lists `quantum-service` as DEBT but crate doesn't exist. Service framework incomplete. | P7-12, P7-16 | Create `crates/quantum-service` implementing `QuantumRuntimeService` trait; integrate with `ServiceManager` | P7-12 |
| U-04 | ExperimentalQpuDevice unsupported | `crates/quantum-hal/src/device.rs` | `ExperimentalQpuDevice::submit` returns `UnsupportedHardware`. No path to real QPU. | P7-07 | Implement vendor adapter trait; add `RigettiQpuAdapter`, `IonQAdapter`, `QuantinuumAdapter` stubs | P7-07 |
| U-05 | GPU probe always unavailable | `crates/qeos-gpu-compute/src/lib.rs` | `probe()` returns `vendor_available: false` unconditionally. No hardware detection. | P7-05 | Implement `vulkan_probe`, `cuda_probe`, `rocm_probe` with dynamic library loading; report capabilities | P7-05 |

---

## 4. Medium Severity Debt Items (Fix During Phase 7) — Part 1

| ID | Title | Location | Description | Impact | Resolution Strategy | Target Milestone |
|---|---|---|---|---|---|---|
| K-01 | Single global allocator | `kernel/src/memory/heap.rs` | One `GlobalAllocator` for all allocations; no per-process/arena isolation. | Performance, fragmentation | Implement `SlabAllocator` for kernel objects; per-process `BumpAllocator` for user heaps | P7-23 |
| K-02 | No KASLR | `kernel/src/arch/x86_64/paging.rs` | Kernel virtual base is fixed; no randomization. | Security (S-01 related) | Add KASLR with bootloader-provided entropy; randomize kernel base at `_start` | P7-02 |
| K-03 | No stack canaries | `kernel/src/process/thread.rs` | Thread stacks have no guard pages or canaries. | Security | Add stack guard pages in `Thread::new`; compile with `-Z stack-protector` | P7-25 |
| K-06 | Timer resolution fixed | `kernel/src/time.rs` | `MonotonicClock` uses fixed 1ms tick; no high-resolution TSC/HPET. | Scheduling precision | Add `TscClock` for x86_64; `GenericTimer` for ARM; calibrate at boot | P7-23 |
| K-07 | No CPU hotplug | `kernel/src/scheduler/scheduler.rs` | Scheduler assumes fixed CPU count; no `CpuHotplugEvent`. | Scalability | Add `CpuHotplug` trait; `Scheduler::cpu_online`/`cpu_offline` | P7-23 |
| K-08 | Interrupt affinity hardcoded | `kernel/src/arch/x86_64/interrupts.rs` | IRQ affinity not configurable; all IRQs to CPU 0. | Performance | Add `IrqAffinity` API; integrate with `PcieHal::set_irq_affinity` | P7-23 |

---

## 5. Low Severity / Opportunistic Debt

| ID | Title | Location | Description | Target Milestone |
|---|---|---|---|---|
| A-01 | Inconsistent error types | Multiple crates | Some use `thiserror`, others `anyhow`, others custom enums. | P7-27 |
| A-02 | Tracing spans not propagated | `crates/system-core/src/bus.rs` | `trace_id` in message but no OpenTelemetry span context. | P7-19 |
| A-03 | No API versioning | `crates/*/src/lib.rs` | No semantic versioning in public traits; breaking changes undetected. | P7-27 |
| A-04 | Benchmarks missing | `benches/` | No `criterion` benchmarks for hot paths (scheduler, ring buffer, simulator). | P7-23 |
| A-05 | Fuzzing harnesses absent | `fuzz/` | No `cargo-fuzz` targets for parsers (ELF, PCI config, Quantum IR). | P7-26 |

---

## 6. Debt Resolution Tracking

| Milestone | CRITICAL | HIGH | MEDIUM | LOW | Notes |
|---|---|---|---|---|---|
| P7-00 (Baseline) | 4 | 10 | 18 | 5 | This audit |
| P7-01 | 2 | 6 | — | — | B-01, B-02, K-04, K-05, K-14, K-15, S-01 |
| P7-03 | — | 3 | — | — | K-11, K-12, K-13 (S-05) |
| P7-05 | — | 1 | 1 | — | U-05, U-07 |
| P7-07 | — | 1 | — | — | U-04 |
| P7-08 | — | — | 1 | — | U-06 |
| P7-09 | — | 1 | 1 | — | K-16, U-03 |
| P7-10 | — | — | 1 | — | U-02 |
| P7-12 | — | 1 | — | — | U-01 |
| P7-15 | 1 | — | — | — | S-01 |
| P7-16 | — | — | 1 | — | U-10 |
| P7-19 | — | — | 1 | 1 | U-08, A-02 |
| P7-21 | — | — | 1 | — | U-08 |
| P7-22 | — | — | 2 | — | K-09, K-10 |
| P7-23 | — | — | 5 | 1 | K-01, K-06, K-07, K-08, U-07, A-04 |
| P7-24 | 2 | — | — | — | B-02, K-14 |
| P7-25 | 1 | — | 1 | — | K-03, U-09 |
| P7-26 | — | — | — | 1 | A-05 |
| P7-27 | — | — | — | 3 | A-01, A-03, A-04 |
| **Total Remaining** | **4** | **10** | **18** | **5** | |

---

## 7. Debt Paydown Principles

1. **No new CRITICAL debt** — Any PR introducing CRITICAL debt must include resolution in same PR.
2. **HIGH debt blocks milestones** — Milestone acceptance criteria include resolution of blocking HIGH items.
3. **MEDIUM debt budget** — Allocate 20% of Phase 7 velocity to MEDIUM debt paydown.
4. **Documentation debt** — Every resolved item updates this register and relevant architecture docs.
5. **Regression prevention** — Each resolved CRITICAL/HIGH item adds a regression test.

---

## 8. Sign-Off

| Role | Name | Date | Status |
|---|---|---|---|
| Principal Architect | [Auditor] | 2026-09-17 | ✅ Baseline Established |
| Security Lead | — | — | ⏳ Pending Review |
| Release Engineer | — | — | ⏳ Pending Review |

---

*This register is a living document. Update on every milestone completion.*