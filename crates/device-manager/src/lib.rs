//! QEOS Phase 4.1 device stack — device manager, PCI/PCIe abstraction and driver lifecycle.
//!
//! ```text
//!                     QEOS DEVICE STACK (Phase 4.1)
//!                                |
//!                    DeviceManager (lifecycle, capabilities)
//!                                |
//!             +------------------+------------------+
//!             |                                     |
//!        DriverRegistry                        PciBus
//!     (DeviceDriver trait)              (PciConfigBackend)
//!             |                                     |
//!      bound devices                    +------------+------------+
//!                                       |                         |
//!                              SimulatedPciBackend        (future) kernel / OS
//!                                (host tests, CI)           config-space backend
//! ```
//!
//! CLASSIFICATION (see `docs/architecture/V.04.md`)
//!
//! - [`manager::DeviceManager`], [`driver::DeviceDriver`], [`pci::PciBus`],
//!   [`pci::bar`], [`pci::capability`]: **REAL** — implemented, host-testable
//!   logic that performs genuine PCI configuration-space decoding.
//! - [`pci::simulated::SimulatedPciBackend`]: **SIMULATED** — a software model of
//!   a PCI bus with fictional device fixtures. It is a test/CI backend and is
//!   never evidence of physical hardware.
//! - A real configuration-space backend (kernel `PcieHal`, `/sys/bus/pci`, or a
//!   vendor SDK) is **ABSTRACT/FUTURE**: it must implement
//!   [`pci::config::PciConfigBackend`]. No such backend is bundled, and no code
//!   in this crate claims to talk to real PCI hardware by itself.
//!
//! GUARANTEES
//!
//! - No `unsafe`, no direct physical-memory or port I/O access.
//! - MMIO access is only possible through [`pci::bar::MmioRegion`] obtained from
//!   a [`pci::bar::MmioMapper`]; the bundled mapper is the simulated one.
//! - Every device-touching operation is gated by a capability
//!   ([`capability::DeviceCapability`]) and IOMMU-aware policy
//!   ([`capability::IommuPolicy`], deny-by-default for unmanaged DMA).
//! - Driver lifecycle transitions are validated; illegal transitions are errors,
//!   never silent no-ops.

#![forbid(unsafe_code)]
// Production paths must not unwrap/expect/panic (§34). Tests may use them, as the
// rest of the workspace does.
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![cfg_attr(test, allow(clippy::unwrap_used, clippy::expect_used, clippy::panic))]

pub mod capability;
pub mod driver;
pub mod error;
pub mod id;
pub mod manager;
pub mod pci;

pub use capability::{DeviceCapability, DeviceCapabilitySet, HotplugPolicy, IommuPolicy};
pub use driver::{DeviceDriver, DriverMatch, DriverState};
pub use error::{DeviceManagerError, DeviceManagerResult, DriverError, DriverResult, PciError, PciResult};
pub use id::{DeviceId, DriverId};
pub use manager::{
    DeviceManager, DeviceManagerConfig, DeviceRecord, DeviceSnapshot, HealthSummary, ManagerCounters,
};
pub use pci::address::{PciAddress, PciClass, PciHeaderType};
pub use pci::bar::{BarKind, MappedBar, MmioMapper, MmioRegion, PciBar, SimulatedMmioMapper};
pub use pci::bus::{BusRange, PciBus};
pub use pci::capability::{LinkSpeed, MsiInfo, MsiVector, MsixInfo, PciCapability, PcieCapability};
pub use pci::config::{BackendSource, ConfigWidth, PciConfigBackend};
pub use pci::device::{DmaReadiness, HotplugSupport, PciDevice, PciDeviceInfo};
pub use pci::simulated::{SimulatedBarSpec, SimulatedDeviceSpec, SimulatedPciBackend};

/// Crate version, reported by diagnostics (Phase 4.8 `qeosctl doctor`).
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_is_reported() {
        assert!(!version().is_empty());
    }
}