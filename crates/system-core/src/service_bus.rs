use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

use crate::error::{IpcError, SystemCoreError};
use crate::Result;

/// Current version of the IPC protocol
pub const IPC_PROTOCOL_VERSION: u32 = 1;

/// Standard message envelope for inter-process communication
///
/// All messages traveling over the service bus follow this standard envelope format,
/// which provides metadata for routing, tracing, and protocol versioning.
///
/// # Fields
/// - `version`: Protocol version for compatibility checks
/// - `message_id`: Unique identifier for this message
/// - `service`: Name of the target service
/// - `event`: Name of the event or RPC method being invoked
/// - `trace_id`: Distributed tracing ID for request correlation
/// - `timestamp`: ISO 8601 timestamp of message creation
/// - `payload`: Arbitrary JSON payload carrying the actual data
///
/// # Example
/// ```ignore
/// let msg = Message {
///     version: 1,
///     message_id: Uuid::new_v4().to_string(),
///     service: "quantum".to_string(),
///     event: "job.submitted".to_string(),
///     trace_id: "trace-12345".to_string(),
///     timestamp: "2026-09-01T10:30:00Z".to_string(),
///     payload: json!({"job_id": "job-001"}),
/// };
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    /// IPC protocol version for compatibility
    pub version: u32,

    /// Unique identifier for this message
    pub message_id: String,

    /// Target service name
    pub service: String,

    /// Event name or RPC method
    pub event: String,

    /// Distributed trace ID for request correlation
    pub trace_id: String,

    /// ISO 8601 timestamp
    pub timestamp: String,

    /// Arbitrary JSON payload
    pub payload: Value,
}

impl Message {
    /// Create a new message with auto-generated IDs and current timestamp
    pub fn new(service: impl Into<String>, event: impl Into<String>, payload: Value) -> Self {
        Self {
            version: IPC_PROTOCOL_VERSION,
            message_id: Uuid::new_v4().to_string(),
            service: service.into(),
            event: event.into(),
            trace_id: Uuid::new_v4().to_string(),
            timestamp: Utc::now().to_rfc3339(),
            payload,
        }
    }

    /// Create a new message with an explicit trace ID for request correlation
    pub fn with_trace(
        service: impl Into<String>,
        event: impl Into<String>,
        trace_id: impl Into<String>,
        payload: Value,
    ) -> Self {
        Self {
            version: IPC_PROTOCOL_VERSION,
            message_id: Uuid::new_v4().to_string(),
            service: service.into(),
            event: event.into(),
            trace_id: trace_id.into(),
            timestamp: Utc::now().to_rfc3339(),
            payload,
        }
    }

    /// Validate the message structure and protocol version
    pub fn validate(&self) -> Result<(), IpcError> {
        if self.version != IPC_PROTOCOL_VERSION {
            return Err(IpcError::InvalidVersion(self.version));
        }

        if self.service.is_empty() {
            return Err(IpcError::ValidationError("service name cannot be empty".to_string()));
        }

        if self.event.is_empty() {
            return Err(IpcError::ValidationError("event name cannot be empty".to_string()));
        }

        if self.message_id.is_empty() {
            return Err(IpcError::ValidationError("message_id cannot be empty".to_string()));
        }

        Ok(())
    }

    /// Serialize the message to JSON
    pub fn to_json(&self) -> Result<String, IpcError> {
        serde_json::to_string(self)
            .map_err(|e| IpcError::SerializationError(format!("message serialization failed: {}", e)))
    }

    /// Deserialize a message from JSON
    pub fn from_json(json: &str) -> Result<Self, IpcError> {
        serde_json::from_str(json)
            .map_err(|e| IpcError::DeserializationError(format!("message deserialization failed: {}", e)))
    }
}

/// Service information and metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceInfo {
    /// Service name
    pub name: String,

    /// Service version
    pub version: String,

    /// Service description
    pub description: String,

    /// Supported events/methods
    pub events: Vec<String>,
}

impl ServiceInfo {
    /// Create a new service information entry
    pub fn new(name: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            version: version.into(),
            description: String::new(),
            events: Vec::new(),
        }
    }

    /// Set the service description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = description.into();
        self
    }

    /// Add a supported event
    pub fn add_event(mut self, event: impl Into<String>) -> Self {
        self.events.push(event.into());
        self
    }
}

/// Message handler function type
pub type MessageHandler = Arc<dyn Fn(Message) -> std::pin::Pin<Box<dyn std::future::Future<Output = Result<Message, IpcError>> + Send>> + Send + Sync>;

