# QuantumEnergyOS IPC Protocol Specification

**Document Version:** 1.0  
**Protocol Version:** 1  
**Last Updated:** 2026-09-01  
**Status:** ACTIVE

---

## Overview

The QuantumEnergyOS Inter-Process Communication (IPC) protocol provides a standardized message format for communication between services in the system. It enables service discovery, request/response patterns, and event publishing across the distributed system architecture.

### Key Principles

- **Service-agnostic:** Protocol is transport-independent
- **Traceable:** All messages carry trace IDs for distributed tracing
- **Versioned:** Protocol version in every message for forward/backward compatibility
- **Validated:** Strict validation of message structure and protocol compliance
- **Observable:** Integration with tracing infrastructure for debugging
- **Type-safe:** Compile-time safety through Rust's type system

---

## Protocol Version History

| Version | Date | Status | Changes |
|---------|------|--------|---------|
| 1 | 2026-09-01 | ACTIVE | Initial specification |

---

## Message Format

### Standard Message Envelope

All IPC messages follow this JSON schema:

```json
{
  "version": 1,
  "message_id": "550e8400-e29b-41d4-a716-446655440000",
  "service": "quantum",
  "event": "job.submitted",
  "trace_id": "trace-12345-abcde",
  "timestamp": "2026-09-01T10:30:45.123456Z",
  "payload": {
    "job_id": "job-001",
    "circuit": "...",
    "priority": "normal"
  }
}
```

### Field Descriptions

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `version` | uint32 | Yes | IPC protocol version (currently 1) |
| `message_id` | string (UUID) | Yes | Unique identifier for this message |
| `service` | string | Yes | Target service name (non-empty) |
| `event` | string | Yes | Event name or RPC method (non-empty) |
| `trace_id` | string | Yes | Distributed trace ID for correlation |
| `timestamp` | string (RFC3339) | Yes | ISO 8601 timestamp of message creation |
| `payload` | JSON object | Yes | Arbitrary JSON payload (can be empty object {}) |

### Validation Rules

A message is considered valid if:

1. ✅ `version == 1` (current protocol version)
2. ✅ `service` is non-empty
3. ✅ `event` is non-empty
4. ✅ `message_id` is non-empty UUID
5. ✅ `trace_id` is non-empty
6. ✅ `timestamp` is valid RFC3339 format
7. ✅ `payload` is valid JSON object

---

## Service Names

Canonical service names in QuantumEnergyOS:

| Service | Domain | Description |
|---------|--------|-------------|
| `auth` | Security | Authentication and credential management |
| `policy` | Security | Policy enforcement and RBAC |
| `identity` | Security | User identity and session management |
| `browser` | UI | Browser manager and profiles |
| `dashboard` | UI | Web-based dashboards and visualizations |
| `telemetry` | Observability | System metrics and events |
| `energy` | Infrastructure | Energy monitoring and optimization |
| `quantum` | Quantum | Quantum runtime and jobs |
| `device` | Hardware | Device management and discovery |
| `scheduler` | Orchestration | Job scheduling and lifecycle |
| `storage` | Persistence | Data storage backend |
| `network` | Infrastructure | Networking and connectivity |

---

## Event Naming Convention

Events follow the pattern: `<entity>.<action>`

### Common Event Patterns

```
# Job lifecycle
quantum.job.submitted
quantum.job.queued
quantum.job.running
quantum.job.completed
quantum.job.failed
quantum.job.cancelled

# Energy events
energy.consumption.reported
energy.consumption.predicted
energy.alert.high_usage
energy.optimization.applied

# System events
system.service.started
system.service.stopped
system.service.health_check
system.error.critical

# Authentication events
auth.login.success
auth.login.failed
auth.logout.requested
auth.token.refreshed

# Telemetry events
telemetry.sample.collected
telemetry.metric.aggregated
telemetry.alert.triggered
```

---

## Message Examples

### 1. Quantum Job Submission

**Request (Client → Quantum Service):**

```json
{
  "version": 1,
  "message_id": "550e8400-e29b-41d4-a716-446655440000",
  "service": "quantum",
  "event": "job.submit",
  "trace_id": "trace-client-123",
  "timestamp": "2026-09-01T10:30:45.123456Z",
  "payload": {
    "circuit": {
      "name": "bell-state",
      "qubits": 2,
      "gates": [
        {"gate": "h", "target": 0},
        {"gate": "cnot", "control": 0, "target": 1}
      ]
    },
    "shots": 1000,
    "backend": "simulator",
    "priority": "normal"
  }
}
```

**Response (Quantum Service → Client):**

```json
{
  "version": 1,
  "message_id": "660f9511-f39c-52e5-b827-557766551111",
  "service": "quantum",
  "event": "job.submitted",
  "trace_id": "trace-client-123",
  "timestamp": "2026-09-01T10:30:45.234567Z",
  "payload": {
    "job_id": "job-20260901-001",
    "status": "queued",
    "position_in_queue": 3,
    "estimated_wait_ms": 5000
  }
}
```

