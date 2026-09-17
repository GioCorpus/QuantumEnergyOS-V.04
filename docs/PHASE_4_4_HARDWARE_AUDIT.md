# QEOS V.04 — Phase 4.4 Hardware Foundation Audit
**Milestone:** 4.4.0  
**Date:** 2026-09-16  
**Status:** COMPLETE (Gated for Milestone 4.4.1)

---

## 1. Executive Summary

This audit establishes the baseline of all hardware abstractions, kernel subsystems, drivers, architecture implementations, build configurations, and memory/concurrency models across the QEOS V.04 workspace. It identifies existing assets, missing capabilities, duplicated types, unsafe code boundaries, and architectural conflicts to guide the incremental implementation of Milestones 4.4.1 through 4.4.9 without regressions or architectural debt.

---

## 2. Existing Components

### 2.1 Kernel Subsystems (`kernel/src/`)
- **`arch/x86_64/`**: Basic CPU state management (`cpu.rs` using `AtomicBool`), interrupt flag controls (`interrupts.rs`), context switching structures (`context.rs`), GDT/IDT tables (`gdt.rs`, `idt.rs`), and paging table scaffolding (`paging.rs`).
- **`hal/mod.rs`**: Initial trait stubs: `CpuHal`, `TimerHal`, `PcieHal`, and reference `NullHal`.
- **`driver/`**:
  - `pci.rs`: `PciAddr` (bus, dev, func), `PciIdentity`, `BarDescriptor`, `InterruptFacts`, `DmaFacts`, `PciFunction`, `PciConfigSource`, and `StubConfigSource` (includes QEMU virtio-net fixture).
  - `mmio.rs`: `MmioWindow` with 32-bit aligned read/write and boundary checks over host memory vector.
  - `interrupt.rs`: `Irq` struct and `IrqHandler` trait with `top_half` contract.
  - `iommu.rs`: `Iommu` trait (`create_domain`, `attach`, `map`, `unmap`), `DomainId`, `IommuPerm`, and `MockIommu`.
  - `device.rs`: `Device`, `DeviceId`, `Driver` trait, and `DriverRegistry`.
  - `lifecycle.rs`: `DeviceLifecycle` enum with 9 states and transition rules.
  - `bus.rs`: `BusKind` enumeration (`Pcie`, `I2c`, `Spi`, `Nvme`, `Qpu`).
- **`dma/mod.rs`**: `DmaBuffer` trait, `DmaRegion` with alignment/size bounds, `DmaMapping` with IOVA validation, and `DmaRing` (fixed-size descriptor ring).
- **`ring/mod.rs`**: `SpscRing<T, N>` with atomic index tracking and overflow policies (`DropNew`, `OverwriteOld`, `Backpressure`).
- **`sync/spin.rs`**: `SpinLock<T>` implementation using `UnsafeCell` and `AtomicBool`.
- **`security/`**, **`telemetry/`**, **`ipc/`**, **`scheduler/`**, **`memory/`**: Kernel-side support infrastructure.

### 2.2 Workspace Crates (`crates/`)
- **`device-manager`**:
  - `pci/`: High-fidelity PCI configuration space parser (`PciConfigSpace`, `PciAddress`, `PciBar`, `BarType`, `PciCapability`, `MsiCapability`, `MsixCapability`), `SimulatedPciBackend`, and `SimulatedMmioMapper`.
  - `manager.rs`: `DeviceManager`, `ManagedDevice`, `DeviceState` (6 states), `CapabilityMask` (fine-grained hardware permission gates).
  - `driver.rs`: `DeviceDriver` trait with lifecycle hooks (`probe`, `bind`, `initialize`, `start`, `stop`, `unbind`, `handle_irq`).
  - `capability.rs`: Explicit flags (`Msi`, `Msix`, `Dma64`, `Ats`, `SrIov`, `PowerManagement`, `ExtendedTags`, etc.).
- **`hardware-abstraction`**:
  - Telemetry and device data structures (`HardwareDevice`, `DeviceClass`, `PciDeviceInfo`, `PciBarInfo`, `PciCapabilityInfo`, `GpuDeviceInfo`, `NvmeInfo`, `PowerSupplyInfo`, `QuantumDeviceInfo`, `SpiDeviceInfo`, `TelemetrySourceInfo`).
