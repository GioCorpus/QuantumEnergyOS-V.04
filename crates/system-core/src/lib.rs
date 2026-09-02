pub mod error;
pub mod service;
pub mod service_bus;
pub mod manager;
pub mod services;

pub use error::{
    DatabaseError, IpcError, QuantumError, ServiceError, SystemCoreError, Result,
};
pub use service::{HealthStatus, QuantumService, ServiceStatus};
pub use service_bus::{Message, MessageBuffer, ServiceInfo, ServiceRegistry, IPC_PROTOCOL_VERSION};
pub use manager::{ServiceManager, SystemHealthReport};
pub use services::{
    AuthService, BrowserService, DashboardService, DeviceService, EnergyService,
    PolicyService, QuantumRuntimeService, SchedulerService, TelemetryService,
};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