### 2. Energy Consumption Report

**Event (Energy Service → Telemetry Service):**

```json
{
  "version": 1,
  "message_id": "770f9511-f39c-52e5-b827-557766552222",
  "service": "telemetry",
  "event": "energy.consumption.reported",
  "trace_id": "trace-energy-periodic-001",
  "timestamp": "2026-09-01T10:31:00.000000Z",
  "payload": {
    "cpu_watts": 45.3,
    "gpu_watts": 0.0,
    "memory_watts": 12.5,
    "total_watts": 57.8,
    "duration_seconds": 60,
    "total_joules": 3468.0
  }
}
```

### 3. Service Health Check

**Query (Dashboard → Quantum Service):**

```json
{
  "version": 1,
  "message_id": "880f9511-f39c-52e5-b827-557766553333",
  "service": "quantum",
  "event": "service.health_check",
  "trace_id": "trace-dashboard-healthcheck-001",
  "timestamp": "2026-09-01T10:32:00.000000Z",
  "payload": {}
}
```

**Response (Quantum Service → Dashboard):**

```json
{
  "version": 1,
  "message_id": "990f9511-f39c-52e5-b827-557766554444",
  "service": "quantum",
  "event": "service.health",
  "trace_id": "trace-dashboard-healthcheck-001",
  "timestamp": "2026-09-01T10:32:00.050000Z",
  "payload": {
    "status": "healthy",
    "uptime_seconds": 3600,
    "backend": "simulator",
    "qubits_available": 64,
    "jobs_in_queue": 3,
    "jobs_completed": 150,
    "jobs_failed": 2,
    "average_latency_ms": 125.5
  }
}
```

### 4. Authentication Request

**Query (Browser → Identity Service):**

```json
{
  "version": 1,
  "message_id": "aa0f9511-f39c-52e5-b827-557766555555",
  "service": "identity",
  "event": "auth.login",
  "trace_id": "trace-browser-login-001",
  "timestamp": "2026-09-01T10:33:00.000000Z",
  "payload": {
    "username": "researcher@quantumenergyos.local",
    "password_hash": "...",
    "client_id": "browser-001",
    "requested_scopes": ["quantum:read", "energy:read"]
  }
}
```

**Response (Identity Service → Browser):**

```json
{
  "version": 1,
  "message_id": "bb0f9511-f39c-52e5-b827-557766556666",
  "service": "identity",
  "event": "auth.login_response",
  "trace_id": "trace-browser-login-001",
  "timestamp": "2026-09-01T10:33:00.200000Z",
  "payload": {
    "success": true,
    "token": "eyJhbGciOiJSUzI1NiIsInR5cCI6IkpXVCJ9...",
    "token_type": "Bearer",
    "expires_in": 3600,
    "refresh_token": "refresh_...",
    "user_id": "user-001",
    "roles": ["developer", "researcher"]
  }
}
```

---

## Transport Implementations

### 1. Unix Domain Sockets (Primary)

Used for local inter-process communication on the same system.

**Features:**
- Low latency
- No network overhead
- Kernel-mediated security
- File-based socket at `/var/run/quantumenergyos/services/<service>.sock`

**Limitations:**
- Single system only
- Requires filesystem permissions

### 2. TCP/Unix (Future)

For remote service communication.

**Features:**
- Network transparent
- Service mesh compatible
- TLS support
- Service registry discovery

**Status:** Deferred to Phase 3

### 3. HTTP/REST (Future)

For external API access.

**Features:**
- HTTP/2 support
- OpenAPI documentation
- Browser accessible
- Standard debugging tools

**Status:** Deferred to Phase 3

---

## Error Handling

### Error Response Format

When a message handler fails, it returns an error response:

```json
{
  "version": 1,
  "message_id": "cc0f9511-f39c-52e5-b827-557766557777",
  "service": "quantum",
  "event": "job.submit_error",
  "trace_id": "trace-client-123",
  "timestamp": "2026-09-01T10:30:45.334567Z",
  "payload": {
    "error": true,
    "error_code": "ALLOCATION_FAILED",
    "error_message": "Failed to allocate 100 qubits: simulator supports maximum 64",
    "error_details": {
      "requested": 100,
      "available": 64,
      "backend": "simulator"
    }
  }
}
```

### Error Codes

| Code | Status | Meaning |
|------|--------|---------|
| `VALIDATION_ERROR` | 400 | Message validation failed |
| `SERVICE_NOT_FOUND` | 404 | Target service not registered |
| `UNAUTHORIZED` | 401 | Authentication/authorization failed |
| `FORBIDDEN` | 403 | Operation not permitted for this user |
| `CONFLICT` | 409 | Resource conflict or state violation |
| `ALLOCATION_FAILED` | 400 | Resource allocation failed |
| `EXECUTION_FAILED` | 500 | Operation execution failed |
| `TIMEOUT` | 504 | Operation timed out |
| `INTERNAL_ERROR` | 500 | Internal service error |