- **`quantum-hal`**:
  - `accelerator.rs`, `device.rs`, `ir.rs`, `job.rs`, `qpu.rs`, `simulator.rs`.
- **`qeos-gpu-compute`**:
  - GPU compute backend abstractions (`GpuComputeBackend`, `CpuGpuSimulator`).
- **`qeos-qpu`**:
  - CLI harness for QPU job execution.
- **`energy-telemetry`**:
  - High-performance lock-free ring buffer (`LockFreeSpscRingBuffer`), telemetry collection, and energy services.

---

## 3. Missing Components

1. **Standalone / Isolated HAL Interface (`qeos-hal`)**:
   - `kernel/src/hal/mod.rs` contains minimal stub traits without clear architectural decoupling between generic hardware interfaces and CPU architectures (`x86_64`, `arm64`).
2. **Safe, Typed MMIO Register Layer**:
   - Existing `MmioWindow` only supports raw 32-bit slices. Missing: `MmioRegion`, `MmioRegister<T>`, volatile read/write semantics for `u8`, `u16`, `u32`, `u64`, bounds validation, and ownership modeling.
3. **Comprehensive In-Kernel PCI Capability Discovery**:
   - In-kernel `pci.rs` has simplified flags but lacks standard PCI capability list traversal, MSI/MSI-X structure decoders, and Power Management capability parsing directly in HAL/Kernel.
4. **Deferred IRQ Bottom-Half Execution**:
   - Kernel `interrupt.rs` defines `IrqHandler::top_half`, but lacks a formal bottom-half deferred work queue / ring model to guarantee interrupt handlers remain non-blocking, non-allocating, and deterministic.
5. **Formal DMA Ownership State Machine**:
   - DMA abstractions lack explicit runtime state tracking for ownership transitions (`CpuOwned` ↔ `DeviceOwned` ↔ `Mapped` ↔ `Unmapped` ↔ `Released`), risking race conditions on device memory buffers.
6. **IOMMU Protection Lifecycle**:
   - `Iommu` trait needs standard device detachment (`detach_device`), domain destruction (`destroy_domain`), dynamic permission modification (`set_permissions`), and strict isolation verification tests.
7. **End-to-End Hardware Integration Test Harness**:
   - Missing a unified `MockPciDevice` combining MMIO registers, DMA buffer transfers, MSI interrupts, and IOMMU domain mapping in a single integration suite.

---

## 4. Broken Components & Linter Warnings

- **`crates/quantum-hal/src/accelerator.rs`**: Unused imports (`QuantumError`, `Result`, `JobHandle`, `JobStatus`, `QuantumJob`, `QuantumResult`, `QuantumDevice`) and unused fields (`info`, `state`, `health`) generate compiler warnings.
- **`kernel/src/driver/mmio.rs`**: Unused field `base` in `MmioWindow`.
- **`crates/energy-telemetry/src/telemetry.rs`**: Unused field `config` in `TelemetryService`.
- **`crates/system-core/src/manager.rs`**: Unused field `MockService.name` in test module.
- **`kernel/src/ring/mod.rs`**: `SpscRing` requires `&mut self` for `push` and `pop`, preventing concurrent single-producer single-consumer execution across threads without external locking.

---

## 5. Duplicated Components

| Area | Kernel Implementation | User-Space / Library Implementation | Resolution Strategy |
|---|---|---|---|
| **PCI Address** | `kernel::driver::pci::PciAddr` (3-tuple: B/D/F) | `device-manager::pci::PciAddress` (4-tuple: Dom/B/D/F) | Unify on 4-tuple `PciAddress` (domain, bus, dev, func) across HAL and user-space. |
| **PCI BAR Descriptor** | `kernel::driver::pci::BarDescriptor` | `device-manager::pci::PciBar`, `BarType` | Standardize on shared BAR representation in HAL. |
| **Device Lifecycle** | `kernel::driver::lifecycle::DeviceLifecycle` (9 states) | `device-manager::DeviceState` (6 states) | Align `DeviceManager` to support all 9 standard lifecycle states. |
| **SPSC Ring Buffer** | `kernel::ring::SpscRing`, `kernel::dma::DmaRing` | `energy-telemetry::LockFreeSpscRingBuffer` | Standardize on lock-free Acquire/Release SPSC ring with Producer/Consumer split handles. |
| **Driver Registry** | `kernel::driver::device::DriverRegistry` | `device-manager::driver::DriverRegistry` | Crisp separation: Kernel maintains low-level device tables; user-space `DeviceManager` enforces policy, authorization, and bindings. |

