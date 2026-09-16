# QEMU Validation Checklist (Phase 4.4)

Scope: host-testable PCI/DMA/IOMMU/MMIO models + QEMU device-shape fixtures.
CLASSIFICATION: SIMULATED unless a real QEMU run log is attached.

## 1. Host fixtures (CI, no QEMU needed)

- `kernel::driver::pci::StubConfigSource::qemu_virtio_net` — 0x1AF4:0x1000,
  one 32-bit memory BAR, MSI without MSI-X, no 64-bit DMA, no ATS/IOMMU.
- `device-manager::SimulatedPciBackend` — config decode + BAR sizing + cap walk.
- `kernel::driver::{DmaMapping, DmaRing, MockIommu, MmioWindow}` — ownership,
  SPSC ring invariants, deny-by-default mapping.

Run: `cargo test -p qeos-kernel -p device-manager`

## 2. QEMU run (manual, attach log)

```powershell
qemu-system-x86_64 -machine q35 -device virtio-net-pci `
  -device qemu-xhci -trace pci_cfg_write -serial stdio -nographic
```

Expect: virtio-net at 00:01.0 (0x1AF4:0x1000), MSI capable, MSI-X absent;
bus-master + memory-space set before DMA; no ATS assumed; IOMMU only when
`-device intel-iommu` is explicitly passed.

## 3. Fault injection (host)

DMA misaligned/oversize rejected; ring full/empty errors; IOMMU unmapped
rejected; MMIO unaligned/out-of-range rejected; absent PCI slot = 0xFFFF.
