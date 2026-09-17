//! PCI capability decoding: MSI, MSI-X, PCI Express and ATS (§5, §7).
//!
//! Capability structures live in the linked list rooted at configuration offset
//! `0x34`. Only ids and register layouts defined by the PCI Local Bus and PCI
//! Express specifications are decoded here; unknown ids are preserved as
//! [`PciCapability::Other`] so nothing is silently dropped.
//!
//! Interrupt *programming* (writing message address/data, binding vectors) is not
//! implemented: interrupts belong to Phase 4.2. This module only reports what the
//! device declares.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Capability ids used by this stack (PCI Local Bus §6.7, PCIe §7.5).
pub mod ids {
    /// Power management capability.
    pub const POWER_MANAGEMENT: u8 = 0x01;
    /// Message Signalled Interrupts.
    pub const MSI: u8 = 0x05;
    /// PCI Express capability.
    pub const PCI_EXPRESS: u8 = 0x10;
    /// MSI-X.
    pub const MSI_X: u8 = 0x11;
    /// Address Translation Services (only meaningful with an IOMMU).
    pub const ADDRESS_TRANSLATION: u8 = 0x0F;
    /// Conventional PCI hot plug capability.
    pub const HOT_PLUG: u8 = 0x0C;
    /// Vendor specific capability.
    pub const VENDOR_SPECIFIC: u8 = 0x09;
}

/// Bits of the MSI message control register.
pub mod msi_control {
    /// MSI enable.
    pub const ENABLE: u16 = 1 << 0;
    /// "Multiple message capable" field.
    pub const MULTIPLE_MESSAGE_CAPABLE: u16 = 0b111 << 1;
    /// "Multiple message enable" field.
    pub const MULTIPLE_MESSAGE_ENABLE: u16 = 0b111 << 4;
    /// 64-bit message address capable.
    pub const ADDRESS_64BIT: u16 = 1 << 7;
    /// Per-vector masking capable.
    pub const PER_VECTOR_MASKING: u16 = 1 << 8;
}

/// Bits of the MSI-X message control register.
pub mod msix_control {
    /// MSI-X enable.
    pub const ENABLE: u16 = 1 << 15;
    /// Function mask (all vectors masked).
    pub const FUNCTION_MASK: u16 = 1 << 14;
    /// Table size field (holds N-1).
    pub const TABLE_SIZE: u16 = 0x07FF;
}

/// PCIe link speed, decoded from the Link Capabilities register.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkSpeed {
    /// 2.5 GT/s.
    Gen1,
    /// 5.0 GT/s.
    Gen2,
    /// 8.0 GT/s.
    Gen3,
    /// 16.0 GT/s.
    Gen4,
    /// 32.0 GT/s.
    Gen5,
    /// 64.0 GT/s.
    Gen6,
    /// Any other encoding, preserved verbatim.
    Unknown(u8),
}

impl LinkSpeed {
    /// Decodes the four-bit max-link-speed field.
    pub const fn from_code(code: u8) -> Self {
        match code {
            1 => LinkSpeed::Gen1,
            2 => LinkSpeed::Gen2,
            3 => LinkSpeed::Gen3,
            4 => LinkSpeed::Gen4,
            5 => LinkSpeed::Gen5,
            6 => LinkSpeed::Gen6,
            other => LinkSpeed::Unknown(other),
        }
    }

    /// Stable label used in snapshots.
    pub const fn label(&self) -> &'static str {
        match self {
            LinkSpeed::Gen1 => "2.5 GT/s",
            LinkSpeed::Gen2 => "5.0 GT/s",
            LinkSpeed::Gen3 => "8.0 GT/s",
            LinkSpeed::Gen4 => "16.0 GT/s",
            LinkSpeed::Gen5 => "32.0 GT/s",
            LinkSpeed::Gen6 => "64.0 GT/s",
            LinkSpeed::Unknown(_) => "unknown",
        }
    }
}

impl fmt::Display for LinkSpeed {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LinkSpeed::Unknown(code) => write!(f, "unknown(0x{code:x})"),
            other => f.write_str(other.label()),
        }
    }
}