---

## 6. Unsafe Code Inventory & Safety Boundaries

| Location | Mechanism | Purpose | Safety Status |
|---|---|---|---|
| `kernel/src/lib.rs:10` | `#![forbid(unsafe_op_in_unsafe_fn)]` | Compiler enforcement | **PASS** — Enforces explicit unsafe blocks inside unsafe functions. |
| `crates/device-manager/src/lib.rs:43` | `#![forbid(unsafe_code)]` | Zero-unsafe policy in user-space | **PASS** — Pure safe Rust for config space decode and policy management. |
| `kernel/src/sync/spin.rs` | `UnsafeCell`, pointer dereferencing in `lock()` / `unlock()` | In-kernel non-sleeping spinlock | **AUDITED** — Properly guarded with `// SAFETY:` invariants and atomic test-and-set. |
| `crates/energy-telemetry/src/ring_buffer.rs` | `UnsafeCell`, `zeroed()`, slot write | Lock-free SPSC telemetry buffer | **AUDITED** — Sound for `Copy` types; Acquire/Release atomic index synchronization prevents data races. |
| `kernel/src/arch/x86_64/cpu.rs` | `AtomicBool` | CPU initialization state | **PASS** — 0 unsafe blocks (migrated from `static mut`). |
| `kernel/src/arch/x86_64/interrupts.rs` | `AtomicBool` | Interrupt system state | **PASS** — 0 unsafe blocks (migrated from `static mut`). |

### Hardware Safety Invariants for Future Milestones:
1. **MMIO Safety**: MMIO memory regions must be bounded, non-overlapping, strictly aligned, and access must use `core::ptr::read_volatile` / `core::ptr::write_volatile`.
2. **DMA Safety**: Physical addresses must never be exposed directly to unprivileged user space. IOVA mappings must be verified via IOMMU bounds.
3. **Interrupt Safety**: Top-half handlers must never allocate on heap, block, or invoke complex algorithms.

---

## 7. Architecture Conflicts & Gaps

1. **In-Kernel vs User-Space Device Management**:
   - *Conflict*: Ambiguity in whether driver probing occurs in the kernel or via IPC to `device-manager`.
   - *Resolution*: Microkernel design pattern: Kernel exposes raw hardware primitives (BAR mapping, IRQ event posting, DMA buffer allocation, IOMMU page table updates) via syscalls/IPC; `device-manager` runs in user space with capability tokens to discover, configure, and orchestrate drivers.
2. **HAL Layering**:
   - *Conflict*: `kernel/src/hal` is currently a minimal stub, while architecture-specific assembly/registers are tied directly into `kernel/src/arch/x86_64`.
   - *Resolution*: Establish a clear trait boundary (`qeos-hal` or `kernel::hal`) exposing generic hardware interfaces (`CpuOps`, `MmioOps`, `DmaOps`, `IrqOps`, `PciOps`), implemented by `arch/x86_64` (and future `arch/arm64`).

---

## 8. Hardware Abstraction & Dependency Map

