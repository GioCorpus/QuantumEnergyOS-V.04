//! Discovered PCI devices: identity, decoded configuration space and derived
//! capability facts (§4, §5).
//!
//! [`PciDevice`] is a *snapshot* of one PCI function as read from configuration
//! space. It contains no handles: nothing here can touch the device. Access is
//! granted later by the device manager through capabilities and MMIO mapping.

use serde::{Deserialize, Serialize};

use super::address::{PciAddress, PciClass, PciHeaderType};
use super::bar::{BarKind, PciBar};
use super::capability::{PciCapability, PcieCapability};
use super::config::{command_bits, status_bits};

/// DMA-related facts derived from configuration space (§5, §6).
///
/// Every field is a decoded bit or a decoded BAR property. Nothing is assumed:
/// when a fact cannot be determined it stays `false`, and the manager refuses to
/// grant DMA access on that basis.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DmaReadiness {
    /// Command register bus-master bit is set (prerequisite for any DMA).
    pub bus_master_enabled: bool,
    /// Command register memory-space bit is set.
    pub memory_space_enabled: bool,
    /// At least one 64-bit memory BAR is implemented.
    pub supports_64bit_addresses: bool,
    /// The function implements MSI or MSI-X (usable for DMA completion).
    pub has_msi_or_msix: bool,
    /// The function advertises Address Translation Services.
    pub has_ats: bool,
}

impl DmaReadiness {
    /// True when the device declares the prerequisites for DMA.
    ///
    /// This is *not* authorisation: the manager additionally requires an explicit
    /// DMA grant and an IOMMU policy that permits it.
    pub const fn is_declared_ready(&self) -> bool {
        self.bus_master_enabled && self.memory_space_enabled
    }

    /// Human readable summary for diagnostics.
    pub fn summary(&self) -> String {
        format!(
            "bus_master={} mem_space={} addr64={} msi={} ats={}",
            self.bus_master_enabled,
            self.memory_space_enabled,
            self.supports_64bit_addresses,
            self.has_msi_or_msix,
            self.has_ats
        )
    }
}

/// Hot-plug support reported by the device (§4: never assume hotplug).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HotplugSupport {
    /// The function exposes no information about hotplug.
    #[default]
    Unknown,
    /// The function sits behind a slot that does not report hotplug.
    NotSupported,
    /// The function reports slot hotplug capability.
    Supported,
}

impl HotplugSupport {
    /// Stable label used in snapshots.
    pub const fn label(self) -> &'static str {
        match self {
            HotplugSupport::Unknown => "unknown",
            HotplugSupport::NotSupported => "not-supported",
            HotplugSupport::Supported => "supported",
        }
    }

    /// True only for [`HotplugSupport::Supported`].
    pub const fn is_supported(self) -> bool {
        matches!(self, HotplugSupport::Supported)
    }
}

impl std::fmt::Display for HotplugSupport {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Identity plus the raw configuration values of one PCI function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PciDeviceInfo {
    /// Where the function lives on the bus.
    pub address: PciAddress,
    /// Vendor id (never the all-ones "absent" sentinel for an enumerated device).
    pub vendor_id: u16,
    /// Device id.
    pub device_id: u16,
    /// Revision id.
    pub revision: u8,
    /// Class/subclass/programming interface triple.
    pub class: PciClass,
    /// Header layout.
    pub header_type: PciHeaderType,
    /// Bit 7 of the header-type register.
    pub multifunction: bool,
    /// Subsystem vendor id, when the header layout provides it.
    pub subsystem_vendor_id: Option<u16>,
    /// Subsystem id, when the header layout provides it.
    pub subsystem_id: Option<u16>,
    /// Command register contents at enumeration time.
    pub command: u16,
    /// Status register contents at enumeration time.
    pub status: u16,
}

impl PciDeviceInfo {
    /// Convenience constructor for a type-0 endpoint.
    pub fn new(address: PciAddress, vendor_id: u16, device_id: u16, class: PciClass) -> Self {
        Self {
            address,
            vendor_id,
            device_id,
            revision: 0,
            class,
            header_type: PciHeaderType::Type0,
            multifunction: false,
            subsystem_vendor_id: None,
            subsystem_id: None,
            command: 0,
            status: 0,
        }
    }

