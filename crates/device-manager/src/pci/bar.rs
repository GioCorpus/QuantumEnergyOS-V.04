//! BAR decoding and the bounded MMIO abstraction (§5).
//!
//! Two halves:
//!
//! 1. **Decoding** — turns raw BAR dwords into [`PciBar`] values. The write-all-ones
//!    sizing algorithm of the PCI Local Bus specification (§6.2.5.1) is implemented
//!    in [`decode`]; when a backend refuses sizing, the size stays `0` (unknown)
//!    instead of being guessed.
//! 2. **Access** — [`MmioRegion`] is the only way to read or write device memory.
//!    Regions are obtained from an [`MmioMapper`]; the bundled mapper is
//!    [`SimulatedMmioMapper`], a memory-backed model that is explicitly flagged as
//!    simulated. Real mapping (kernel page tables, IOMMU domains) is FUTURE.
//!
//! There is no `unsafe`, no raw pointer and no port I/O in this file.

use std::collections::HashSet;

use serde::{Deserialize, Serialize};

use crate::error::{PciError, PciResult};

use super::address::PciAddress;

/// Kind of a Base Address Register, as encoded in its low bits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BarKind {
    /// 32-bit memory-mapped BAR (type bits `0b000`).
    Memory32,
    /// 64-bit memory-mapped BAR, consuming two dwords (type bits `0b100`).
    Memory64,
    /// I/O port BAR (bit 0 set). Not mappable as memory.
    IoPort,
    /// The slot reads back zero: the function does not implement this BAR.
    Unimplemented,
}

impl BarKind {
    /// True for memory BAR kinds.
    pub const fn is_memory(self) -> bool {
        matches!(self, BarKind::Memory32 | BarKind::Memory64)
    }

    /// Stable label used in snapshots and diagnostics.
    pub const fn label(self) -> &'static str {
        match self {
            BarKind::Memory32 => "mem32",
            BarKind::Memory64 => "mem64",
            BarKind::IoPort => "io",
            BarKind::Unimplemented => "none",
        }
    }
}

impl std::fmt::Display for BarKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.label())
    }
}

/// A decoded Base Address Register.
///
/// `size == 0` means "unknown", which happens when the backend cannot perform BAR
/// sizing. Such a BAR is never mapped and never reported as usable memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PciBar {
    /// BAR slot index (0..6 for type-0 headers).
    pub index: u8,
    /// Memory/IO kind.
    pub kind: BarKind,
    /// Address associated by firmware/OS.
    pub base: u64,
    /// Region size, or `0` when unknown.
    pub size: u64,
    /// Prefetchable bit (memory BARs only).
    pub prefetchable: bool,
}

impl PciBar {
    /// Builds a BAR from decoded fields.
    pub const fn new(index: u8, kind: BarKind, base: u64, size: u64, prefetchable: bool) -> Self {
        Self {
            index,
            kind,
            base,
            size,
            prefetchable,
        }
    }

    /// Returns the same BAR with a known size.
    pub const fn with_size(self, size: u64) -> Self {
        Self { size, ..self }
    }

    /// True when this BAR is memory-mapped.
    pub const fn is_memory(&self) -> bool {
        self.kind.is_memory()
    }

    /// True when the BAR can be handed to an [`MmioMapper`]: memory, sized, present.
    pub const fn is_mappable(&self) -> bool {
        self.is_memory() && self.size > 0
    }

    /// Highest address covered by the region, exclusive.
    pub const fn end(&self) -> u64 {
        self.base.saturating_add(self.size)
    }

    /// True when `offset` (relative to the BAR base) lies inside the region.
    pub const fn contains_offset(&self, offset: u64) -> bool {
        offset < self.size
    }
}

/// BAR decoding helpers derived from the PCI specification.
pub mod decode {
    use super::{BarKind, PciBar};

