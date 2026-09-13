# Kernel Drivers (Host Stubs)

`kernel/src/driver/` (`BusKind`, `Device`, `Driver::probe`, `DriverRegistry`) + `kernel/src/device/`.

- `stub_for_host_tests()` (`enumerate_stub`) returns fake `0:1:0` for host tests only; NOT real PCI. Must never be interpreted as hardware support.
- No BAR/caps/IRQ/DMA binding yet; lifecycle (init/start/stop/reset/remove) is future (S15/S18).
- DMA checks in `kernel/src/dma/` + `driver/dma.rs`: alignment + 64MiB bound, `Drop` unmap, IOMMU declared absent (deny-by-default).