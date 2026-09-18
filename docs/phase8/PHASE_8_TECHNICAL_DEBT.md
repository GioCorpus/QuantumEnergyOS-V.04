# Phase 8 Technical Debt — Inherited + New Risks

**Date:** 2026-09-17
**Scope:** Technical debt catalog for Phase 8 planning

---

## Classification

| Severity | Description |
|---|---|
| **CRITICAL** | Blocks production deployment, security vulnerability, data loss risk |
| **HIGH** | Blocks Phase 8 milestone, significant functional gap |
| **MEDIUM** | Degrades quality, maintainability, or performance |
| **LOW** | Nice-to-have, cosmetic, documentation |

---

## 1. Inherited from Phase 7 (Carried Forward)

### 1.1 Critical (Must Fix Before P8-31 Production Deployment)

| ID | Title | Location | Description | Phase 8 Impact |
|---|---|---|---|---|
| B-01 | Toolchain not pinned | Missing `rust-toolchain.toml` | No reproducible builds across CI/local | All milestones |
| B-02 | Kernel not built for bare-metal in CI | `.github/workflows/ci.yml` | Only host simulation tested | P8-31 Production Deployment |
| S-01 | No secure/measured boot | `kernel/src/boot/` | No TPM integration, no boot attestation | P8-31, P8-28 Security |
| S-05 | No hardware IOMMU driver | `kernel/src/driver/iommu.rs` | `MockIommu` only, no DMA isolation | P8-08 Edge, P8-18 GPU AI |

### 1.2 High (Blocks Phase 8 Milestones)

| ID | Title | Location | Description | Blocks |
|---|---|---|---|---|
| K-04 | Panic handler not real | `kernel/src/panic.rs` | `panic_info` not `#[panic_handler]` | P8-01 API Gateway reliability |
| K-05 | No `_start` symbol | `kernel/boot/linker.ld` | `ENTRY(_start)` but no symbol | P8-31 Production Deployment |
| K-11 | MockIommu only | `kernel/src/driver/iommu.rs` | Software-only, no hardware IOMMU | P8-08 Edge Runtime, P8-18 GPU AI |
| K-12 | Driver DMA integration stub | `kernel/src/driver/dma.rs` | `todo!()` in driver DMA methods | P8-18 GPU AI, P8-20 Hybrid |
| K-14 | VFS operations unsupported | `kernel/src/fs/` | Most ops return `NotSupported` | P8-13 Data Plane, P8-23 Marketplace |
| K-15 | No process supervisor | `kernel/src/process/process.rs` | No restart, health monitoring | P8-07 Cloud Control Plane |
| K-16 | No IPC disconnect/flow control | `kernel/src/ipc/channel.rs` | Channels lack backpressure | P8-09 Sync, P8-13 Data Plane |
| U-01 | No `quantum-service` crate | Workspace Cargo.toml | Declared but missing | P8-12 Federation Scheduler |
| U-04 | QPU returns UnsupportedHardware | `crates/quantum-hal/src/lib.rs` | `ExperimentalQpuDevice` stub | P8-20 AI↔QPU Hybrid |
| U-05 | GPU probe unavailable | `crates/qeos-gpu-compute/src/lib.rs` | `probe()` always returns unavailable | P8-18 GPU AI |

### 1.3 Medium (Degrades Phase 8 Quality)

| ID | Title | Location | Description | Target |
|---|---|---|---|---|
| A-01 | Inconsistent error types | Across crates | Different error enums, no common trait | P8-24 SDK |
| A-02 | Tracing spans not propagated | `kernel/src/tracing/`, services | No distributed trace context | P8-27 Observability |
| A-03 | No API versioning | `system-core/src/gateway/` | Breaking changes risk | P8-02 API Contracts |
| A-04 | Benchmarks missing | All crates | No performance baselines | P8-29 Chaos/Validation |
| A-05 | Fuzzing harnesses absent | All crates | No automated security testing | P8-28 Security |