/// Service registry for IPC service discovery
///
/// Maintains a mapping of service names to their handlers and metadata.
/// Provides registration, discovery, and routing capabilities.
#[derive(Clone)]
pub struct ServiceRegistry {
    /// Map of service name to (handler, info)
    services: Arc<RwLock<HashMap<String, (MessageHandler, ServiceInfo)>>>,
}

impl ServiceRegistry {
    /// Create a new service registry
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a service with a handler
    pub async fn register(
        &self,
        name: impl Into<String>,
        handler: MessageHandler,
        info: ServiceInfo,
    ) -> Result<()> {
        let name = name.into();
        let mut services = self.services.write().await;

        if services.contains_key(&name) {
            warn!("service '{}' already registered, overwriting", name);
        }

        services.insert(name.clone(), (handler, info));
        info!("registered service: {}", name);
        Ok(())
    }

    /// Unregister a service
    pub async fn unregister(&self, name: &str) -> Result<()> {
        let mut services = self.services.write().await;
        services.remove(name);
        info!("unregistered service: {}", name);
        Ok(())
    }

    /// Get service information
    pub async fn get_info(&self, name: &str) -> Result<ServiceInfo> {
        let services = self.services.read().await;
        services
            .get(name)
            .map(|(_, info)| info.clone())
            .ok_or_else(|| SystemCoreError::IpcServiceNotRegistered(name.to_string()).into())
    }

    /// List all registered services
    pub async fn list_services(&self) -> Result<Vec<String>> {
        let services = self.services.read().await;
        Ok(services.keys().cloned().collect())
    }

    /// Route a message to the appropriate handler
    pub async fn route(&self, message: Message) -> Result<Message, IpcError> {
        // Validate message
        message.validate()?;

        let services = self.services.read().await;
        let (handler, _) = services
            .get(&message.service)
            .ok_or_else(|| {
                error!("service not registered: {}", message.service);
                IpcError::ServiceNotRegistered(message.service.clone())
            })?;

        debug!(
            message_id = %message.message_id,
            service = %message.service,
            event = %message.event,
            "routing message"
        );

        // Call handler
        handler(message).await
    }
}

impl Default for ServiceRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple in-memory message buffer for testing and local communication.
///
/// Uses a bounded capacity to prevent unbounded memory growth.
/// When full, the oldest message is dropped (FIFO).
#[derive(Debug, Clone)]
pub struct MessageBuffer {
    messages: Arc<RwLock<VecDeque<Message>>>,
    capacity: usize,
}

impl MessageBuffer {
    /// Create a new message buffer with the specified capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            messages: Arc::new(RwLock::new(VecDeque::with_capacity(capacity))),
            capacity,
        }
    }

    /// Add a message to the buffer.
    /// If the buffer is full, the oldest message is dropped.
    pub async fn push(&self, message: Message) {
        let mut msgs = self.messages.write().await;
        if msgs.len() >= self.capacity {
            msgs.pop_front();
        }
        msgs.push_back(message);
    }

    /// Get all buffered messages.
    pub async fn all(&self) -> Vec<Message> {
        let msgs = self.messages.read().await;
        msgs.iter().cloned().collect()
    }

    /// Get the number of buffered messages.
    pub async fn len(&self) -> usize {
        let msgs = self.messages.read().await;
        msgs.len()
    }

    /// Check if buffer is empty.
    pub async fn is_empty(&self) -> bool {
        let msgs = self.messages.read().await;
        msgs.is_empty()
    }

    /// Clear all messages.
    pub async fn clear(&self) {
        let mut msgs = self.messages.write().await;
        msgs.clear();
    }
}

