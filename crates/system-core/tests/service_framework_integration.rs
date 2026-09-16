// Integration tests for the service framework
// Located in crates/system-core/tests/

use system_core::{
    AuthService, BrowserService, DashboardService, DeviceService, EnergyService,
    PolicyService, QuantumRuntimeService, SchedulerService, TelemetryService,
    ServiceManager, HealthStatus, ServiceStatus,
};

#[tokio::test]
async fn test_single_service_complete_lifecycle() {
    let manager = ServiceManager::new();

    // Register a service
    let auth_service = Box::new(AuthService::new());
    manager.register("auth", auth_service).await.unwrap();

    // Verify registration
    assert!(manager.has_service("auth").await);
    assert_eq!(manager.service_count().await, 1);

    // Initialize
    manager.initialize().await.unwrap();
    let status = manager.status("auth").await.unwrap();
    assert_eq!(status, ServiceStatus::Stopped);

    // Start
    manager.start_all().await.unwrap();
    let status = manager.status("auth").await.unwrap();
    assert_eq!(status, ServiceStatus::Running);
    let health = manager.health("auth").await.unwrap();
    assert_eq!(health, HealthStatus::Healthy);

    // Stop
    manager.stop_all().await.unwrap();
    let status = manager.status("auth").await.unwrap();
    assert_eq!(status, ServiceStatus::Stopped);
}

#[tokio::test]
async fn test_multiple_services_orchestration() {
    let manager = ServiceManager::new();

    // Register multiple services
    manager.register("auth", Box::new(AuthService::new())).await.unwrap();
    manager.register("policy", Box::new(PolicyService::new())).await.unwrap();
    manager.register("telemetry", Box::new(TelemetryService::new())).await.unwrap();

    assert_eq!(manager.service_count().await, 3);

    // Initialize all
    manager.initialize().await.unwrap();

    // Verify all initialized
    let statuses = manager.status_all().await.unwrap();
    assert_eq!(statuses.len(), 3);
    for status in statuses.values() {
        assert_eq!(*status, ServiceStatus::Stopped);
    }

    // Start all
    manager.start_all().await.unwrap();

    // Verify all running
    let statuses = manager.status_all().await.unwrap();
    for status in statuses.values() {
        assert_eq!(*status, ServiceStatus::Running);
    }

    // Stop all
    manager.stop_all().await.unwrap();

    // Verify all stopped
    let statuses = manager.status_all().await.unwrap();
    for status in statuses.values() {
        assert_eq!(*status, ServiceStatus::Stopped);
    }
}

#[tokio::test]
async fn test_quantum_system_services() {
    let manager = ServiceManager::new();

    // Register quantum subsystem services
    manager.register("quantum", Box::new(QuantumRuntimeService::new())).await.unwrap();
    manager.register("scheduler", Box::new(SchedulerService::new())).await.unwrap();
    manager.register("device", Box::new(DeviceService::new())).await.unwrap();

    manager.initialize().await.unwrap();
    manager.start_all().await.unwrap();

    // Verify quantum services are running
    let services = manager.list_services().await;
    assert!(services.contains(&"quantum".to_string()));
    assert!(services.contains(&"scheduler".to_string()));
    assert!(services.contains(&"device".to_string()));

    manager.stop_all().await.unwrap();
}

#[tokio::test]
async fn test_observability_services() {
    let manager = ServiceManager::new();

    // Register observability services
    manager.register("telemetry", Box::new(TelemetryService::new())).await.unwrap();
    manager.register("energy", Box::new(EnergyService::new())).await.unwrap();
    manager.register("dashboard", Box::new(DashboardService::new())).await.unwrap();

    manager.initialize().await.unwrap();
    manager.start_all().await.unwrap();

    // Verify all are healthy
    let health_report = manager.system_health().await.unwrap();
    assert_eq!(health_report.total_services, 3);
    assert_eq!(health_report.healthy_services, 3);
    assert!(health_report.is_healthy());

    manager.stop_all().await.unwrap();
}

#[tokio::test]
async fn test_full_system_startup() {
    let manager = ServiceManager::new();

    // Register all major services (simulating full system)
    manager.register("auth", Box::new(AuthService::new())).await.unwrap();
    manager.register("policy", Box::new(PolicyService::new())).await.unwrap();
    manager.register("telemetry", Box::new(TelemetryService::new())).await.unwrap();
    manager.register("energy", Box::new(EnergyService::new())).await.unwrap();
    manager.register("quantum", Box::new(QuantumRuntimeService::new())).await.unwrap();
    manager.register("scheduler", Box::new(SchedulerService::new())).await.unwrap();
    manager.register("device", Box::new(DeviceService::new())).await.unwrap();
    manager.register("browser", Box::new(BrowserService::new())).await.unwrap();
    manager.register("dashboard", Box::new(DashboardService::new())).await.unwrap();

    assert_eq!(manager.service_count().await, 9);

    // Initialize all
    manager.initialize().await.unwrap();

    // Start all
    manager.start_all().await.unwrap();

    // Verify system health
    let health_report = manager.system_health().await.unwrap();
    assert_eq!(health_report.total_services, 9);
    assert_eq!(health_report.healthy_services, 9);
    assert_eq!(health_report.warning_services, 0);
    assert_eq!(health_report.unhealthy_services, 0);
    assert!(health_report.is_healthy());

    // Verify all services list
    let services = manager.list_services().await;
    assert_eq!(services.len(), 9);

    // Verify each service status
    for service_name in &["auth", "policy", "telemetry", "energy", "quantum", "scheduler", "device", "browser", "dashboard"] {
        let status = manager.status(service_name).await.unwrap();
        assert_eq!(status, ServiceStatus::Running);
        let health = manager.health(service_name).await.unwrap();
        assert_eq!(health, HealthStatus::Healthy);
    }

    // Shutdown all
    manager.stop_all().await.unwrap();

    // Verify all stopped
    for service_name in &["auth", "policy", "telemetry", "energy", "quantum", "scheduler", "device", "browser", "dashboard"] {
        let status = manager.status(service_name).await.unwrap();
        assert_eq!(status, ServiceStatus::Stopped);
    }
}

