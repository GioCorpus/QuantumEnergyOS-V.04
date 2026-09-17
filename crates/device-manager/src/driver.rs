//! Driver trait, match rules and lifecycle state machine (§4, §5).
//!
//! A device driver in QEOS is decoupled from the kernel core. It registers with the
//! [`crate::manager::DeviceManager`], matches against discovered [`PciDeviceInfo`] records,
//! and receives state transitions through explicit methods:
//!
//! ```text
//!       [Unbound]
//!           │  probe() & bind
//!           ▼
//!        [Bound]
//!           │  initialize()
//!           ▼
//!     [Initialized]
//!        │        ▲
//! start()│        │ stop()
//!        ▼        │
//!      [Running] ─┘
//!           │
//!           ├── reset() ──> [Initialized]
//!           │
//!           └── error ───> [Failed]
//! ```
//!
//! Every transition is validated; illegal transitions return [`crate::error::DriverError::InvalidTransition`].

use serde::{Deserialize, Serialize};

use hardware_abstraction::DeviceHealth;

use crate::error::{DriverError, DriverResult};
use crate::id::DeviceId;
use crate::pci::device::{PciDevice, PciDeviceInfo};

/// Lifecycle state of a driver bound to a device.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DriverState {
    /// Device is discovered but no driver is bound.
    #[default]
    Unbound,
    /// Driver matched and probed successfully, ready for initialization.
    Bound,
    /// Hardware structures and registers initialized.
    Initialized,
    /// Actively processing requests and servicing interrupts.
    Running,
    /// Suspended or stopped cleanly.
    Stopped,
    /// Encountered a hardware or communication failure.
    Failed(String),
}

impl DriverState {
    /// True when actively running.
    pub const fn is_running(&self) -> bool {
        matches!(self, DriverState::Running)
    }

    /// True when bound to a driver.
    pub const fn is_bound(&self) -> bool {
        !matches!(self, DriverState::Unbound)
    }

    /// True when in a failed state.
    pub const fn is_failed(&self) -> bool {
        matches!(self, DriverState::Failed(_))
    }

    /// Stable label used in snapshots and logs.
    pub fn label(&self) -> &'static str {
        match self {
            DriverState::Unbound => "unbound",
            DriverState::Bound => "bound",
            DriverState::Initialized => "initialized",
            DriverState::Running => "running",
            DriverState::Stopped => "stopped",
            DriverState::Failed(_) => "failed",
        }
    }

    /// Validates a requested state transition, returning [`DriverError::InvalidTransition`]
    /// if the transition is illegal.
    pub fn check_transition(device: DeviceId, from: &Self, to: &Self) -> DriverResult<()> {
        let legal = match (from, to) {
            (DriverState::Unbound, DriverState::Bound) => true,
            (DriverState::Bound, DriverState::Initialized) => true,
            (DriverState::Bound, DriverState::Unbound) => true,
            (DriverState::Initialized, DriverState::Running) => true,
            (DriverState::Initialized, DriverState::Unbound) => true,
            (DriverState::Running, DriverState::Stopped) => true,
            (DriverState::Running, DriverState::Initialized) => true, // via reset
            (DriverState::Stopped, DriverState::Running) => true,
            (DriverState::Stopped, DriverState::Initialized) => true,
            (DriverState::Stopped, DriverState::Unbound) => true,
            (_, DriverState::Failed(_)) => true, // Any state can transition to Failed on error
            (DriverState::Failed(_), DriverState::Initialized) => true, // after reset/recovery
            (DriverState::Failed(_), DriverState::Unbound) => true,
            _ => false,
        };

        if legal {
            Ok(())
        } else {
            Err(DriverError::InvalidTransition {
                device,
                from: from.clone(),
                to: to.clone(),
            })
        }
    }
}

impl std::fmt::Display for DriverState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DriverState::Failed(reason) => write!(f, "failed({reason})"),
            other => f.write_str(other.label()),
        }
    }
}

/// Match criteria used by a driver to declare which PCI devices it supports.
#[derive(Debug, Clone)]
pub enum DriverMatch {
    /// Matches any device (generic/diagnostic drivers).
    Any,
    /// Matches exact vendor and device ID.
    VendorProduct { vendor_id: u16, device_id: u16 },
    /// Matches PCI base class and sub-class.
    Class { base: u8, subclass: u8 },
    /// Matches by vendor ID only.
    Vendor(u16),
}

