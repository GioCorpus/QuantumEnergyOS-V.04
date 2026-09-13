use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::RwLock;
use tracing::{debug, warn};

use crate::error::{IpcError, ServiceError, SystemCoreError};
use crate::service_bus::{Message, ServiceInfo, ServiceRegistry};

use serde_json::json;

pub trait ServiceAccessPolicy: Send + Sync {
    fn authorize(&self, service: &str, user: Option<&str>) -> Result<(), ServiceError>;
}

#[derive(Debug, Clone, Default)]
pub struct AllowAllPolicy;

impl ServiceAccessPolicy for AllowAllPolicy {
    fn authorize(&self, _service: &str, _user: Option<&str>) -> Result<(), ServiceError> {
        Ok(())
    }
}

pub struct RequireAuthPolicy;

impl ServiceAccessPolicy for RequireAuthPolicy {
    fn authorize(&self, service: &str, user: Option<&str>) -> Result<(), ServiceError> {
        match user {
            Some(_) => Ok(()),
            None => {
                warn!(service, "authorization rejected: missing user");
                Err(ServiceError::AuthenticationFailed)
            }
        }
    }
}

pub struct ServiceRateLimiter {
    capacity: u32,
    window: Duration,
    counts: Arc<RwLock<HashMap<String, (u32, Instant)>>>,
}

