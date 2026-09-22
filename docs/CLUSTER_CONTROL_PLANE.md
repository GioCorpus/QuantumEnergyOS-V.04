# QEOS V.04 — Cluster Control Plane

**Status:** STABLE (as of Phase 7.5)

## 1. Authority

The **control plane** is the single authority over membership, heartbeats,
health, capabilities, configuration and jobs. It is the sole writer of
membership state, which avoids split-brain assumptions.

## 2. Node Registration Lifecycle

```text
DISCOVERED -> PENDING -> REGISTERED -> HEALTHY <-> DEGRADED
                                      ^    |
                                      |____|   (re-healthy)
active states -> OFFLINE <-> (reconnect -> PENDING -> re-register)
active/offline -> REVOKED (terminal)
```

- Reconnect requires re-authentication (`PENDING`).
- Revocation is terminal.

## 3. Heartbeats & Failure Detection

- Nodes send authenticated heartbeats with a **session generation**.
- A stale (older-generation) heartbeat is rejected (`StaleHeartbeat`).
- The control-plane tick marks nodes behind the heartbeat deadline as OFFLINE.

## 4. Failure Handling Matrix

| Failure | Detection | Response |
|---|---|---|
| Node crash / heartbeat loss | heartbeat timeout | mark OFFLINE |
| Duplicate active registration | admit check | reject |
| Stale/duplicate connection | generation mismatch | reject |
| Credential mismatch | SHA-256 fingerprint | reject |
| Network failure | absence of heartbeats | mark OFFLINE |

## 5. Auditability

Membership transitions (register, drain, revoke, remove) are driven by explicit
methods on `ControlPlane`; every privileged action is a typed operation that can
be wrapped in an audit event by `identity-service` at higher layers (P7.7).