    /// Memory-space indicator bit (`0` means memory BAR).
    pub const MEMORY_INDICATOR: u32 = 1 << 0;
    /// Memory type field mask (bits 1..3).
    pub const MEMORY_TYPE_MASK: u32 = 0b111 << 1;
    /// Type field value selecting a 64-bit BAR.
    pub const MEMORY_TYPE_64BIT: u32 = 0b100 << 1;
    /// Prefetchable bit (bit 3).
    pub const PREFETCHABLE: u32 = 1 << 3;
    /// Address mask for a 32-bit memory BAR (low four bits are attributes).
    pub const MEMORY_BASE_MASK_32: u32 = !0x0F;
    /// Address mask for an I/O BAR (low two bits are attributes).
    pub const IO_BASE_MASK: u32 = !0x03;
    /// Mask applied to a memory BAR read-back before sizing.
    pub const SIZING_MASK_LOW: u32 = !0x0F;

    /// Decodes the *shape* of a BAR from its first dword.
    ///
    /// `upper` must be `Some(low dword of the high half)` for 64-bit BARs, which
    /// occupy a pair of dwords. The size is filled in later, during enumeration,
    /// by the write-all-ones probe (see [`super::decode::size_from_mask`]).
    pub fn from_raw(index: u8, raw: u32, upper: Option<u32>) -> PciBar {
        if raw == 0 {
            return PciBar::new(index, BarKind::Unimplemented, 0, 0, false);
        }
        if raw & MEMORY_INDICATOR != 0 {
            return PciBar::new(index, BarKind::IoPort, u64::from(raw & IO_BASE_MASK), 0, false);
        }
        let prefetchable = raw & PREFETCHABLE != 0;
        if raw & MEMORY_TYPE_MASK == MEMORY_TYPE_64BIT {
            let high = upper.unwrap_or(0);
            let base = u64::from(raw & MEMORY_BASE_MASK_32) | (u64::from(high) << 32);
            PciBar::new(index, BarKind::Memory64, base, 0, prefetchable)
        } else {
            PciBar::new(
                index,
                BarKind::Memory32,
                u64::from(raw & MEMORY_BASE_MASK_32),
                0,
                prefetchable,
            )
        }
    }

    /// Number of BAR dwords consumed by this kind (`2` for 64-bit memory BARs).
    pub const fn dwords(kind: BarKind) -> u8 {
        match kind {
            BarKind::Memory64 => 2,
            _ => 1,
        }
    }

    /// Derives the region size from the write-all-ones read-back values.
    ///
    /// Returns `0` when the slot is not implemented: every bit the function does
    /// not implement reads back as `0` after writing all ones.
    pub fn size_from_mask(mask_low: u32, mask_high: Option<u32>) -> u64 {
        if mask_low == 0 {
            return 0;
        }
        let low = u64::from(!mask_low & SIZING_MASK_LOW).wrapping_add(1);
        if low != 0 {
            return low & 0xFFFF_FFFF;
        }
        // 64-bit BAR whose low half is fully implemented: the size lives in the
        // upper dword (the standard probe yields zero below 4 GiB).
        match mask_high {
            Some(mask) => u64::from(!mask).wrapping_add(1) << 32,
            None => 0,
        }
    }
}

/// Validates an MMIO access against a region: natural alignment and bounds.
///
/// Every [`MmioRegion`] implementation must call this before touching memory, so
/// that out-of-range or misaligned accesses are errors instead of silent
/// corruption.
pub fn check_mmio_access(size: usize, offset: usize, width: usize) -> PciResult<()> {
    if width == 0 || offset % width != 0 {
        return Err(PciError::MmioMisaligned { offset, width });
    }
    match offset.checked_add(width) {
        Some(end) if end <= size => Ok(()),
        _ => Err(PciError::MmioOutOfRange {
            offset,
            width,
            size,
        }),
    }
}

/// Bounded read/write access to one mapped device memory region.
///
/// Implementations must be `Send` (a region may be handed to another thread) and
/// must reject out-of-range/misaligned access with [`PciError`]. They must not
/// expose raw pointers: this trait is the MMIO boundary of the device stack.
pub trait MmioRegion: Send {
    /// Short label, e.g. `simulated-mmio`.
    fn label(&self) -> &'static str;

