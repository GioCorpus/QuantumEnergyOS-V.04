# QEOS V.04 — Kernel ↔ User Space Contract

**Document Version:** 1.0  
**Milestone:** 5.1 — Kernel ↔ User Space Contract  
**Status:** ACTIVE SPECIFICATION & CONTRACT  

---

## 1. Objective & Scope

This contract establishes the formal, immutable boundary between **Kernel Space** and **User Space** in QuantumEnergyOS V.04. It strictly defines subsystem responsibilities, calling interfaces, handle semantics, memory validation rules, capability boundaries, and forbidden abstractions.

---

## 2. Core Separation of Responsibilities

```text
+-------------------------------------------------------------------------------+
| KERNEL SPACE RESPONSIBILITIES                                                 |
| - Physical & Virtual Memory Management (Paging, Page Tables, Bounds)          |
| - Process & Thread Scheduling (Priority Runqueues, Time Slices)               |
| - Hardware Interrupts & Monotonic Clock Timers                                |
| - IPC Ring Buffers & Syscall Dispatcher                                       |
| - Capability Access Control (CapSet, Permission Verification)                 |
| - Hardware Drivers (PCIe Config, Safe MMIO Windows, DMA Buffers, IOMMU)       |
| - Power & Energy Hardware Telemetry Counters                                  |
+-------------------------------------------------------------------------------+
                                      ||
                 [ RIGID BOUNDARY: SYSCALL ABI & HANDLES ]
                                      ||
+-------------------------------------------------------------------------------+
| USER SPACE RESPONSIBILITIES                                                   |
| - System Services (Identity, Policy, Telemetry, Energy, Device Orchestration) |
| - Service Bus & Service Gateway (Tokio, Async IPC, Rate Limiting)             |
| - Quantum Execution Engine (Quantum IR AST, Compiler, Optimizer, Schedulers)  |
| - Mathematical Simulators (Topological Majorana Simulator, CPU State Vector)  |
| - GPU Acceleration & Compute Queues (wgpu, compute shaders, CPU fallback)     |
| - Desktop Environment (tinywl / KDE Plasma Compositor)                        |
| - Browser Manager & Sandboxed Profiles                                        |
| - Dashboard Manager & Observability Visualizers                               |
| - Developer Tooling (qeos-cli, qpu-cli, Python/Rust SDKs, Applications)       |
+-------------------------------------------------------------------------------+
```

---

## 3. Strict Boundary Rules & Invariants

### Rule 1: Zero Information Leakage of Kernel Internals
Under no circumstances may the kernel expose:
- Raw kernel pointers (virtual addresses in the kernel higher-half).
- Physical memory addresses (e.g., `PhysPage`, raw DMA addresses).
- Internal kernel structs (e.g., `Process`, `Thread`, `Vfs`, `Inode`, `SpinLock`).
- Raw hardware registers, port I/O, or raw MMIO pointers.
- Direct DMA descriptor tables.

### Rule 2: Opaque Handle & Descriptor Model
All kernel-managed resources accessible to user space are identified exclusively by opaque numerical handles:
- **`FileHandle`**: Index into process-local open file table.
- **`ChannelHandle`**: Opaque token referencing kernel IPC channel endpoints.
- **`ProcessHandle` / `Tid`**: Numerical process/thread identifier with access gated by PID ownership.
- **`DeviceHandle`**: Opaque identifier representing a registered device; operations are gated by capability tokens.

### Rule 3: Comprehensive User Memory Validation
User-supplied pointers and slices must be validated before access:
- **Null Check**: Buffer address `ptr != 0`.
- **Overflow Check**: Address arithmetic `ptr.checked_add(len)` must not overflow `usize`.
- **Range Check**: Buffer must lie entirely within the process's assigned user-space address window (`end <= max_user_address`).
- **Permission Check**: Virtual memory pages spanned by the buffer must have appropriate read/write flags.

### Rule 4: Capability-Gated Operations
Privileged operations (device interaction, DMA mapping, raw telemetry access, system administration) require specific capabilities in the process `CapSet`:
- `Capability::DeviceRead` / `Capability::DeviceWrite` for hardware inspection.
- `Capability::Dma` for DMA ring interactions.
- `Capability::Pci` for PCIe topology queries.
- `Capability::Telemetry` for kernel performance and energy counters.
- `Capability::Quantum` for low-level quantum accelerator device access.
- `Capability::Admin` for kernel configuration and process management.

### Rule 5: Dependency Isolation
The `kernel` crate must maintain **zero dependencies** on user-space service crates (`system-core`, `quantum-runtime`, `identity-service`, `device-manager`, `quartz5d`). User space crates consume kernel features only through standard syscall wrappers or IPC protocols.

---

## 4. API Definitions

### 4.1 Kernel API (Internal Kernel Services)
- `kernel::memory`: `PhysicalMemoryManager`, `VirtualMemoryManager`, `HeapAllocator`
- `kernel::scheduler`: `Scheduler`, `Thread`, `Priority`, `SchedClass`
- `kernel::security`: `CapSet`, `Capability`, `require(caps, access)`
- `kernel::driver`: `DriverRegistry`, `MmioWindow`, `Iommu`, `PciAddr`
- `kernel::ipc`: `Channel`, `Message`, `SpscRing`

### 4.2 Syscall ABI (Boundary Interface)
- Defined in `docs/SYSCALL_ABI.md` and implemented in `kernel::syscall`.
- `dispatch_syscall(no, a0, a1, a2) -> Result<u64, SyscallError>`

### 4.3 IPC Protocol (User Space Service Bus)
- Defined in `docs/IPC_ARCHITECTURE.md` and `docs/IPC_PROTOCOL.md`.
- Envelope: `version`, `message_id`, `service`, `event`, `trace_id`, `timestamp`, `payload`.

### 4.4 Device API
- Low-level control managed by kernel drivers via typed MMIO slices and IOMMU domain mappings.
- User-space discovery and driver lifecycle management handled by `crates/device-manager` without direct port I/O or unchecked raw memory dereferencing.

---

## 5. Verification & Acceptance Criteria

| Criteria | Verification Method | Status |
|---|---|---|
| Rigid Kernel/User Boundary Defined | Documented in `KERNEL_USERSPACE_CONTRACT.md` and `SYSCALL_ABI.md` | **PASS** |
| Zero Circular Dependencies | Verified via Cargo workspace dependency graph | **PASS** |
| Kernel Independent of User-Space Crates | Kernel `Cargo.toml` has 0 dependencies on `crates/*` | **PASS** |
| Pointer & Range Validation | Tested via unit & boundary integration test suites | **PASS** |
| Capability Checks Enforced | Validated via `validate::require` test cases | **PASS** |
| Opaque Handle Model Enforced | VFS and IPC handle isolation tests | **PASS** |
