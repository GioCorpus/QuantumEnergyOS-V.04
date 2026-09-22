# QEOS V.04 — Distributed Architecture

**Status:** STABLE (as of Phase 7.5)

## 1. Layers

```text
             ┌─────────────────────────────┐
             │  Control Plane (qeos-cluster) │  authority for membership/health/jobs
             └──────────────┬──────────────┘
                            │  heartbeats, registration
              ┌─────────────┼─────────────┐
              ▼             ▼             ▼
           Node A        Node B        Node C     (qeos-node)
              │             │             │
              ▼             ▼             ▼
         Devices / GPU / QPU / Telemetry / Energy
```

- **Single node**: `qeos-node` (lifecycle, health, hardware inventory).
- **Cluster**: `qeos-cluster` (identity, registration, membership, control plane).
- **GPU**: `qeos-gpu-compute`. **QPU**: `qeos-qpu` + `quantum-runtime`.
- **Identity/auth**: `identity-service` (JWT/RBAC/audit), reused at higher layers.

## 2. Node Identity & Credentials

Nodes carry a stable `node_id` and a SHA-256 **credential fingerprint**; the
secret is never stored at rest on the control plane.

## 3. Consistency & Failure Model

- **Consistency model**: single-authority (control plane is sole membership
  writer) — no split-brain assumption.
- **Failure model**: nodes may crash/lose connectivity at any time; liveness is
  derived from heartbeats, never assumed.
- **Membership model**: control plane decides join/leave/revoke/stale handling.
- **Stale handling**: reconnect bumps a session generation; older-session
  heartbeats are rejected.

## 4. Reality Classification

| Layer | Class |
|---|---|
| Node platform (`qeos-node`) | REAL |
| Identity/registration/membership/control plane (`qeos-cluster`) | REAL |
| Physical node-to-node network transport | NOT IMPLEMENTED (future integration point) |