    /// True when the region is a software model, not device memory.
    fn is_simulated(&self) -> bool;

    /// Region size in bytes.
    fn size(&self) -> usize;

    /// Reads a 32-bit word.
    fn read_u32(&self, offset: usize) -> PciResult<u32>;

    /// Reads a 64-bit word.
    fn read_u64(&self, offset: usize) -> PciResult<u64>;

    /// Writes a 32-bit word.
    fn write_u32(&mut self, offset: usize, value: u32) -> PciResult<()>;

    /// Writes a 64-bit word.
    fn write_u64(&mut self, offset: usize, value: u64) -> PciResult<()>;
}

/// Allocates [`MmioRegion`] handles for device BARs.
///
/// The bundled implementation is [`SimulatedMmioMapper`]. A real mapper (kernel
/// page tables, IOMMU domain, `/sys/bus/pci/.../resourceN` mapping) is FUTURE and
/// would implement this trait without changing any caller.
pub trait MmioMapper: Send {
    /// Short label, e.g. `simulated-mmio-mapper`.
    fn label(&self) -> &'static str;

    /// True when the mapper produces software models only.
    fn is_simulated(&self) -> bool;

    /// Maps a BAR and returns the region handle. Double mapping is an error.
    fn map(&mut self, device: PciAddress, bar: &PciBar) -> PciResult<Box<dyn MmioRegion>>;

    /// Releases a previously mapped BAR.
    fn unmap(&mut self, device: PciAddress, bar_index: u8) -> PciResult<()>;

    /// True when the (device, BAR) pair is currently mapped.
    fn is_mapped(&self, device: PciAddress, bar_index: u8) -> bool;
}

/// Memory-backed MMIO region used by tests and CI.
///
/// HONEST LIMITATIONS
/// - The buffer is private to the handle: writes never reach a device, and other
///   handles do not observe them.
/// - It models address-space access only; no interrupt, DMA or cache semantics.
#[derive(Debug)]
pub struct SimulatedMmio {
    bytes: Vec<u8>,
    active: bool,
}

impl SimulatedMmio {
    /// A zeroed region of `size` bytes.
    pub fn zeroed(size: usize) -> Self {
        Self {
            bytes: vec![0u8; size],
            active: true,
        }
    }

    /// A region initialized with `bytes`.
    pub fn from_bytes(bytes: Vec<u8>) -> Self {
        Self {
            bytes,
            active: true,
        }
    }

    /// Marks the region as released; further accesses fail.
    pub fn release(&mut self) {
        self.active = false;
    }

    /// Read-only view of the backing bytes (test/diagnostic helper).
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

impl MmioRegion for SimulatedMmio {
    fn label(&self) -> &'static str {
        "simulated-mmio"
    }

    fn is_simulated(&self) -> bool {
        true
    }

    fn size(&self) -> usize {
        self.bytes.len()
    }

    fn read_u32(&self, offset: usize) -> PciResult<u32> {
        check_mmio_access(self.size(), offset, 4)?;
        let mut raw = [0u8; 4];
        raw.copy_from_slice(&self.bytes[offset..offset + 4]);
        Ok(u32::from_le_bytes(raw))
    }

    fn read_u64(&self, offset: usize) -> PciResult<u64> {
        check_mmio_access(self.size(), offset, 8)?;
        let mut raw = [0u8; 8];
        raw.copy_from_slice(&self.bytes[offset..offset + 8]);
        Ok(u64::from_le_bytes(raw))
    }

    fn write_u32(&mut self, offset: usize, value: u32) -> PciResult<()> {
        check_mmio_access(self.size(), offset, 4)?;
        self.bytes[offset..offset + 4].copy_from_slice(&value.to_le_bytes());
        Ok(())
    }

    fn write_u64(&mut self, offset: usize, value: u64) -> PciResult<()> {
        check_mmio_access(self.size(), offset, 8)?;
        self.bytes[offset..offset + 8].copy_from_slice(&value.to_le_bytes());
        Ok(())
    }
}

