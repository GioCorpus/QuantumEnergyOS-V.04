//! Discovered PCI devices: identity, decoded configuration space and derived
//! capability facts (§4, §5).
//!
//! [`PciDevice`] is a *snapshot* of one PCI function as read from configuration
//! space. It contains no handles: nothing here can touch the device. Access is
//! granted later by the device manager through capabilities and MMIO mapping.

use serde::{Deserialize, Serialize};

use crate::capability::{DeviceCapability, DeviceCapabilitySet};

use super::address::{PciAddress, PciClass, PciHeaderType};
use super::bar::{BarKind, PciBar};
use super::capability::{ids, PciCapability, PcieCapability};
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

/// A fully decoded snapshot of one PCI/PCIe function.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PciDevice {
    /// Core configuration space info.
    pub info: PciDeviceInfo,
    /// Decoded Base Address Registers.
    pub bars: Vec<PciBar>,
    /// Decoded capabilities list.
    pub capabilities: Vec<PciCapability>,
    /// DMA facts derived from configuration space.
    pub dma_readiness: DmaReadiness,
    /// Hot-plug support status.
    pub hotplug: HotplugSupport,
}

impl PciDevice {
    /// Constructs a `PciDevice` from its parts, deriving DMA readiness and hotplug support.
    pub fn new(info: PciDeviceInfo, bars: Vec<PciBar>, capabilities: Vec<PciCapability>) -> Self {
        let dma_readiness = DmaReadiness {
            bus_master_enabled: info.bus_master_enabled(),
            memory_space_enabled: info.memory_space_enabled(),
            supports_64bit_addresses: facts::has_64bit_bar(&bars),
            has_msi_or_msix: facts::supports_interrupts(&capabilities),
            has_ats: facts::has_ats(&capabilities),
        };
        let hotplug = facts::hotplug_support(&capabilities);

        Self {
            info,
            bars,
            capabilities,
            dma_readiness,
            hotplug,
        }
    }

    /// Bus address.
    pub fn address(&self) -> PciAddress {
        self.info.address
    }

    /// Vendor ID.
    pub fn vendor_id(&self) -> u16 {
        self.info.vendor_id
    }

    /// Device ID.
    pub fn device_id(&self) -> u16 {
        self.info.device_id
    }

    /// Device class.
    pub fn class(&self) -> PciClass {
        self.info.class
    }

    /// Core info reference.
    pub fn info(&self) -> &PciDeviceInfo {
        &self.info
    }

    /// Slice of BARs.
    pub fn bars(&self) -> &[PciBar] {
        &self.bars
    }

    /// Looks up a BAR by slot index.
    pub fn bar(&self, index: u8) -> Option<&PciBar> {
        self.bars.iter().find(|b| b.index == index)
    }

    /// Slice of capabilities.
    pub fn capabilities(&self) -> &[PciCapability] {
        &self.capabilities
    }

    /// Checks if a capability id is present.
    pub fn has_capability(&self, id: u8) -> bool {
        facts::has_capability(&self.capabilities, id)
    }

    /// DMA readiness facts.
    pub fn dma_readiness(&self) -> DmaReadiness {
        self.dma_readiness
    }

    /// Hotplug support status.
    pub fn hotplug_support(&self) -> HotplugSupport {
        self.hotplug
    }

