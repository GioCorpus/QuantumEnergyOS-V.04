# QEOS V.04 — SDK Architecture

**Status:** STABLE (as of Phase 7.8)

## 1. Primary language

Rust. The QEOS SDK is the set of public workspace crates, each with a bounded,
forbidding-unsafe surface:

| Crate | Responsibility |
|---|---|
| `qeos-node` | node lifecycle, health, hardware inventory |
| `qeos-gpu-compute` | GPU device API, memory safety, CPU reference |
| `qeos-qpu` | generic QPU interface, job isolation, hardware readiness |
| `qeos-cluster` | node identity, membership, scheduler, jobs, remote admin, fleet |
| `qeos-research` | experiments, datasets, artifacts, reproducibility, workflows |
| `quantum-runtime` | Majorana model, measurement, error correction |
| `identity-service` | JWT/RBAC/authentication/audit |

## 2. Layering

```text
Research (qeos-research) / CLI (qeos-cli)
            │
            ▼
Fleet (qeos-cluster): scheduler, jobs, admin, telemetry, energy, trust
            │
            ▼
Compute (qeos-gpu-compute, qeos-qpu)   Node platform (qeos-node)
            │
            ▼
Identity (identity-service)   Quantum runtime (quantum-runtime)
```

## 3. Python / TypeScript

Only used where justified; none added in P7.8 (no Python/TS runtime is bundled).

## 4. Reality

All crates are `#![forbid(unsafe_code)]`; hardware backends are SIMULATED/
UNAVAILABLE where no real device exists (GPU/QPU).
