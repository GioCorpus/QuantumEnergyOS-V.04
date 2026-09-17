//! Memory-backed PCI configuration space model (§28).
//!
//! [`SimulatedPciBackend`] simulates a PCI bus for host-tests and CI without
//! requiring physical hardware or privileged OS APIs.
//!
//! Features:
//! - Full 256-byte standard configuration space per device.
//! - Accurate BAR sizing simulation (write all ones returns size mask).
//! - Standard capability linked list formatting (Power Management, MSI, MSI-X, PCIe).
//! - Realistic hardware test fixtures (Host Bridge, NVMe, GPU, NIC, Telemetry).

use std::collections::{BTreeMap, HashSet};

use serde::{Deserialize, Serialize};

use crate::error::{PciError, PciResult};

use super::address::{PciAddress, PciClass, PciHeaderType};
use super::bar::BarKind;
use super::capability::{
    msi_control, msix_control, LinkSpeed, MsiInfo, MsixInfo, PciCapability, PcieCapability,
};
use super::config::{
    command_bits, offsets, status_bits, BackendSource, ConfigWidth, PciConfigBackend,
};

/// Specification for a simulated Base Address Register.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulatedBarSpec {
    /// Slot index (0..5).
    pub index: u8,
    /// BAR kind.
    pub kind: BarKind,
    /// Simulated base address.
    pub base: u64,
    /// Simulated region size in bytes (must be power of two).
    pub size: u64,
    /// Prefetchable memory bit.
    pub prefetchable: bool,
}

/// Specification for adding a device to [`SimulatedPciBackend`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SimulatedDeviceSpec {
    /// Bus address.
    pub address: PciAddress,
    /// Vendor ID.
    pub vendor_id: u16,
    /// Device ID.
    pub device_id: u16,
    /// Revision ID.
    pub revision: u8,
    /// Class code triple.
    pub class: PciClass,
    /// Header layout.
    pub header_type: PciHeaderType,
    /// Multi-function device flag.
    pub multifunction: bool,
    /// Subsystem vendor ID.
    pub subsystem_vendor_id: Option<u16>,
    /// Subsystem ID.
    pub subsystem_id: Option<u16>,
    /// Initial command register.
    pub command: u16,
    /// Initial status register.
    pub status: u16,
    /// Configured BARs.
    pub bars: Vec<SimulatedBarSpec>,
    /// Configured capabilities.
    pub capabilities: Vec<PciCapability>,
}

