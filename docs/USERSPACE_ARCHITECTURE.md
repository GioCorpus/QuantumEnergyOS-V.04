# QEOS V.04 — User Space Architecture Specification

**Document Version:** 4.0  
**Status:** ACTIVE SPECIFICATION  

---

## 1. Executive Summary

User Space in QuantumEnergyOS V.04 is designed as a modular, secure, microservice-oriented platform. It contains all high-level business logic, quantum simulation and compilation, identity management, policy enforcement, energy telemetry collectors, browser management, dashboard services, and developer tooling.

User space interacts with the underlying kernel **exclusively** through the Syscall ABI and kernel IPC primitives.

---

## 2. Component Diagram

```text
+-----------------------------------------------------------------------------+
|                            USER SPACE PLATFORM                              |
|                                                                             |
|  +------------------------+   +--------------------+   +-----------------+  |
|  |     CLI / Dev Tools    |   |     Desktop UI     |   |   Applications  |  |
|  | qeos-cli | qpu-cli SDK |   | tinywl / KDE / Web |   |  Experiments    |  |
|  +-----------+------------+   +---------+----------+   +--------+--------+  |
|              |                          |                       |           |
|              +--------------------------+-----------------------+           |
|                                         |                                   |
|                                         v                                   |
|                        +--------------------------------+                   |
|                        |        SERVICE GATEWAY         |                   |
|                        | Authentication | Rate Limiting |                   |
|                        +----------------+---------------+                   |
|                                         |                                   |
|                                         v                                   |
|                        +--------------------------------+                   |
|                        |       SERVICE BUS (IPC)        |                   |
|                        | Routing | Tracing | Envelope   |                   |
|                        +----------------+---------------+                   |
|                                         |                                   |
|         +---------------+---------------+---------------+                   |
|         |               |               |               |                   |
|         v               v               v               v                   |
|  +-------------+ +-------------+ +-------------+ +-------------+            |
|  |  Identity   | |   Policy    | |  Telemetry  | |   Energy    |            |
|  |   Service   | |   Service   | |   Service   | |   Service   |            |
|  |  (Argon2id) | |   (RBAC)    | | (SPSC Ring) | |(Provenance) |            |
|  +-------------+ +-------------+ +-------------+ +-------------+            |
|         |               |               |               |                   |
|         +---------------+---------------+---------------+                   |
|                                         |                                   |
|                                         v                                   |
|                        +--------------------------------+                   |
|                        |    QUANTUM EXECUTION ENGINE    |                   |
|                        | IR | Compiler | Majorana Sim   |                   |
|                        | Schedulers | GPU / CPU Backend |                   |
|                        +--------------------------------+                   |
+-----------------------------------------------------------------------------+
                                         |
                                (Syscall ABI / IPC)
                                         |
                                         v
                                  [ KERNEL SPACE ]
```

---

## 3. Core User Space Services

### 3.1 Service Framework & Lifecycle Manager (`system-core`)
The `ServiceManager` coordinates the lifecycle of all registered system services:
- **States**: `Unregistered -> Registered -> Initialized -> Running -> Paused -> Stopped -> Failed`
- **Health Monitoring**: Continuous heartbeat checking, aggregated system health reports, and crash recovery with exponential backoff.
- **Service Registry**: Dynamic capability-based service lookup.

### 3.2 Identity & Policy Architecture (`identity-service`, `system-core::services::policy`)
- **Authentication**: Password verification via Argon2id with random salts. Cryptographic session tokens generated using RS256 with JWKS key rotation.
- **Authorization**: Role-Based Access Control (RBAC) with granular roles (`Admin`, `Developer`, `Researcher`, `Operator`, `Guest`) and capability mapping (`DeviceRead`, `DeviceWrite`, `Dma`, `Pci`, `Telemetry`, `Quantum`, `Admin`).

### 3.3 Quantum Execution Platform (`quantum-runtime`, `quantum-hal`)
The quantum execution pipeline operates completely decoupled from kernel memory:
1. **Frontend / API**: Accepts quantum circuits in high-level representations.
2. **Intermediate Representation (IR)**: Universal IR AST supporting standard unitary gates, topological braiding operators, and parity measurements.
3. **Compiler & Optimizer**: Gate cancellation, commutation analysis, and topological decomposition passes.
4. **Schedulers**: Priority-aware quantum job scheduling with timeout and shot allocation.
5. **Backends**:
   - **CPU Deterministic Simulator**: High-precision state vector simulation for golden baseline tests.
   - **Majorana Topological Simulator**: Fermionic mode algebra ($\{\gamma_i, \gamma_j\} = 2\delta_{ij}$), Tetron layout, parity measurements ($P_{ij} = i\gamma_i\gamma_j$), and quasiparticle noise models.
   - **GPU Compute Backend (`qeos-gpu-compute`)**: Asynchronous compute pipelines with CPU fallback validation.
   - **Future Vendor QPU Adapter (`VendorQpuAdapter`)**: Interface for physical hardware drivers without mock emulation masquerading as real hardware.

### 3.4 Telemetry & Energy Pipeline (`energy-telemetry`)
- SPSC lock-free telemetry ring buffer for real-time metric capture.
- Provenance tagging on every energy metric:
  - `MEASURED`: Directly acquired from hardware sensors.
  - `ESTIMATED`: Calculated using analytical power models.
  - `SIMULATED`: Generated from software workloads.
  - `UNAVAILABLE`: Hardware counters unequipped or unreadable.

### 3.5 Desktop & Browser Integration
- **Desktop Flavors**: Minimal embedded compositor (`tinywl`) and full desktop environment (`KDE Plasma`).
- **Browser Manager**: Profiles for Developer, AI, Digital Twin, Energy, and Research with sandboxed policies and no direct kernel hardware access.
- **Dashboard Platform**: Unified web-based control panel integrating telemetry streams, job queues, and security monitoring.

### 3.6 Developer CLI Platform (`qeos-cli`, `qpu-cli`)
- Interactive CLI for system monitoring, service inspection, quantum job submission, and energy profile audits.
