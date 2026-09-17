//! PCI/PCIe addressing, class codes and header types (§5).
//!
//! All values here are decoded from configuration space or from the address
//! itself; nothing is guessed. Unknown encodings are reported explicitly
//! (`Unknown(..)`, `"unknown"`) instead of being mapped to a plausible device.

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::error::{PciError, PciResult};

/// Address of one PCI function: `bus:device.function`.
///
/// The PCIe segment/domain is not modelled yet; the host model uses a single
/// domain (rendered as `0000`). Multi-segment systems are FUTURE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct PciAddress {
    /// Bus number (0..=255).
    pub bus: u8,
    /// Device number (0..=31).
    pub device: u8,
    /// Function number (0..=7).
    pub function: u8,
}

impl PciAddress {
    /// Highest legal device number on a bus.
    pub const MAX_DEVICE: u8 = 31;
    /// Highest legal function number on a device.
    pub const MAX_FUNCTION: u8 = 7;
    /// Number of functions a multifunction device may expose.
    pub const FUNCTION_COUNT: u8 = 8;
    /// Number of devices on a bus.
    pub const DEVICE_COUNT: u8 = 32;

    /// Builds an address. Validity is checked by [`PciAddress::validate`].
    pub const fn new(bus: u8, device: u8, function: u8) -> Self {
        Self {
            bus,
            device,
            function,
        }
    }

    /// True when device/function are within the architectural limits.
    pub const fn is_valid(&self) -> bool {
        self.device <= Self::MAX_DEVICE && self.function <= Self::MAX_FUNCTION
    }

    /// Returns [`PciError::InvalidAddress`] when the address is impossible.
    pub fn validate(&self) -> PciResult<()> {
        if self.is_valid() {
            Ok(())
        } else {
            Err(PciError::InvalidAddress(self.to_string()))
        }
    }

    /// Canonical enumeration key: `bus << 8 | device << 3 | function`.
    pub const fn config_key(&self) -> u16 {
        ((self.bus as u16) << 8) | ((self.device as u16) << 3) | self.function as u16
    }

    /// Inverse of [`PciAddress::config_key`].
    pub const fn from_config_key(key: u16) -> Self {
        Self {
            bus: (key >> 8) as u8,
            device: ((key >> 3) & 0x1F) as u8,
            function: (key & 0x07) as u8,
        }
    }

    /// Function 0 of the same device (used to read the header type first).
    pub const fn function_zero(&self) -> Self {
        Self {
            bus: self.bus,
            device: self.device,
            function: 0,
        }
    }

    /// True when this is function 0.
    pub const fn is_function_zero(&self) -> bool {
        self.function == 0
    }
}

impl fmt::Display for PciAddress {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "0000:{:02x}:{:02x}.{}",
            self.bus, self.device, self.function
        )
    }
}

/// Class code triple (`class` / `subclass` / `programming interface`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct PciClass {
    /// Base class code (configuration offset `0x0B`).
    pub base: u8,
    /// Sub-class code (offset `0x0A`).
    pub subclass: u8,
    /// Programming interface (offset `0x09`).
    pub prog_if: u8,
}

impl PciClass {
    /// Builds a class triple.
    pub const fn new(base: u8, subclass: u8, prog_if: u8) -> Self {
        Self {
            base,
            subclass,
            prog_if,
        }
    }

    /// Human readable name for the subset of class codes this stack cares about.
    ///
    /// Anything not in the table is reported as `"unknown"`; use
    /// [`PciClass::hex`] for the raw encoding. Guessing a device type from an
    /// unrecognised code would violate the "no invented hardware" rule.
    pub const fn name(&self) -> &'static str {
        match (self.base, self.subclass) {
            (0x00, 0x00) => "unclassified",
            (0x01, 0x01) => "IDE controller",
            (0x01, 0x06) => "SATA controller",
            (0x01, 0x08) => "NVMe controller",
            (0x02, 0x00) => "Ethernet controller",
            (0x02, 0x80) => "network controller",
            (0x03, 0x00) => "VGA controller",
            (0x03, 0x02) => "3D controller",
            (0x04, 0x01) => "multimedia audio controller",
            (0x06, 0x00) => "host bridge",
            (0x06, 0x01) => "ISA bridge",
            (0x06, 0x04) => "PCI-to-PCI bridge",
            (0x06, 0x80) => "system peripheral",
            (0x0C, 0x03) => "USB controller",
            (0x0C, 0x05) => "SMBus controller",
            (0x11, 0x80) => "signal processing controller",
            (0xFF, _) => "unassigned class",
            _ => "unknown",
        }
    }

    /// Raw encoding, always available even for unknown classes.
    pub fn hex(&self) -> String {
        format!(
            "{:02x}:{:02x}:{:02x}",
            self.base, self.subclass, self.prog_if
        )
    }

    /// True for the PCI-to-PCI bridge class.
    pub const fn is_bridge(&self) -> bool {
        self.base == 0x06 && self.subclass == 0x04
    }

    /// True for the mass-storage base class.
    pub const fn is_storage(&self) -> bool {
        self.base == 0x01
    }

    /// True for the network base class.
    pub const fn is_network(&self) -> bool {
        self.base == 0x02
    }

    /// True for the display base class.
    pub const fn is_display(&self) -> bool {
        self.base == 0x03
    }
}

