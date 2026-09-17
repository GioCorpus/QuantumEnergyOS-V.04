pub mod cpu;
pub mod device;
pub mod error;
pub mod gpu;
pub mod nvme;
pub mod pci;
pub mod power;
pub mod quantum;
pub mod spi;
pub mod telemetry;

pub use cpu::{CpuDevice, CpuHealth, CpuInfo};
pub use device::{DeviceHealth, DeviceInfo, HardwareDevice};
pub use error::{HardwareError, Result};
pub use gpu::{GpuDevice, GpuHealth, GpuInfo};
pub use nvme::{NvmeDevice, NvmeHealth, NvmeInfo};
pub use pci::{PciDevice, PciInfo};
pub use power::{PowerDevice, PowerHealth, PowerInfo};
pub use quantum::{QuantumDeviceHealth, QuantumDeviceInfo};
pub use spi::{I2cDevice, I2cInfo, SpiDevice, SpiInfo};
pub use telemetry::{TelemetryHardware, TelemetryInfo};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