/// MSI capability contents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsiInfo {
    /// Raw message control register.
    pub message_control: u16,
    /// Log2 of the number of vectors requested (field bits 1..3).
    pub vectors_requested_log2: u8,
    /// True when the device supports a 64-bit message address.
    pub address_64bit: bool,
    /// True when per-vector masking is implemented.
    pub per_vector_masking: bool,
    /// True when MSI is currently enabled.
    pub enabled: bool,
}

impl MsiInfo {
    /// Decodes the message control register.
    pub const fn from_control(message_control: u16) -> Self {
        Self {
            message_control,
            vectors_requested_log2: ((message_control & msi_control::MULTIPLE_MESSAGE_CAPABLE) >> 1)
                as u8,
            address_64bit: message_control & msi_control::ADDRESS_64BIT != 0,
            per_vector_masking: message_control & msi_control::PER_VECTOR_MASKING != 0,
            enabled: message_control & msi_control::ENABLE != 0,
        }
    }

    /// Number of vectors the device can request (`1 << log2`).
    pub const fn vector_count(&self) -> u16 {
        1u16 << (self.vectors_requested_log2 & 0x07)
    }
}

/// MSI-X capability contents.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsixInfo {
    /// Raw message control register.
    pub message_control: u16,
    /// BAR index holding the MSI-X vector table.
    pub table_bar: u8,
    /// Offset of the vector table inside `table_bar`.
    pub table_offset: u32,
    /// BAR index holding the pending-bit array.
    pub pba_bar: u8,
    /// True when MSI-X is enabled.
    pub enabled: bool,
    /// True when the whole function is masked.
    pub function_masked: bool,
}

impl MsixInfo {
    /// Number of MSI-X vectors (the table-size field holds N-1).
    pub const fn vector_count(&self) -> u16 {
        (self.message_control & msix_control::TABLE_SIZE).wrapping_add(1)
    }
}

/// A configured MSI / MSI-X interrupt vector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct MsiVector {
    /// Vector index (0..N).
    pub index: u16,
    /// Target destination core / APIC ID.
    pub destination_apic_id: u32,
    /// Message address (32-bit or 64-bit MMIO/APIC address).
    pub address: u64,
    /// Message data (interrupt vector number and delivery mode).
    pub data: u32,
    /// Masked status.
    pub masked: bool,
}

impl MsiVector {
    /// Constructs an unmasked MSI vector.
    pub const fn new(index: u16, destination_apic_id: u32, address: u64, data: u32) -> Self {
        Self {
            index,
            destination_apic_id,
            address,
            data,
            masked: false,
        }
    }
}

/// PCI Express capability contents (slot and link information).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PcieCapability {
    /// Capability version (low nibble of the PCIe capability register).
    pub version: u8,
    /// Device/port type (bits 4..7 of the PCIe capability register).
    pub device_port_type: u8,
    /// The link is connected to a slot (PCIe capabilities register bit 8).
    pub slot_implemented: bool,
    /// Slot capabilities report attention-button/power-controller hotplug.
    pub hotplug_capable: bool,
    /// Slot capabilities report surprise removal support.
    pub hotplug_surprise: bool,
    /// Maximum link speed reported by the link capabilities register.
    pub link_speed: LinkSpeed,
    /// Maximum link width reported by the link capabilities register.
    pub link_width: u8,
}

/// One entry of the capability list.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum PciCapability {
    /// Power management (capability id `0x01`).
    PowerManagement { version: u8 },
    /// Message Signalled Interrupts (id `0x05`).
    Msi(MsiInfo),
    /// MSI-X (id `0x11`).
    Msix(MsixInfo),
    /// PCI Express (id `0x10`).
    Pcie(PcieCapability),
    /// Address Translation Services (id `0x0F`).
    AddressTranslation,
    /// Conventional PCI hot plug (id `0x0C`).
    HotPlug,
    /// Vendor specific capability (id `0x09`).
    VendorSpecific { id: u8 },
    /// Any capability this stack does not decode, preserved with its id.
    Other { id: u8, offset: u16 },
}

