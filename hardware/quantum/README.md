# Hardware Quantum (V.04 §13)

Canonical implementation: `crates/hardware-abstraction/src/quantum.rs` (`QuantumHardware`, `QuantumDeviceInfo` via `HardwareDevice` trait).

Defaults to SIMULATION/EMULATION; PHYSICAL requires documented API and `enable_physical()` capability gate. Never assumes undocumented registers or control signals.