/// MMIO mapper backed by process memory: the SIMULATED mapper (§28).
///
/// - Allocates a zeroed buffer per mapped BAR, capped by `max_region_bytes`.
/// - Tracks mapped `(device, BAR)` pairs so double mapping and unmapping an
///   unmapped BAR are errors instead of silent state corruption.
/// - Never touches device memory: there is no device. This is the host model used
///   by tests, CI and the `qeos-devices` diagnostic binary.
#[derive(Debug)]
pub struct SimulatedMmioMapper {
    mapped: HashSet<(PciAddress, u8)>,
    max_region_bytes: u64,
}

impl SimulatedMmioMapper {
    /// Default cap for a simulated region (64 KiB), keeping tests fast.
    pub const DEFAULT_MAX_REGION_BYTES: u64 = 64 * 1024;

    /// Mapper with the default cap.
    pub fn new() -> Self {
        Self {
            mapped: HashSet::new(),
            max_region_bytes: Self::DEFAULT_MAX_REGION_BYTES,
        }
    }

    /// Mapper with an explicit cap (withdrawn BARs stay unmappable above it).
    pub fn with_limit(max_region_bytes: u64) -> Self {
        Self {
            mapped: HashSet::new(),
            max_region_bytes,
        }
    }

    /// Configured cap.
    pub const fn limit(&self) -> u64 {
        self.max_region_bytes
    }

    /// Number of currently mapped BARs.
    pub fn mapped_count(&self) -> usize {
        self.mapped.len()
    }
}

impl Default for SimulatedMmioMapper {
    fn default() -> Self {
        Self::new()
    }
}

impl MmioMapper for SimulatedMmioMapper {
    fn label(&self) -> &'static str {
        "simulated-mmio-mapper"
    }

    fn is_simulated(&self) -> bool {
        true
    }

    fn map(&mut self, device: PciAddress, bar: &PciBar) -> PciResult<Box<dyn MmioRegion>> {
        if bar.kind == BarKind::Unimplemented {
            return Err(PciError::BarUnavailable {
                addr: device,
                index: bar.index,
            });
        }
        if !bar.is_memory() {
            return Err(PciError::BarNotMemoryMapped {
                addr: device,
                index: bar.index,
                kind: bar.kind,
            });
        }
        if bar.size == 0 {
            return Err(PciError::BarSizeUnknown {
                addr: device,
                index: bar.index,
            });
        }
        if bar.size > self.max_region_bytes {
            return Err(PciError::BarTooLarge {
                addr: device,
                index: bar.index,
                size: bar.size,
                limit: self.max_region_bytes,
            });
        }
        let key = (device, bar.index);
        if !self.mapped.insert(key) {
            return Err(PciError::BarAlreadyMapped {
                addr: device,
                index: bar.index,
            });
        }
        let size = bar.size as usize;
        Ok(Box::new(SimulatedMmio::zeroed(size)))
    }

    fn unmap(&mut self, device: PciAddress, bar_index: u8) -> PciResult<()> {
        if self.mapped.remove(&(device, bar_index)) {
            Ok(())
        } else {
            Err(PciError::BarNotMapped {
                addr: device,
                index: bar_index,
            })
        }
    }

    fn is_mapped(&self, device: PciAddress, bar_index: u8) -> bool {
        self.mapped.contains(&(device, bar_index))
    }
}

/// A mapped BAR: the device address, the BAR descriptor and the region handle.
///
/// `MappedBar` is the token a driver must hold to touch device memory. It is
/// deliberately not `Clone`: mapping is tracked per (device, BAR) in the mapper.
pub struct MappedBar {
    /// Address of the device that owns the BAR.
    pub device: PciAddress,
    /// The BAR descriptor this region was mapped from.
    pub bar: PciBar,
    region: Box<dyn MmioRegion>,
}

impl MappedBar {
    /// Builds a mapped BAR from its parts.
    pub fn new(device: PciAddress, bar: PciBar, region: Box<dyn MmioRegion>) -> Self {
        Self {
            device,
            bar,
            region,
        }
    }

    /// Region handle.
    pub fn region(&self) -> &dyn MmioRegion {
        self.region.as_ref()
    }