impl SimulatedDeviceSpec {
    /// Builds raw 256-byte configuration space and BAR map from the specification.
    pub fn build_config_space(&self) -> ([u8; 256], Vec<SimulatedBarSpec>) {
        let mut config = [0u8; 256];

        // 0x00: Vendor ID
        config[0..2].copy_from_slice(&self.vendor_id.to_le_bytes());
        // 0x02: Device ID
        config[2..4].copy_from_slice(&self.device_id.to_le_bytes());
        // 0x04: Command
        config[4..6].copy_from_slice(&self.command.to_le_bytes());
        // 0x06: Status
        let status = if !self.capabilities.is_empty() {
            self.status | status_bits::CAPABILITY_LIST
        } else {
            self.status
        };
        config[6..8].copy_from_slice(&status.to_le_bytes());

        // 0x08: Revision
        config[8] = self.revision;
        // 0x09: Prog IF
        config[9] = self.class.prog_if;
        // 0x0A: Subclass
        config[10] = self.class.subclass;
        // 0x0B: Class code
        config[11] = self.class.base;

        // 0x0E: Header type
        let raw_hdr = match self.header_type {
            PciHeaderType::Type0 => 0x00,
            PciHeaderType::Type1 => 0x01,
            PciHeaderType::Type2 => 0x02,
            PciHeaderType::Unknown(u) => u,
        } | if self.multifunction { 0x80 } else { 0x00 };
        config[0x0E] = raw_hdr;

        // 0x2C: Subsystem Vendor ID / 0x2E: Subsystem ID
        if let Some(svid) = self.subsystem_vendor_id {
            config[0x2C..0x2E].copy_from_slice(&svid.to_le_bytes());
        }
        if let Some(sid) = self.subsystem_id {
            config[0x2E..0x30].copy_from_slice(&sid.to_le_bytes());
        }

        // BAR Registers
        for bar in &self.bars {
            if let Some(offset) = offsets::bar(bar.index) {
                let off = offset as usize;
                match bar.kind {
                    BarKind::Memory32 => {
                        let val =
                            (bar.base as u32 & !0x0F) | if bar.prefetchable { 0x08 } else { 0x00 };
                        config[off..off + 4].copy_from_slice(&val.to_le_bytes());
                    }
                    BarKind::Memory64 => {
                        let low = (bar.base as u32 & !0x0F)
                            | 0x04
                            | if bar.prefetchable { 0x08 } else { 0x00 };
                        let high = (bar.base >> 32) as u32;
                        config[off..off + 4].copy_from_slice(&low.to_le_bytes());
                        if off + 8 <= 256 {
                            config[off + 4..off + 8].copy_from_slice(&high.to_le_bytes());
                        }
                    }
                    BarKind::IoPort => {
                        let val = (bar.base as u32 & !0x03) | 0x01;
                        config[off..off + 4].copy_from_slice(&val.to_le_bytes());
                    }
                    BarKind::Unimplemented => {}
                }
            }
        }

        // Capabilities linked list starting at offset 0x40
        if !self.capabilities.is_empty() {
            config[offsets::CAPABILITY_POINTER as usize] = 0x40;
            let mut cap_offset = 0x40usize;

            for (i, cap) in self.capabilities.iter().enumerate() {
                let next_offset = if i + 1 < self.capabilities.len() {
                    (cap_offset + 24).min(0xF0) as u8
                } else {
                    0u8
                };

                config[cap_offset] = cap.id();
                config[cap_offset + 1] = next_offset;

                match cap {
                    PciCapability::PowerManagement { version } => {
                        let val = *version as u16;
                        config[cap_offset + 2..cap_offset + 4].copy_from_slice(&val.to_le_bytes());
                    }
                    PciCapability::Msi(info) => {
                        config[cap_offset + 2..cap_offset + 4]
                            .copy_from_slice(&info.message_control.to_le_bytes());
                    }
                    PciCapability::Msix(info) => {
                        config[cap_offset + 2..cap_offset + 4]
                            .copy_from_slice(&info.message_control.to_le_bytes());
                        let table_val =
                            (info.table_offset & !0x07) | (info.table_bar as u32 & 0x07);
                        let pba_val = info.pba_bar as u32 & 0x07;
                        config[cap_offset + 4..cap_offset + 8]
                            .copy_from_slice(&table_val.to_le_bytes());
                        config[cap_offset + 8..cap_offset + 12]
                            .copy_from_slice(&pba_val.to_le_bytes());
                    }
                    PciCapability::Pcie(info) => {
                        let pcie_caps = (info.version as u16 & 0x0F)
                            | ((info.device_port_type as u16 & 0x0F) << 4)
                            | if info.slot_implemented { 1 << 8 } else { 0 };
                        config[cap_offset + 2..cap_offset + 4]
                            .copy_from_slice(&pcie_caps.to_le_bytes());

                        // Link caps at offset + 12
                        let speed_code = match info.link_speed {
                            LinkSpeed::Gen1 => 1,
                            LinkSpeed::Gen2 => 2,
                            LinkSpeed::Gen3 => 3,
                            LinkSpeed::Gen4 => 4,
                            LinkSpeed::Gen5 => 5,
                            LinkSpeed::Gen6 => 6,
                            LinkSpeed::Unknown(c) => c,
                        };
                        let link_caps =
                            (speed_code as u32 & 0x0F) | ((info.link_width as u32 & 0x3F) << 4);
                        config[cap_offset + 12..cap_offset + 16]
                            .copy_from_slice(&link_caps.to_le_bytes());

                        // Slot caps at offset + 20
                        if info.slot_implemented {
                            let slot_caps: u32 = if info.hotplug_capable {
                                1u32 << 6
                            } else {
                                0u32
                            } | if info.hotplug_surprise {
                                1u32 << 5
                            } else {
                                0u32
                            };
                            config[cap_offset + 20..cap_offset + 24]
                                .copy_from_slice(&slot_caps.to_le_bytes());
                        }
                    }
                    PciCapability::AddressTranslation
                    | PciCapability::HotPlug
                    | PciCapability::VendorSpecific { .. }
                    | PciCapability::Other { .. } => {}
                }

                cap_offset = next_offset as usize;
                if cap_offset == 0 {
                    break;
                }
            }
        }

        (config, self.bars.clone())
    }
}

