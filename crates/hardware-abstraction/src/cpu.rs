use serde::{Deserialize, Serialize};

use crate::device::{DeviceHealth, DeviceInfo, HardwareDevice};
use crate::error::HardwareError;

/// CPU device information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuInfo {
    pub core_count: u32,
    pub thread_count: u32,
    pub base_frequency_mhz: u64,
    pub max_frequency_mhz: u64,
    pub architecture: String,
    pub features: Vec<String>,
}

/// CPU health monitoring data.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CpuHealth {
    pub temperature_celsius: f64,
    pub utilization_percent: f64,
    pub power_watts: f64,
    pub throttling: bool,
}

/// CPU device abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuDevice {
    info: DeviceInfo,
    cpu_info: CpuInfo,
    health: CpuHealth,
    state: crate::device::DeviceState,
}

impl CpuDevice {
    pub fn new(info: DeviceInfo, cpu_info: CpuInfo) -> Self {
        Self {
            info,
            cpu_info,
            health: CpuHealth::default(),
            state: crate::device::DeviceState::Uninitialized,
        }
    }

    /// Get CPU-specific information.
    pub fn cpu_info(&self) -> &CpuInfo {
        &self.cpu_info
    }

    /// Get CPU health data.
    pub fn cpu_health(&self) -> &CpuHealth {
        &self.health
    }

    /// Update CPU health data.
    pub fn update_health(&mut self, health: CpuHealth) {
        self.health = health;
    }

    /// Get current CPU temperature.
    pub fn temperature(&self) -> f64 {
        self.health.temperature_celsius
    }

    /// Get current CPU utilization.
    pub fn utilization(&self) -> f64 {
        self.health.utilization_percent
    }

    /// Check if CPU is throttling.
    pub fn is_throttling(&self) -> bool {
        self.health.throttling
    }

    /// Get the number of cores.
    pub fn core_count(&self) -> u32 {
        self.cpu_info.core_count
    }

    /// Get the number of threads.
    pub fn thread_count(&self) -> u32 {
        self.cpu_info.thread_count
    }
}

impl HardwareDevice for CpuDevice {
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
        if self.health.temperature_celsius > 95.0 || self.health.throttling {
            DeviceHealth::Warning
        } else if self.health.temperature_celsius > 85.0 {
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
    fn test_cpu_device_creation() {
        let info = DeviceInfo {
            id: "cpu0".to_string(),
            name: "Test CPU".to_string(),
            vendor: "TestVendor".to_string(),
            model: "X1000".to_string(),
            serial_number: None,
            firmware_version: None,
        };

        let cpu_info = CpuInfo {
            core_count: 8,
            thread_count: 16,
            base_frequency_mhz: 2400,
            max_frequency_mhz: 5000,
            architecture: "x86_64".to_string(),
            features: vec!["sse".to_string(), "avx".to_string()],
        };

        let device = CpuDevice::new(info, cpu_info);
        assert_eq!(device.core_count(), 8);
        assert_eq!(device.thread_count(), 16);
    }

    #[test]
    fn test_cpu_health_update() {
        let info = DeviceInfo::default();
        let cpu_info = CpuInfo::default();
        let mut device = CpuDevice::new(info, cpu_info);

        device.update_health(CpuHealth {
            temperature_celsius: 75.0,
            utilization_percent: 50.0,
            power_watts: 95.0,
            throttling: false,
        });

        assert!((device.temperature() - 75.0).abs() < 1e-10);
        assert!((device.utilization() - 50.0).abs() < 1e-10);
    }

    #[test]
    fn test_cpu_health_status() {
        let info = DeviceInfo::default();
        let cpu_info = CpuInfo::default();
        let mut device = CpuDevice::new(info, cpu_info);

        device.update_health(CpuHealth {
            temperature_celsius: 60.0,
            ..Default::default()
        });
        assert_eq!(device.health(), DeviceHealth::Healthy);

        device.update_health(CpuHealth {
            temperature_celsius: 90.0,
            ..Default::default()
        });
        assert_eq!(device.health(), DeviceHealth::Degraded);

        device.update_health(CpuHealth {
            temperature_celsius: 98.0,
            ..Default::default()
        });
        assert_eq!(device.health(), DeviceHealth::Warning);
    }

    #[test]
    fn test_cpu_initialization() {
        let info = DeviceInfo::default();
        let cpu_info = CpuInfo::default();
        let mut device = CpuDevice::new(info, cpu_info);

        assert!(device.initialize().is_ok());
        assert!(device.is_ready());
    }
}
