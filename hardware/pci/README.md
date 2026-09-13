# Hardware PCI (V.04 §13)

Canonical implementation: `crates/hardware-abstraction/src/pci.rs` (`PciDevice`, `PciInfo` via `HardwareDevice` trait).

Access via `identify/initialize/health` only. No undocumented config-space assumptions.