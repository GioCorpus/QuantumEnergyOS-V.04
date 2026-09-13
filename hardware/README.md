# Hardware Abstraction (V.04 §13)

Canonical implementation: `crates/hardware-abstraction`
(`HardwareDevice` trait: `identify` / `initialize` / `health`).

- `cpu/`, `gpu/`, `pci/`, `nvme/`, `spi/`, `i2c/`, `power/`, `telemetry/`, `quantum/`
- All access via typed interfaces; no undocumented registers.
- QPU devices default to SIMULATION/EMULATION; PHYSICAL requires documented API.

See `HARDWARE_ABSTRACTION.md` for policy.