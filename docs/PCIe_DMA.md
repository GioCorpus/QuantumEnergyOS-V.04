# PCIe / DMA (Phase 4.4)

- Discovery: `kernel::driver::pci::{PciAddr, PciIdentity, BarDescriptor,
  InterruptFacts, DmaFacts, PciFunction, PciConfigSource, StubConfigSource}`.
  Absent slot = 0xFFFF. No MSI-X / 64-bit / IOMMU assumed.
- Decode: `device-manager::pci` (config decode, BAR sizing, cap walk incl.
  MSI/MSI-X, `SimulatedPciBackend` transport, `SimulatedMmioMapper`).
- MMIO: `kernel::driver::mmio::MmioWindow` (aligned + bounded host model).
  Real volatile access is arch-specific FUTURE work with documented SAFETY.
- DMA: `kernel::dma::{DmaRegion, DmaMapping, DmaRing}` — aligned/lifetime
  tracked, unmap-on-drop, SPSC ring only (MPMC/MPSC/SPMC need separate impl).
- Rule: raw physical addresses never reach user space (IOVA handles only).
