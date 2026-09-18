# Phase 8 Dependency Graph

**Date:** 2026-09-17
**Scope:** Crate dependencies for Phase 8 implementation

---

## 1. Current Workspace Dependencies (Phase 7)

```
kernel (CRITICAL)
  ├── crates/system-core (HIGH)
  │     ├── crates/identity-service (HIGH)
  │     ├── crates/energy-telemetry (HIGH)
  │     ├── crates/hardware-abstraction (HIGH)
  │     └── crates/device-manager (HIGH)
  │
  ├── crates/quantum-runtime (MEDIUM)
  │     ├── crates/quantum-hal (HIGH)
  │     └── crates/hardware-abstraction (HIGH)
  │
  ├── crates/qeos-gpu-compute (MEDIUM)
  │     ├── crates/hardware-abstraction (HIGH)
  │     └── kernel (DMA/IOMMU traits)
  │
  ├── crates/identity-service (HIGH) — standalone
  │
  ├── crates/energy-telemetry (HIGH) — standalone
  │
  ├── crates/hardware-abstraction (HIGH) — standalone (traits only)
  │
  ├── crates/device-manager (HIGH)
  │     ├── crates/hardware-abstraction (HIGH)
  │     └── kernel (PCI/DMA traits)
  │
  ├── crates/quantum-hal (HIGH)
  │     ├── crates/quantum-runtime (MEDIUM)
  │     └── crates/hardware-abstraction (HIGH)
  │
  ├── crates/quartz5d (EXPERIMENTAL) — standalone
  │
  └── crates/qeos-qpu (LOW)
        ├── crates/quantum-runtime (MEDIUM)
        ├── crates/quantum-hal (HIGH)
        └── crates/system-core (HIGH)

MISSING: crates/quantum-service (declared in workspace, no source)
```

---

## 2. Phase 8 New Crate Dependency Graph

### 2.1 Control Plane Layer

```
qeos-control-plane (P0)
  ├── crates/system-core
  ├── crates/identity-service
  ├── crates/device-manager
  ├── crates/energy-telemetry
  └── crates/hardware-abstraction

qeos-federation (P0)
  ├── qeos-control-plane
  ├── crates/identity-service
  └── crates/hardware-abstraction (capability traits)

qeos-data-plane (P0)
  ├── qeos-control-plane
  ├── crates/energy-telemetry
  ├── crates/system-core (storage service)
  └── crates/hardware-abstraction

### 2.3 AI/ML Runtime Layer

```
qeos-ai-runtime (P2)
  ├── crates/qeos-gpu-compute
  ├── crates/quantum-hal
  ├── crates/system-core
  └── crates/hardware-abstraction

qeos-ml-training (P2)
  ├── qeos-ai-runtime
  ├── qeos-data-plane
  ├── qeos-federation
  └── crates/energy-telemetry

qeos-ml-inference (P2)
  ├── qeos-ai-runtime
  ├── qeos-control-plane
  └── crates/energy-telemetry

qeos-model-registry (P2)
  ├── qeos-data-plane
  ├── crates/identity-service
  └── crates/energy-telemetry

qeos-ai-scheduler (P2)
  ├── qeos-control-plane
  ├── qeos-federation
  ├── crates/qeos-gpu-compute
  └── crates/energy-telemetry
```

### 2.4 GPU/QPU Integration Layer (Enhanced)

```
qeos-gpu-compute (ENHANCED, P2)
  ├── crates/hardware-abstraction
  ├── kernel (DMA/IOMMU traits)
  └── [NEW] cuda/rocm/vulkan backend crates

qeos-qpu (ENHANCED, P2)
  ├── crates/quantum-hal
  ├── crates/quantum-runtime
  ├── kernel (QPU hook)
  └── [NEW] hardware backend crates

qeos-hybrid-runtime (NEW, P2)
  ├── qeos-ai-runtime
  ├── qeos-qpu
  ├── crates/quantum-runtime
  └── crates/energy-telemetry
```

### 2.5 Research/Developer Platform Layer

```
qeos-experiment (P3)
  ├── crates/quantum-runtime
  ├── crates/energy-telemetry
  ├── qeos-data-plane
  └── crates/hardware-abstraction

qeos-workflow (P3)
  ├── qeos-experiment
  ├── qeos-control-plane
  └── crates/energy-telemetry

qeos-notebook (P3)
  ├── qeos-experiment
  ├── qeos-sdk-python
  └── crates/energy-telemetry

