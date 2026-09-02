use serde::{Deserialize, Serialize};
use crate::error::ServiceError;

/// Current operational state of a service
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    /// Service is in initialization phase
    Initializing,
    /// Service is running normally
    Running,
    /// Service is stopped
    Stopped,
    /// Service is running but in a degraded state
    Degraded,
}

/// Health status assessment of a service
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    /// Service is healthy and operational
    Healthy,
    /// Service is operational but with warnings
    Warning,
    /// Service is unhealthy or non-operational
    Unhealthy,
}

/// Common interface for all quantum services in QuantumEnergyOS
///
/// This trait defines the lifecycle and health monitoring interface
/// that all services must implement. Services are initialized, started,
/// stopped, and monitored through this common interface.
///
/// # Example
///
/// ```ignore
/// struct MyQuantumService {
///     status: ServiceStatus,
/// }
///
/// impl QuantumService for MyQuantumService {
///     fn initialize(&mut self) -> Result<(), ServiceError> {
///         // Initialize service resources
///         Ok(())
///     }
///
///     fn start(&mut self) -> Result<(), ServiceError> {
///         self.status = ServiceStatus::Running;
///         Ok(())
///     }
///
///     fn stop(&mut self) -> Result<(), ServiceError> {
///         self.status = ServiceStatus::Stopped;
///         Ok(())
///     }
///
///     fn status(&self) -> ServiceStatus {
///         self.status
///     }
///
///     fn health(&self) -> HealthStatus {
///         match self.status {
///             ServiceStatus::Running => HealthStatus::Healthy,
///             ServiceStatus::Degraded => HealthStatus::Warning,
///             _ => HealthStatus::Unhealthy,
///         }
///     }
/// }
/// ```
pub trait QuantumService {
    /// Initialize the service and prepare resources for startup
    fn initialize(&mut self) -> Result<(), ServiceError>;

    /// Start the service and begin operation
    fn start(&mut self) -> Result<(), ServiceError>;

    /// Stop the service and clean up resources
    fn stop(&mut self) -> Result<(), ServiceError>;

    /// Get the current operational status of the service
    fn status(&self) -> ServiceStatus;

    /// Get the current health assessment of the service
    fn health(&self) -> HealthStatus;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_service_status_ordering() {
        let statuses = vec![
            ServiceStatus::Initializing,
            ServiceStatus::Running,
            ServiceStatus::Stopped,
            ServiceStatus::Degraded,
        ];
        assert_eq!(statuses.len(), 4);
    }

    #[test]
    fn test_health_status_values() {
        let statuses = vec![
            HealthStatus::Healthy,
            HealthStatus::Warning,
            HealthStatus::Unhealthy,
        ];
        assert_eq!(statuses.len(), 3);
    }

    #[test]
    fn test_status_serialization() {
        let status = ServiceStatus::Running;
        let json = serde_json::to_string(&status).unwrap();
        assert_eq!(json, "\"Running\"");
    }

    #[test]
    fn test_health_status_serialization() {
        let health = HealthStatus::Healthy;
        let json = serde_json::to_string(&health).unwrap();
        assert_eq!(json, "\"Healthy\"");
    }
}

