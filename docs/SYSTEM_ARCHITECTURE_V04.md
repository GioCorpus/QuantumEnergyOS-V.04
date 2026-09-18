# QuantumEnergyOS V.04 — System Architecture Specification

**Document Version:** 4.0  
**Status:** ACTIVE SPECIFICATION  
**Audience:** Kernel Architects, Runtime Engineers, Security Reviewers, Platform Developers  

---

## 1. Architectural Philosophy & Core Axiom

```text
               +---------------------------------------------------+
               |                    QEOS V.04                      |
               +-------------------------+-------------------------+
                                         |
                     +-------------------+-------------------+
                     |                                       |
             [ KERNEL SPACE ]                         [ USER SPACE ]
                     |                                       |
    +----------------+----------------+      +---------------+---------------+
    |                |                |      |               |               |
[ Memory ]     [ Scheduler ]       [ IPC ]  [ Services ] [ Quantum Runtime ][ Desktop / UI ]
    |                |                |      |               |               |
    +----------------+----------------+      | Identity      | Majorana Sim  | Wayland / tinywl
                     |                       | Policy        | Compiler      | KDE Plasma
               [ Drivers ]                   | Telemetry     | QPU Adapters  | Browser Mgr
         +-----------+-----------+           | Device Mgr    | Schedulers    | Dashboard Mgr
         |           |           |           | Energy Svc    | CPU/GPU Exec  | Applications
      [ PCIe ]    [ DMA ]     [ GPU ]        +---------------+---------------+
         |           |           |                           |
         +-----------+-----------+             +-------------+-------------+
                     |                         | Energy Telemetry Pipeline |
                 [ IOMMU ]                     | Provenance: Measured/Sim  |
                                               +---------------------------+
```

### The Fundamental Rule of Isolation:
> **The Kernel is an Operating System Kernel.**  
> It manages physical memory, virtual address spaces, hardware interrupts, scheduling, processes, threads, IPC, device drivers, IOMMU protection, and low-level power/energy telemetry hooks.
>
> **The Quantum Platform Lives in User Space.**  
> Quantum IR compilation, optimization passes, gate execution, Majorana mathematical simulation, topological braiding, error correction decoders, experiments orchestration, and future hardware vendor drivers reside strictly in **user space**.

---

## 2. Layered Architecture Stack

```text
+-----------------------------------------------------------------------------------+
| 1. APPLICATIONS & DEVELOPER TOOLS                                                 |
|    qeos-cli | qpu-cli | Web Applications | JupyterLab | Python SDK | Dashboards   |
+-----------------------------------------------------------------------------------+
| 2. USER-SPACE DESKTOP & MANAGERS                                                  |
|    Wayland Compositor (tinywl / Plasma) | Browser Manager | Dashboard Manager     |
+-----------------------------------------------------------------------------------+
| 3. QUANTUM EXECUTION PLATFORM (crates/quantum-runtime, crates/quantum-hal)        |
|    Application API -> Quantum IR -> Compiler / Optimizer -> Scheduler            |
|    Backends: CPU Deterministic Simulator | Majorana Simulator | GPU Compute Queue |
+-----------------------------------------------------------------------------------+
| 4. SYSTEM SERVICES & SECURITY PLATFORM (crates/system-core, identity-service)     |
|    Service Manager | Service Bus (Tokio / IPC) | Service Gateway (Rate Limiting)  |
|    Identity Service (Argon2id, JWT, RS256) | Policy Service (RBAC, Capabilities)  |
|    Energy & Telemetry Service (Provenance Tracking: Measured/Estimated/Simulated) |
+-----------------------------------------------------------------------------------+
| 5. BOUNDARY: SYSCALL ABI & IPC PROTOCOL                                           |
|    Syscall Dispatcher | Pointer Validation | Capability Checks | Opaque Handles   |
+-----------------------------------------------------------------------------------+
| 6. KERNEL SPACE (kernel/)                                                         |
|    Process & Thread Model | Scheduler (Priority Runqueues) | Memory Allocators    |
|    Virtual Memory Manager | IPC Rings | Virtual File System | Security CapSets   |
+-----------------------------------------------------------------------------------+
| 7. DRIVERS & HARDWARE ABSTRACTION                                                 |
|    Driver Registry | Device Lifecycle (9-State) | PCIe Config | MMIO Windows      |
|    DMA Region & Rings | IOMMU Domain Isolation | Timer Hal | Interrupt Handlers   |
+-----------------------------------------------------------------------------------+
| 8. PHYSICAL HARDWARE / HOST ENVIRONMENT                                           |
|    x86_64 / AArch64 / RISC-V CPU | GPU Accelerator | PCIe Devices | Host Simulator|
+-----------------------------------------------------------------------------------+
```

