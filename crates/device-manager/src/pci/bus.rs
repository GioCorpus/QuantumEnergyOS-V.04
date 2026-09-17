//! PCI/PCIe bus walker and device enumerator (§5).
//!
//! [`PciBus`] discovers devices by scanning configuration space through any
//! [`PciConfigBackend`]. It performs genuine configuration-space decoding:
//!
//! - Multi-function probing (function 0 checked first, functions 1..7 only if
//!   bit 7 of the header type register is set).
//! - Base Address Register (BAR) decoding and write-all-ones sizing.
//! - Linked-list capability walking with cycle detection and boundary bounds.
//!
//! No `unsafe`, no direct hardware I/O.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::error::{PciError, PciResult};

use super::address::{PciAddress, PciClass, PciHeaderType};
use super::bar::{decode, BarKind, PciBar};
use super::capability::{
    ids, msix_control, LinkSpeed, MsiInfo, MsixInfo, PciCapability, PcieCapability,
};
use super::config::{command_bits, offsets, status_bits, PciConfigBackend, PciConfigReadExt};
use super::device::{PciDevice, PciDeviceInfo};

/// Range of bus numbers to scan during enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BusRange {
    /// Starting bus number (inclusive).
    pub start: u8,
    /// Ending bus number (inclusive).
    pub end: u8,
}

impl BusRange {
    /// Scan a single bus (e.g. root bus 0).
    pub const fn single(bus: u8) -> Self {
        Self {
            start: bus,
            end: bus,
        }
    }

    /// Scan all 256 buses in the PCI domain.
    pub const fn all() -> Self {
        Self { start: 0, end: 255 }
    }

    /// Scan an explicit range.
    pub const fn range(start: u8, end: u8) -> Self {
        Self { start, end }
    }

    /// Iterator over the bus numbers in the range.
    pub fn iter(&self) -> impl Iterator<Item = u8> {
        self.start..=self.end
    }
}

impl Default for BusRange {
    fn default() -> Self {
        Self::single(0)
    }
}

/// PCI/PCIe bus driver and enumerator.
pub struct PciBus<B: PciConfigBackend> {
    backend: B,
    range: BusRange,
}

impl<B: PciConfigBackend> PciBus<B> {
    /// Creates a new `PciBus` with the given backend and default root bus range (bus 0).
    pub fn new(backend: B) -> Self {
        Self {
            backend,
            range: BusRange::default(),
        }
    }

    /// Creates a new `PciBus` with an explicit bus scan range.
    pub fn with_bus_range(backend: B, range: BusRange) -> Self {
        Self { backend, range }
    }

