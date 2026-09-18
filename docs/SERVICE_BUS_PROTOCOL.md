# QEOS V.04 — Service Bus Protocol Specification

**Document Version:** 1.0
**Phase Milestone:** P5-03 — IPC Foundation
**Protocol Version:** 1
**Status:** ACTIVE SPECIFICATION

---

## 1. Overview & Architecture

The QuantumEnergyOS Service Bus provides an asynchronous, strongly typed, and trace-correlated communication bus for user-space daemons, services, runtimes, and applications.

```text
+-------------------------------------------------------------------------------+
|                             CLIENT / APPLICATION                              |
+-------------------------------------------------------------------------------+
                                       |
                   (Request / Response / Event Stream)
                                       |
                                       v
+-------------------------------------------------------------------------------+
|                              SERVICE GATEWAY                                  |
| - Authentication Token Verification (JWT / RS256)                             |
| - Role-Based Access Control (RBAC)                                            |
| - Rate Limiter (Token Bucket)                                                 |
+-------------------------------------------------------------------------------+
                                       |
                                (Validated Msg)
                                       |
                                       v
+-------------------------------------------------------------------------------+
|                             SERVICE REGISTRY & BUS                            |
| - Routing Dispatcher                                                          |
| - Distributed Trace Propagation (trace_id)                                    |
| - Backpressure Bounded Message Buffers                                        |
| - Timeout & Cancellation Monitors                                             |
+-------------------------------------------------------------------------------+
       |                               |                               |
       v                               v                               v
+--------------+               +---------------+               +---------------+
| Auth Service |               | Quantum Engine|               | Telemetry Svc |
+--------------+               +---------------+               +---------------+
```

---

## 2. Standard Message Envelope Specification

Every message traversing the Service Bus is encapsulated in a canonical JSON envelope:

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

### Field Definitions:
- **`version` (uint32, required)**: Protocol version. Receivers MUST reject `version != 1`.
- **`message_id` (string/UUID, required)**: Unique identifier generated per message.
- **`service` (string, required)**: Target service canonical name (`auth`, `policy`, `quantum`, `telemetry`, `energy`, `device`, `browser`, `dashboard`).
- **`event` (string, required)**: Method or event name formatted as `<entity>.<action>`.
- **`trace_id` (string, required)**: Distributed trace identifier propagated across all downstream calls.
- **`timestamp` (string/RFC3339, required)**: UTC creation timestamp.
- **`payload` (JSON object, required)**: Arbitrary structured data payload.

---

## 3. Communication Patterns

### 3.1 Request-Response (RPC)
1. Caller creates a `Message` with `service`, `event`, and a fresh or inherited `trace_id`.
2. Caller awaits a response from the service handler within a bounded `timeout` (e.g., 5000 ms).
3. Service processes the request and responds with a matching `trace_id` and correlated payload.

### 3.2 Asynchronous Events (Pub/Sub)
1. Publishers broadcast state changes (`energy.consumption.reported`, `job.completed`, `service.health`).
2. Subscribed consumers receive the event payload via non-blocking bounded channels.

### 3.3 Backpressure & Bounded Buffers
- Each service inbox is protected by a bounded `MessageBuffer` (default capacity: 1000 items).
- If the queue exceeds capacity, new messages return `IpcError::BufferFull` (or are dropped based on overflow policy).

### 3.4 Cancellation & Timeouts
- All RPC handlers accept `tokio::time::Duration` timeouts and cancellation tokens.
- Stalled requests are aborted cleanly without leaking handler resources.

---

## 4. Error Codes & Representation

Errors are transmitted either as standard RPC error responses or as typed IPC exceptions:

```json
{
  "version": 1,
  "message_id": "cc0f9511-f39c-52e5-b827-557766557777",
  "service": "quantum",
  "event": "job.submit_error",
  "trace_id": "trace-exp-4492-alpha",
  "timestamp": "2026-09-17T18:00:01.000000Z",
  "payload": {
    "error": true,
    "error_code": "RESOURCE_EXHAUSTED",
    "error_message": "QPU queue is full. Maximum concurrent jobs reached."
  }
}
```
