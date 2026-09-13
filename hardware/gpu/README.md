# Hardware GPU (V.04 §13)

Canonical implementation: `crates/hardware-abstraction/src/gpu.rs` (`GpuDevice`, `GpuInfo`, `GpuHealth` via `HardwareDevice` trait).

Access via `identify/initialize/health` only. No undocumented registers.