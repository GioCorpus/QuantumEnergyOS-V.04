//! PCI configuration-space transport (§5).
//!
//! [`PciConfigBackend`] is the *only* way this crate reads or writes PCI
//! configuration space. Everything else (enumeration, BAR probing, capability
//! walking) is written against this trait, which is what makes the bus layer
//! testable on a host and replaceable by a real kernel transport.
//!
//! Backends must be honest about what they are:
//!
//! - [`BackendSource::Simulated`] — a software model (tests/CI).
//! - [`BackendSource::Physical`] — real hardware behind a kernel/OS service.
//! - [`BackendSource::Undeclared`] — the default: not trusted for hardware claims.

use serde::{Deserialize, Serialize};

use crate::error::{PciError, PciResult};

use super::address::PciAddress;

/// Width of one configuration-space access.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConfigWidth {
    /// 8-bit access.
    U8,
    /// 16-bit access.
    U16,
    /// 32-bit access.
    U32,
}

impl ConfigWidth {
    /// Access size in bytes.
    pub const fn bytes(self) -> u8 {
        match self {
            ConfigWidth::U8 => 1,
            ConfigWidth::U16 => 2,
            ConfigWidth::U32 => 4,
        }
    }

    /// Stable label used in diagnostics.
    pub const fn label(self) -> &'static str {
        match self {
            ConfigWidth::U8 => "u8",
            ConfigWidth::U16 => "u16",
            ConfigWidth::U32 => "u32",
        }
    }
}

/// Configuration-space register offsets (PCI Local Bus 3.0 §6.1 / PCIe §7.5).
///
/// Only offsets this crate actually uses are listed. Nothing here implies access
/// to undocumented registers.
pub mod offsets {
    /// Vendor id (16-bit).
    pub const VENDOR_ID: u16 = 0x00;
    /// Device id (16-bit).
    pub const DEVICE_ID: u16 = 0x02;
    /// Command register (16-bit).
    pub const COMMAND: u16 = 0x04;
    /// Status register (16-bit).
    pub const STATUS: u16 = 0x06;
    /// Revision id (8-bit).
    pub const REVISION_ID: u16 = 0x08;
    /// Programming interface (8-bit).
    pub const PROG_IF: u16 = 0x09;
    /// Sub-class code (8-bit).
    pub const SUBCLASS: u16 = 0x0A;
    /// Base class code (8-bit).
    pub const CLASS_CODE: u16 = 0x0B;
    /// Header type (8-bit; bit 7 = multifunction).
    pub const HEADER_TYPE: u16 = 0x0E;
    /// First BAR slot (dword).
    pub const BAR0: u16 = 0x10;
    /// Subsystem vendor id (16-bit, type 0/1 header).
    pub const SUBSYSTEM_VENDOR_ID: u16 = 0x2C;
    /// Subsystem id (16-bit, type 0/1 header).
    pub const SUBSYSTEM_ID: u16 = 0x2E;
    /// Capability list pointer (8-bit, offset into the legacy 256-byte space).
    pub const CAPABILITY_POINTER: u16 = 0x34;
    /// Interrupt line (8-bit).
    pub const INTERRUPT_LINE: u16 = 0x3C;
    /// Interrupt pin (8-bit).
    pub const INTERRUPT_PIN: u16 = 0x3D;
    /// Number of BAR dwords in a type-0 header.
    pub const BAR_SLOTS: u8 = 6;
    /// Offset of BAR slot `index`, or `None` when out of range.
    pub const fn bar(index: u8) -> Option<u16> {
        if index < BAR_SLOTS {
            Some(BAR0 + (index as u16) * 4)
        } else {
            None
        }
    }
}

