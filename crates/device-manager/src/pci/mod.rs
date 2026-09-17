//! PCI/PCIe bus abstraction (§5).
//!
//! The bus layer decodes *real* configuration-space semantics (vendor/device ids,
//! class codes, header types, BARs, capability lists, MSI/MSI-X, PCIe link and
//! slot information) through one narrow transport trait:
//! [`config::PciConfigBackend`].
//!
//! What is real and what is not:
//!
//! - decoding, enumeration, BAR sizing (the PCI Local Bus write-all-ones
//!   algorithm) and capability walking: implemented here, host-testable;
//! - the transport: [`simulated::SimulatedPciBackend`] is a software model used
//!   by tests/CI. A production transport (kernel `PcieHal`, `/sys/bus/pci`,
//!   firmware tables) is FUTURE and must implement [`config::PciConfigBackend`];
//! - MMIO: [`bar::MmioMapper`] is the only way to reach a BAR. The bundled mapper
//!   is [`bar::SimulatedMmioMapper`] (a memory-backed model); real mapping is the
//!   kernel's responsibility and is not implemented here.
//!
//! Nothing in this module performs port I/O, `unsafe` MMIO or physical-memory
//! access.

pub mod address;
pub mod bar;
pub mod bus;
pub mod capability;
pub mod config;
pub mod device;
pub mod simulated;

pub use address::{PciAddress, PciClass, PciHeaderType};
pub use bar::{BarKind, MappedBar, MmioMapper, MmioRegion, PciBar, SimulatedMmioMapper};
pub use bus::{BusRange, PciBus};
pub use capability::{LinkSpeed, MsiInfo, MsiVector, MsixInfo, PciCapability, PcieCapability};
pub use config::{BackendSource, ConfigWidth, PciConfigBackend};
pub use device::{DmaReadiness, HotplugSupport, PciDevice, PciDeviceInfo};
pub use simulated::{SimulatedBarSpec, SimulatedDeviceSpec, SimulatedPciBackend};