    /// Stable identification string, `vendor:device`.
    pub fn ids_label(&self) -> String {
        format!("{:04x}:{:04x}", self.vendor_id, self.device_id)
    }

    /// True when the bus-master bit is set.
    pub fn bus_master_enabled(&self) -> bool {
        self.command & command_bits::BUS_MASTER != 0
    }

    /// True when the memory-space bit is set.
    pub fn memory_space_enabled(&self) -> bool {
        self.command & command_bits::MEMORY_SPACE != 0
    }

    /// True when the function implements a capability list.
    pub fn has_capability_list(&self) -> bool {
        self.status & status_bits::CAPABILITY_LIST != 0
    }
}

/// Decoded-fact helpers over a capability/BAR list.
///
/// Kept free of device state so enumeration, drivers, diagnostics and tests all
/// derive facts the same way.
pub mod facts {
    use super::*;
    use crate::pci::capability::{ids, MsiInfo, MsixInfo};

    /// True when a decoded capability with `id` is present.
    pub fn has_capability(capabilities: &[PciCapability], id: u8) -> bool {
        capabilities
            .iter()
            .any(|capability| capability.id() == id)
    }

    /// True when MSI or MSI-X is present.
    pub fn supports_interrupts(capabilities: &[PciCapability]) -> bool {
        has_capability(capabilities, ids::MSI) || has_capability(capabilities, ids::MSI_X)
    }

    /// True when Address Translation Services are advertised.
    pub fn has_ats(capabilities: &[PciCapability]) -> bool {
        has_capability(capabilities, ids::ADDRESS_TRANSLATION)
    }

    /// True when any memory BAR is 64-bit.
    pub fn has_64bit_bar(bars: &[PciBar]) -> bool {
        bars.iter().any(|bar| bar.kind == BarKind::Memory64)
    }

    /// PCIe capability, when present.
    pub fn pcie(capabilities: &[PciCapability]) -> Option<&PcieCapability> {
        capabilities.iter().find_map(|capability| match capability {
            PciCapability::Pcie(info) => Some(info),
            _ => None,
        })
    }

    /// MSI capability, when present.
    pub fn msi(capabilities: &[PciCapability]) -> Option<&MsiInfo> {
        capabilities.iter().find_map(|capability| match capability {
            PciCapability::Msi(info) => Some(info),
            _ => None,
        })
    }

    /// MSI-X capability, when present.
    pub fn msix(capabilities: &[PciCapability]) -> Option<&MsixInfo> {
        capabilities.iter().find_map(|capability| match capability {
            PciCapability::Msix(info) => Some(info),
            _ => None,
        })
    }

    /// Link description from the PCIe capability, or `unknown` when absent.
    pub fn link_label(capabilities: &[PciCapability]) -> String {
        match pcie(capabilities) {
            Some(pcie) => format!("{} x{}", pcie.link_speed, pcie.link_width),
            None => "unknown".to_string(),
        }
    }

    /// Hot-plug support derived from the PCIe slot registers.
    ///
    /// Returns [`HotplugSupport::Unknown`] when the function exposes no PCIe
    /// capability, so a caller cannot mistake "no information" for "supported".
    pub fn hotplug_support(capabilities: &[PciCapability]) -> HotplugSupport {
        match pcie(capabilities) {
            Some(pcie) if pcie.slot_implemented && pcie.hotplug_capable => {
                HotplugSupport::Supported
            }
            Some(pcie) if pcie.slot_implemented => HotplugSupport::NotSupported,
            _ => HotplugSupport::Unknown,
        }
    }
}