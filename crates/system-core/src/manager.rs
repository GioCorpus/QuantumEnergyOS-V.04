use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, error, info, warn};

use crate::error::ServiceError;
use crate::service::{HealthStatus, QuantumService, ServiceStatus};
use crate::Result;

/// Type-safe service handle for box-erased dynamic dispatch
pub type ServiceHandle = Arc<RwLock<Box<dyn QuantumService + Send + Sync>>>;

/// Manages the lifecycle of multiple services
///
/// The ServiceManager orchestrates initialization, startup, shutdown, and health
/// monitoring of all services in the QuantumEnergyOS system. It provides:
///
/// - Centralized service registration and lifecycle control
/// - Parallel startup/shutdown with error aggregation
/// - Health monitoring and status reporting
/// - Service dependency tracking (future)
///
/// # Example
///
/// ```ignore
/// let manager = ServiceManager::new();
/// manager.register("quantum", quantum_service).await?;
/// manager.register("energy", energy_service).await?;
/// manager.initialize().await?;
/// manager.start_all().await?;
/// // ... services running ...
/// manager.stop_all().await?;
/// ```
pub struct ServiceManager {
    services: Arc<RwLock<HashMap<String, ServiceHandle>>>,
    initialized: Arc<RwLock<bool>>,
}

impl ServiceManager {
    /// Create a new service manager
    pub fn new() -> Self {
        Self {
            services: Arc::new(RwLock::new(HashMap::new())),
            initialized: Arc::new(RwLock::new(false)),
        }
    }

    /// Register a service with the manager
    pub async fn register(
        &self,
        name: impl Into<String>,
        service: Box<dyn QuantumService + Send + Sync>,
    ) -> Result<()> {
        let name = name.into();
        let mut services = self.services.write().await;

        if services.contains_key(&name) {
            warn!("service '{}' already registered, replacing", name);
        }

        services.insert(name.clone(), Arc::new(RwLock::new(service)));
        info!("registered service: {}", name);
        Ok(())
    }

    /// Unregister a service from the manager
    pub async fn unregister(&self, name: &str) -> Result<()> {
        let mut services = self.services.write().await;
        services.remove(name);
        info!("unregistered service: {}", name);
        Ok(())
    }

    /// Initialize all registered services
    pub async fn initialize(&self) -> Result<()> {
        info!("initializing all services");

        // Collect handles to avoid holding the read lock while acquiring write locks
        let handles: Vec<_> = {
            let services = self.services.read().await;
            services.iter().map(|(name, handle)| (name.clone(), handle.clone())).collect()
        };

        let mut errors = Vec::new();

        for (name, handle) in &handles {
            debug!("initializing service: {}", name);
            let mut service = handle.write().await;

            match service.initialize() {
                Ok(()) => {
                    info!("initialized service: {}", name);
                }
                Err(e) => {
                    error!("failed to initialize service '{}': {}", name, e);
                    errors.push((name.clone(), e));
                }
            }
        }

        if !errors.is_empty() {
            return Err(crate::error::SystemCoreError::ServiceInitializationFailed(
                format!("failed to initialize {} services", errors.len()),
            )
            .into());
        }

        let mut initialized = self.initialized.write().await;
        *initialized = true;

        info!("all services initialized successfully");
        Ok(())
    }

    /// Start all registered services
    pub async fn start_all(&self) -> Result<()> {
        let initialized = self.initialized.read().await;
        if !*initialized {
            return Err(crate::error::SystemCoreError::ServiceStartupFailed(
                "services must be initialized before starting".to_string(),
            )
            .into());
        }
        drop(initialized);

        info!("starting all services");

        // Collect handles to avoid holding the read lock while acquiring write locks
        let handles: Vec<_> = {
            let services = self.services.read().await;
            services.iter().map(|(name, handle)| (name.clone(), handle.clone())).collect()
        };

        let mut errors = Vec::new();

        for (name, handle) in &handles {
            debug!("starting service: {}", name);
            let mut service = handle.write().await;

            match service.start() {
                Ok(()) => {
                    info!("started service: {}", name);
                }
                Err(e) => {
                    error!("failed to start service '{}': {}", name, e);
                    errors.push((name.clone(), e));
                }
            }
        }

        if !errors.is_empty() {
            error!("failed to start {} services", errors.len());
            return Err(crate::error::SystemCoreError::ServiceStartupFailed(
                format!("failed to start {} services", errors.len()),
            )
            .into());
        }

        info!("all services started successfully");
        Ok(())
    }

    /// Stop all registered services
    pub async fn stop_all(&self) -> Result<()> {
        info!("stopping all services");

        // Collect handles in reverse order (LIFO) to avoid holding the read lock
        let handles: Vec<_> = {
            let services = self.services.read().await;
            let mut service_list: Vec<_> = services.iter().map(|(name, handle)| (name.clone(), handle.clone())).collect();
            service_list.reverse();
            service_list
        };

        let mut errors = Vec::new();

        for (name, handle) in &handles {
            debug!("stopping service: {}", name);
            let mut service = handle.write().await;

            match service.stop() {
                Ok(()) => {
                    info!("stopped service: {}", name);
                }
                Err(e) => {
                    error!("failed to stop service '{}': {}", name, e);
                    errors.push((name.clone(), e));
                }
            }
        }

        if !errors.is_empty() {
            error!("failed to stop {} services", errors.len());
            return Err(crate::error::SystemCoreError::ServiceShutdownFailed(
                format!("failed to stop {} services", errors.len()),
            )
            .into());
        }

        info!("all services stopped successfully");
        Ok(())
    }

