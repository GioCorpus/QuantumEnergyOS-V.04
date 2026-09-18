//! QEOS V.04 — IPC Foundation & Service Bus Test Suite (P5-03).
//!
//! Validates:
//! 1. Envelope serialization & deserialization roundtrip
//! 2. Message structure validation & rejection of invalid fields
//! 3. Protocol version mismatch handling
//! 4. Cross-task asynchronous message dispatch via ServiceRegistry
//! 5. Distributed trace_id propagation
//! 6. Message buffer capacity limits & FIFO behavior
//! 7. Service Gateway authorization policy & token bucket rate limiting

use serde_json::json;
use std::sync::Arc;
use system_core::{
    service_bus::{Message, MessageBuffer, ServiceInfo, ServiceRegistry, IPC_PROTOCOL_VERSION},
    service_gateway::{RequireAuthPolicy, ServiceGateway, ServiceRateLimiter},
    IpcError,
};

// ============================================================================
// 1. Serialization & Envelope Tests
// ============================================================================

#[test]
fn test_ipc_envelope_roundtrip() {
    let payload = json!({
        "job_id": "job-test-449",
        "qubits": 4,
        "backend": "majorana_sim"
    });

    let original = Message::with_trace("quantum", "job.submit", "trace-abc-123", payload);
    assert_eq!(original.version, IPC_PROTOCOL_VERSION);
    assert_eq!(original.service, "quantum");
    assert_eq!(original.event, "job.submit");
    assert_eq!(original.trace_id, "trace-abc-123");

    let json_str = original.to_json().expect("Serialization must succeed");
    let recovered = Message::from_json(&json_str).expect("Deserialization must succeed");

    assert_eq!(recovered.version, original.version);
    assert_eq!(recovered.message_id, original.message_id);
    assert_eq!(recovered.service, original.service);
    assert_eq!(recovered.event, original.event);
    assert_eq!(recovered.trace_id, original.trace_id);
    assert_eq!(recovered.payload, original.payload);
}

#[test]
fn test_ipc_validation_rejects_invalid_messages() {
    // Empty service
    let mut msg = Message::new("", "test.event", json!({}));
    assert!(matches!(msg.validate(), Err(IpcError::ValidationError(_))));

    // Empty event
    msg = Message::new("quantum", "", json!({}));
    assert!(matches!(msg.validate(), Err(IpcError::ValidationError(_))));

    // Empty message_id
    msg = Message::new("quantum", "test.event", json!({}));
    msg.message_id = "".to_string();
    assert!(matches!(msg.validate(), Err(IpcError::ValidationError(_))));
}

#[test]
fn test_ipc_version_mismatch_rejection() {
    let mut msg = Message::new("quantum", "job.submit", json!({}));
    msg.version = 999; // Incompatible future version

    match msg.validate() {
        Err(IpcError::InvalidVersion(v)) => assert_eq!(v, 999),
        other => panic!("Expected IpcError::InvalidVersion, got {:?}", other),
    }
}

// ============================================================================
// 2. Service Registry & Async Routing Tests
// ============================================================================

#[tokio::test]
async fn test_ipc_async_routing_and_trace_propagation() {
    let registry = ServiceRegistry::new();
    let info = ServiceInfo::new("telemetry", "1.0.0")
        .with_description("Telemetry collector")
        .add_event("sample.collect");

    // Register handler that echoes the trace_id in its response
    registry
        .register(
            "telemetry",
            Arc::new(|msg| {
                Box::pin(async move {
                    let resp = Message::with_trace(
                        "client",
                        "sample.acknowledged",
                        &msg.trace_id,
                        json!({"received_event": msg.event}),
                    );
                    Ok(resp)
                })
            }),
            info,
        )
        .await
        .expect("Registration should succeed");

    let trace_id = "trace-req-99901";
    let req = Message::with_trace(
        "telemetry",
        "sample.collect",
        trace_id,
        json!({"cpu_watts": 45.2}),
    );

    let response = registry.route(req).await.expect("Routing should succeed");

    assert_eq!(response.trace_id, trace_id);
    assert_eq!(response.event, "sample.acknowledged");
    assert_eq!(response.payload["received_event"], "sample.collect");
}

#[tokio::test]
async fn test_ipc_routing_to_unregistered_service_fails() {
    let registry = ServiceRegistry::new();
    let req = Message::new("nonexistent", "ping", json!({}));

    let result = registry.route(req).await;
    assert!(result.is_err());
}

// ============================================================================
// 3. Message Buffer & Backpressure Tests
// ============================================================================

#[tokio::test]
async fn test_ipc_message_buffer_fifo_and_capacity() {
    let buffer = MessageBuffer::new(3);
    assert_eq!(buffer.len().await, 0);
    assert!(buffer.is_empty().await);

    let m1 = Message::new("svc", "e1", json!(1));
    let m2 = Message::new("svc", "e2", json!(2));
    let m3 = Message::new("svc", "e3", json!(3));
    let m4 = Message::new("svc", "e4", json!(4));

    buffer.push(m1).await;
    buffer.push(m2).await;
    buffer.push(m3).await;
    assert_eq!(buffer.len().await, 3);

    // Buffer full - oldest message is dropped (FIFO)
    buffer.push(m4).await;
    assert_eq!(buffer.len().await, 3);

    // Get all messages - should have e2, e3, e4 (e1 was dropped)
    let all = buffer.all().await;
    assert_eq!(all.len(), 3);
    assert_eq!(all[0].event, "e2");
    assert_eq!(all[1].event, "e3");
    assert_eq!(all[2].event, "e4");

    buffer.clear().await;
    assert!(buffer.is_empty().await);
}

// ============================================================================
// 4. Service Gateway Authorization & Rate Limiting Tests
// ============================================================================

#[tokio::test]
async fn test_ipc_gateway_auth_policy_enforcement() {
    let registry = Arc::new(ServiceRegistry::new());
    let info = ServiceInfo::new("secure_qpu", "1.0.0").add_event("execute");

    registry
        .register(
            "secure_qpu",
            Arc::new(|_msg| {
                Box::pin(async move {
                    Ok(Message::new(
                        "client",
                        "execute.ok",
                        json!({"status": "done"}),
                    ))
                })
            }),
            info,
        )
        .await
        .unwrap();

    let auth_policy = Arc::new(RequireAuthPolicy);
    let limiter = ServiceRateLimiter::new(10, 60);
    let gateway = ServiceGateway::new((*registry).clone(), auth_policy, limiter);

    // Request without user identity is rejected
    let unauthenticated_msg = Message::new("secure_qpu", "execute", json!({}));
    let res = gateway.route_authorized(unauthenticated_msg, None).await;
    assert!(res.is_err());

    // Request with valid user credentials passes
    let authenticated_msg =
        Message::new("secure_qpu", "execute", json!({"user": "admin_researcher"}));
    let res = gateway
        .route_authorized(authenticated_msg, Some("admin_researcher"))
        .await;
    assert!(res.is_ok());
}

#[tokio::test]
async fn test_ipc_gateway_rate_limiter() {
    let limiter = ServiceRateLimiter::new(2, 60); // 2 ops capacity, 60s window

    // First 2 requests allowed
    assert!(limiter.check("client_1").await.is_ok());
    assert!(limiter.check("client_1").await.is_ok());

    // 3rd request blocked by rate limiter
    assert!(limiter.check("client_1").await.is_err());

    // Different client has its own bucket
    assert!(limiter.check("client_2").await.is_ok());
}
