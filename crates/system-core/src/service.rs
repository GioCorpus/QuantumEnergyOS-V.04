use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceStatus {
    Initializing,
    Running,
    Stopped,
    Degraded,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Warning,
    Unhealthy,
}

#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("service initialization failed")]
    InitializationFailed,
    #[error("service startup failed")]
    StartFailed,
    #[error("service shutdown failed")]
    StopFailed,
}

pub trait QuantumService {
    fn initialize(&mut self) -> Result<(), ServiceError>;
    fn start(&mut self) -> Result<(), ServiceError>;
    fn stop(&mut self) -> Result<(), ServiceError>;
    fn status(&self) -> ServiceStatus;
    fn health(&self) -> HealthStatus;
}