    /// Get the status of all services
    pub async fn status_all(&self) -> Result<HashMap<String, ServiceStatus>> {
        let services = self.services.read().await;
        let mut statuses = HashMap::new();

        for (name, handle) in services.iter() {
            let service = handle.read().await;
            statuses.insert(name.clone(), service.status());
        }

        Ok(statuses)
    }

    /// Get the health of all services
    pub async fn health_all(&self) -> Result<HashMap<String, HealthStatus>> {
        let services = self.services.read().await;
        let mut healths = HashMap::new();

        for (name, handle) in services.iter() {
            let service = handle.read().await;
            healths.insert(name.clone(), service.health());
        }

        Ok(healths)
    }

    /// Get the status of a specific service
    pub async fn status(&self, name: &str) -> Result<ServiceStatus> {
        let services = self.services.read().await;
        let handle = services
            .get(name)
            .ok_or_else(|| crate::error::SystemCoreError::ServiceNotFound(name.to_string()))?;

        let service = handle.read().await;
        Ok(service.status())
    }

    /// Get the health of a specific service
    pub async fn health(&self, name: &str) -> Result<HealthStatus> {
        let services = self.services.read().await;
        let handle = services
            .get(name)
            .ok_or_else(|| crate::error::SystemCoreError::ServiceNotFound(name.to_string()))?;

        let service = handle.read().await;
        Ok(service.health())
    }

    /// List all registered services
    pub async fn list_services(&self) -> Vec<String> {
        let services = self.services.read().await;
        services.keys().cloned().collect()
    }

    /// Get the number of registered services
    pub async fn service_count(&self) -> usize {
        let services = self.services.read().await;
        services.len()
    }

    /// Check if a service is registered
    pub async fn has_service(&self, name: &str) -> bool {
        let services = self.services.read().await;
        services.contains_key(name)
    }

    /// Check if all services are healthy
    pub async fn all_healthy(&self) -> bool {
        let services = self.services.read().await;
        for handle in services.values() {
            let service = handle.read().await;
            if service.health() != HealthStatus::Healthy {
                return false;
            }
        }
        true
    }

    /// Get summary of system health
    pub async fn system_health(&self) -> Result<SystemHealthReport> {
        let services = self.services.read().await;
        let mut healthy = 0;
        let mut warning = 0;
        let mut unhealthy = 0;

        for handle in services.values() {
            let service = handle.read().await;
            match service.health() {
                HealthStatus::Healthy => healthy += 1,
                HealthStatus::Warning => warning += 1,
                HealthStatus::Unhealthy => unhealthy += 1,
            }
        }

        Ok(SystemHealthReport {
            total_services: services.len(),
            healthy_services: healthy,
            warning_services: warning,
            unhealthy_services: unhealthy,
        })
    }
}

impl Default for ServiceManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Summary report of system health across all services
#[derive(Debug, Clone)]
pub struct SystemHealthReport {
    /// Total number of registered services
    pub total_services: usize,

    /// Number of services in healthy state
    pub healthy_services: usize,

    /// Number of services with warnings
    pub warning_services: usize,

    /// Number of services in unhealthy state
    pub unhealthy_services: usize,
}

impl SystemHealthReport {
    /// Check if the overall system is healthy
    pub fn is_healthy(&self) -> bool {
        self.unhealthy_services == 0 && self.warning_services == 0
    }

    /// Check if the overall system is degraded (some warnings)
    pub fn is_degraded(&self) -> bool {
        self.unhealthy_services == 0 && self.warning_services > 0
    }