impl DriverMatch {
    /// Tests if a discovered device matches these criteria.
    pub fn matches(&self, dev: &PciDeviceInfo) -> bool {
        match self {
            DriverMatch::Any => true,
            DriverMatch::VendorProduct {
                vendor_id,
                device_id,
            } => dev.vendor_id == *vendor_id && dev.device_id == *device_id,
            DriverMatch::Class { base, subclass } => {
                dev.class.base == *base && dev.class.subclass == *subclass
            }
            DriverMatch::Vendor(vendor) => dev.vendor_id == *vendor,
        }
    }
}

/// Interface that all device drivers must implement.
///
/// Drivers must be `Send + Sync` so they can be registered with the central device manager
/// and safely accessed across worker threads.
pub trait DeviceDriver: Send + Sync {
    /// Unique human-readable name of the driver (e.g. `qeos-nvme-driver`).
    fn name(&self) -> &'static str;

    /// Checks if this driver claims the given device.
    fn matches(&self, device: &PciDeviceInfo) -> bool;

    /// Probes the device to verify hardware presence and readiness.
    fn probe(&mut self, device: &PciDevice) -> DriverResult<()>;

    /// Initializes hardware registers, BAR buffers, and descriptor queues.
    fn initialize(&mut self, device: &PciDevice) -> DriverResult<()>;

    /// Starts device operation and enables event/interrupt processing.
    fn start(&mut self, device: &PciDevice) -> DriverResult<()>;

    /// Stops device operation cleanly.
    fn stop(&mut self, device: &PciDevice) -> DriverResult<()>;

    /// Resets device state to default initialized condition.
    fn reset(&mut self, device: &PciDevice) -> DriverResult<()>;

    /// Reports current device health.
    fn health(&self) -> DeviceHealth;
}

// =========================================================================
// STANDARD BUILT-IN SIMULATED DRIVERS
// =========================================================================

/// Null / generic driver used for testing and unmanaged devices.
#[derive(Debug, Default)]
pub struct NullDriver {
    name: &'static str,
    health: DeviceHealth,
}

impl NullDriver {
    /// Creates a null driver with a custom name.
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            health: DeviceHealth::Healthy,
        }
    }
}

impl DeviceDriver for NullDriver {
    fn name(&self) -> &'static str {
        self.name
    }

    fn matches(&self, _device: &PciDeviceInfo) -> bool {
        true
    }

    fn probe(&mut self, _device: &PciDevice) -> DriverResult<()> {
        Ok(())
    }

    fn initialize(&mut self, _device: &PciDevice) -> DriverResult<()> {
        Ok(())
    }

    fn start(&mut self, _device: &PciDevice) -> DriverResult<()> {
        Ok(())
    }

    fn stop(&mut self, _device: &PciDevice) -> DriverResult<()> {
        Ok(())
    }

    fn reset(&mut self, _device: &PciDevice) -> DriverResult<()> {
        Ok(())
    }

    fn health(&self) -> DeviceHealth {
        self.health
    }
}

/// Simulated NVMe storage controller driver.
#[derive(Debug, Default)]
pub struct SimulatedNvmeDriver {
    health: DeviceHealth,
    initialized: bool,
    running: bool,
}

impl SimulatedNvmeDriver {
    /// Creates a new NVMe driver instance.
    pub fn new() -> Self {
        Self {
            health: DeviceHealth::Healthy,
            initialized: false,
            running: false,
        }
    }
}

impl DeviceDriver for SimulatedNvmeDriver {
    fn name(&self) -> &'static str {
        "qeos-nvme-driver"
    }

    fn matches(&self, device: &PciDeviceInfo) -> bool {
        device.class.is_storage() && device.class.subclass == 0x08 // NVMe subclass
    }

    fn probe(&mut self, device: &PciDevice) -> DriverResult<()> {
        if !device.bars().iter().any(|b| b.is_mappable()) {
            return Err(DriverError::ProbeFailed {
                driver: self.name().to_string(),
                device: DeviceId::from_raw(device.address().config_key() as u32),
                reason: "NVMe device has no mappable BARs".to_string(),
            });
        }
        Ok(())
    }

    fn initialize(&mut self, _device: &PciDevice) -> DriverResult<()> {
        self.initialized = true;
        Ok(())
    }

    fn start(&mut self, _device: &PciDevice) -> DriverResult<()> {
        if !self.initialized {
            return Err(DriverError::StartFailed {
                driver: self.name().to_string(),
                device: DeviceId::from_raw(0),
                reason: "driver not initialized".to_string(),
            });
        }
        self.running = true;
        Ok(())
    }

    fn stop(&mut self, _device: &PciDevice) -> DriverResult<()> {
        self.running = false;
        Ok(())
    }

    fn reset(&mut self, _device: &PciDevice) -> DriverResult<()> {
        self.running = false;
        self.initialized = true;
        Ok(())
    }

    fn health(&self) -> DeviceHealth {
        self.health
    }
}

