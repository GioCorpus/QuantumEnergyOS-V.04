# Hardware Architecture (Phase 4.4–4.9)

```text
QEOS APPS / services / qeos-qpu CLI
  GPU runtime (qeos-gpu-compute) | QPU runtime (quantum-runtime + quantum-hal)
  HAL facts (hardware-abstraction) | Device lifecycle (device-manager)
  kernel: driver(pci/dma/iommu/mmio/lifecycle) + dma + telemetry + security
```

Rules: kernel holds memory/scheduler/IRQ/PCIe/DMA/IOMMU/lifecycle/telemetry
primitives only. GPU/QPU/Majorana/EC/simulation live in user-space crates.
Vendor stacks (CUDA/ROCm/Vulkan/physical QPU) are FUTURE behind `probe()`.
See PCIe_DMA.md, IOMMU.md, GPU_RUNTIME.md, QPU_RUNTIME.md, QUANTUM_IR.md,
MAJORANA_SIMULATOR.md, ERROR_CORRECTION.md, TELEMETRY.md, SECURITY_MODEL.md,
EXPERIMENT_ENGINE.md.