    /// Get overall health status
    pub fn overall_status(&self) -> HealthStatus {
        if self.unhealthy_services > 0 {
            HealthStatus::Unhealthy
        } else if self.warning_services > 0 {
            HealthStatus::Warning
        } else {
            HealthStatus::Healthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::{HealthStatus, ServiceStatus};

    /// Mock service for testing
    #[derive(Debug)]
    struct MockService {
        name: String,
        status: ServiceStatus,
        health: HealthStatus,
    }

    impl MockService {
        fn new(name: impl Into<String>) -> Self {
            Self {
                name: name.into(),
                status: ServiceStatus::Initializing,
                health: HealthStatus::Unhealthy,
            }
        }
    }

    impl QuantumService for MockService {
        fn initialize(&mut self) -> Result<(), ServiceError> {
            self.status = ServiceStatus::Stopped;
            self.health = HealthStatus::Healthy;
            Ok(())
        }

        fn start(&mut self) -> Result<(), ServiceError> {
            self.status = ServiceStatus::Running;
            Ok(())
        }

        fn stop(&mut self) -> Result<(), ServiceError> {
            self.status = ServiceStatus::Stopped;
            Ok(())
        }

        fn status(&self) -> ServiceStatus {
            self.status
        }

        fn health(&self) -> HealthStatus {
            self.health
        }
    }

    #[tokio::test]
    async fn test_service_manager_creation() {
        let manager = ServiceManager::new();
        assert_eq!(manager.service_count().await, 0);
    }

    #[tokio::test]
    async fn test_register_service() {
        let manager = ServiceManager::new();
        let service = Box::new(MockService::new("test"));
        manager.register("test", service).await.unwrap();

        assert!(manager.has_service("test").await);
        assert_eq!(manager.service_count().await, 1);
    }

    #[tokio::test]
    async fn test_unregister_service() {
        let manager = ServiceManager::new();
        let service = Box::new(MockService::new("test"));
        manager.register("test", service).await.unwrap();
        assert_eq!(manager.service_count().await, 1);

        manager.unregister("test").await.unwrap();
        assert!(!manager.has_service("test").await);
        assert_eq!(manager.service_count().await, 0);
    }

    #[tokio::test]
    async fn test_service_lifecycle() {
        let manager = ServiceManager::new();
        let service = Box::new(MockService::new("test"));
        manager.register("test", service).await.unwrap();

        // Should not be initialized yet
        let status = manager.status("test").await.unwrap();
        assert_eq!(status, ServiceStatus::Initializing);

        // Initialize
        manager.initialize().await.unwrap();
        let status = manager.status("test").await.unwrap();
        assert_eq!(status, ServiceStatus::Stopped);

        // Start
        manager.start_all().await.unwrap();
        let status = manager.status("test").await.unwrap();
        assert_eq!(status, ServiceStatus::Running);

        // Stop
        manager.stop_all().await.unwrap();
        let status = manager.status("test").await.unwrap();
        assert_eq!(status, ServiceStatus::Stopped);
    }

    #[tokio::test]
    async fn test_initialize_without_services() {
        let manager = ServiceManager::new();
        let result = manager.initialize().await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_start_without_initialization() {
        let manager = ServiceManager::new();
        let service = Box::new(MockService::new("test"));
        manager.register("test", service).await.unwrap();

        let result = manager.start_all().await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_status_all() {
        let manager = ServiceManager::new();
        manager
            .register("service1", Box::new(MockService::new("service1")))
            .await
            .unwrap();
        manager
            .register("service2", Box::new(MockService::new("service2")))
            .await
            .unwrap();

        let statuses = manager.status_all().await.unwrap();
        assert_eq!(statuses.len(), 2);
    }

    #[tokio::test]
    async fn test_health_all() {
        let manager = ServiceManager::new();
        manager
            .register("service1", Box::new(MockService::new("service1")))
            .await
            .unwrap();
        manager
            .register("service2", Box::new(MockService::new("service2")))
            .await
            .unwrap();

        let healths = manager.health_all().await.unwrap();
        assert_eq!(healths.len(), 2);
    }

    #[tokio::test]
    async fn test_all_healthy() {
        let manager = ServiceManager::new();
        let service = Box::new(MockService::new("test"));
        manager.register("test", service).await.unwrap();

        manager.initialize().await.unwrap();
        assert!(manager.all_healthy().await);
    }

    #[tokio::test]
    async fn test_system_health_report() {
        let manager = ServiceManager::new();
        manager
            .register("service1", Box::new(MockService::new("service1")))
            .await
            .unwrap();
        manager
            .register("service2", Box::new(MockService::new("service2")))
            .await
            .unwrap();

        manager.initialize().await.unwrap();

        let report = manager.system_health().await.unwrap();
        assert_eq!(report.total_services, 2);
        assert_eq!(report.healthy_services, 2);
        assert_eq!(report.warning_services, 0);
        assert_eq!(report.unhealthy_services, 0);
        assert!(report.is_healthy());
    }

    #[tokio::test]
    async fn test_list_services() {
        let manager = ServiceManager::new();
        manager
            .register("service1", Box::new(MockService::new("service1")))
            .await
            .unwrap();
        manager
            .register("service2", Box::new(MockService::new("service2")))
            .await
            .unwrap();

        let services = manager.list_services().await;
        assert_eq!(services.len(), 2);
        assert!(services.contains(&"service1".to_string()));
        assert!(services.contains(&"service2".to_string()));
    }

    #[tokio::test]
    async fn test_service_not_found() {
        let manager = ServiceManager::new();
        let result = manager.status("nonexistent").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_multiple_services_startup_order() {
        let manager = ServiceManager::new();
        manager
            .register("first", Box::new(MockService::new("first")))
            .await
            .unwrap();
        manager
            .register("second", Box::new(MockService::new("second")))
            .await
            .unwrap();
        manager
            .register("third", Box::new(MockService::new("third")))
            .await
            .unwrap();

        manager.initialize().await.unwrap();
        manager.start_all().await.unwrap();

        let statuses = manager.status_all().await.unwrap();
        for status in statuses.values() {
            assert_eq!(*status, ServiceStatus::Running);
        }
    }
}