/// Simulated GPU / display controller driver.
#[derive(Debug, Default)]
pub struct SimulatedGpuDriver {
    health: DeviceHealth,
    running: bool,
}

impl SimulatedGpuDriver {
    /// Creates a new GPU driver instance.
    pub fn new() -> Self {
        Self {
            health: DeviceHealth::Healthy,
            running: false,
        }
    }
}

impl DeviceDriver for SimulatedGpuDriver {
    fn name(&self) -> &'static str {
        "qeos-gpu-driver"
    }

    fn matches(&self, device: &PciDeviceInfo) -> bool {
        device.class.is_display()
    }

    fn probe(&mut self, _device: &PciDevice) -> DriverResult<()> {
        Ok(())
    }

    fn initialize(&mut self, _device: &PciDevice) -> DriverResult<()> {
        Ok(())
    }

    fn start(&mut self, _device: &PciDevice) -> DriverResult<()> {
        self.running = true;
        Ok(())
    }

    fn stop(&mut self, _device: &PciDevice) -> DriverResult<()> {
        self.running = false;
        Ok(())
    }

    fn reset(&mut self, _device: &PciDevice) -> DriverResult<()> {
        self.running = false;
        Ok(())
    }

    fn health(&self) -> DeviceHealth {
        self.health
    }
}

/// Simulated energy telemetry driver.
#[derive(Debug, Default)]
pub struct SimulatedTelemetryDriver {
    health: DeviceHealth,
    running: bool,
}

impl SimulatedTelemetryDriver {
    /// Creates a new telemetry driver instance.
    pub fn new() -> Self {
        Self {
            health: DeviceHealth::Healthy,
            running: false,
        }
    }
}

impl DeviceDriver for SimulatedTelemetryDriver {
    fn name(&self) -> &'static str {
        "qeos-telemetry-driver"
    }

    fn matches(&self, device: &PciDeviceInfo) -> bool {
        device.vendor_id == 0x51E0 || (device.class.base == 0x11 && device.class.subclass == 0x80)
    }

    fn probe(&mut self, _device: &PciDevice) -> DriverResult<()> {
        Ok(())
    }

    fn initialize(&mut self, _device: &PciDevice) -> DriverResult<()> {
        Ok(())
    }

    fn start(&mut self, _device: &PciDevice) -> DriverResult<()> {
        self.running = true;
        Ok(())
    }

    fn stop(&mut self, _device: &PciDevice) -> DriverResult<()> {
        self.running = false;
        Ok(())
    }

    fn reset(&mut self, _device: &PciDevice) -> DriverResult<()> {
        self.running = false;
        Ok(())
    }

    fn health(&self) -> DeviceHealth {
        self.health
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pci::address::{PciAddress, PciClass};

    #[test]
    fn driver_state_transitions() {
        let dev = DeviceId::from_raw(1);

        assert!(
            DriverState::check_transition(dev, &DriverState::Unbound, &DriverState::Bound).is_ok()
        );
        assert!(
            DriverState::check_transition(dev, &DriverState::Bound, &DriverState::Initialized)
                .is_ok()
        );
        assert!(DriverState::check_transition(
            dev,
            &DriverState::Initialized,
            &DriverState::Running
        )
        .is_ok());
        assert!(
            DriverState::check_transition(dev, &DriverState::Running, &DriverState::Stopped)
                .is_ok()
        );
        assert!(
            DriverState::check_transition(dev, &DriverState::Stopped, &DriverState::Running)
                .is_ok()
        );

        // Illegal transitions
        assert!(
            DriverState::check_transition(dev, &DriverState::Unbound, &DriverState::Running)
                .is_err()
        );
        assert!(
            DriverState::check_transition(dev, &DriverState::Bound, &DriverState::Running).is_err()
        );
    }

    #[test]
    fn driver_matches_correctly() {
        let nvme_info = PciDeviceInfo::new(
            PciAddress::new(0, 1, 0),
            0x1B36,
            0x0010,
            PciClass::new(0x01, 0x08, 0x02),
        );
        let gpu_info = PciDeviceInfo::new(
            PciAddress::new(0, 2, 0),
            0x10DE,
            0x1EB8,
            PciClass::new(0x03, 0x00, 0x00),
        );

        let nvme_driver = SimulatedNvmeDriver::new();
        assert!(nvme_driver.matches(&nvme_info));
        assert!(!nvme_driver.matches(&gpu_info));

        let gpu_driver = SimulatedGpuDriver::new();
        assert!(gpu_driver.matches(&gpu_info));
        assert!(!gpu_driver.matches(&nvme_info));
    }
}