/// Simulated PCI configuration space backend.
#[derive(Debug, Clone)]
pub struct SimulatedPciBackend {
    /// Configuration space memory per device key (bus << 8 | dev << 3 | func).
    configs: BTreeMap<u16, [u8; 256]>,
    /// BAR specifications for accurate sizing.
    bars: BTreeMap<(u16, u8), SimulatedBarSpec>,
    /// Tracks BARs currently probed with 0xFFFFFFFF for sizing.
    sizing_state: HashSet<(u16, u8)>,
}

impl SimulatedPciBackend {
    /// Creates a simulated backend pre-populated with standard QEOS test fixtures.
    pub fn new() -> Self {
        let mut backend = Self::empty();
        backend.populate_default_fixtures();
        backend
    }

    /// Creates an empty simulated backend with no devices.
    pub fn empty() -> Self {
        Self {
            configs: BTreeMap::new(),
            bars: BTreeMap::new(),
            sizing_state: HashSet::new(),
        }
    }

    /// Adds a device specification to the simulated bus.
    pub fn add_device(&mut self, spec: SimulatedDeviceSpec) {
        let key = spec.address.config_key();
        let (config, bars) = spec.build_config_space();
        self.configs.insert(key, config);

        for bar in bars {
            self.bars.insert((key, bar.index), bar);
        }
    }

    /// Removes a device at `addr` from the simulated bus (simulates hot unplug).
    pub fn remove_device(&mut self, addr: PciAddress) -> bool {
        let key = addr.config_key();
        let removed = self.configs.remove(&key).is_some();
        if removed {
            for slot in 0..6 {
                self.bars.remove(&(key, slot));
                self.sizing_state.remove(&(key, slot));
            }
        }
        removed
    }