### 1.4 Low (Documentation/Polish)

| ID | Title | Location | Description |
|---|---|---|---|
| D-01 | Missing architecture decision records | `docs/` | No ADR log for major decisions |
| D-02 | Incomplete API documentation | All public APIs | Missing examples, error codes |
---

## 2. New Phase 8 Technical Debt (Projected)

### 2.1 Critical (Architecture-Level Risks)

| ID | Title | Risk | Mitigation |
|---|---|---|---|
| P8-C-01 | **Federation split-brain** | Network partitions cause divergent state | CRDTs, quorum reads, fencing tokens |
| P8-C-02 | **Capability confusion** | Delegated capabilities not properly bounded | Formal capability calculus, runtime verification |
| P8-C-03 | **AI model supply chain** | Unverified models deployed to production | Sigstore signing, SBOM, attestation |
| P8-C-04 | **Quantum-classical sync** | Hybrid workflows lose coherence | Deterministic replay, checkpointing |

### 2.2 High (Milestone-Level Risks)

| ID | Title | Risk | Mitigation |
|---|---|---|---|
| P8-H-01 | **Control plane scalability** | etcd/Raft bottleneck at 1000+ nodes | Sharding, lease-based caching |
| P8-H-02 | **Data plane consistency** | Artifact registry eventual consistency | Version vectors, conflict resolution |
| P8-H-03 | **GPU driver stability** | Vendor kernel drivers crash host | User-space drivers (Vulkan), isolation |
| P8-H-04 | **QPU calibration drift** | Quantum hardware fidelity varies | Continuous calibration, error budgets |
| P8-H-05 | **Edge sync conflicts** | Offline mutations conflict on reconnect | CRDTs, application-level merge |
| P8-H-06 | **Multi-tenant isolation** | Side-channels in shared GPU/QPU | Time-slicing, cache flushing, MIG |
| P8-H-07 | **Plugin sandbox escape** | WASM vulnerabilities break isolation | Capability-based syscall filtering |

### 2.3 Medium (Quality Risks)

| ID | Title | Risk | Mitigation |
|---|---|---|---|
| P8-M-01 | **API version sprawl** | Too many versions to maintain | Semantic versioning, deprecation policy |
| P8-M-02 | **SDK language drift** | Rust/Python/TS APIs diverge | Code generation from single source |
| P8-M-03 | **Observability cardinality** | High-cardinality labels explode costs | Label sanitization, aggregation rules |
| P8-M-04 | **Chaos test flakiness** | Non-deterministic failures | Deterministic simulation, seed control |

### 2.4 Low (Operational Risks)

| ID | Title | Risk |
|---|---|---|
| P8-L-01 | **Documentation lag** | Architecture out of date |
| P8-L-02 | **Example rot** | Tutorials break with API changes |
| P8-L-03 | **Dependency hell** | Transitive dependency conflicts |
---

## 3. Debt Paydown Plan (Phase 8 Milestones)

### P8-00 Baseline (This Document)
- [x] Catalog all inherited debt
- [x] Identify new Phase 8 risks
- [x] Create `rust-toolchain.toml` (B-01)
- [x] Document all unsafe code (Done in P8-00-REPORT)

### P8-01 to P8-06 (Foundation)
- [ ] Fix panic handler (K-04)
- [ ] Add API versioning (A-03)
- [ ] Implement `_start` symbol (K-05)
- [ ] Create quantum-service crate (U-01)

### P8-07 to P8-13 (Cloud/Edge/Federation)
- [ ] Process supervisor (K-15)
- [ ] IPC flow control (K-16)
- [ ] VFS implementation (K-14)
- [ ] Distributed tracing (A-02)

### P8-14 to P8-20 (AI/GPU/QPU)
- [ ] Hardware IOMMU driver (S-05, K-11)
- [ ] Driver DMA integration (K-12)
- [ ] GPU vendor backends (U-05)
- [ ] QPU hardware interface (U-04)
- [ ] Multi-tenant GPU isolation (P8-H-06)