---

## Distributed Tracing

### Trace ID Propagation

The `trace_id` field enables request tracing across service boundaries:

```
Client Request
  ↓ trace-id: "trace-user-123"
[Browser Service]
  ↓ (forward same trace_id)
[Identity Service]
  ↓ (forward same trace_id)
[Database]
  ↓ (correlate logs)
[Auth Service]
```

### Correlation Example

**Request Chain:**

1. Browser submits job with `trace_id: "trace-user-job-001"`
2. Scheduler receives message, logs with same trace ID
3. Scheduler routes to Quantum service, preserves trace ID
4. Quantum service logs execution, preserves trace ID
5. Telemetry receives notification, logs with trace ID

All logs can be correlated by searching for `trace-user-job-001` across all services.

---

## Backwards Compatibility

### Protocol Version Handling

Services MUST:

1. ✅ Accept and validate protocol version in all messages
2. ✅ Reject messages with version > current (forward compatibility break)
3. ✅ Handle messages with version < current (backward compatibility)
4. ⚠️ Document version-specific behavior differences

### Upgrade Path

When introducing Protocol Version 2:

1. Services should accept v1 and v2 messages simultaneously
2. Responses preserve the version of the incoming request
3. Deprecation period: Services support previous version for 2 releases
4. Migration: Documentation provided for v1 → v2 upgrade

---

## Security Considerations

### Message Integrity

Messages should be validated for:

1. ✅ Protocol compliance (structure validation)
2. ✅ Service existence (registry lookup)
3. ✅ Permission (RBAC checks)
4. ✅ Rate limiting (prevent DoS)

### Sensitive Data Handling

- ❌ DO NOT log full messages containing passwords
- ❌ DO NOT transmit credentials in plaintext
- ✅ DO use JWT tokens for authentication
- ✅ DO use TLS for network transports
- ✅ DO hash passwords with Argon2id

### Trace ID Privacy

Trace IDs should NOT contain:
- User names
- Session tokens
- Secrets or credentials
- Personal information

---

## Operational Guidelines

### Message Validation Checklist

Before processing, handlers must verify:

- [ ] Message protocol version matches current
- [ ] Service name matches receiver
- [ ] Required fields are non-empty
- [ ] Timestamp is reasonable (not future, not too old)
- [ ] Payload is valid JSON
- [ ] Sender has permission to send this event
- [ ] Rate limits are not exceeded

### Logging Best Practices

Log all messages with context:

```rust
debug!(
    message_id = %message.message_id,
    service = %message.service,
    event = %message.event,
    trace_id = %message.trace_id,
    "processing message"
);
```

### Timeout Recommendations

| Operation | Timeout |
|-----------|---------|
| Service discovery | 100ms |
| Simple RPC call | 1000ms |
| Quantum job submission | 5000ms |
| Long-running operation | 30000ms |
| Health check | 500ms |

---

## Appendix A: Service Registry

### Service Discovery Flow

```
┌─────────┐
│ Client  │
└────┬────┘
     │ query: "quantum"
     ↓
┌──────────────────┐
│ ServiceRegistry  │
└────┬─────────────┘
     │ returns: ServiceInfo
     ↓
┌────────────┐
│ Connect to │
│  Quantum   │
│  Service   │
└────────────┘
```

### Service Registration Example

```rust
let registry = ServiceRegistry::new();
let info = ServiceInfo::new("quantum", "0.1.0")
    .with_description("Quantum runtime service")
    .add_event("job.submit")
    .add_event("job.status")
    .add_event("job.results");

registry.register("quantum", handler, info).await?;
```

---

## Appendix B: Common Service APIs

### Quantum Service

```
quantum.job.submit       → quantum.job.submitted | quantum.job.error
quantum.job.status       → quantum.job.status_response | quantum.job.error
quantum.job.results      → quantum.job.results_response | quantum.job.error
quantum.service.health   → quantum.service.health_response
quantum.service.info     → quantum.service.info_response
```

### Energy Service

```
energy.consumption.report
energy.consumption.predict     → energy.prediction_response
energy.alert.subscribe         → energy.alert_subscribed
energy.optimization.apply      → energy.optimization_applied
```

### Identity Service

```
identity.auth.login            → identity.auth.login_response | identity.auth.error
identity.auth.logout           → identity.auth.logout_response
identity.token.refresh         → identity.token.refresh_response | identity.auth.error
identity.rbac.check            → identity.rbac.check_response
```

---

## References

- **RFC 3339:** Date and Time on the Internet: Timestamps (https://tools.ietf.org/html/rfc3339)
- **RFC 4122:** UUID Format (https://tools.ietf.org/html/rfc4122)
- **JSON Schema:** https://json-schema.org/
- **Distributed Tracing:** https://opentelemetry.io/

---

**Document Author:** QuantumEnergyOS Development Team  
**Approval Status:** Phase 2 Implementation  
**Next Review:** 2026-12-01
