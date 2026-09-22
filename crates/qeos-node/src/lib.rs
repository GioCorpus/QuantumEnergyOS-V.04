//! # QEOS Node Operations Platform (Phase 7.2)
//!
//! The node-level operational foundation of QEOS V.04. This crate exposes the
//! production operations model, normalized node health, hardware discovery,
//! capability inventory and inventory persistence.
//!
//! ## Reality classification
//!
//! - [`lifecycle`] — **REAL** state machine (host-testable).
//! - [`health`] — **REAL** normalization model; values are only present when a
//!   measurement/reading exists, otherwise the field is `None`.
//! - [`discovery::HostDiscoverySource`] — **REAL** only for CPU parallelism;
//!   everything else is `UNAVAILABLE` from the generic host source.
//! - [`discovery::SimulatedDiscoverySource`] — **SIMULATED** (tests/CI only).
//! - [`inventory`] — **REAL** persistence and change detection.

#![forbid(unsafe_code)]

pub mod device;
pub mod device_lifecycle;
pub mod discovery;
pub mod error;
pub mod handles;
pub mod health;
pub mod inventory;
pub mod lifecycle;
pub mod node;

pub use device::{
    Device, DeviceCapabilities, DeviceClass, DeviceIdentity, DeviceOperationalState,
    DeviceTelemetry, DeviceTopology,
};
pub use device_lifecycle::{DeviceLifecycle, DeviceState};
pub use discovery::{
    DeviceClassTag, DiscoveryEntry, DiscoveryError, HardwareSource, HostDiscoverySource,
    SimulatedDiscoverySource,
};
pub use error::{NodeError, Result};
pub use handles::{DeviceHandle, HandleCapability, HandleManager};
pub use health::{
    ComponentHealth, EnergyHealth, HealthStatus, MeasurementSource, NodeHealth,
    ServiceHealthSummary,
};
pub use inventory::{
    reconcile, HardwareInventory, InventoryChange, InventoryRecord, InventoryStore,
    JsonFileInventoryStore, MemoryInventoryStore,
};
pub use lifecycle::{
    Backoff, MaintenanceMode, NodeLifecycleState, NodeRuntime, RestartPolicy,
    ServiceLifecycleState, ServiceSpec, ServiceStatus,
};

/// QEOS node crate version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
