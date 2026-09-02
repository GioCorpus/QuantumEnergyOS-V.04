use serde::{Deserialize, Serialize};

use crate::device::{DeviceInfo, DeviceHealth, HardwareDevice};
use crate::error::HardwareError;

/// Power monitoring device information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PowerInfo {
    pub channels: u32,
    pub max_voltage: f64,
    pub max_current: f64,
    pub max_power: f64,
    pub measurement_resolution: f64,
}

/// Power monitoring health data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PowerHealth {
    pub input_voltage: f64,
    pub input_current: f64,
    pub input_power: f64,
    pub temperature_celsius: f64,
    pub overcurrent: bool,
    pub overvoltage: bool,
    pub undervoltage: bool,
}

/// Power monitoring device abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PowerDevice {
    info: DeviceInfo,
    power_info: PowerInfo,
    health: PowerHealth,
    state: crate::device::DeviceState,
}

impl PowerDevice {
    pub fn new(info: DeviceInfo, power_info: PowerInfo) -> Self {
        Self {
            info,
            power_info,
            health: PowerHealth::default(),
            state: crate::device::DeviceState::Uninitialized,
        }
    }

    /// Get power-specific information.
    pub fn power_info(&self) -> &PowerInfo {
        &self.power_info
    }

    /// Get power health data.
    pub fn power_health(&self) -> &PowerHealth {
        &self.health
    }

    /// Update power health data.
    pub fn update_health(&mut self, health: PowerHealth) {
        self.health = health;
    }

    /// Get current input voltage.
    pub fn voltage(&self) -> f64 {
        self.health.input_voltage
    }

    /// Get current input current.
    pub fn current(&self) -> f64 {
        self.health.input_current
    }

    /// Get current input power.
    pub fn power(&self) -> f64 {
        self.health.input_power
    }

    /// Check if any fault condition is present.
    pub fn has_fault(&self) -> bool {
        self.health.overcurrent || self.health.overvoltage || self.health.undervoltage
    }

    /// Get the number of channels.
    pub fn channels(&self) -> u32 {
        self.power_info.channels
    }
}

impl HardwareDevice for PowerDevice {
    fn identify(&self) -> DeviceInfo {
        self.info.clone()
    }

    fn initialize(&mut self) -> Result<(), HardwareError> {
        if self.state == crate::device::DeviceState::Ready {
            return Ok(());
        }
        self.state = crate::device::DeviceState::Ready;
        Ok(())
    }

    fn health(&self) -> DeviceHealth {
        if self.has_fault() {
            DeviceHealth::Error
        } else if self.health.temperature_celsius > 70.0 {
            DeviceHealth::Warning
        } else {
            DeviceHealth::Healthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_power_device_creation() {
        let info = DeviceInfo {
            id: "power0".to_string(),
            name: "Power Monitor".to_string(),
            vendor: "TestVendor".to_string(),
            model: "INA219".to_string(),
            serial_number: None,
            firmware_version: None,
        };

        let power_info = PowerInfo {
            channels: 1,
            max_voltage: 26.0,
            max_current: 3.2,
            max_power: 83.2,
            measurement_resolution: 0.001,
        };

        let device = PowerDevice::new(info, power_info);
        assert_eq!(device.channels(), 1);
    }

    #[test]
    fn test_power_health_update() {
        let info = DeviceInfo::default();
        let power_info = PowerInfo::default();
        let mut device = PowerDevice::new(info, power_info);

        device.update_health(PowerHealth {
            input_voltage: 12.0,
            input_current: 2.5,
            input_power: 30.0,
            temperature_celsius: 45.0,
            ..Default::default()
        });

        assert!((device.voltage() - 12.0).abs() < 1e-10);
        assert!((device.current() - 2.5).abs() < 1e-10);
        assert!((device.power() - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_power_fault_detection() {
        let info = DeviceInfo::default();
        let power_info = PowerInfo::default();
        let mut device = PowerDevice::new(info, power_info);

        device.update_health(PowerHealth {
            overcurrent: true,
            ..Default::default()
        });

        assert!(device.has_fault());
        assert_eq!(device.health(), DeviceHealth::Error);
    }

    #[test]
    fn test_power_initialization() {
        let info = DeviceInfo::default();
        let power_info = PowerInfo::default();
        let mut device = PowerDevice::new(info, power_info);

        assert!(device.initialize().is_ok());
        assert!(device.is_ready());
    }
}