### P8-21 to P8-27 (Research/Dev Platform)
- [ ] Unified SDK error types (A-01)
- [ ] Code-generated SDKs (P8-M-02)
- [ ] Benchmarks for all runtimes (A-04)
- [ ] Fuzzing harnesses (A-05)

### P8-28 to P8-33 (Security/Validation/Release)
- [ ] Secure boot (S-01)
- [ ] Bare-metal CI (B-02)
- [ ] Chaos engineering (P8-M-04)
- [ ] Supply chain security (P8-C-03)
- [ ] Federation split-brain testing (P8-C-01)
- [ ] Full integration validation (P8-30)

---

## 4. Debt Metrics Tracking

| Metric | Phase 7 Baseline | Phase 8 Target | Measurement |
|---|---|---|---|
| Critical debt items | 4 | 0 | Count |
| High debt items | 10 | 0 | Count |
| Unsafe code locations | 2 | 0 new | `cargo-geiger` |
| Test coverage (line) | ~78% | >85% | `cargo-tarpaulin` |
| API stability index | N/A | 1.0 (semver) | Breaking changes / release |
| Mean time to detect (MTTD) | N/A | <5 min | Observability |
| Mean time to recover (MTTR) | N/A | <30 min | Chaos engineering |

---

## 5. Decision Log (Architecture Decisions Affecting Debt)

| ADR | Title | Status | Debt Impact |
|---|---|---|---|
| ADR-001 | Host-testable kernel simulation | Accepted | Defers bare-metal debt (B-02, K-05) |
| ADR-002 | Capability-based security | Accepted | Prevents ambient authority debt |
| ADR-003 | Simulation-first quantum | Accepted | Defers QPU hardware debt (U-04) |
| ADR-004 | CPU fallback for GPU | Accepted | Defers vendor GPU debt (U-05) |
| ADR-005 | CRDTs for edge sync | Proposed | Mitigates P8-H-05 |
| ADR-006 | WASM for plugin sandbox | Proposed | Mitigates P8-H-07 |
| ADR-007 | Sigstore for model signing | Proposed | Mitigates P8-C-03 |

---

## 6. Remediation Ownership

| Component Area | Owner | Review Cadence |
|---|---|---|
| Kernel (memory, scheduler, IPC) | Kernel Team | Sprint |
| Control Plane | Platform Team | Sprint |
| Federation | Distributed Systems Team | Sprint |
| AI/ML Runtime | ML Platform Team | Sprint |
| GPU/QPU Integration | Hardware Team | Sprint |
| Research Platform | Developer Experience Team | Sprint |
| Security/Observability | SRE/Security Team | Sprint |

---

## 7. Gate Criteria for Phase 8 Progression

| Gate | Required Debt State |
|---|---|
| P8-01 Start | B-01 fixed, K-04 fixed, A-03 designed |
| P8-07 Start | K-15 fixed, K-16 designed |
| P8-14 Start | U-05 prototype, S-05/K-11/K-12 designed |
| P8-21 Start | A-01 unified, A-02 implemented |
| P8-28 Start | S-01 implemented, B-02 in CI, P8-C-01 tested |
| P8-31 Release | All CRITICAL/HIGH resolved, metrics at target |

---

## 8. Summary

**Total Inherited Debt:** 19 items (4 Critical, 10 High, 4 Medium, 1 Low)
**Projected New Phase 8 Debt:** 15 items (4 Critical, 7 High, 4 Medium, 3 Low)

**Key Insight:** Phase 8's distributed nature amplifies existing kernel/IPC gaps (K-11, K-12, K-14, K-15, K-16). The federation, AI/GPU/QPU, and multi-tenancy milestones **require** these to be resolved. The simulation-first approach (ADR-003, ADR-004) has deferred hardware debt but created integration debt that must be paid in P8-18/P8-20.
| D-03 | No migration guides | Phase 7→8 | Upgrade path undocumented |