impl PciCapability {
    /// The raw capability id.
    pub const fn id(&self) -> u8 {
        match self {
            PciCapability::PowerManagement { .. } => ids::POWER_MANAGEMENT,
            PciCapability::Msi(_) => ids::MSI,
            PciCapability::Msix(_) => ids::MSI_X,
            PciCapability::Pcie(_) => ids::PCI_EXPRESS,
            PciCapability::AddressTranslation => ids::ADDRESS_TRANSLATION,
            PciCapability::HotPlug => ids::HOT_PLUG,
            PciCapability::VendorSpecific { id } => *id,
            PciCapability::Other { id, .. } => *id,
        }
    }

    /// Stable label used in snapshots and diagnostics.
    pub const fn label(&self) -> &'static str {
        match self {
            PciCapability::PowerManagement { .. } => "power-management",
            PciCapability::Msi(_) => "msi",
            PciCapability::Msix(_) => "msi-x",
            PciCapability::Pcie(_) => "pcie",
            PciCapability::AddressTranslation => "ats",
            PciCapability::HotPlug => "hot-plug",
            PciCapability::VendorSpecific { .. } => "vendor-specific",
            PciCapability::Other { .. } => "other",
        }
    }

    /// True when the capability signals interrupt support.
    pub const fn is_interrupt_capability(&self) -> bool {
        matches!(self, PciCapability::Msi(_) | PciCapability::Msix(_))
    }
}

impl fmt::Display for PciCapability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PciCapability::Other { id, offset } => {
                write!(f, "other(0x{id:02x} @ 0x{offset:02x})")
            }
            other => f.write_str(other.label()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn link_speed_decoding_covers_known_and_unknown_codes() {
        assert_eq!(LinkSpeed::from_code(1), LinkSpeed::Gen1);
        assert_eq!(LinkSpeed::from_code(3), LinkSpeed::Gen3);
        assert_eq!(LinkSpeed::from_code(6), LinkSpeed::Gen6);
        assert_eq!(LinkSpeed::from_code(9), LinkSpeed::Unknown(9));
        assert_eq!(LinkSpeed::Gen3.to_string(), "8.0 GT/s");
        assert_eq!(LinkSpeed::Unknown(9).to_string(), "unknown(0x9)");
    }

    #[test]
    fn msi_control_decodes_vectors_and_flags() {
        // enable + 4 vectors (log2 = 2) + 64-bit address + per-vector masking
        let control = msi_control::ENABLE
            | (2 << 1)
            | msi_control::ADDRESS_64BIT
            | msi_control::PER_VECTOR_MASKING;
        let msi = MsiInfo::from_control(control);
        assert!(msi.enabled);
        assert!(msi.address_64bit);
        assert!(msi.per_vector_masking);
        assert_eq!(msi.vectors_requested_log2, 2);
        assert_eq!(msi.vector_count(), 4);
    }

    #[test]
    fn msix_table_size_field_holds_n_minus_one() {
        let info = MsixInfo {
            message_control: msix_control::ENABLE | 15,
            table_bar: 1,
            table_offset: 0x2000,
            pba_bar: 1,
            enabled: true,
            function_masked: false,
        };
        assert_eq!(info.vector_count(), 16);
    }

    #[test]
    fn capability_ids_and_labels_are_stable() {
        let msi = PciCapability::Msi(MsiInfo::from_control(0));
        assert_eq!(msi.id(), ids::MSI);
        assert!(msi.is_interrupt_capability());
        assert_eq!(msi.to_string(), "msi");

        let other = PciCapability::Other {
            id: 0x42,
            offset: 0x60,
        };
        assert_eq!(other.id(), 0x42);
        assert!(other.to_string().contains("0x42"));
        assert!(!other.is_interrupt_capability());

        assert_eq!(
            PciCapability::Pcie(PcieCapability {
                version: 2,
                device_port_type: 0,
                slot_implemented: false,
                hotplug_capable: false,
                hotplug_surprise: false,
                link_speed: LinkSpeed::Gen4,
                link_width: 16,
            })
            .label(),
            "pcie"
        );
        assert_eq!(PciCapability::AddressTranslation.label(), "ats");
    }
}
