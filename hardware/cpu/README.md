# Hardware CPU (V.04 §13)

Canonical implementation: `crates/hardware-abstraction/src/cpu.rs` (`CpuDevice`, `CpuInfo`, `CpuHealth` via `HardwareDevice` trait).

Access via `identify/initialize/health` only. No undocumented registers.