//! Error vocabulary of the device stack (§34).
//!
//! Three separate types with one umbrella:
//!
//! - [`PciError`] — configuration space, BARs, capabilities, MMIO mapping.
//! - [`DriverError`] — driver lifecycle, binding and capability policy.
//! - [`DeviceManagerError`] — what manager entry points return.
//!
//! All variants carry context (address, device id, driver name, reason) so that
//! diagnostics (`qeosctl doctor`, dashboards) never need to guess.

use thiserror::Error;

use crate::capability::DeviceCapability;
use crate::driver::DriverState;
use crate::id::{DeviceId, DriverId};
use crate::pci::address::PciAddress;
use crate::pci::bar::BarKind;
use crate::pci::config::ConfigWidth;

/// Result alias for PCI bus operations.
pub type PciResult<T> = std::result::Result<T, PciError>;
/// Result alias for driver lifecycle operations.
pub type DriverResult<T> = std::result::Result<T, DriverError>;
/// Result alias for device manager operations.
pub type DeviceManagerResult<T> = std::result::Result<T, DeviceManagerError>;

/// Failure of a PCI/PCIe configuration-space or BAR operation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum PciError {
    /// No function responds at this address (vendor id `0xFFFF`).
    #[error("no PCI function responds at {addr}")]
    DeviceAbsent { addr: PciAddress },

    /// The address itself is impossible (device > 31 or function > 7).
    #[error("invalid PCI address: {0}")]
    InvalidAddress(String),

    /// Configuration space content violates the layout the walker relies on.
    #[error("malformed PCI configuration space at {addr}: {reason}")]
    MalformedConfigSpace { addr: PciAddress, reason: String },

    /// The capability list does not terminate, repeats, or points out of range.
    #[error("malformed PCI capability list at {addr}: {reason}")]
    MalformedCapabilityList { addr: PciAddress, reason: String },

    /// Offset/width combination the backend cannot serve (misaligned access).
    #[error("unsupported PCI configuration access at {addr}: offset={offset:#04x} width={width:?}")]
    UnsupportedAccess {
        addr: PciAddress,
        offset: u16,
        width: ConfigWidth,
    },

    /// The backend itself failed (transport, kernel service, simulated model).
    #[error("PCI backend '{backend}' failed: {reason}")]
    Backend { backend: String, reason: String },

    /// The BAR slot is not implemented by this function.
    #[error("BAR {index} is not implemented by {addr}")]
    BarUnavailable { addr: PciAddress, index: u8 },

    /// The BAR exists but is not memory-mapped memory (I/O port BAR).
    #[error("BAR {index} of {addr} is not memory-mapped (kind: {kind:?})")]
    BarNotMemoryMapped {
        addr: PciAddress,
        index: u8,
        kind: BarKind,
    },

    /// The BAR size is unknown because the backend refuses BAR sizing.
    #[error("BAR {index} of {addr} has unknown size; backend does not support BAR sizing")]
    BarSizeUnknown { addr: PciAddress, index: u8 },

    /// The BAR was already handed out by the MMIO mapper.
    #[error("BAR {index} of {addr} is already mapped")]
    BarAlreadyMapped { addr: PciAddress, index: u8 },

    /// The BAR is not currently mapped.
    #[error("BAR {index} of {addr} is not mapped")]
    BarNotMapped { addr: PciAddress, index: u8 },

    /// The BAR region exceeds the mapper's configured limit.
    #[error("BAR {index} of {addr} is {size} bytes, above the mapper limit of {limit}")]
    BarTooLarge {
        addr: PciAddress,
        index: u8,
        size: u64,
        limit: u64,
    },

    /// MMIO access outside the mapped region.
    #[error("MMIO access at offset {offset} (width {width}) exceeds region size {size}")]
    MmioOutOfRange {
        offset: usize,
        width: usize,
        size: usize,
    },

    /// MMIO access that is not naturally aligned for its width.
    #[error("MMIO access at offset {offset} is not aligned for width {width}")]
    MmioMisaligned { offset: usize, width: usize },

    /// Requested capability id is not present in the list.
    #[error("PCI capability id {id:#04x} not found on {addr}")]
    CapabilityNotFound { addr: PciAddress, id: u8 },

    /// Enumeration produced more devices than the configured guard allows.
    #[error("PCI enumeration exceeded the device limit ({limit})")]
    EnumerationLimitExceeded { limit: usize },
}