    /// Pre-populates default realistic test fixtures for QEOS V.04.
    fn populate_default_fixtures(&mut self) {
        // 1. Host Bridge at 0000:00:00.0
        self.add_device(SimulatedDeviceSpec {
            address: PciAddress::new(0, 0, 0),
            vendor_id: 0x8086,
            device_id: 0x1234,
            revision: 0x01,
            class: PciClass::new(0x06, 0x00, 0x00),
            header_type: PciHeaderType::Type0,
            multifunction: false,
            subsystem_vendor_id: None,
            subsystem_id: None,
            command: command_bits::BUS_MASTER | command_bits::MEMORY_SPACE,
            status: 0,
            bars: Vec::new(),
            capabilities: Vec::new(),
        });

        // 2. NVMe Storage Controller at 0000:00:01.0
        self.add_device(SimulatedDeviceSpec {
            address: PciAddress::new(0, 1, 0),
            vendor_id: 0x1B36,
            device_id: 0x0010,
            revision: 0x02,
            class: PciClass::new(0x01, 0x08, 0x02),
            header_type: PciHeaderType::Type0,
            multifunction: false,
            subsystem_vendor_id: Some(0x1B36),
            subsystem_id: Some(0x0001),
            command: command_bits::BUS_MASTER | command_bits::MEMORY_SPACE,
            status: 0,
            bars: vec![SimulatedBarSpec {
                index: 0,
                kind: BarKind::Memory64,
                base: 0x1000_0000,
                size: 0x4000, // 16 KiB
                prefetchable: false,
            }],
            capabilities: vec![PciCapability::Msix(MsixInfo {
                message_control: msix_control::ENABLE | 15,
                table_bar: 0,
                table_offset: 0x2000,
                pba_bar: 0,
                enabled: true,
                function_masked: false,
            })],
        });

        // 3. GPU / 3D Controller at 0000:00:02.0
        self.add_device(SimulatedDeviceSpec {
            address: PciAddress::new(0, 2, 0),
            vendor_id: 0x10DE,
            device_id: 0x1EB8,
            revision: 0xA1,
            class: PciClass::new(0x03, 0x00, 0x00),
            header_type: PciHeaderType::Type0,
            multifunction: false,
            subsystem_vendor_id: Some(0x10DE),
            subsystem_id: Some(0x1234),
            command: command_bits::BUS_MASTER | command_bits::MEMORY_SPACE,
            status: 0,
            bars: vec![
                SimulatedBarSpec {
                    index: 0,
                    kind: BarKind::Memory64,
                    base: 0x2000_0000,
                    size: 0x1_0000, // 64 KiB
                    prefetchable: true,
                },
                SimulatedBarSpec {
                    index: 2,
                    kind: BarKind::Memory32,
                    base: 0x3000_0000,
                    size: 0x4000, // 16 KiB
                    prefetchable: false,
                },
            ],
            capabilities: vec![
                PciCapability::PowerManagement { version: 3 },
                PciCapability::Msi(MsiInfo::from_control(
                    msi_control::ENABLE | (2 << 1) | msi_control::ADDRESS_64BIT,
                )),
                PciCapability::Pcie(PcieCapability {
                    version: 2,
                    device_port_type: 0,
                    slot_implemented: true,
                    hotplug_capable: true,
                    hotplug_surprise: false,
                    link_speed: LinkSpeed::Gen4,
                    link_width: 16,
                }),
            ],
        });

        // 4. Ethernet NIC at 0000:00:03.0
        self.add_device(SimulatedDeviceSpec {
            address: PciAddress::new(0, 3, 0),
            vendor_id: 0x8086,
            device_id: 0x100E,
            revision: 0x03,
            class: PciClass::new(0x02, 0x00, 0x00),
            header_type: PciHeaderType::Type0,
            multifunction: false,
            subsystem_vendor_id: Some(0x8086),
            subsystem_id: Some(0x001E),
            command: command_bits::BUS_MASTER | command_bits::MEMORY_SPACE,
            status: 0,
            bars: vec![SimulatedBarSpec {
                index: 0,
                kind: BarKind::Memory32,
                base: 0x4000_0000,
                size: 0x8000, // 32 KiB
                prefetchable: false,
            }],
            capabilities: vec![PciCapability::Msi(MsiInfo::from_control(
                msi_control::ENABLE,
            ))],
        });

        // 5. Energy Telemetry Sensor at 0000:00:04.0
        self.add_device(SimulatedDeviceSpec {
            address: PciAddress::new(0, 4, 0),
            vendor_id: 0x51E0,
            device_id: 0x0001,
            revision: 0x01,
            class: PciClass::new(0x11, 0x80, 0x00),
            header_type: PciHeaderType::Type0,
            multifunction: false,
            subsystem_vendor_id: Some(0x51E0),
            subsystem_id: Some(0x0001),
            command: command_bits::BUS_MASTER | command_bits::MEMORY_SPACE,
            status: 0,
            bars: vec![SimulatedBarSpec {
                index: 0,
                kind: BarKind::Memory32,
                base: 0x5000_0000,
                size: 0x1000, // 4 KiB
                prefetchable: false,
            }],
            capabilities: vec![PciCapability::Msi(MsiInfo::from_control(
                msi_control::ENABLE,
            ))],
        });
    }
}

