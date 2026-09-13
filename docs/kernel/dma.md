# Kernel DMA and IOMMU (Host Model)

`kernel/src/dma/` + `kernel/src/driver/dma.rs` (`DmaRegion`, `DmaBuffer`).

- Validation: 4KiB-aligned phys, 0 < size <= 64MiB; `map/unmap` booleans, `Drop` forces unmap (invariant I1).
- No IOMMU hardware: declared `IOMMU=absent`, deny-by-default. Future DMA must bind `owner: DeviceId` + `CapSet` and require unmap-on-drop.