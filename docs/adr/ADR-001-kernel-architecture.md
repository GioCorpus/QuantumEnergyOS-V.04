# ADR-001 — Kernel stays GP-OS; QPU/GPU/Energy as devices + userspace runtimes

Status: accepted. Context: avoid turning kernel into quantum simulator/lab.
Decision: kernel exposes memory/sched/IPC/DMA/PCIe/IRQ/device/security/timer/sync;
QPU runtime, Majorana sim, AI, dashboards live in userspace via IPC + device API.
Consequences: `qpu` module is trait-only; `pci::enumerate_stub` host-only;
energy is telemetry primitives only. Ref: §1-2, §40-41, §78-79, §90.