qeos-sdk-rust (P3)
  ├── qeos-control-plane
  ├── qeos-federation
  ├── qeos-data-plane
  ├── qeos-ai-runtime
  ├── qeos-ml-training
  ├── qeos-ml-inference
  ├── qeos-model-registry
  ├── qeos-gpu-compute
  ├── qeos-qpu
  ├── qeos-hybrid-runtime
  ├── qeos-experiment
  ├── qeos-workflow
  ├── crates/identity-service
  ├── crates/energy-telemetry
  └── crates/system-core

qeos-sdk-python (P3)
  └── qeos-sdk-rust (via PyO3)

qeos-sdk-typescript (P3)
  └── qeos-sdk-rust (via WASM)

qeos-cli (P3)
  └── qeos-sdk-rust

qeos-marketplace (P3)
  ├── qeos-data-plane
  ├── crates/identity-service
  └── crates/energy-telemetry

qeos-docs (P3)
  └── ALL CRATES (doc generation)
```

### 2.6 Observability & Security Layer

```
qeos-observability (P4)
  ├── crates/energy-telemetry
  ├── kernel (tracing)
  ├── qeos-data-plane
  └── crates/system-core

qeos-security (P4)
  ├── kernel (capabilities)
  ├── crates/identity-service
  ├── qeos-control-plane
  ├── qeos-federation
  └── crates/hardware-abstraction

qeos-chaos (P4)
  ├── qeos-control-plane
  ├── crates/energy-telemetry (fault injection)
  └── qeos-federation
```
qeos-cloud-provider (P1)
  ├── qeos-control-plane
  └── qeos-federation
```

### 2.2 Edge Runtime Layer

