use serde::{Deserialize, Serialize};

use crate::device::{DeviceInfo, DeviceHealth, HardwareDevice};
use crate::error::HardwareError;

/// SPI device information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SpiInfo {
    pub bus: u32,
    pub chip_select: u32,
    pub max_speed_hz: u32,
    pub mode: SpiMode,
    pub bits_per_word: u8,
}

/// SPI bus mode.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SpiMode {
    #[default]
    Mode0,
    Mode1,
    Mode2,
    Mode3,
}

/// SPI device abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SpiDevice {
    info: DeviceInfo,
    spi_info: SpiInfo,
    state: crate::device::DeviceState,
}

impl SpiDevice {
    pub fn new(info: DeviceInfo, spi_info: SpiInfo) -> Self {
        Self {
            info,
            spi_info,
            state: crate::device::DeviceState::Uninitialized,
        }
    }

    /// Get SPI-specific information.
    pub fn spi_info(&self) -> &SpiInfo {
        &self.spi_info
    }

    /// Get the bus number.
    pub fn bus(&self) -> u32 {
        self.spi_info.bus
    }

    /// Get the chip select.
    pub fn chip_select(&self) -> u32 {
        self.spi_info.chip_select
    }

    /// Get the bus identifier.
    pub fn bus_id(&self) -> String {
        format!("spi{}.{}", self.spi_info.bus, self.spi_info.chip_select)
    }
}

impl HardwareDevice for SpiDevice {
    fn identify(&self) -> DeviceInfo {
        self.info.clone()
    }

    fn initialize(&mut self) -> Result<(), HardwareError> {
        self.state = crate::device::DeviceState::Ready;
        Ok(())
    }

    fn health(&self) -> DeviceHealth {
        match self.state {
            crate::device::DeviceState::Ready => DeviceHealth::Healthy,
            crate::device::DeviceState::Error => DeviceHealth::Error,
            _ => DeviceHealth::Unknown,
        }
    }
}

/// I2C device information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct I2cInfo {
    pub bus: u32,
    pub address: u8,
    pub speed_khz: u32,
}

/// I2C device abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct I2cDevice {
    info: DeviceInfo,
    i2c_info: I2cInfo,
    state: crate::device::DeviceState,
}

impl I2cDevice {
    pub fn new(info: DeviceInfo, i2c_info: I2cInfo) -> Self {
        Self {
            info,
            i2c_info,
            state: crate::device::DeviceState::Uninitialized,
        }
    }

    /// Get I2C-specific information.
    pub fn i2c_info(&self) -> &I2cInfo {
        &self.i2c_info
    }

    /// Get the bus number.
    pub fn bus(&self) -> u32 {
        self.i2c_info.bus
    }

    /// Get the device address.
    pub fn address(&self) -> u8 {
        self.i2c_info.address
    }

    /// Get the bus identifier.
    pub fn bus_id(&self) -> String {
        format!("i2c-{}.{}", self.i2c_info.bus, self.i2c_info.address)
    }
}

impl HardwareDevice for I2cDevice {
    fn identify(&self) -> DeviceInfo {
        self.info.clone()
    }

    fn initialize(&mut self) -> Result<(), HardwareError> {
        self.state = crate::device::DeviceState::Ready;
        Ok(())
    }

    fn health(&self) -> DeviceHealth {
        match self.state {
            crate::device::DeviceState::Ready => DeviceHealth::Healthy,
            crate::device::DeviceState::Error => DeviceHealth::Error,
            _ => DeviceHealth::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_spi_device_creation() {
        let info = DeviceInfo {
            id: "spi0".to_string(),
            name: "Temperature Sensor".to_string(),
            vendor: "TestVendor".to_string(),
            model: "TMP100".to_string(),
            serial_number: None,
            firmware_version: None,
        };

        let spi_info = SpiInfo {
            bus: 0,
            chip_select: 0,
            max_speed_hz: 1000000,
            mode: SpiMode::Mode0,
            bits_per_word: 8,
        };

        let device = SpiDevice::new(info, spi_info);
        assert_eq!(device.bus(), 0);
        assert_eq!(device.chip_select(), 0);
        assert_eq!(device.bus_id(), "spi0.0");
    }

    #[test]
    fn test_spi_initialization() {
        let info = DeviceInfo::default();
        let spi_info = SpiInfo::default();
        let mut device = SpiDevice::new(info, spi_info);

        assert!(device.initialize().is_ok());
        assert_eq!(device.health(), DeviceHealth::Healthy);
    }

    #[test]
    fn test_i2c_device_creation() {
        let info = DeviceInfo {
            id: "i2c0".to_string(),
            name: "Power Monitor".to_string(),
            vendor: "TestVendor".to_string(),
            model: "INA219".to_string(),
            serial_number: None,
            firmware_version: None,
        };

        let i2c_info = I2cInfo {
            bus: 1,
            address: 0x40,
            speed_khz: 400,
        };

        let device = I2cDevice::new(info, i2c_info);
        assert_eq!(device.bus(), 1);
        assert_eq!(device.address(), 0x40);
        assert_eq!(device.bus_id(), "i2c-1.64");
    }

    #[test]
    fn test_i2c_initialization() {
        let info = DeviceInfo::default();
        let i2c_info = I2cInfo::default();
        let mut device = I2cDevice::new(info, i2c_info);

        assert!(device.initialize().is_ok());
        assert_eq!(device.health(), DeviceHealth::Healthy);
    }

    #[test]
    fn test_spi_mode() {
        assert_eq!(SpiMode::default(), SpiMode::Mode0);
        assert_ne!(SpiMode::Mode0, SpiMode::Mode3);
    }
}