/// Bits of the command register (offset [`offsets::COMMAND`]).
pub mod command_bits {
    /// Device responds to I/O space accesses.
    pub const IO_SPACE: u16 = 1 << 0;
    /// Device responds to memory space accesses.
    pub const MEMORY_SPACE: u16 = 1 << 1;
    /// Device may act as a bus master (required for DMA).
    pub const BUS_MASTER: u16 = 1 << 2;
    /// Special cycle monitoring.
    pub const SPECIAL_CYCLES: u16 = 1 << 3;
    /// Memory write-and-invalidate enable.
    pub const MEMORY_WRITE_INVALIDATE: u16 = 1 << 4;
    /// VGA palette snoop.
    pub const VGA_PALETTE_SNOOP: u16 = 1 << 5;
    /// Parity error response.
    pub const PARITY_ERROR_RESPONSE: u16 = 1 << 6;
    /// SERR# driver enable.
    pub const SERR_ENABLE: u16 = 1 << 8;
    /// Interrupt disable (masks legacy INTx).
    pub const INTERRUPT_DISABLE: u16 = 1 << 10;
}

/// Bits of the status register (offset [`offsets::STATUS`]).
pub mod status_bits {
    /// The function implements a capability list (pointer at `0x34`).
    pub const CAPABILITY_LIST: u16 = 1 << 4;
    /// The function is capable of 66 MHz operation.
    pub const FAST_BACK_TO_BACK: u16 = 1 << 7;
}

/// Size of the legacy configuration space, in bytes.
pub const LEGACY_CONFIG_LEN: u16 = 0x100;
/// Size of the PCIe extended configuration space, in bytes.
pub const PCIE_CONFIG_LEN: u16 = 0x1000;
/// Value read for the vendor id when no function responds.
pub const VENDOR_ID_ABSENT: u16 = 0xFFFF;

/// How a backend reaches configuration space (§28: never assume hardware).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BackendSource {
    /// Software model. Nothing reached through it is physical hardware.
    #[default]
    Simulated,
    /// Real hardware behind a kernel/OS service.
    Physical,
    /// Not declared by the backend: treated as untrusted for hardware claims.
    Undeclared,
}

impl BackendSource {
    /// True when the backend can only produce simulated data.
    pub const fn is_simulated(self) -> bool {
        matches!(self, BackendSource::Simulated)
    }

    /// Stable label used in snapshots and CLI output.
    pub const fn label(self) -> &'static str {
        match self {
            BackendSource::Simulated => "simulated",
            BackendSource::Physical => "physical",
            BackendSource::Undeclared => "undeclared",
        }
    }
}

impl std::fmt::Display for BackendSource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// Transport used to read/write PCI configuration space.
///
/// Implementations decide how the bytes are reached (simulated model, kernel
/// service, firmware table). They must not perform `unsafe` MMIO: MMIO belongs to
/// [`crate::pci::bar::MmioMapper`], configuration space to this trait.
pub trait PciConfigBackend {
    /// Short backend label, e.g. `simulated` or `kernel-pcie`.
    fn label(&self) -> &'static str;

    /// Declares whether this backend models or reaches real hardware.
    fn source(&self) -> BackendSource {
        BackendSource::Undeclared
    }

    /// Reads `width` bytes at `offset`.
    ///
    /// Must return [`PciError::DeviceAbsent`] when nothing responds at `addr`, so
    /// enumeration can skip the function instead of inventing a device.
    fn read(&mut self, addr: PciAddress, offset: u16, width: ConfigWidth) -> PciResult<u32>;

    /// Writes `width` bytes at `offset`.
    ///
    /// Writes exist for documented setup steps only: BAR sizing, command-register
    /// enable bits, and (later) MSI/MSI-X programming. Backends should keep the
    /// set of writable offsets as small as their model allows.
    fn write(
        &mut self,
        addr: PciAddress,
        offset: u16,
        width: ConfigWidth,
        value: u32,
    ) -> PciResult<()>;

    /// True when the PCI write-all-ones BAR sizing algorithm is legal here.
    ///
    /// When false, the enumerator leaves BAR sizes at `0` (unknown) instead of
    /// guessing, and MMIO mapping of those BARs fails with
    /// [`PciError::BarSizeUnknown`].
    fn supports_bar_sizing(&self) -> bool {
        false
    }