---

## 3. Subsystem Breakdown

### 3.1 Kernel Subsystems
- **Memory Management (`kernel::memory`)**: Multi-tier architecture comprising physical frame allocation (`PhysicalMemoryManager`), virtual address space mapping with permissions (`VirtualMemoryManager`), bounded heap allocator (`HeapAllocator`), and out-of-memory handlers (`OomHandler`).
- **Process & Scheduler (`kernel::process`, `kernel::scheduler`)**: Process table with PID/TID tracking, credentials, and capabilities. Preemptive priority-based runqueues with thread state transitions (`Created -> Ready -> Running -> Sleeping -> Terminated`).
- **IPC Subsystem (`kernel::ipc`, `kernel::ring`)**: High-throughput message channels backed by atomic SPSC ring buffers for inter-process synchronization without kernel lock contention.
- **Security & Capabilities (`kernel::security`)**: Strict capability model (`CapSet`) gating administrative, hardware, and device operations.
- **Hardware Abstraction & Drivers (`kernel::driver`, `kernel::hal`)**: Device registry with formal 9-state machine (`Discovered -> Configured -> ResourceAllocated -> Initialized -> Active -> Paused -> ErrorRecovery -> Teardown -> Removed`), safe bounds-checked MMIO windows, IOMMU page table isolation, and interrupt acknowledgement.

### 3.2 User Space Platform
- **Microkernel Service Framework (`crates/system-core`)**: Centralized service orchestration, dependency management, service bus envelope serialization/deserialization, gateway authentication policies, and rate limiting.
- **Identity & Access Management (`crates/identity-service`)**: Cryptographic user authentication using Argon2id password hashing, RS256 JWT generation and validation, JWKS endpoint integration, session lifecycle tracking, and Role-Based Access Control (RBAC).
- **Quantum Execution Platform (`crates/quantum-runtime`, `crates/quantum-hal`, `crates/qeos-qpu`)**: Quantum intermediate representation (IR), circuit compilation, topological Majorana mode simulator with parity measurements, noise models, quantum error correction, and backend scheduling.
- **GPU & Compute Engine (`crates/qeos-gpu-compute`)**: Asynchronous compute queue abstraction supporting GPU acceleration with strict CPU fallback validation.
- **Energy & Observability Pipeline (`crates/energy-telemetry`)**: Lock-free telemetry ring buffer delivering structured energy, thermal, CPU, memory, and quantum execution metrics with mandatory data provenance classification (`measured`, `estimated`, `simulated`, `unavailable`).

---

## 4. Hardware Interaction & Realism Policy

QEOS V.04 enforces a strict honesty principle:
1. **Never simulate as real hardware**: Components operating under simulation are explicitly labeled `SIMULATED` or `MOCK`.
2. **Deterministic Majorana Simulator**: The Majorana mathematical model ($\{\gamma_i, \gamma_j\} = 2\delta_{ij}$, $P_{ij} = i\gamma_i\gamma_j$) operates purely in user space as a research simulator; it is never represented as physical topological hardware.
3. **Hardware Vendor Abstraction (`VendorQpuAdapter`)**: Future quantum hardware will be integrated via documented user-space device adapters and HAL interfaces without fictitious registers.
4. **Energy Provenance**: Every energy measurement published through the telemetry bus contains a provenance tag indicating its physical source.