```
qeos-edge-runtime (P1)
  ├── crates/system-core
  ├── crates/energy-telemetry
  ├── crates/device-manager
  └── crates/hardware-abstraction

qeos-sync (P1)
  ├── qeos-edge-runtime
  ├── qeos-data-plane
  └── crates/energy-telemetry (CRDT state)
---

## 3. Dependency Matrix (New Crates × Existing Crates)

| New Crate | kernel | system-core | identity | energy-tel | hw-abs | device-mgr | quantum-rt | quantum-hal | gpu-compute |
|---|---|---|---|---|---|---|---|---|---|
| qeos-control-plane | | ✅ | ✅ | ✅ | ✅ | ✅ | | | |
| qeos-federation | | ✅ | ✅ | | ✅ | | | | |
| qeos-data-plane | | ✅ | | ✅ | ✅ | | | | |
| qeos-cloud-provider | | ✅ | | | | | | | |
| qeos-edge-runtime | | ✅ | | ✅ | ✅ | ✅ | | | |
| qeos-sync | | | | ✅ | | | | | |
| qeos-ai-runtime | | ✅ | | | ✅ | | | ✅ | ✅ |
| qeos-ml-training | | | | ✅ | | | | | |
| qeos-ml-inference | | ✅ | | ✅ | | | | | |
| qeos-model-registry | | | ✅ | ✅ | | | | | |
| qeos-ai-scheduler | | ✅ | | ✅ | | | | | ✅ |
| qeos-gpu-compute (enh) | ✅ | | | | ✅ | | | | |
| qeos-qpu (enh) | ✅ | | | | | | ✅ | ✅ | |
| qeos-hybrid-runtime | | | | ✅ | | | ✅ | | ✅ |
| qeos-experiment | | | | ✅ | ✅ | | ✅ | | |
| qeos-workflow | | ✅ | | ✅ | | | | | |
| qeos-notebook | | | | ✅ | | | | | |
| qeos-sdk-rust | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| qeos-sdk-python | | | | | | | | | |
| qeos-sdk-typescript | | | | | | | | | |
| qeos-cli | | | | | | | | | |
| qeos-marketplace | | | ✅ | ✅ | | | | | |
| qeos-docs | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ | ✅ |
| qeos-observability | ✅ | ✅ | | ✅ | | | | | |
| qeos-security | ✅ | ✅ | ✅ | | ✅ | | | | |
| qeos-chaos | ✅ | ✅ | | ✅ | | | | | |

---

## 4. Build Order (Topological Sort)

### Phase 0: Foundation (Parallel)
```
1. crates/hardware-abstraction (traits only, no deps)
2. crates/identity-service (standalone)
3. crates/energy-telemetry (standalone)
4. kernel (core, no user-space deps)
```

### Phase 1: Core Services (After Phase 0)
```
5. crates/device-manager (needs kernel, hw-abs)
6. crates/system-core (needs identity, energy, hw-abs, device-mgr)
7. crates/quantum-hal (needs quantum-rt, hw-abs)
8. crates/quantum-runtime (needs quantum-hal, hw-abs)
9. crates/qeos-gpu-compute (needs kernel, hw-abs)
10. crates/quartz5d (standalone)
11. crates/qeos-qpu (needs quantum-rt, quantum-hal, system-core)
```

### Phase 2: Control Plane (P0 Milestones)
```
12. qeos-control-plane (needs system-core, identity, device-mgr, energy, hw-abs)
13. qeos-federation (needs control-plane, identity, hw-abs)
14. qeos-data-plane (needs control-plane, energy, system-core, hw-abs)
```

### Phase 3: Edge/Cloud (P1 Milestones)
```
15. qeos-cloud-provider (needs control-plane, federation)
16. qeos-edge-runtime (needs system-core, energy, device-mgr, hw-abs)
17. qeos-sync (needs edge-runtime, data-plane, energy)
```

### Phase 4: AI/GPU/QPU (P2 Milestones)
```
18. qeos-ai-runtime (needs gpu-compute, quantum-hal, system-core, hw-abs)
19. qeos-ml-training (needs ai-runtime, data-plane, federation, energy)
20. qeos-ml-inference (needs ai-runtime, control-plane, energy)
21. qeos-model-registry (needs data-plane, identity, energy)
22. qeos-ai-scheduler (needs control-plane, federation, gpu-compute, energy)
23. qeos-gpu-compute enhanced (needs kernel, hw-abs, vendor backends)
24. qeos-qpu enhanced (needs kernel, quantum-rt, quantum-hal, hw backends)
25. qeos-hybrid-runtime (needs ai-runtime, qpu, quantum-rt, energy)
```

### Phase 5: Research/Dev Platform (P3 Milestones)
```
26. qeos-experiment (needs quantum-rt, energy, data-plane, hw-abs)
27. qeos-workflow (needs experiment, control-plane, energy)
28. qeos-notebook (needs experiment, sdk-python, energy)
29. qeos-sdk-rust (needs ALL control plane + runtime crates)
30. qeos-sdk-python (needs sdk-rust)
31. qeos-sdk-typescript (needs sdk-rust)
32. qeos-cli (needs sdk-rust)
33. qeos-marketplace (needs data-plane, identity, energy)
34. qeos-docs (needs ALL)
```

### Phase 6: Observability/Security (P4 Milestones)
```
35. qeos-observability (needs energy, kernel, data-plane, system-core)
36. qeos-security (needs kernel, identity, control-plane, federation, hw-abs)
37. qeos-chaos (needs control-plane, energy, federation)
```

---

## 5. Circular Dependency Analysis

**Confirmed Acyclic:** All dependencies flow from lower layers to higher layers:
- Kernel → Hardware Abstraction → Runtime → Services → Control Plane → Federation → Platform APIs → SDK/CLI

**No cycles detected** in the proposed graph.

**Watch Points:**
- `qeos-sdk-rust` depends on everything — must be built last
- `qeos-docs` depends on everything — must be built last
- `qeos-hybrid-runtime` depends on both `qeos-ai-runtime` and `qeos-qpu` — both must be built first
---

## 6. Feature Flags for Incremental Builds

```toml
# Cargo.toml workspace features
[workspace]
members = [
    "kernel",
    "crates/*",
    "qeos-control-plane",
    "qeos-federation",
    "qeos-data-plane",
    "qeos-cloud-provider",
    "qeos-edge-runtime",
    "qeos-sync",
    "qeos-ai-runtime",
    "qeos-ml-training",
    "qeos-ml-inference",
    "qeos-model-registry",
    "qeos-ai-scheduler",
    "qeos-gpu-compute",
    "qeos-qpu",
    "qeos-hybrid-runtime",
    "qeos-experiment",
    "qeos-workflow",
    "qeos-notebook",
    "qeos-sdk-rust",
    "qeos-sdk-python",
    "qeos-sdk-typescript",
    "qeos-cli",
    "qeos-marketplace",
    "qeos-docs",
    "qeos-observability",
    "qeos-security",
    "qeos-chaos",
]

# Per-crate features for optional dependencies
[features]
# qeos-gpu-compute
default = []
cuda-backend = ["cust", "cuda-sys"]
rocm-backend = ["hip-sys", "rocm-sys"]
vulkan-backend = ["ash", "vk-sys"]
cpu-fallback = []  # Always available

# qeos-qpu
simulator-backend = ["quantum-runtime/simulator"]
hardware-backend = ["qeos-qpu-hardware"]  # Future

# qeos-ai-runtime
training = ["qeos-ml-training"]
inference = ["qeos-ml-inference"]
hybrid = ["qeos-hybrid-runtime"]

