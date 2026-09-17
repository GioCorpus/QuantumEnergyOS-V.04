use serde::{Deserialize, Serialize};

use crate::device::{DeviceHealth, DeviceInfo, HardwareDevice};
use crate::error::HardwareError;

/// PCI device information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PciInfo {
    pub bus: u8,
    pub device: u8,
    pub function: u8,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub revision: u8,
    pub link_speed: String,
    pub link_width: u32,
}

/// PCI device abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PciDevice {
    info: DeviceInfo,
    pci_info: PciInfo,
    state: crate::device::DeviceState,
}

impl PciDevice {
    pub fn new(info: DeviceInfo, pci_info: PciInfo) -> Self {
        Self {
            info,
            pci_info,
            state: crate::device::DeviceState::Uninitialized,
        }
    }

    /// Get PCI-specific information.
    pub fn pci_info(&self) -> &PciInfo {
        &self.pci_info
    }

    /// Get the PCI address as a string (e.g., "00:1f.0").
    pub fn address(&self) -> String {
        format!(
            "{:02x}:{:02x}.{}",
            self.pci_info.bus, self.pci_info.device, self.pci_info.function
        )
    }

    /// Get the device's vendor and device ID.
    pub fn ids(&self) -> (u16, u16) {
        (self.pci_info.vendor_id, self.pci_info.device_id)
    }
}

impl HardwareDevice for PciDevice {
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
    fn test_pci_device_creation() {
        let info = DeviceInfo {
            id: "pci0".to_string(),
            name: "Test PCI Device".to_string(),
            vendor: "TestVendor".to_string(),
            model: "PCI100".to_string(),
            serial_number: None,
            firmware_version: None,
        };

        let pci_info = PciInfo {
            bus: 0,
            device: 31,
            function: 0,
            vendor_id: 0x8086,
            device_id: 0x1234,
            class_code: 6,
            subclass: 1,
            revision: 0,
            link_speed: "8 GT/s".to_string(),
            link_width: 4,
        };

        let device = PciDevice::new(info, pci_info);
        assert_eq!(device.address(), "00:1f.0");
        assert_eq!(device.ids(), (0x8086, 0x1234));
    }

    #[test]
    fn test_pci_initialization() {
        let info = DeviceInfo::default();
        let pci_info = PciInfo::default();
        let mut device = PciDevice::new(info, pci_info);

        assert!(device.initialize().is_ok());
        assert_eq!(device.health(), DeviceHealth::Healthy);
    }
}