    /// Read-only access to the underlying backend.
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Mutable access to the underlying backend.
    pub fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }

    /// Consumes the bus and returns the underlying backend.
    pub fn into_backend(self) -> B {
        self.backend
    }

    /// Current bus scan range.
    pub fn range(&self) -> BusRange {
        self.range
    }

    /// Updates the bus scan range.
    pub fn set_range(&mut self, range: BusRange) {
        self.range = range;
    }

    /// Reads and decodes a single PCI function at `addr`.
    ///
    /// Returns [`PciError::DeviceAbsent`] if no function responds.
    pub fn read_device(&mut self, addr: PciAddress) -> PciResult<PciDevice> {
        addr.validate()?;

        let vendor_id = self.backend.read_u16(addr, offsets::VENDOR_ID)?;
        if vendor_id == 0xFFFF || vendor_id == 0x0000 {
            return Err(PciError::DeviceAbsent { addr });
        }

        let device_id = self.backend.read_u16(addr, offsets::DEVICE_ID)?;
        let command = self.backend.read_u16(addr, offsets::COMMAND)?;
        let status = self.backend.read_u16(addr, offsets::STATUS)?;
        let revision = self.backend.read_u8(addr, offsets::REVISION_ID)?;
        let prog_if = self.backend.read_u8(addr, offsets::PROG_IF)?;
        let subclass = self.backend.read_u8(addr, offsets::SUBCLASS)?;
        let class_base = self.backend.read_u8(addr, offsets::CLASS_CODE)?;
        let raw_header = self.backend.read_u8(addr, offsets::HEADER_TYPE)?;

        let header_type = PciHeaderType::from_raw(raw_header);
        let multifunction = PciHeaderType::is_multifunction(raw_header);
        let class = PciClass::new(class_base, subclass, prog_if);

        let (subsystem_vendor_id, subsystem_id) = match header_type {
            PciHeaderType::Type0 => {
                let svid = self.backend.read_u16(addr, offsets::SUBSYSTEM_VENDOR_ID)?;
                let sid = self.backend.read_u16(addr, offsets::SUBSYSTEM_ID)?;
                (Some(svid), Some(sid))
            }
            _ => (None, None),
        };

        let info = PciDeviceInfo {
            address: addr,
            vendor_id,
            device_id,
            revision,
            class,
            header_type,
            multifunction,
            subsystem_vendor_id,
            subsystem_id,
            command,
            status,
        };

        // BAR Decoding & Sizing
        let bars = self.decode_bars(addr, header_type)?;

        // Capability List Walking
        let capabilities = if status & status_bits::CAPABILITY_LIST != 0 {
            self.decode_capabilities(addr)?
        } else {
            Vec::new()
        };

        Ok(PciDevice::new(info, bars, capabilities))
    }

    /// Decodes all BARs for a device function according to its header layout.
    fn decode_bars(
        &mut self,
        addr: PciAddress,
        header_type: PciHeaderType,
    ) -> PciResult<Vec<PciBar>> {
        let bar_slots = header_type.bar_slots();
        let mut bars = Vec::new();
        let supports_sizing = self.backend.supports_bar_sizing();

        let mut slot = 0u8;
        while slot < bar_slots {
            let offset = match offsets::bar(slot) {
                Some(o) => o,
                None => break,
            };

            let raw_low = self.backend.read_u32(addr, offset)?;
            if raw_low == 0 {
                // Unimplemented or zero base
                bars.push(PciBar::new(slot, BarKind::Unimplemented, 0, 0, false));
                slot += 1;
                continue;
            }

            let is_mem64 = (raw_low & 0x01 == 0) && ((raw_low & 0x06) == 0x04);
            let raw_high = if is_mem64 && slot + 1 < bar_slots {
                let high_offset = offset + 4;
                Some(self.backend.read_u32(addr, high_offset)?)
            } else {
                None
            };

            let mut bar = decode::from_raw(slot, raw_low, raw_high);

            // Sizing probe (PCI write-all-ones algorithm)
            if supports_sizing && bar.is_memory() {
                // Write all ones to probe size
                self.backend.write_u32(addr, offset, 0xFFFF_FFFF)?;
                let mask_low = self.backend.read_u32(addr, offset)?;
                // Restore original low dword
                self.backend.write_u32(addr, offset, raw_low)?;

                let mask_high = if is_mem64 && raw_high.is_some() {
                    let high_offset = offset + 4;
                    self.backend.write_u32(addr, high_offset, 0xFFFF_FFFF)?;
                    let m_high = self.backend.read_u32(addr, high_offset)?;
                    self.backend
                        .write_u32(addr, high_offset, raw_high.unwrap_or(0))?;
                    Some(m_high)
                } else {
                    None
                };

                let size = decode::size_from_mask(mask_low, mask_high);
                bar = bar.with_size(size);
            }

            let consumed_slots = decode::dwords(bar.kind);
            bars.push(bar);
            slot += consumed_slots;
        }

        Ok(bars)
    }

    /// Walks the PCI capability list starting at offset 0x34.
    fn decode_capabilities(&mut self, addr: PciAddress) -> PciResult<Vec<PciCapability>> {
        let mut caps = Vec::new();
        let mut seen = HashSet::new();

        let mut ptr = self.backend.read_u8(addr, offsets::CAPABILITY_POINTER)? & 0xFC;
        let mut count = 0;

        while ptr != 0 {
            if ptr < 0x40 {
                return Err(PciError::MalformedCapabilityList {
                    addr,
                    reason: format!(
                        "capability pointer 0x{ptr:02x} is below the 0x40 legacy header limit"
                    ),
                });
            }

            if !seen.insert(ptr) {
                return Err(PciError::MalformedCapabilityList {
                    addr,
                    reason: format!("cycle detected at capability pointer 0x{ptr:02x}"),
                });
            }

            count += 1;
            if count > 48 {
                return Err(PciError::MalformedCapabilityList {
                    addr,
                    reason: "capability list exceeded maximum length (48)".to_string(),
                });
            }

            let cap_id = self.backend.read_u8(addr, ptr as u16)?;
            let next_ptr = self.backend.read_u8(addr, (ptr as u16) + 1)? & 0xFC;

            let decoded = match cap_id {
                ids::POWER_MANAGEMENT => {
                    let pm_cap = self.backend.read_u16(addr, (ptr as u16) + 2)?;
                    PciCapability::PowerManagement {
                        version: (pm_cap & 0x07) as u8,
                    }
                }
                ids::MSI => {
                    let msg_ctrl = self.backend.read_u16(addr, (ptr as u16) + 2)?;
                    PciCapability::Msi(MsiInfo::from_control(msg_ctrl))
                }
                ids::MSI_X => {
                    let msg_ctrl = self.backend.read_u16(addr, (ptr as u16) + 2)?;
                    let table_val = self.backend.read_u32(addr, (ptr as u16) + 4)?;
                    let pba_val = self.backend.read_u32(addr, (ptr as u16) + 8)?;

                    PciCapability::Msix(MsixInfo {
                        message_control: msg_ctrl,
                        table_bar: (table_val & 0x07) as u8,
                        table_offset: table_val & !0x07,
                        pba_bar: (pba_val & 0x07) as u8,
                        enabled: msg_ctrl & msix_control::ENABLE != 0,
                        function_masked: msg_ctrl & msix_control::FUNCTION_MASK != 0,
                    })
                }
                ids::PCI_EXPRESS => {
                    let pcie_caps = self.backend.read_u16(addr, (ptr as u16) + 2)?;
                    let dev_port_type = ((pcie_caps >> 4) & 0x0F) as u8;
                    let slot_implemented = (pcie_caps & (1 << 8)) != 0;

                    let link_caps = self.backend.read_u32(addr, (ptr as u16) + 12)?;
                    let link_speed = LinkSpeed::from_code((link_caps & 0x0F) as u8);
                    let link_width = ((link_caps >> 4) & 0x3F) as u8;

                    let (hotplug_capable, hotplug_surprise) = if slot_implemented {
                        let slot_caps = self.backend.read_u32(addr, (ptr as u16) + 20)?;
                        ((slot_caps & (1 << 6)) != 0, (slot_caps & (1 << 5)) != 0)
                    } else {
                        (false, false)
                    };

                    PciCapability::Pcie(PcieCapability {
                        version: (pcie_caps & 0x0F) as u8,
                        device_port_type: dev_port_type,
                        slot_implemented,
                        hotplug_capable,
                        hotplug_surprise,
                        link_speed,
                        link_width,
                    })
                }
                ids::ADDRESS_TRANSLATION => PciCapability::AddressTranslation,
                ids::HOT_PLUG => PciCapability::HotPlug,
                ids::VENDOR_SPECIFIC => PciCapability::VendorSpecific {
                    id: ids::VENDOR_SPECIFIC,
                },
                other => PciCapability::Other {
                    id: other,
                    offset: ptr as u16,
                },
            };

            caps.push(decoded);
            ptr = next_ptr;
        }

        Ok(caps)
    }

    /// Enumerates all devices on the configured bus range.
    ///
    /// Scans devices 0..=31 and functions 0..=7 (skipping functions 1..7 if function 0
    /// is not multifunction).
    pub fn enumerate(&mut self) -> PciResult<Vec<PciDevice>> {
        let mut devices = Vec::new();

        for bus in self.range.iter() {
            for dev in 0..=PciAddress::MAX_DEVICE {
                let func0_addr = PciAddress::new(bus, dev, 0);
                match self.read_device(func0_addr) {
                    Ok(dev0) => {
                        let is_mf = dev0.info.multifunction;
                        devices.push(dev0);

                        if is_mf {
                            for func in 1..=PciAddress::MAX_FUNCTION {
                                let func_addr = PciAddress::new(bus, dev, func);
                                if let Ok(func_dev) = self.read_device(func_addr) {
                                    devices.push(func_dev);
                                }
                            }
                        }
                    }
                    Err(PciError::DeviceAbsent { .. }) => continue,
                    Err(e) => return Err(e),
                }
            }
        }

        Ok(devices)
    }

    /// Enables the bus-master bit in the device's command register.
    pub fn enable_bus_master(&mut self, addr: PciAddress) -> PciResult<()> {
        let cmd = self.backend.read_u16(addr, offsets::COMMAND)?;
        if cmd & command_bits::BUS_MASTER == 0 {
            self.backend
                .write_u16(addr, offsets::COMMAND, cmd | command_bits::BUS_MASTER)?;
        }
        Ok(())
    }

    /// Enables the memory-space decoding bit in the device's command register.
    pub fn enable_memory_space(&mut self, addr: PciAddress) -> PciResult<()> {
        let cmd = self.backend.read_u16(addr, offsets::COMMAND)?;
        if cmd & command_bits::MEMORY_SPACE == 0 {
            self.backend
                .write_u16(addr, offsets::COMMAND, cmd | command_bits::MEMORY_SPACE)?;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pci::simulated::SimulatedPciBackend;

    #[test]
    fn bus_range_iterates_correctly() {
        let r = BusRange::range(0, 3);
        let buses: Vec<u8> = r.iter().collect();
        assert_eq!(buses, vec![0, 1, 2, 3]);

        let s = BusRange::single(5);
        assert_eq!(s.iter().collect::<Vec<u8>>(), vec![5]);
    }

    #[test]
    fn pci_bus_enumerates_simulated_devices() {
        let backend = SimulatedPciBackend::new();
        let mut bus = PciBus::new(backend);
        let devices = bus.enumerate().unwrap();

        assert!(!devices.is_empty());
        // Verify host bridge exists at 0000:00:00.0
        let host_bridge = devices
            .iter()
            .find(|d| d.address() == PciAddress::new(0, 0, 0));
        assert!(host_bridge.is_some());
    }
}
