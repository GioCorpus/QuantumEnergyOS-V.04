use serde::{Deserialize, Serialize};

use crate::device::{DeviceInfo, DeviceHealth, HardwareDevice};
use crate::error::HardwareError;

/// GPU device information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GpuInfo {
    pub vram_mb: u64,
    pub core_count: u32,
    pub base_clock_mhz: u64,
    pub boost_clock_mhz: u64,
    pub pcie_lanes: u32,
    pub driver_version: String,
}

/// GPU health monitoring data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GpuHealth {
    pub temperature_celsius: f64,
    pub hotspot_temperature_celsius: f64,
    pub utilization_percent: f64,
    pub vram_usage_mb: u64,
    pub power_watts: f64,
    pub fan_speed_percent: f64,
    pub throttling: bool,
}

/// GPU device abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuDevice {
    info: DeviceInfo,
    gpu_info: GpuInfo,
    health: GpuHealth,
    state: crate::device::DeviceState,
}

impl GpuDevice {
    pub fn new(info: DeviceInfo, gpu_info: GpuInfo) -> Self {
        Self {
            info,
            gpu_info,
            health: GpuHealth::default(),
            state: crate::device::DeviceState::Uninitialized,
        }
    }

    /// Get GPU-specific information.
    pub fn gpu_info(&self) -> &GpuInfo {
        &self.gpu_info
    }

    /// Get GPU health data.
    pub fn gpu_health(&self) -> &GpuHealth {
        &self.health
    }

    /// Update GPU health data.
    pub fn update_health(&mut self, health: GpuHealth) {
        self.health = health;
    }

    /// Get current GPU temperature.
    pub fn temperature(&self) -> f64 {
        self.health.temperature_celsius
    }

    /// Get current GPU utilization.
    pub fn utilization(&self) -> f64 {
        self.health.utilization_percent
    }

    /// Get current VRAM usage.
    pub fn vram_usage(&self) -> u64 {
        self.health.vram_usage_mb
    }

    /// Check if GPU is throttling.
    pub fn is_throttling(&self) -> bool {
        self.health.throttling
    }

    /// Get total VRAM.
    pub fn vram_total(&self) -> u64 {
        self.gpu_info.vram_mb
    }
}

impl HardwareDevice for GpuDevice {
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
        if self.health.temperature_celsius > 90.0
            || self.health.hotspot_temperature_celsius > 100.0
            || self.health.throttling
        {
            DeviceHealth::Warning
        } else if self.health.temperature_celsius > 80.0 {
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
    fn test_gpu_device_creation() {
        let info = DeviceInfo {
            id: "gpu0".to_string(),
            name: "Test GPU".to_string(),
            vendor: "TestVendor".to_string(),
            model: "RTX9000".to_string(),
            serial_number: None,
            firmware_version: None,
        };

        let gpu_info = GpuInfo {
            vram_mb: 24576,
            core_count: 10240,
            base_clock_mhz: 1410,
            boost_clock_mhz: 1770,
            pcie_lanes: 16,
            driver_version: "535.0".to_string(),
        };

        let device = GpuDevice::new(info, gpu_info);
        assert_eq!(device.vram_total(), 24576);
    }

    #[test]
    fn test_gpu_health_update() {
        let info = DeviceInfo::default();
        let gpu_info = GpuInfo::default();
        let mut device = GpuDevice::new(info, gpu_info);

        device.update_health(GpuHealth {
            temperature_celsius: 70.0,
            utilization_percent: 80.0,
            vram_usage_mb: 8192,
            power_watts: 250.0,
            fan_speed_percent: 60.0,
            ..Default::default()
        });

        assert!((device.temperature() - 70.0).abs() < 1e-10);
        assert!((device.utilization() - 80.0).abs() < 1e-10);
        assert_eq!(device.vram_usage(), 8192);
    }

    #[test]
    fn test_gpu_health_status() {
        let info = DeviceInfo::default();
        let gpu_info = GpuInfo::default();
        let mut device = GpuDevice::new(info, gpu_info);

        device.update_health(GpuHealth {
            temperature_celsius: 65.0,
            ..Default::default()
        });
        assert_eq!(device.health(), DeviceHealth::Healthy);

        device.update_health(GpuHealth {
            temperature_celsius: 85.0,
            ..Default::default()
        });
        assert_eq!(device.health(), DeviceHealth::Degraded);

        device.update_health(GpuHealth {
            temperature_celsius: 95.0,
            ..Default::default()
        });
        assert_eq!(device.health(), DeviceHealth::Warning);
    }

    #[test]
    fn test_gpu_initialization() {
        let info = DeviceInfo::default();
        let gpu_info = GpuInfo::default();
        let mut device = GpuDevice::new(info, gpu_info);

        assert!(device.initialize().is_ok());
        assert!(device.is_ready());
    }
}