#[tokio::test]
async fn test_service_replacement() {
    let manager = ServiceManager::new();

    // Register initial service
    manager.register("auth", Box::new(AuthService::new())).await.unwrap();
    assert_eq!(manager.service_count().await, 1);

    // Replace with new instance
    manager.register("auth", Box::new(AuthService::new())).await.unwrap();
    assert_eq!(manager.service_count().await, 1); // Should still be 1

    manager.unregister("auth").await.unwrap();
    assert_eq!(manager.service_count().await, 0);
}

#[tokio::test]
async fn test_service_error_handling() {
    let manager = ServiceManager::new();

    // Try to start without initialization
    manager.register("auth", Box::new(AuthService::new())).await.unwrap();
    let result = manager.start_all().await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_health_report_generation() {
    let manager = ServiceManager::new();

    // Register multiple services
    manager.register("auth", Box::new(AuthService::new())).await.unwrap();
    manager.register("policy", Box::new(PolicyService::new())).await.unwrap();
    manager.register("quantum", Box::new(QuantumRuntimeService::new())).await.unwrap();

    manager.initialize().await.unwrap();

    // Before starting, services are unhealthy
    let report = manager.system_health().await.unwrap();
    assert_eq!(report.unhealthy_services, 3);
    assert!(!report.is_healthy());

    // After starting, services are healthy
    manager.start_all().await.unwrap();
    let report = manager.system_health().await.unwrap();
    assert_eq!(report.healthy_services, 3);
    assert!(report.is_healthy());

    manager.stop_all().await.unwrap();
}

#[tokio::test]
async fn test_nonexistent_service_queries() {
    let manager = ServiceManager::new();

    // Query nonexistent service
    let result = manager.status("nonexistent").await;
    assert!(result.is_err());

    let result = manager.health("nonexistent").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_service_count_tracking() {
    let manager = ServiceManager::new();

    assert_eq!(manager.service_count().await, 0);

    manager.register("service1", Box::new(AuthService::new())).await.unwrap();
    assert_eq!(manager.service_count().await, 1);

    manager.register("service2", Box::new(PolicyService::new())).await.unwrap();
    assert_eq!(manager.service_count().await, 2);

    manager.register("service3", Box::new(BrowserService::new())).await.unwrap();
    assert_eq!(manager.service_count().await, 3);

    manager.unregister("service1").await.unwrap();
    assert_eq!(manager.service_count().await, 2);

    manager.unregister("service2").await.unwrap();
    assert_eq!(manager.service_count().await, 1);

    manager.unregister("service3").await.unwrap();
    assert_eq!(manager.service_count().await, 0);
}

#[tokio::test]
async fn test_service_all_healthy_flag() {
    let manager = ServiceManager::new();

    // Empty system is not healthy
    assert!(!manager.all_healthy().await);

    manager.register("auth", Box::new(AuthService::new())).await.unwrap();
    manager.initialize().await.unwrap();
    manager.start_all().await.unwrap();

    // After startup, all should be healthy
    assert!(manager.all_healthy().await);

    manager.stop_all().await.unwrap();

    // After stop, no longer all healthy
    assert!(!manager.all_healthy().await);
}

#[tokio::test]
async fn test_concurrent_service_operations() {
    let manager = std::sync::Arc::new(ServiceManager::new());

    // Register services from multiple tasks
    let mut handles = vec![];

    for i in 0..5 {
        let mgr = manager.clone();
        let handle = tokio::spawn(async move {
            let service_name = format!("service-{}", i);
            mgr.register(&service_name, Box::new(AuthService::new())).await.unwrap();
        });
        handles.push(handle);
    }

    // Wait for all registrations
    for handle in handles {
        handle.await.unwrap();
    }

    // Verify all services registered
    assert_eq!(manager.service_count().await, 5);

    // Initialize and start all
    manager.initialize().await.unwrap();
    manager.start_all().await.unwrap();

    // Verify all running
    let health_report = manager.system_health().await.unwrap();
    assert_eq!(health_report.healthy_services, 5);

    manager.stop_all().await.unwrap();
}
