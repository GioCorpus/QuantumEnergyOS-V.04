# Hardware Abstraction

## Goal

Provide hardware interfaces for CPU, GPU, PCIe, NVMe, SPI, I2C, power, telemetry, and quantum adapter devices without hard-coding vendor-specific behavior at the service layer.

## Core trait

```rust
pub trait HardwareDevice {
    fn identify(&self) -> DeviceInfo;
    fn initialize(&mut self) -> Result<(), HardwareError>;
    fn health(&self) -> DeviceHealth;
}
```

## Device categories

- CPU and system health
- GPU and accelerator access
- PCI enumeration and device registry
- NVMe drive health and IO telemetry
- SPI/I2C sensor interfaces
- power and electrical monitoring
- telemetry ring buffers and streaming drivers
- quantum adapter interfaces that remain capability-gated

## Constraint

Quantum hardware support is only a formal adapter contract. Any physical device integration must be gated behind documented APIs and real compatibility checks.
