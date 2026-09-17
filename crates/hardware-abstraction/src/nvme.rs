use serde::{Deserialize, Serialize};

use crate::device::{DeviceHealth, DeviceInfo, HardwareDevice};
use crate::error::HardwareError;

/// NVMe device information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NvmeInfo {
    pub capacity_bytes: u64,
    pub model_number: String,
    pub firmware_revision: String,
    pub serial_number: String,
    pub namespace_id: u32,
    pub sector_size: u32,
    pub max_transfer_size: u32,
}

/// NVMe health monitoring data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NvmeHealth {
    pub temperature_celsius: f64,
    pub available_spare_percent: u8,
    pub available_spare_threshold: u8,
    pub percentage_used: u8,
    pub data_units_read: u128,
    pub data_units_written: u128,
    pub host_read_commands: u128,
    pub host_write_commands: u128,
    pub power_cycles: u128,
    pub power_on_hours: u128,
    pub unsafe_shutdowns: u128,
    pub media_errors: u128,
    pub error_log_entries: u128,
}

/// NVMe device abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NvmeDevice {
    info: DeviceInfo,
    nvme_info: NvmeInfo,
    health: NvmeHealth,
    state: crate::device::DeviceState,
}

impl NvmeDevice {
    pub fn new(info: DeviceInfo, nvme_info: NvmeInfo) -> Self {
        Self {
            info,
            nvme_info,
            health: NvmeHealth::default(),
            state: crate::device::DeviceState::Uninitialized,
        }
    }

    /// Get NVMe-specific information.
    pub fn nvme_info(&self) -> &NvmeInfo {
        &self.nvme_info
    }

    /// Get NVMe health data.
    pub fn nvme_health(&self) -> &NvmeHealth {
        &self.health
    }

    /// Update NVMe health data.
    pub fn update_health(&mut self, health: NvmeHealth) {
        self.health = health;
    }

    /// Get total capacity in bytes.
    pub fn capacity(&self) -> u64 {
        self.nvme_info.capacity_bytes
    }

    /// Get current temperature.
    pub fn temperature(&self) -> f64 {
        self.health.temperature_celsius
    }

    /// Get available spare percentage.
    pub fn available_spare(&self) -> u8 {
        self.health.available_spare_percent
    }

    /// Get percentage used (wear indicator).
    pub fn percentage_used(&self) -> u8 {
        self.health.percentage_used
    }

    /// Check if the drive is nearing end of life.
    pub fn nearing_eol(&self) -> bool {
        self.health.percentage_used >= 90
            || self.health.available_spare_percent <= self.health.available_spare_threshold
    }

    /// Check if the device is in Ready state.
    pub fn is_ready(&self) -> bool {
        self.state == crate::device::DeviceState::Ready
    }
}

impl HardwareDevice for NvmeDevice {
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
        if self.nearing_eol() {
            DeviceHealth::Warning
        } else if self.health.media_errors > 0 || self.health.error_log_entries > 10 {
            DeviceHealth::Degraded
        } else {
            DeviceHealth::Healthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nvme_device_creation() {
        let info = DeviceInfo {
            id: "nvme0".to_string(),
            name: "Test NVMe".to_string(),
            vendor: "TestVendor".to_string(),
            model: "NVMe5000".to_string(),
            serial_number: Some("S123456".to_string()),
            firmware_version: Some("1.0".to_string()),
        };

        let nvme_info = NvmeInfo {
            capacity_bytes: 1024 * 1024 * 1024 * 1024,
            model_number: "NVMe5000".to_string(),
            firmware_revision: "1.0".to_string(),
            serial_number: "S123456".to_string(),
            namespace_id: 1,
            sector_size: 512,
            max_transfer_size: 131072,
        };

        let device = NvmeDevice::new(info, nvme_info);
        assert_eq!(device.capacity(), 1024 * 1024 * 1024 * 1024);
    }

    #[test]
    fn test_nvme_health_status() {
        let info = DeviceInfo::default();
        let nvme_info = NvmeInfo::default();
        let mut device = NvmeDevice::new(info, nvme_info);

        device.update_health(NvmeHealth {
            temperature_celsius: 45.0,
            available_spare_percent: 100,
            percentage_used: 10,
            ..Default::default()
        });

        assert_eq!(device.health(), DeviceHealth::Healthy);
        assert!(!device.nearing_eol());
    }

    #[test]
    fn test_nvme_nearing_eol() {
        let info = DeviceInfo::default();
        let nvme_info = NvmeInfo::default();
        let mut device = NvmeDevice::new(info, nvme_info);

        device.update_health(NvmeHealth {
            percentage_used: 95,
            ..Default::default()
        });

        assert!(device.nearing_eol());
        assert_eq!(device.health(), DeviceHealth::Warning);
    }

    #[test]
    fn test_nvme_initialization() {
        let info = DeviceInfo::default();
        let nvme_info = NvmeInfo::default();
        let mut device = NvmeDevice::new(info, nvme_info);

        assert!(device.initialize().is_ok());
        assert!(device.is_ready());
    }
}
