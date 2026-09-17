use serde::{Deserialize, Serialize};

/// Information about a hardware device.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub vendor: String,
    pub model: String,
    pub serial_number: Option<String>,
    pub firmware_version: Option<String>,
}

/// Health status of a hardware device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DeviceHealth {
    Healthy,
    Degraded,
    Warning,
    Error,
    #[default]
    Unknown,
}

/// Core trait for all hardware devices in QuantumEnergyOS.
///
/// All hardware access must flow through this trait, providing a consistent
/// interface for device identification, initialization, and health monitoring.
///
/// Implementations must be thread-safe (Send + Sync) as devices may be
/// accessed from multiple system services.
pub trait HardwareDevice: Send + Sync {
    /// Get device information.
    fn identify(&self) -> DeviceInfo;

    /// Initialize the device.
    fn initialize(&mut self) -> Result<(), crate::error::HardwareError>;

    /// Get device health status.
    fn health(&self) -> DeviceHealth;

    /// Check if the device is ready for operation.
    fn is_ready(&self) -> bool {
        matches!(
            self.health(),
            DeviceHealth::Healthy | DeviceHealth::Degraded
        )
    }
}

/// Device capability flags.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub supports_dma: bool,
    pub supports_interrupts: bool,
    pub supports_thermal_monitoring: bool,
    pub supports_power_monitoring: bool,
    pub supports_error_correction: bool,
}

/// Device operational state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DeviceState {
    #[default]
    Uninitialized,
    Initializing,
    Ready,
    Busy,
    Error,
    Suspended,
    Removed,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_device_info_default() {
        let info = DeviceInfo::default();
        assert!(info.id.is_empty());
        assert!(info.name.is_empty());
    }

    #[test]
    fn test_device_health() {
        assert_ne!(DeviceHealth::Healthy, DeviceHealth::Error);
        assert_eq!(DeviceHealth::default(), DeviceHealth::Unknown);
    }

    #[test]
    fn test_device_state() {
        assert_eq!(DeviceState::default(), DeviceState::Uninitialized);
    }

    #[test]
    fn test_device_capabilities() {
        let caps = DeviceCapabilities::default();
        assert!(!caps.supports_dma);
    }
}