    /// Computes the set of capabilities *declared* by this hardware.
    ///
    /// Every enumerated device declares `Enumerate`, `Inspect`, `Operate`, and `Reset`.
    /// Optional capabilities (`MmioAccess`, `DmaAccess`, `InterruptControl`,
    /// `PowerManagement`, `Hotplug`) are included only if the hardware facts justify them.
    pub fn declared_capabilities(&self) -> DeviceCapabilitySet {
        let mut caps = DeviceCapabilitySet::baseline();
        caps.grant(DeviceCapability::Reset);

        // MMIO access is declared if any memory BAR is present & mappable
        if self.bars.iter().any(|b| b.is_mappable()) {
            caps.grant(DeviceCapability::MmioAccess);
        }

        // DMA access is declared if bus master & memory space are enabled
        if self.dma_readiness.is_declared_ready() {
            caps.grant(DeviceCapability::DmaAccess);
        }

        // Interrupt control is declared if MSI or MSI-X is present
        if self.dma_readiness.has_msi_or_msix {
            caps.grant(DeviceCapability::InterruptControl);
        }

        // Power management is declared if power capability is present
        if self.has_capability(ids::POWER_MANAGEMENT) {
            caps.grant(DeviceCapability::PowerManagement);
        }

        // Hotplug is declared if slot reports hotplug
        if self.hotplug.is_supported() {
            caps.grant(DeviceCapability::Hotplug);
        }

        caps
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
        capabilities.iter().any(|capability| capability.id() == id)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pci::capability::{LinkSpeed, MsiInfo};

    #[test]
    fn pci_device_info_properties() {
        let addr = PciAddress::new(0, 1, 0);
        let class = PciClass::new(0x01, 0x08, 0x02);
        let mut info = PciDeviceInfo::new(addr, 0x1B36, 0x0010, class);
        info.command = command_bits::BUS_MASTER | command_bits::MEMORY_SPACE;

        assert_eq!(info.ids_label(), "1b36:0010");
        assert!(info.bus_master_enabled());
        assert!(info.memory_space_enabled());
        assert!(!info.has_capability_list());
    }

    #[test]
    fn pci_device_derives_facts_and_capabilities() {
        let addr = PciAddress::new(0, 2, 0);
        let class = PciClass::new(0x03, 0x00, 0x00);
        let mut info = PciDeviceInfo::new(addr, 0x10DE, 0x1EB8, class);
        info.command = command_bits::BUS_MASTER | command_bits::MEMORY_SPACE;
        info.status = status_bits::CAPABILITY_LIST;

        let bars = vec![
            PciBar::new(0, BarKind::Memory64, 0x1000_0000, 0x10000, false),
            PciBar::new(2, BarKind::Memory32, 0x2000_0000, 0x2000, false),
        ];

        let capabilities = vec![
            PciCapability::PowerManagement { version: 3 },
            PciCapability::Msi(MsiInfo::from_control(0x0001)),
            PciCapability::Pcie(PcieCapability {
                version: 2,
                device_port_type: 0,
                slot_implemented: true,
                hotplug_capable: true,
                hotplug_surprise: false,
                link_speed: LinkSpeed::Gen4,
                link_width: 16,
            }),
        ];

        let dev = PciDevice::new(info, bars, capabilities);

        assert_eq!(dev.address(), addr);
        assert_eq!(dev.vendor_id(), 0x10DE);
        assert_eq!(dev.device_id(), 0x1EB8);
        assert_eq!(dev.bars().len(), 2);
        assert!(dev.bar(0).is_some());
        assert!(dev.bar(5).is_none());
        assert!(dev.dma_readiness().is_declared_ready());
        assert!(dev.dma_readiness().supports_64bit_addresses);
        assert!(dev.dma_readiness().has_msi_or_msix);
        assert_eq!(dev.hotplug_support(), HotplugSupport::Supported);

        let declared = dev.declared_capabilities();
        assert!(declared.contains(DeviceCapability::Enumerate));
        assert!(declared.contains(DeviceCapability::Inspect));
        assert!(declared.contains(DeviceCapability::Operate));
        assert!(declared.contains(DeviceCapability::Reset));
        assert!(declared.contains(DeviceCapability::MmioAccess));
        assert!(declared.contains(DeviceCapability::DmaAccess));
        assert!(declared.contains(DeviceCapability::InterruptControl));
        assert!(declared.contains(DeviceCapability::PowerManagement));
        assert!(declared.contains(DeviceCapability::Hotplug));
    }
}