# qeos-sdk-rust
full = [
    "qeos-control-plane",
    "qeos-federation",
    "qeos-data-plane",
    "qeos-ai-runtime",
    "qeos-ml-training",
    "qeos-ml-inference",
    "qeos-model-registry",
    "qeos-gpu-compute",
    "qeos-qpu",
    "qeos-hybrid-runtime",
    "qeos-experiment",
    "qeos-workflow",
]
minimal = ["qeos-control-plane", "crates/identity-service"]
```

---

## 7. Integration Test Dependencies

| Test Suite | Requires Crates | Environment |
|---|---|---|
| `control-plane-integration` | qeos-control-plane, system-core, identity, device-mgr | Local cluster (3 nodes) |
| `federation-integration` | qeos-federation, control-plane, identity | Multi-cluster (2+) |
| `data-plane-integration` | qeos-data-plane, control-plane, energy | S3-compatible + DB |
| `edge-sync-integration` | qeos-edge-runtime, qeos-sync, data-plane | Simulated partition |
| `ai-training-integration` | qeos-ml-training, ai-runtime, gpu-compute, data-plane | GPU cluster |
| `ai-inference-integration` | qeos-ml-inference, ai-runtime, model-registry | CPU/GPU |
| `hybrid-ai-qpu-integration` | qeos-hybrid-runtime, ai-runtime, qpu, quantum-rt | QPU simulator |
| `experiment-workflow-integration` | qeos-experiment, qeos-workflow, data-plane | Local |
| `sdk-integration` | qeos-sdk-rust, ALL | Against running cluster |
| `chaos-integration` | qeos-chaos, control-plane, federation, energy | Dedicated test cluster |

---

## 8. Cargo Workspace Configuration (Target)

```toml
# Cargo.toml (workspace root)
[workspace]
resolver = "2"
members = [
    "kernel",
    "crates/system-core",
    "crates/quantum-runtime",
    "crates/quantum-hal",
    "crates/identity-service",
    "crates/energy-telemetry",
    "crates/hardware-abstraction",
    "crates/device-manager",
    "crates/qeos-gpu-compute",
    "crates/quartz5d",
    "crates/qeos-qpu",
    # Phase 8 new crates:
    "qeos-control-plane",
    "qeos-federation",
    "qeos-data-plane",
    "qeos-cloud-provider",
    "qeos-edge-runtime",
    "qeos-sync",
    "qeos-ai-runtime",
    "qeos-ml-training",
    "qeos-ml-inference",
    "qeos-model-registry",
    "qeos-ai-scheduler",
    "qeos-hybrid-runtime",
    "qeos-experiment",
    "qeos-workflow",
    "qeos-notebook",
    "qeos-sdk-rust",
    "qeos-sdk-python",
    "qeos-sdk-typescript",
    "qeos-cli",
    "qeos-marketplace",
    "qeos-docs",
    "qeos-observability",
    "qeos-security",
    "qeos-chaos",
]

[workspace.dependencies]
# Core
tokio = { version = "1.38", features = ["full", "rt-multi-thread"] }
tonic = { version = "0.12", features = ["tls", "prost-codec"] }
prost = "0.12"
serde = { version = "1.0", features = ["derive"] }
thiserror = "1.0"
tracing = "0.1"
tracing-subscriber = { version = "0.3", features = ["env-filter", "json"] }
metrics = "0.21"
prometheus = "0.13"

# Hardware
wgpu = "0.20"
ash = "0.37"  # Vulkan
cust = "0.7"  # CUDA
hip-sys = "0.2"  # ROCm

# Quantum
ndarray = "0.15"
nalgebra = "0.32"

# Security
ring = "0.17"
argon2 = "0.5"
jsonwebtoken = "9.0"

# Testing
proptest = "1.0"
criterion = "0.5"

[workspace.lints]
rust = "2021"
clippy = { all = "allow", pedantic = "allow", nursery = "allow" }
rustdoc = { missing_docs = "warn" }

[profile.release]
lto = "fat"
codegen-units = 1
panic = "abort"
strip = true

[profile.bench]
debug = true
```

---

## 9. Dependency Validation Checklist

- [ ] No circular dependencies in crate graph
- [ ] All new crates have `hardware-abstraction` as dependency (trait contracts)
- [ ] `qeos-sdk-rust` is leaf node (depends on all, nothing depends on it)
- [ ] `qeos-docs` is leaf node
- [ ] Kernel only depended upon, never depends on user-space crates
- [ ] Feature flags allow building subsets (CI, embedded, minimal)
- [ ] All trait definitions in `hardware-abstraction` or `quantum-hal`
- [ ] No direct kernel → user-space crate dependencies (only via traits)
- [ ] Version compatibility: all crates use same major version of shared deps
```