impl Default for SimulatedPciBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl PciConfigBackend for SimulatedPciBackend {
    fn label(&self) -> &'static str {
        "simulated-pci"
    }

    fn source(&self) -> BackendSource {
        BackendSource::Simulated
    }

    fn read(&mut self, addr: PciAddress, offset: u16, width: ConfigWidth) -> PciResult<u32> {
        let key = addr.config_key();
        let config = match self.configs.get(&key) {
            Some(c) => c,
            None => return Err(PciError::DeviceAbsent { addr }),
        };

        let off = offset as usize;
        let w = width.bytes() as usize;
        if off + w > 256 {
            return Err(PciError::UnsupportedAccess {
                addr,
                offset,
                width,
            });
        }

        // BAR Sizing response
        for slot in 0..6u8 {
            if let Some(bar_off) = offsets::bar(slot) {
                if offset == bar_off && self.sizing_state.contains(&(key, slot)) {
                    if let Some(spec) = self.bars.get(&(key, slot)) {
                        let size_mask = !(spec.size.wrapping_sub(1)) as u32;
                        let attr_mask = match spec.kind {
                            BarKind::Memory32 | BarKind::Memory64 => 0x0F,
                            BarKind::IoPort => 0x03,
                            BarKind::Unimplemented => 0,
                        };
                        return Ok(size_mask & !attr_mask);
                    }
                }
            }
        }

        let slice = &config[off..off + w];
        let val = match width {
            ConfigWidth::U8 => slice[0] as u32,
            ConfigWidth::U16 => {
                let mut b = [0u8; 2];
                b.copy_from_slice(slice);
                u16::from_le_bytes(b) as u32
            }
            ConfigWidth::U32 => {
                let mut b = [0u8; 4];
                b.copy_from_slice(slice);
                u32::from_le_bytes(b)
            }
        };

        Ok(val)
    }

    fn write(
        &mut self,
        addr: PciAddress,
        offset: u16,
        width: ConfigWidth,
        value: u32,
    ) -> PciResult<()> {
        let key = addr.config_key();
        let config = match self.configs.get_mut(&key) {
            Some(c) => c,
            None => return Err(PciError::DeviceAbsent { addr }),
        };

        let off = offset as usize;
        let w = width.bytes() as usize;
        if off + w > 256 {
            return Err(PciError::UnsupportedAccess {
                addr,
                offset,
                width,
            });
        }

        // Check for BAR sizing write (writing 0xFFFFFFFF to BAR offset)
        for slot in 0..6u8 {
            if let Some(bar_off) = offsets::bar(slot) {
                if offset == bar_off && width == ConfigWidth::U32 {
                    if value == 0xFFFF_FFFF {
                        self.sizing_state.insert((key, slot));
                    } else {
                        self.sizing_state.remove(&(key, slot));
                    }
                }
            }
        }

        let bytes = value.to_le_bytes();
        config[off..off + w].copy_from_slice(&bytes[..w]);
        Ok(())
    }

    fn supports_bar_sizing(&self) -> bool {
        true
    }

    fn iommu_enforced(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pci::config::PciConfigReadExt;

    #[test]
    fn simulated_backend_reports_devices_and_absent() {
        let mut backend = SimulatedPciBackend::new();
        let root = PciAddress::new(0, 0, 0);
        let vendor = backend.read_u16(root, 0).unwrap();
        assert_eq!(vendor, 0x8086);

        let absent = PciAddress::new(0, 31, 0);
        assert!(matches!(
            backend.read_u16(absent, 0),
            Err(PciError::DeviceAbsent { .. })
        ));
    }

    #[test]
    fn simulated_backend_supports_bar_sizing() {
        let mut backend = SimulatedPciBackend::new();
        let nvme = PciAddress::new(0, 1, 0);

        // Read initial BAR0
        let orig_bar0 = backend.read_u32(nvme, offsets::BAR0).unwrap();
        assert_ne!(orig_bar0, 0);

        // Write all ones
        backend.write_u32(nvme, offsets::BAR0, 0xFFFF_FFFF).unwrap();
        let mask = backend.read_u32(nvme, offsets::BAR0).unwrap();
        // 16 KiB size mask: !(16384 - 1) & !0x0F = !0x3FFF & !0x0F = 0xFFFF_C000
        assert_eq!(mask, 0xFFFF_C000);

        // Restore BAR0
        backend.write_u32(nvme, offsets::BAR0, orig_bar0).unwrap();
        assert_eq!(backend.read_u32(nvme, offsets::BAR0).unwrap(), orig_bar0);
    }
}