    /// True when an IOMMU translation domain is enforced for this bus.
    ///
    /// Only a kernel backend can answer `true`; the simulated backend does not.
    fn iommu_enforced(&self) -> bool {
        false
    }
}

/// Typed convenience accessors over any [`PciConfigBackend`].
pub trait PciConfigReadExt: PciConfigBackend {
    /// Reads a byte.
    fn read_u8(&mut self, addr: PciAddress, offset: u16) -> PciResult<u8> {
        Ok(self.read(addr, offset, ConfigWidth::U8)? as u8)
    }

    /// Reads a 16-bit word.
    fn read_u16(&mut self, addr: PciAddress, offset: u16) -> PciResult<u16> {
        Ok(self.read(addr, offset, ConfigWidth::U16)? as u16)
    }

    /// Reads a 32-bit dword.
    fn read_u32(&mut self, addr: PciAddress, offset: u16) -> PciResult<u32> {
        self.read(addr, offset, ConfigWidth::U32)
    }

    /// Writes a 16-bit word.
    fn write_u16(&mut self, addr: PciAddress, offset: u16, value: u16) -> PciResult<()> {
        self.write(addr, offset, ConfigWidth::U16, u32::from(value))
    }

    /// Writes a 32-bit dword.
    fn write_u32(&mut self, addr: PciAddress, offset: u16, value: u32) -> PciResult<()> {
        self.write(addr, offset, ConfigWidth::U32, value)
    }
}

impl<T: PciConfigBackend + ?Sized> PciConfigReadExt for T {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn widths_report_their_size() {
        assert_eq!(ConfigWidth::U8.bytes(), 1);
        assert_eq!(ConfigWidth::U16.bytes(), 2);
        assert_eq!(ConfigWidth::U32.bytes(), 4);
        assert_eq!(ConfigWidth::U32.label(), "u32");
    }

    #[test]
    fn bar_offsets_stay_inside_the_legacy_header() {
        assert_eq!(offsets::bar(0), Some(0x10));
        assert_eq!(offsets::bar(5), Some(0x24));
        assert_eq!(offsets::bar(6), None);
        for index in 0..offsets::BAR_SLOTS {
            if let Some(offset) = offsets::bar(index) {
                assert!(offset + 4 <= offsets::SUBSYSTEM_VENDOR_ID);
            }
        }
    }

    #[test]
    fn command_bits_are_distinct() {
        let bits = [
            command_bits::IO_SPACE,
            command_bits::MEMORY_SPACE,
            command_bits::BUS_MASTER,
            command_bits::SPECIAL_CYCLES,
            command_bits::MEMORY_WRITE_INVALIDATE,
            command_bits::VGA_PALETTE_SNOOP,
            command_bits::PARITY_ERROR_RESPONSE,
            command_bits::SERR_ENABLE,
            command_bits::INTERRUPT_DISABLE,
        ];
        let mut seen = 0u16;
        for bit in bits {
            assert_eq!(seen & bit, 0, "duplicate command bit {bit:#06x}");
            seen |= bit;
        }
    }

    #[test]
    fn backend_source_default_is_simulated_and_labeled() {
        assert_eq!(BackendSource::default(), BackendSource::Simulated);
        assert!(BackendSource::Simulated.is_simulated());
        assert!(!BackendSource::Physical.is_simulated());
        assert_eq!(BackendSource::Undeclared.to_string(), "undeclared");
    }

    #[test]
    fn sentinel_values_match_the_specifications() {
        assert_eq!(VENDOR_ID_ABSENT, 0xFFFF);
        assert_eq!(LEGACY_CONFIG_LEN, 256);
        assert_eq!(PCIE_CONFIG_LEN, 4096);
        assert_eq!(status_bits::CAPABILITY_LIST, 0x10);
    }
}