impl Default for MessageBuffer {
    fn default() -> Self {
        Self::new(1000)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_message_creation() {
        let msg = Message::new("quantum", "job.submitted", json!({"job_id": "123"}));
        assert_eq!(msg.version, IPC_PROTOCOL_VERSION);
        assert_eq!(msg.service, "quantum");
        assert_eq!(msg.event, "job.submitted");
        assert!(!msg.message_id.is_empty());
        assert!(!msg.trace_id.is_empty());
    }

    #[test]
    fn test_message_with_trace() {
        let trace_id = "my-trace-123";
        let msg = Message::with_trace(
            "energy",
            "consumption.reported",
            trace_id,
            json!({"watts": 100}),
        );
        assert_eq!(msg.trace_id, trace_id);
        assert_eq!(msg.service, "energy");
    }

    #[test]
    fn test_message_validation_valid() {
        let msg = Message::new("quantum", "job.submitted", json!({}));
        assert!(msg.validate().is_ok());
    }

    #[test]
    fn test_message_validation_empty_service() {
        let mut msg = Message::new("quantum", "job.submitted", json!({}));
        msg.service = String::new();
        assert!(msg.validate().is_err());
    }

    #[test]
    fn test_message_validation_empty_event() {
        let mut msg = Message::new("quantum", "job.submitted", json!({}));
        msg.event = String::new();
        assert!(msg.validate().is_err());
    }

    #[test]
    fn test_message_validation_invalid_version() {
        let mut msg = Message::new("quantum", "job.submitted", json!({}));
        msg.version = 999;
        assert!(msg.validate().is_err());
    }

    #[test]
    fn test_message_serialization() {
        let msg = Message::new("quantum", "job.submitted", json!({"id": "123"}));
        let json = msg.to_json().unwrap();
        assert!(json.contains("\"service\":\"quantum\""));
        assert!(json.contains("\"event\":\"job.submitted\""));
    }

    #[test]
    fn test_message_deserialization() {
        let original = Message::new("quantum", "job.submitted", json!({"id": "123"}));
        let json = original.to_json().unwrap();
        let deserialized = Message::from_json(&json).unwrap();
        assert_eq!(deserialized.service, original.service);
        assert_eq!(deserialized.event, original.event);
        assert_eq!(deserialized.version, original.version);
    }

    #[tokio::test]
    async fn test_message_buffer_operations() {
        let buffer = MessageBuffer::new(100);
        assert!(buffer.is_empty().await);

        let msg1 = Message::new("quantum", "job.submitted", json!({"id": "1"}));
        let msg2 = Message::new("energy", "consumption.reported", json!({"watts": 100}));

        buffer.push(msg1.clone()).await;
        buffer.push(msg2.clone()).await;

        assert_eq!(buffer.len().await, 2);
        assert!(!buffer.is_empty().await);

        let all = buffer.all().await;
        assert_eq!(all.len(), 2);

        buffer.clear().await;
        assert!(buffer.is_empty().await);
    }

    #[tokio::test]
    async fn test_service_info_construction() {
        let info = ServiceInfo::new("quantum", "0.1.0")
            .with_description("Quantum runtime service")
            .add_event("job.submitted")
            .add_event("job.completed");

        assert_eq!(info.name, "quantum");
        assert_eq!(info.version, "0.1.0");
        assert_eq!(info.description, "Quantum runtime service");
        assert_eq!(info.events.len(), 2);
    }

    #[tokio::test]
    async fn test_service_registry_operations() {
        let registry = ServiceRegistry::new();

        // Create a simple handler
        let handler: MessageHandler = Arc::new(|msg| {
            Box::pin(async move {
                Ok(Message::new(
                    msg.service.clone() + "_response",
                    msg.event.clone() + "_response",
                    json!({"status": "success"}),
                ))
            })
        });

        let info = ServiceInfo::new("test", "1.0.0")
            .with_description("Test service")
            .add_event("test.event");

        // Register service
        registry.register("test", handler, info).await.unwrap();

        // Verify registration
        let services = registry.list_services().await.unwrap();
        assert!(services.contains(&"test".to_string()));

        // Get service info
        let retrieved_info = registry.get_info("test").await.unwrap();
        assert_eq!(retrieved_info.name, "test");

        // Unregister service
        registry.unregister("test").await.unwrap();
        let services = registry.list_services().await.unwrap();
        assert!(!services.contains(&"test".to_string()));
    }

    #[tokio::test]
    async fn test_service_registry_routing() {
        let registry = ServiceRegistry::new();

        // Create a handler that echoes the message
        let handler: MessageHandler = Arc::new(|msg| {
            Box::pin(async move {
                let mut response = msg.clone();
                response.event = format!("{}_response", msg.event);
                Ok(response)
            })
        });

        let info = ServiceInfo::new("echo", "1.0.0");
        registry.register("echo", handler, info).await.unwrap();

        // Send a message
        let msg = Message::new("echo", "test.event", json!({"data": "hello"}));
        let response = registry.route(msg).await.unwrap();

        assert_eq!(response.service, "echo");
        assert_eq!(response.event, "test.event_response");
    }

    #[tokio::test]
    async fn test_service_registry_unregistered_service() {
        let registry = ServiceRegistry::new();
        let msg = Message::new("nonexistent", "test.event", json!({}));
        let result = registry.route(msg).await;
        assert!(result.is_err());
    }
}