/// Failure of a driver lifecycle, binding or capability-policy operation.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum DriverError {
    /// The operation needs a bound driver, but the device is unbound.
    #[error("no driver is bound to {device}")]
    NoDriverBound { device: DeviceId },

    /// A second driver tried to bind an already-bound device.
    #[error("{device} is already bound to driver '{driver}'")]
    DriverAlreadyBound { device: DeviceId, driver: String },

    /// The driver's match criteria reject this device.
    #[error("driver '{driver}' does not support {device}")]
    UnsupportedDevice { driver: String, device: DeviceId },

    /// Two drivers were registered under the same name.
    #[error("driver name '{0}' is already registered")]
    DriverNameTaken(String),

    /// `probe` rejected the device.
    #[error("driver '{driver}' probe failed for {device}: {reason}")]
    ProbeFailed {
        driver: String,
        device: DeviceId,
        reason: String,
    },

    /// `initialize` failed.
    #[error("driver '{driver}' initialize failed for {device}: {reason}")]
    InitializeFailed {
        driver: String,
        device: DeviceId,
        reason: String,
    },

    /// `start` failed.
    #[error("driver '{driver}' start failed for {device}: {reason}")]
    StartFailed {
        driver: String,
        device: DeviceId,
        reason: String,
    },

    /// `stop` failed.
    #[error("driver '{driver}' stop failed for {device}: {reason}")]
    StopFailed {
        driver: String,
        device: DeviceId,
        reason: String,
    },

    /// `reset` failed.
    #[error("driver '{driver}' reset failed for {device}: {reason}")]
    ResetFailed {
        driver: String,
        device: DeviceId,
        reason: String,
    },

    /// Lifecycle transition rejected by the state machine.
    #[error("illegal lifecycle transition for {device}: {from:?} -> {to:?}")]
    InvalidTransition {
        device: DeviceId,
        from: DriverState,
        to: DriverState,
    },

    /// The capability was not granted, with the reason recorded.
    #[error("capability '{capability}' denied for {device}: {reason}")]
    CapabilityDenied {
        device: DeviceId,
        capability: DeviceCapability,
        reason: String,
    },

    /// The device does not declare the capability, so it cannot be granted.
    #[error("{device} does not declare capability '{capability}' (declared: {declared:?})")]
    CapabilityNotDeclared {
        device: DeviceId,
        capability: DeviceCapability,
        declared: Vec<&'static str>,
    },

    /// Removal refused by hotplug policy.
    #[error("hot-removal denied for {device}: {reason}")]
    HotplugDenied { device: DeviceId, reason: String },

    /// The device does not report hotplug support.
    #[error("{device} does not report hotplug support")]
    HotplugUnsupported { device: DeviceId },

    /// Unknown device id.
    #[error("unknown device {device}")]
    DeviceNotFound { device: DeviceId },

    /// Unknown driver id.
    #[error("unknown driver {driver}")]
    DriverNotFound { driver: DriverId },

    /// PCI failure surfaced through a driver operation.
    #[error("PCI subsystem error: {0}")]
    Pci(#[from] PciError),

    /// Internal invariant violation (never silently ignored).
    #[error("internal device stack error: {0}")]
    Internal(String),
}

/// Umbrella error returned by [`crate::manager::DeviceManager`] entry points.
#[derive(Debug, Error)]
pub enum DeviceManagerError {
    /// PCI bus/enumeration failure.
    #[error(transparent)]
    Pci(#[from] PciError),

    /// Driver lifecycle/policy failure.
    #[error(transparent)]
    Driver(#[from] DriverError),

    /// Hardware abstraction layer failure (identity/health bridges).
    #[error(transparent)]
    Hal(#[from] hardware_abstraction::HardwareError),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pci_errors_report_address_and_reason() {
        let addr = PciAddress::new(0, 3, 0);
        let err = PciError::DeviceAbsent { addr };
        assert!(err.to_string().contains("0000:00:03.0"));

        let malformed = PciError::MalformedCapabilityList {
            addr,
            reason: "cycle".to_string(),
        };
        assert!(malformed.to_string().contains("cycle"));
    }

    #[test]
    fn driver_errors_report_device_and_capability() {
        let device = DeviceId::from_raw(1);
        let err = DriverError::CapabilityDenied {
            device,
            capability: DeviceCapability::DmaAccess,
            reason: "no IOMMU".to_string(),
        };
        let text = err.to_string();
        assert!(text.contains("device-0001"));
        assert!(text.contains("dma_access"));
        assert!(text.contains("no IOMMU"));
    }

    #[test]
    fn driver_error_converts_from_pci_error() {
        let err: DriverError = PciError::InvalidAddress("bad".to_string()).into();
        assert!(matches!(err, DriverError::Pci(_)));
    }

    #[test]
    fn manager_error_is_transparent_over_pci_and_driver() {
        let device = DeviceId::from_raw(2);
        let from_pci: DeviceManagerError = PciError::InvalidAddress("bad".to_string()).into();
        assert!(from_pci.to_string().contains("bad"));

        let from_driver: DeviceManagerError = DriverError::NoDriverBound { device }.into();
        assert!(from_driver.to_string().contains("device-0002"));
    }
}