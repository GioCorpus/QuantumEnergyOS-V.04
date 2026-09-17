pub mod error;
pub mod manager;
pub mod service;
pub mod service_bus;
pub mod service_gateway;
pub mod services;

pub use error::{DatabaseError, IpcError, QuantumError, Result, ServiceError, SystemCoreError};
pub use manager::{ServiceManager, SystemHealthReport};
pub use service::{HealthStatus, QuantumService, ServiceStatus};
pub use service_bus::{Message, MessageBuffer, ServiceInfo, ServiceRegistry, IPC_PROTOCOL_VERSION};
pub use service_gateway::{
    AllowAllPolicy, RequireAuthPolicy, ServiceAccessPolicy, ServiceGateway, ServiceRateLimiter,
};
pub use services::{
    AuthService, BrowserService, DashboardService, DeviceService, EnergyService, PolicyService,
    QuantumRuntimeService, SchedulerService, TelemetryService,
};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