impl ServiceRateLimiter {
    pub fn new(capacity: u32, window_secs: u64) -> Self {
        Self {
            capacity,
            window: Duration::from_secs(window_secs),
            counts: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn check(&self, service: &str) -> Result<(), ServiceError> {
        let now = Instant::now();
        let mut counts = self.counts.write().await;
        let (count, start) = counts.entry(service.to_string()).or_insert((0, now));

        if now.duration_since(*start) > self.window {
            *count = 0;
            *start = now;
        }

        if *count >= self.capacity {
            warn!(service, count, "rate limit exceeded");
            return Err(ServiceError::StartFailed);
        }

        *count += 1;
        debug!(service, count, "request allowed");
        Ok(())
    }
}

pub struct ServiceGateway {
    registry: ServiceRegistry,
    policy: Arc<dyn ServiceAccessPolicy>,
    limiter: ServiceRateLimiter,
}

impl ServiceGateway {
    pub fn new(
        registry: ServiceRegistry,
        policy: Arc<dyn ServiceAccessPolicy>,
        limiter: ServiceRateLimiter,
    ) -> Self {
        Self {
            registry,
            policy,
            limiter,
        }
    }

    pub async fn discover(&self, name: &str) -> Result<ServiceInfo, ServiceError> {
        self.registry.get_info(name).await.map_err(|e| match e {
            SystemCoreError::IpcServiceNotRegistered(_) => ServiceError::AuthenticationFailed,
            SystemCoreError::ServiceNotFound(_) => ServiceError::AuthenticationFailed,
            _ => ServiceError::StartFailed,
        })
    }

    pub async fn list_services(&self) -> Result<Vec<String>, ServiceError> {
        self.registry.list_services().await.map_err(|_| ServiceError::StartFailed)
    }

    pub async fn route_authorized(
        &self,
        message: Message,
        user: Option<&str>,
    ) -> Result<Message, ServiceError> {
        self.policy.authorize(&message.service, user)?;
        self.limiter.check(&message.service).await?;

        let response = self.registry.route(message).await.map_err(|e| match e {
            IpcError::ServiceNotRegistered(_) => ServiceError::AuthenticationFailed,
            _ => ServiceError::StartFailed,
        })?;

        debug!(message_id = %response.message_id, "authorized message routed");
        Ok(response)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allow_all_policy_always_authorizes() {
        let policy = AllowAllPolicy;
        assert!(policy.authorize("quantum", None).is_ok());
        assert!(policy.authorize("quantum", Some("user")).is_ok());
    }

    #[test]
    fn test_require_auth_policy_rejects_missing_user() {
        let policy = RequireAuthPolicy;
        let result = policy.authorize("quantum", None);
        assert!(result.is_err());
    }

    #[test]
    fn test_require_auth_policy_allows_user() {
        let policy = RequireAuthPolicy;
        assert!(policy.authorize("quantum", Some("user")).is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_allows_within_capacity() {
        let limiter = ServiceRateLimiter::new(2, 60);
        assert!(limiter.check("quantum").await.is_ok());
        assert!(limiter.check("quantum").await.is_ok());
    }

    #[tokio::test]
    async fn test_rate_limiter_blocks_over_capacity() {
        let limiter = ServiceRateLimiter::new(1, 60);
        assert!(limiter.check("quantum").await.is_ok());
        assert!(limiter.check("quantum").await.is_err());
    }

    #[tokio::test]
    async fn test_service_gateway_route_authorized() {
        let registry = ServiceRegistry::new();
        let gateway = ServiceGateway::new(
            registry.clone(),
            Arc::new(AllowAllPolicy),
            ServiceRateLimiter::new(10, 60),
        );

        let handler: crate::service_bus::MessageHandler = Arc::new(|msg| {
            Box::pin(async move {
                Ok(Message::new(
                    msg.service.clone(),
                    msg.event.clone() + "_response",
                    json!({"ok": true}),
                ))
            })
        });

        let info = ServiceInfo::new("quantum", "0.1.0");
        registry.register("quantum", handler, info).await.unwrap();

        let message = Message::new("quantum", "job.submit", json!({}));
        let response = gateway.route_authorized(message, Some("user")).await.unwrap();
        assert_eq!(response.event, "job.submit_response");
    }

    #[tokio::test]
    async fn test_service_gateway_blocks_unauthorized() {
        let registry = ServiceRegistry::new();
        let gateway = ServiceGateway::new(
            registry,
            Arc::new(RequireAuthPolicy),
            ServiceRateLimiter::new(10, 60),
        );

        let message = Message::new("quantum", "job.submit", json!({}));
        let result = gateway.route_authorized(message, None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_service_gateway_enforces_rate_limit() {
        let registry = ServiceRegistry::new();
        let gateway = ServiceGateway::new(
            registry,
            Arc::new(AllowAllPolicy),
            ServiceRateLimiter::new(1, 60),
        );

        let handler: crate::service_bus::MessageHandler = Arc::new(|msg| {
            Box::pin(async move { Ok(msg.clone()) })
        });

        let info = ServiceInfo::new("quantum", "0.1.0");
        registry.register("quantum", handler, info).await.unwrap();

        let message1 = Message::new("quantum", "job.submit", json!({}));
        assert!(gateway.route_authorized(message1, Some("user")).await.is_ok());

        let message2 = Message::new("quantum", "job.submit", json!({}));
        assert!(gateway.route_authorized(message2, Some("user")).await.is_err());
    }

    #[tokio::test]
    async fn test_service_gateway_discover() {
        let registry = ServiceRegistry::new();
        let gateway = ServiceGateway::new(
            registry.clone(),
            Arc::new(AllowAllPolicy),
            ServiceRateLimiter::new(10, 60),
        );

        let info = ServiceInfo::new("quantum", "0.1.0");
        let handler: crate::service_bus::MessageHandler = Arc::new(|msg| {
            Box::pin(async move { Ok(msg.clone()) })
        });
        registry.register("quantum", handler, info).await.unwrap();

        let discovered = gateway.discover("quantum").await.unwrap();
        assert_eq!(discovered.name, "quantum");
        assert_eq!(discovered.version, "0.1.0");
    }

    #[tokio::test]
    async fn test_service_gateway_list_services() {
        let registry = ServiceRegistry::new();
        let gateway = ServiceGateway::new(
            registry.clone(),
            Arc::new(AllowAllPolicy),
            ServiceRateLimiter::new(10, 60),
        );

        let info = ServiceInfo::new("quantum", "0.1.0");
        let handler: crate::service_bus::MessageHandler = Arc::new(|msg| {
            Box::pin(async move { Ok(msg.clone()) })
        });
        registry.register("quantum", handler, info).await.unwrap();

        let services = gateway.list_services().await.unwrap();
        assert!(services.contains(&"quantum".to_string()));
    }
}