```text
+-------------------------------------------------------------------------+
|                              Applications                               |
|          (qpu-cli, Desktop, Energy Dashboard, Quantum Runtime)          |
+-------------------------------------------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                  User-Space Device & Service Framework                  |
|  - crates/device-manager  (PCI decode, capability masks, driver policy) |
|  - crates/energy-telemetry (Lock-free telemetry collection & power)     |
|  - crates/system-core     (Service bus, gateway, lifecycle management)  |
+-------------------------------------------------------------------------+
                                     | (Syscall / IPC Boundary)
                                     v
+-------------------------------------------------------------------------+
|                            QEOS Kernel Core                             |
|  - kernel::driver::lifecycle (9-state transition engine)                |
|  - kernel::dma               (Ownership state machine & IOVA mapping)   |
|  - kernel::driver::iommu     (Domain isolation & page permissions)      |
|  - kernel::driver::interrupt (Top-half acknowledge & event dispatch)   |
|  - kernel::driver::mmio      (Safe volatile MmioRegion & registers)     |
+-------------------------------------------------------------------------+
                                     |
                                     v
+-------------------------------------------------------------------------+
|                     Hardware Abstraction Layer (HAL)                    |
|                        (kernel::hal / qeos-hal)                         |
|  - Generic Traits: CpuHal, MmioHal, DmaHal, IrqHal, PciHal              |
+-------------------------------------------------------------------------+
                                     |
               +---------------------+---------------------+
               |                                           |
               v                                           v
+-----------------------------+             +-----------------------------+
|    x86_64 Architecture      |             |     ARM64 Architecture      |
| (GDT, IDT, LAPIC, Port I/O) |             |        (Future Target)      |
+-----------------------------+             +-----------------------------+
```

---

## 9. Recommended Migration Roadmap (Milestones 4.4.1–4.4.9)

1. **Milestone 4.4.1 (HAL Cleanup)**:
   - Expand `kernel::hal` with complete generic hardware traits (`CpuHal`, `MmioHal`, `DmaHal`, `IrqHal`, `PciHal`).
   - Isolate `arch/x86_64` implementation behind these traits.
2. **Milestone 4.4.2 (PCI Discovery)**:
   - Enhance PCI discovery with standard BDF addressing, header type parsing, capability scanning (MSI, MSI-X, PCIe, PM), and robust mock fixtures.
3. **Milestone 4.4.3 (BAR / MMIO)**:
   - Implement safe `MmioRegion` and typed `MmioRegister` with bounds, alignment, volatile read/write, and documented `SAFETY`.
4. **Milestone 4.4.4 (Interrupt / MSI / MSI-X Foundation)**:
   - Build device-independent IRQ/MSI/MSI-X abstractions with top-half minimal ACK and lock-free bottom-half deferred work queue.
5. **Milestone 4.4.5 (DMA Core)**:
   - Implement `DmaBuffer`, `DmaRegion`, and `DmaMapping` with explicit CPU/Device ownership state transitions.
6. **Milestone 4.4.6 (DMA Ring)**:
   - Implement production-grade lock-free SPSC DMA Ring with Acquire/Release semantics, wraparound handling, and concurrency tests.
7. **Milestone 4.4.7 (IOMMU Abstraction)**:
   - Complete `Iommu` trait (`create_domain`, `attach_device`, `detach_device`, `destroy_domain`, `map`, `unmap`, `set_permissions`) with `MockIommu` and isolation validation.
8. **Milestone 4.4.8 (Device Manager)**:
   - Reconcile the 9-state lifecycle (`Discovered` -> `Probing` -> `Initialized` -> `Ready` -> `Running` -> `Suspended` -> `Stopping` -> `Removed` -> `Failed`) with driver registration and failure recovery.
9. **Milestone 4.4.9 (PCI/DMA Integration & Phase Gate)**:
   - Create comprehensive `MockPciDevice` integration test combining PCI discovery, MMIO registers, DMA buffer transfers, MSI interrupts, and IOMMU domain mapping.
   - Run full workspace verification (`fmt`, `check`, `test`, `clippy`) and produce `docs/PHASE_4_4_REPORT.md`.

---

## 10. Risks & Mitigations

| Risk | Impact | Mitigation |
|---|---|---|
| **Raw Pointer Dereference in MMIO** | Undefined behavior / CPU fault | Strict `MmioRegion` abstraction with bounds/alignment checking before any volatile access. |
| **Unsynchronized DMA Buffer Access** | Host/device data corruption | Explicit `DmaOwnership` state machine (`CpuOwned` vs `DeviceOwned`) enforced at compile/runtime. |
| **Interrupt Latency Degradation** | Missed deadlines / system stall | Strict top-half handler rule: only ACK hardware and enqueue event to lock-free SPSC ring. |
| **IOMMU Translation Faults** | Memory corruption / access violation | Deny-by-default domain isolation and IOVA range validation in `Iommu::map`. |