impl fmt::Display for PciClass {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} [{}]", self.name(), self.hex())
    }
}

/// Header layout of a PCI function (configuration offset `0x0E`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PciHeaderType {
    /// Type 0: endpoint device (six BAR slots).
    Type0,
    /// Type 1: PCI-to-PCI bridge (two BAR slots plus bus number registers).
    Type1,
    /// Type 2: CardBus bridge.
    Type2,
    /// Any other value, reported verbatim.
    Unknown(u8),
}

impl PciHeaderType {
    /// Decodes the low seven bits of the header-type register.
    pub const fn from_raw(raw: u8) -> Self {
        match raw & 0x7F {
            0x00 => PciHeaderType::Type0,
            0x01 => PciHeaderType::Type1,
            0x02 => PciHeaderType::Type2,
            other => PciHeaderType::Unknown(other),
        }
    }

    /// Bit 7 of the header-type register: the device has multiple functions.
    pub const fn is_multifunction(raw: u8) -> bool {
        raw & 0x80 != 0
    }

    /// Number of BAR slots defined by this header layout.
    pub const fn bar_slots(&self) -> u8 {
        match self {
            PciHeaderType::Type0 => 6,
            PciHeaderType::Type1 => 2,
            _ => 0,
        }
    }

    /// Stable label used in snapshots and diagnostics.
    pub const fn label(&self) -> &'static str {
        match self {
            PciHeaderType::Type0 => "type0",
            PciHeaderType::Type1 => "type1",
            PciHeaderType::Type2 => "type2",
            PciHeaderType::Unknown(_) => "unknown",
        }
    }
}

impl fmt::Display for PciHeaderType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PciHeaderType::Unknown(raw) => write!(f, "unknown(0x{raw:02x})"),
            other => f.write_str(other.label()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn config_key_round_trips() {
        for (bus, device, function) in [(0u8, 0u8, 0u8), (0, 31, 7), (1, 2, 3), (255, 31, 7)] {
            let addr = PciAddress::new(bus, device, function);
            assert_eq!(PciAddress::from_config_key(addr.config_key()), addr);
        }
    }

    #[test]
    fn display_uses_lspci_layout() {
        assert_eq!(PciAddress::new(0, 31, 0).to_string(), "0000:00:1f.0");
        assert_eq!(PciAddress::new(3, 0, 2).to_string(), "0000:03:00.2");
    }

    #[test]
    fn validation_rejects_impossible_addresses() {
        assert!(PciAddress::new(0, 31, 7).validate().is_ok());
        assert!(PciAddress::new(0, 32, 0).validate().is_err());
        assert!(PciAddress::new(0, 0, 8).validate().is_err());
        assert!(matches!(
            PciAddress::new(0, 32, 0).validate(),
            Err(PciError::InvalidAddress(_))
        ));
    }

    #[test]
    fn function_helpers_work() {
        let addr = PciAddress::new(1, 4, 3);
        assert_eq!(addr.function_zero(), PciAddress::new(1, 4, 0));
        assert!(!addr.is_function_zero());
        assert!(addr.function_zero().is_function_zero());
    }

    #[test]
    fn class_names_are_explicit_for_unknown_codes() {
        assert_eq!(PciClass::new(0x01, 0x08, 0x02).name(), "NVMe controller");
        assert_eq!(PciClass::new(0x06, 0x04, 0x00).name(), "PCI-to-PCI bridge");
        assert_eq!(PciClass::new(0x7E, 0x7E, 0x00).name(), "unknown");
        assert_eq!(PciClass::new(0x7E, 0x7E, 0x00).hex(), "7e:7e:00");
        assert!(PciClass::new(0x01, 0x08, 0x02).is_storage());
        assert!(PciClass::new(0x06, 0x04, 0x00).is_bridge());
        assert!(!PciClass::new(0x02, 0x00, 0x00).is_display());
    }

    #[test]
    fn header_type_decodes_low_bits_and_multifunction() {
        assert_eq!(PciHeaderType::from_raw(0x00), PciHeaderType::Type0);
        assert_eq!(PciHeaderType::from_raw(0x81), PciHeaderType::Type1);
        assert!(PciHeaderType::is_multifunction(0x81));
        assert!(!PciHeaderType::is_multifunction(0x01));
        assert_eq!(PciHeaderType::from_raw(0x7F), PciHeaderType::Unknown(0x7F));
        assert_eq!(PciHeaderType::Unknown(0x7F).to_string(), "unknown(0x7f)");
        assert_eq!(PciHeaderType::Type0.bar_slots(), 6);
        assert_eq!(PciHeaderType::Type1.bar_slots(), 2);
    }
}