    /// Mutable region handle.
    pub fn region_mut(&mut self) -> &mut dyn MmioRegion {
        self.region.as_mut()
    }

    /// True when the region is a software model.
    pub fn is_simulated(&self) -> bool {
        self.region.is_simulated()
    }

    /// Region size in bytes.
    pub fn size(&self) -> usize {
        self.region.size()
    }

    /// Reads a 32-bit word from the region.
    pub fn read_u32(&self, offset: usize) -> PciResult<u32> {
        self.region.read_u32(offset)
    }

    /// Writes a 32-bit word to the region.
    pub fn write_u32(&mut self, offset: usize, value: u32) -> PciResult<()> {
        self.region.write_u32(offset, value)
    }
}

impl std::fmt::Debug for MappedBar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("MappedBar")
            .field("device", &self.device)
            .field("bar", &self.bar)
            .field("region", &self.region.label())
            .field("bytes", &self.region.size())
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn addr() -> PciAddress {
        PciAddress::new(0, 3, 0)
    }

    #[test]
    fn decodes_a_32bit_memory_bar() {
        let bar = decode::from_raw(0, 0xF000_000C, None);
        assert_eq!(bar.kind, BarKind::Memory32);
        assert_eq!(bar.base, 0xF000_0000);
        assert!(bar.prefetchable);
        assert_eq!(bar.size, 0, "size is unknown until the sizing probe runs");
        assert!(!bar.is_mappable(), "a size-0 BAR must never be mappable");
    }

    #[test]
    fn decodes_a_64bit_memory_bar_from_two_dwords() {
        let bar = decode::from_raw(2, 0x0000_1004, Some(0x0000_0002));
        assert_eq!(bar.kind, BarKind::Memory64);
        assert_eq!(bar.base, 0x2_0000_1000);
        assert_eq!(decode::dwords(bar.kind), 2);
    }

    #[test]
    fn decodes_an_io_bar() {
        let bar = decode::from_raw(4, 0x0000_C001, None);
        assert_eq!(bar.kind, BarKind::IoPort);
        assert_eq!(bar.base, 0xC000);
        assert!(!bar.is_memory());
    }

    #[test]
    fn decodes_an_unimplemented_bar() {
        let bar = decode::from_raw(5, 0, None);
        assert_eq!(bar.kind, BarKind::Unimplemented);
        assert!(!bar.is_mappable());
        assert_eq!(bar.kind.to_string(), "none");
    }

    #[test]
    fn derives_size_from_the_write_all_ones_mask() {
        // A 4 KiB region: low 13 bits hardwired to zero (attributes masked off).
        assert_eq!(decode::size_from_mask(!0xFFFu32 & !0xFu32, None), 0x1000);
        // 64 KiB.
        assert_eq!(decode::size_from_mask(!0xFFFFu32 & !0xFu32, None), 0x10000);
        // Unimplemented slot.
        assert_eq!(decode::size_from_mask(0, None), 0);
        // Fully implemented low half: the size comes from the upper dword.
        assert_eq!(
            decode::size_from_mask(0x0000_0000, Some(!0x3_u32)),
            0x4_0000_0000
        );
    }

    #[test]
    fn bar_helpers_report_the_region() {
        let bar = PciBar::new(1, BarKind::Memory32, 0x1000, 0x2000, false);
        assert!(bar.is_mappable());
        assert_eq!(bar.end(), 0x3000);
        assert!(bar.contains_offset(0x1FFF));
        assert!(!bar.contains_offset(0x2000));
        assert_eq!(bar.with_size(0x4000).size, 0x4000);
        assert_eq!(bar.kind.to_string(), "mem32");
    }

    #[test]
    fn mmio_bounds_check_rejects_misaligned_and_out_of_range() {
        assert!(check_mmio_access(16, 0, 4).is_ok());
        assert!(matches!(
            check_mmio_access(16, 2, 4),
            Err(PciError::MmioMisaligned { .. })
        ));
        assert!(matches!(
            check_mmio_access(16, 16, 4),
            Err(PciError::MmioOutOfRange { .. })
        ));
        assert!(matches!(
            check_mmio_access(16, usize::MAX, 4),
            Err(PciError::MmioOutOfRange { .. })
        ));
    }

    #[test]
    fn simulated_region_round_trips_words() {
        let mut region = SimulatedMmio::zeroed(16);
        assert_eq!(region.size(), 16);
        assert_eq!(region.label(), "simulated-mmio");
        assert!(region.is_simulated());
        assert_eq!(region.read_u32(0).unwrap(), 0);
        region.write_u32(4, 0xDEAD_BEEF).unwrap();
        assert_eq!(region.read_u32(4).unwrap(), 0xDEAD_BEEF);
        region.write_u64(8, 0x0123_4567_89AB_CDEF).unwrap();
        assert_eq!(region.read_u64(8).unwrap(), 0x0123_4567_89AB_CDEF);
        assert_eq!(region.bytes().len(), 16);
        assert!(region.write_u32(14, 1).is_err());
    }

    #[test]
    fn simulated_mapper_tracks_mappings_and_refuses_double_map() {
        let mut mapper = SimulatedMmioMapper::new();
        assert_eq!(mapper.limit(), SimulatedMmioMapper::DEFAULT_MAX_REGION_BYTES);
        assert!(mapper.is_simulated());
        let bar = PciBar::new(0, BarKind::Memory32, 0, 0x1000, false);

        assert!(mapper.map(addr(), &bar).is_ok());
        assert!(mapper.is_mapped(addr(), 0));
        assert_eq!(mapper.mapped_count(), 1);
        assert!(matches!(
            mapper.map(addr(), &bar),
            Err(PciError::BarAlreadyMapped { .. })
        ));

        assert!(mapper.unmap(addr(), 0).is_ok());
        assert!(!mapper.is_mapped(addr(), 0));
        assert!(matches!(
            mapper.unmap(addr(), 0),
            Err(PciError::BarNotMapped { .. })
        ));
        assert_eq!(mapper.label(), "simulated-mmio-mapper");
        assert_eq!(SimulatedMmioMapper::default().mapped_count(), 0);
    }

    #[test]
    fn simulated_mapper_refuses_unusable_bars() {
        let mut mapper = SimulatedMmioMapper::new();
        let unsized_bar = PciBar::new(0, BarKind::Memory32, 0, 0, false);
        assert!(matches!(
            mapper.map(addr(), &unsized_bar),
            Err(PciError::BarSizeUnknown { .. })
        ));

        let io_bar = PciBar::new(1, BarKind::IoPort, 0xC000, 0x100, false);
        assert!(matches!(
            mapper.map(addr(), &io_bar),
            Err(PciError::BarNotMemoryMapped { .. })
        ));

        let none_bar = PciBar::new(5, BarKind::Unimplemented, 0, 0, false);
        assert!(matches!(
            mapper.map(addr(), &none_bar),
            Err(PciError::BarUnavailable { .. })
        ));

        let mut small = SimulatedMmioMapper::with_limit(0x100);
        let big_bar = PciBar::new(0, BarKind::Memory32, 0, 0x1000, false);
        assert!(matches!(
            small.map(addr(), &big_bar),
            Err(PciError::BarTooLarge { .. })
        ));
    }

    #[test]
    fn mapped_bar_wraps_the_region() {
        let mut mapper = SimulatedMmioMapper::new();
        let bar = PciBar::new(0, BarKind::Memory32, 0, 0x1000, false);
        let region = mapper.map(addr(), &bar).unwrap();
        let mut mapped = MappedBar::new(addr(), bar, region);
        assert!(mapped.is_simulated());
        assert_eq!(mapped.size(), 0x1000);
        mapped.write_u32(0, 7).unwrap();
        assert_eq!(mapped.read_u32(0).unwrap(), 7);
        assert!(mapped.region().is_simulated());
        mapped.region_mut().write_u32(8, 9).unwrap();
        assert_eq!(mapped.region().read_u32(8).unwrap(), 9);
        assert!(format!("{mapped:?}").contains("simulated-mmio"));
    }
}