# QEOS V.04 — IPC Architecture Specification

**Document Version:** 4.0  
**Status:** ACTIVE SPECIFICATION  

---

## 1. Overview & Dual-Layer IPC Model

QuantumEnergyOS V.04 employs a dual-layer IPC architecture tailored for performance, modularity, and security:

```text
+-----------------------------------------------------------------------+
| USER-SPACE SERVICE BUS (crates/system-core)                           |
| - Asynchronous message passing (Tokio, Unix Domain Sockets)           |
| - Distributed tracing via trace_id                                    |
| - Service Gateway with Rate Limiting & Auth Policies                  |
| - JSON / Typed envelope serialization                                 |
+-----------------------------------------------------------------------+
                                   |
                 (Syscalls: SyscallNo::Send / Recv)
                                   |
+-----------------------------------------------------------------------+
| KERNEL IPC CHANNELS (kernel/src/ipc, kernel/src/ring)                 |
| - High-performance atomic SPSC ring buffers                           |
| - Bounded channel queues with overflow backpressure                   |
| - Zero copy descriptor transfers                                      |
| - Capability-checked channel endpoints                                |
+-----------------------------------------------------------------------+
```

---

## 2. Standard Message Envelope Schema (User Space)

All inter-service messages on the Service Bus adhere to the standard envelope schema:

```json
{
  "version": 1,
  "message_id": "9b1deb4d-3b7d-4bad-9bdd-2b0d7b3dcb6d",
  "service": "quantum",
  "event": "job.submit",
  "trace_id": "trace-exp-4492-alpha",
  "timestamp": "2026-09-17T18:00:00.000000Z",
  "payload": {
    "job_id": "job-001",
    "circuit_name": "majorana_parity_check",
    "shots": 2048,
    "priority": "normal"
  }
}
```

### Schema Invariants:
1. `version`: Mandatory integer indicating protocol compatibility (currently `1`).
2. `message_id`: Unique UUIDv4 string per message.
3. `service`: Target service identifier (non-empty string).
4. `event`: Action or RPC verb (`<entity>.<action>`).
5. `trace_id`: Distributed correlation ID propagated across all microservices.
6. `timestamp`: RFC3339 UTC timestamp.
7. `payload`: Structured JSON payload.

---

## 3. Kernel IPC Channels (`kernel::ipc`)

The kernel layer provides lightweight, bounded channel queues:
- **`Channel`**: Bounded message queue with capacity limit `cap` and state management (`open`/`closed`).
- **`Message`**: Kernel message containing source TID, destination TID, message kind, and raw payload byte vector.
- **`SpscRing`**: Lock-free single-producer single-consumer ring buffer implementing configurable overflow policies (`DropNew`, `OverwriteOld`, `Block`).

---

## 4. Service Gateway & Protection

The `ServiceGateway` in `crates/system-core` controls access to services:
- **Rate Limiting (`ServiceRateLimiter`)**: Leaky-bucket algorithm preventing message flooding and denial-of-service.
- **Access Policies (`ServiceAccessPolicy`)**: Enforces authentication tokens and verified claims before forwarding requests to target services.
- **Routing & Discovery**: Dynamic registry dispatching events to registered service handlers.
