# Hardware NVMe (V.04 §13)

Canonical implementation: `crates/hardware-abstraction/src/nvme.rs` (`NvmeDevice`, `NvmeInfo`, `NvmeHealth` via `HardwareDevice` trait).

Access via `identify/initialize/health` only. No undocumented registers.