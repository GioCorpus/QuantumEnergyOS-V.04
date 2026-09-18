//! Central Device Manager: device discovery, driver binding, lifecycle management
//! and capability enforcement (§4, §20).
//!
//! [`DeviceManager`] coordinates:
//! - Discovered PCI devices and stable assigned [`DeviceId`] handles.
//! - Registered drivers implementing [`crate::driver::DeviceDriver`].
//! - Explicit capability granting/revocation gated by [`IommuPolicy`] and hardware facts.
//! - Device lifecycle state transitions with rollback on failure.
//! - Hotplug management gated by [`HotplugPolicy`].

use serde::{Deserialize, Serialize};

use hardware_abstraction::DeviceHealth;

use crate::capability::{DeviceCapability, DeviceCapabilitySet, HotplugPolicy, IommuPolicy};
use crate::driver::{
    DeviceDriver, DriverState, NullDriver, SimulatedGpuDriver, SimulatedNvmeDriver,
    SimulatedTelemetryDriver,
};
use crate::error::{DeviceManagerResult, DriverError, DriverResult, PciError};
use crate::id::{DeviceId, DriverId};
use crate::pci::bus::PciBus;
use crate::pci::config::PciConfigBackend;
use crate::pci::device::PciDevice;

/// Configuration for the [`DeviceManager`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceManagerConfig {
    /// Automatically match and bind drivers upon enumeration.
    pub auto_bind: bool,
    /// IOMMU policy gating DMA capability grants.
    pub iommu_policy: IommuPolicy,
    /// Policy gating hot-removal of devices.
    pub hotplug_policy: HotplugPolicy,
    /// Maximum number of devices allowed.
    pub max_devices: usize,
}

impl Default for DeviceManagerConfig {
    fn default() -> Self {
        Self {
            auto_bind: true,
            iommu_policy: IommuPolicy::DenyUnmanagedDma,
            hotplug_policy: HotplugPolicy::DenyRemoval,
            max_devices: 64,
        }
    }
}

/// A managed device record held by the manager.
pub struct DeviceRecord {
    /// Stable assigned device identifier.
    pub id: DeviceId,
    /// Fully decoded PCI device information.
    pub device: PciDevice,
    /// Driver bound to this device, if any.
    pub driver_id: Option<DriverId>,
    /// Driver name for fast lookup and display.
    pub driver_name: Option<String>,
    /// Current driver lifecycle state.
    pub driver_state: DriverState,
    /// Set of capabilities explicitly granted to this device.
    pub granted_capabilities: DeviceCapabilitySet,
    /// Current device health status.
    pub health: DeviceHealth,
}

/// Serializable snapshot of a device for diagnostics and CLI output.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceSnapshot {
    /// Assigned device ID.
    pub id: DeviceId,
    /// Bus address in standard `0000:bb:dd.f` format.
    pub address: String,
    /// Vendor ID.
    pub vendor_id: u16,
    /// Device ID.
    pub device_id: u16,
    /// PCI class name.
    pub class_name: String,
    /// Name of bound driver, or `None`.
    pub driver_name: Option<String>,
    /// Current lifecycle state.
    pub driver_state: String,
    /// Capabilities declared by hardware.
    pub declared_capabilities: Vec<String>,
    /// Capabilities explicitly granted by manager.
    pub granted_capabilities: Vec<String>,
    /// DMA readiness summary.
    pub dma_ready: bool,
    /// Hotplug support label.
    pub hotplug: String,
    /// Health label.
    pub health: String,
}

/// Cumulative counters tracked by the manager.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ManagerCounters {
    /// Number of devices discovered.
    pub enumerated: usize,
    /// Number of devices bound to drivers.
    pub bound: usize,
    /// Number of devices initialized.
    pub initialized: usize,
    /// Number of devices actively running.
    pub running: usize,
    /// Number of device/driver errors encountered.
    pub failed: usize,
    /// Number of hotplug events processed.
    pub hotplug_events: usize,
}

/// Health summary across all managed devices.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct HealthSummary {
    /// Total devices managed.
    pub total_devices: usize,
    /// Devices in Healthy status.
    pub healthy: usize,
    /// Devices in Warning status.
    pub warning: usize,
    /// Devices in Degraded status.
    pub degraded: usize,
    /// Devices in Failed status.
    pub failed: usize,
    /// Devices in Unknown status.
    pub unknown: usize,
}

/// Central device manager.
pub struct DeviceManager {
    config: DeviceManagerConfig,
    drivers: Vec<(DriverId, Box<dyn DeviceDriver>)>,
    devices: Vec<DeviceRecord>,
    next_device_id: u32,
    next_driver_id: u32,
    counters: ManagerCounters,
}

impl DeviceManager {
    /// Creates a new device manager with the given configuration.
    pub fn new(config: DeviceManagerConfig) -> Self {
        Self {
            config,
            drivers: Vec::new(),
            devices: Vec::new(),
            next_device_id: 1,
            next_driver_id: 1,
            counters: ManagerCounters::default(),
        }
    }

    /// Creates a device manager pre-registered with default simulated drivers.
    pub fn with_default_drivers(config: DeviceManagerConfig) -> Self {
        let mut mgr = Self::new(config);
        let _ = mgr.register_driver(SimulatedNvmeDriver::new());
        let _ = mgr.register_driver(SimulatedGpuDriver::new());
        let _ = mgr.register_driver(SimulatedTelemetryDriver::new());
        let _ = mgr.register_driver(NullDriver::new("qeos-generic-driver"));
        mgr
    }

    /// Manager configuration.
    pub fn config(&self) -> &DeviceManagerConfig {
        &self.config
    }

    /// Mutable manager configuration.
    pub fn config_mut(&mut self) -> &mut DeviceManagerConfig {
        &mut self.config
    }

    /// Registers a driver with the manager.
    pub fn register_driver<D: DeviceDriver + 'static>(
        &mut self,
        driver: D,
    ) -> DriverResult<DriverId> {
        self.register_driver_boxed(Box::new(driver))
    }

    /// Registers a boxed driver.
    pub fn register_driver_boxed(
        &mut self,
        driver: Box<dyn DeviceDriver>,
    ) -> DriverResult<DriverId> {
        let name = driver.name();
        if self.drivers.iter().any(|(_, d)| d.name() == name) {
            return Err(DriverError::DriverNameTaken(name.to_string()));
        }

        let id = DriverId::from_raw(self.next_driver_id);
        self.next_driver_id += 1;
        self.drivers.push((id, driver));
        Ok(id)
    }

    /// Enumerates a PCI bus, records discovered devices, and optionally auto-binds drivers.
    pub fn enumerate_bus<B: PciConfigBackend>(
        &mut self,
        bus: &mut PciBus<B>,
    ) -> DeviceManagerResult<usize> {
        let discovered = bus.enumerate()?;
        let mut count = 0;

        for pci_dev in discovered {
            if self.devices.len() >= self.config.max_devices {
                return Err(PciError::EnumerationLimitExceeded {
                    limit: self.config.max_devices,
                }
                .into());
            }

            // Check if device already registered
            if self
                .devices
                .iter()
                .any(|r| r.device.address() == pci_dev.address())
            {
                continue;
            }

            let id = DeviceId::from_raw(self.next_device_id);
            self.next_device_id += 1;

            let declared = pci_dev.declared_capabilities();
            let mut baseline = DeviceCapabilitySet::baseline();
            // Baseline cannot exceed declared capabilities
            baseline = baseline.difference(baseline.difference(declared));

            let record = DeviceRecord {
                id,
                device: pci_dev,
                driver_id: None,
                driver_name: None,
                driver_state: DriverState::Unbound,
                granted_capabilities: baseline,
                health: DeviceHealth::Unknown,
            };

            self.devices.push(record);
            self.counters.enumerated += 1;
            count += 1;

            if self.config.auto_bind {
                let _ = self.bind_device(id);
            }
        }

        Ok(count)
    }

    /// Binds the best matching registered driver to `dev_id`.
    pub fn bind_device(&mut self, dev_id: DeviceId) -> DriverResult<()> {
        let dev_idx = self
            .devices
            .iter()
            .position(|r| r.id == dev_id)
            .ok_or(DriverError::DeviceNotFound { device: dev_id })?;

        if self.devices[dev_idx].driver_state.is_bound() {
            return Err(DriverError::DriverAlreadyBound {
                device: dev_id,
                driver: self.devices[dev_idx]
                    .driver_name
                    .clone()
                    .unwrap_or_default(),
            });
        }

        // Find matching driver
        let info = self.devices[dev_idx].device.info().clone();
        let pci_dev = self.devices[dev_idx].device.clone();

        let mut matched = None;
        for (drv_id, driver) in &mut self.drivers {
            if driver.matches(&info) {
                match driver.probe(&pci_dev) {
                    Ok(()) => {
                        matched = Some((*drv_id, driver.name().to_string(), driver.health()));
                        break;
                    }
                    Err(e) => {
                        // Driver rejected during probe, continue search
                        tracing::debug!(
                            "Driver '{}' probe rejected device: {:?}",
                            driver.name(),
                            e
                        );
                    }
                }
            }
        }

        if let Some((drv_id, drv_name, health)) = matched {
            let record = &mut self.devices[dev_idx];
            DriverState::check_transition(dev_id, &record.driver_state, &DriverState::Bound)?;
            record.driver_id = Some(drv_id);
            record.driver_name = Some(drv_name);
            record.driver_state = DriverState::Bound;
            record.health = health;
            self.counters.bound += 1;
            Ok(())
        } else {
            Err(DriverError::UnsupportedDevice {
                driver: "no matching driver".to_string(),
                device: dev_id,
            })
        }
    }

    /// Initializes hardware and resources for `dev_id`.
    pub fn initialize_device(&mut self, dev_id: DeviceId) -> DriverResult<()> {
        self.check_capability(dev_id, DeviceCapability::Operate)?;

        let dev_idx = self
            .devices
            .iter()
            .position(|r| r.id == dev_id)
            .ok_or(DriverError::DeviceNotFound { device: dev_id })?;

        let drv_id = self.devices[dev_idx]
            .driver_id
            .ok_or(DriverError::NoDriverBound { device: dev_id })?;

        let pci_dev = self.devices[dev_idx].device.clone();

        let driver = self
            .drivers
            .iter_mut()
            .find(|(id, _)| *id == drv_id)
            .map(|(_, d)| d)
            .ok_or(DriverError::DriverNotFound { driver: drv_id })?;

        driver.initialize(&pci_dev).inspect_err(|e| {
            self.counters.failed += 1;
            self.devices[dev_idx].driver_state = DriverState::Failed(e.to_string());
        })?;

        let record = &mut self.devices[dev_idx];
        DriverState::check_transition(dev_id, &record.driver_state, &DriverState::Initialized)?;
        record.driver_state = DriverState::Initialized;
        record.health = driver.health();
        self.counters.initialized += 1;
        Ok(())
    }

    /// Starts active operation on `dev_id`.
    pub fn start_device(&mut self, dev_id: DeviceId) -> DriverResult<()> {
        self.check_capability(dev_id, DeviceCapability::Operate)?;

        let dev_idx = self
            .devices
            .iter()
            .position(|r| r.id == dev_id)
            .ok_or(DriverError::DeviceNotFound { device: dev_id })?;

        let drv_id = self.devices[dev_idx]
            .driver_id
            .ok_or(DriverError::NoDriverBound { device: dev_id })?;

        let pci_dev = self.devices[dev_idx].device.clone();

        let driver = self
            .drivers
            .iter_mut()
            .find(|(id, _)| *id == drv_id)
            .map(|(_, d)| d)
            .ok_or(DriverError::DriverNotFound { driver: drv_id })?;

        driver.start(&pci_dev).inspect_err(|e| {
            self.counters.failed += 1;
            self.devices[dev_idx].driver_state = DriverState::Failed(e.to_string());
        })?;

        let record = &mut self.devices[dev_idx];
        DriverState::check_transition(dev_id, &record.driver_state, &DriverState::Running)?;
        record.driver_state = DriverState::Running;
        record.health = driver.health();
        self.counters.running += 1;
        Ok(())
    }

    /// Stops active operation on `dev_id`.
    pub fn stop_device(&mut self, dev_id: DeviceId) -> DriverResult<()> {
        self.check_capability(dev_id, DeviceCapability::Operate)?;

        let dev_idx = self
            .devices
            .iter()
            .position(|r| r.id == dev_id)
            .ok_or(DriverError::DeviceNotFound { device: dev_id })?;

        let drv_id = self.devices[dev_idx]
            .driver_id
            .ok_or(DriverError::NoDriverBound { device: dev_id })?;

        let pci_dev = self.devices[dev_idx].device.clone();

        let driver = self
            .drivers
            .iter_mut()
            .find(|(id, _)| *id == drv_id)
            .map(|(_, d)| d)
            .ok_or(DriverError::DriverNotFound { driver: drv_id })?;

        driver.stop(&pci_dev).inspect_err(|e| {
            self.counters.failed += 1;
            self.devices[dev_idx].driver_state = DriverState::Failed(e.to_string());
        })?;

        let record = &mut self.devices[dev_idx];
        DriverState::check_transition(dev_id, &record.driver_state, &DriverState::Stopped)?;
        record.driver_state = DriverState::Stopped;
        record.health = driver.health();
        Ok(())
    }

    /// Resets `dev_id` back to default initialized state.
    pub fn reset_device(&mut self, dev_id: DeviceId) -> DriverResult<()> {
        self.check_capability(dev_id, DeviceCapability::Reset)?;

        let dev_idx = self
            .devices
            .iter()
            .position(|r| r.id == dev_id)
            .ok_or(DriverError::DeviceNotFound { device: dev_id })?;

        let drv_id = self.devices[dev_idx]
            .driver_id
            .ok_or(DriverError::NoDriverBound { device: dev_id })?;

        let pci_dev = self.devices[dev_idx].device.clone();

        let driver = self
            .drivers
            .iter_mut()
            .find(|(id, _)| *id == drv_id)
            .map(|(_, d)| d)
            .ok_or(DriverError::DriverNotFound { driver: drv_id })?;

        driver.reset(&pci_dev).inspect_err(|e| {
            self.counters.failed += 1;
            self.devices[dev_idx].driver_state = DriverState::Failed(e.to_string());
        })?;

        let record = &mut self.devices[dev_idx];
        DriverState::check_transition(dev_id, &record.driver_state, &DriverState::Initialized)?;
        record.driver_state = DriverState::Initialized;
        record.health = driver.health();
        Ok(())
    }

    /// Hot-removes `dev_id` from the manager (gated by [`HotplugPolicy`]).
    pub fn remove_device(&mut self, dev_id: DeviceId) -> DriverResult<()> {
        let dev_idx = self
            .devices
            .iter()
            .position(|r| r.id == dev_id)
            .ok_or(DriverError::DeviceNotFound { device: dev_id })?;

        let record = &self.devices[dev_idx];

        // Check hotplug capability & policy
        match self.config.hotplug_policy {
            HotplugPolicy::DenyRemoval => {
                return Err(DriverError::HotplugDenied {
                    device: dev_id,
                    reason: "hot-removal denied by policy (DenyRemoval)".to_string(),
                });
            }
            HotplugPolicy::AllowReported => {
                if !record.device.hotplug_support().is_supported() {
                    return Err(DriverError::HotplugUnsupported { device: dev_id });
                }
            }
            HotplugPolicy::AllowAny => {}
        }

        // Stop device if running
        if record.driver_state.is_running() {
            self.stop_device(dev_id)?;
        }

        self.devices.remove(dev_idx);
        self.counters.hotplug_events += 1;
        Ok(())
    }

    /// Explicitly grants a capability to `dev_id`.
    ///
    /// For [`DeviceCapability::DmaAccess`], validates that `IommuPolicy` permits it.
    pub fn grant_capability(
        &mut self,
        dev_id: DeviceId,
        capability: DeviceCapability,
    ) -> DriverResult<()> {
        let record = self
            .devices
            .iter_mut()
            .find(|r| r.id == dev_id)
            .ok_or(DriverError::DeviceNotFound { device: dev_id })?;

        let declared = record.device.declared_capabilities();
        if !declared.contains(capability) {
            return Err(DriverError::CapabilityNotDeclared {
                device: dev_id,
                capability,
                declared: declared.labels(),
            });
        }

        // Gated IOMMU policy for DMA
        if capability == DeviceCapability::DmaAccess
            && !self.config.iommu_policy.permits_dma_grant()
        {
            return Err(DriverError::CapabilityDenied {
                device: dev_id,
                capability,
                reason: format!(
                    "IOMMU policy '{}' refuses unmanaged DMA access",
                    self.config.iommu_policy
                ),
            });
        }

        record.granted_capabilities.grant(capability);
        Ok(())
    }

    /// Revokes a capability from `dev_id`.
    pub fn revoke_capability(
        &mut self,
        dev_id: DeviceId,
        capability: DeviceCapability,
    ) -> DriverResult<()> {
        let record = self
            .devices
            .iter_mut()
            .find(|r| r.id == dev_id)
            .ok_or(DriverError::DeviceNotFound { device: dev_id })?;

        record.granted_capabilities.revoke(capability);
        Ok(())
    }

    /// Verifies that `dev_id` holds `capability`.
    pub fn check_capability(
        &self,
        dev_id: DeviceId,
        capability: DeviceCapability,
    ) -> DriverResult<()> {
        let record = self
            .devices
            .iter()
            .find(|r| r.id == dev_id)
            .ok_or(DriverError::DeviceNotFound { device: dev_id })?;

        if record.granted_capabilities.contains(capability) {
            Ok(())
        } else {
            Err(DriverError::CapabilityDenied {
                device: dev_id,
                capability,
                reason: format!(
                    "capability '{}' has not been granted (granted: {})",
                    capability, record.granted_capabilities
                ),
            })
        }
    }

    /// Read-only slice of managed device records.
    pub fn devices(&self) -> &[DeviceRecord] {
        &self.devices
    }

    /// Read-only access to a specific device record.
    pub fn device(&self, dev_id: DeviceId) -> Option<&DeviceRecord> {
        self.devices.iter().find(|r| r.id == dev_id)
    }

    /// Generates snapshots of all managed devices for JSON / CLI / diagnostics.
    pub fn snapshot(&self) -> Vec<DeviceSnapshot> {
        self.devices
            .iter()
            .map(|r| {
                let pci = &r.device;
                let declared = pci.declared_capabilities();
                let granted_labels = r
                    .granted_capabilities
                    .labels()
                    .into_iter()
                    .map(String::from)
                    .collect();
                let declared_labels = declared.labels().into_iter().map(String::from).collect();

                DeviceSnapshot {
                    id: r.id,
                    address: pci.address().to_string(),
                    vendor_id: pci.vendor_id(),
                    device_id: pci.device_id(),
                    class_name: pci.class().name().to_string(),
                    driver_name: r.driver_name.clone(),
                    driver_state: r.driver_state.to_string(),
                    declared_capabilities: declared_labels,
                    granted_capabilities: granted_labels,
                    dma_ready: pci.dma_readiness().is_declared_ready(),
                    hotplug: pci.hotplug_support().to_string(),
                    health: format!("{:?}", r.health),
                }
            })
            .collect()
    }

    /// Calculates overall health summary across managed devices.
    pub fn health_summary(&self) -> HealthSummary {
        let mut summary = HealthSummary {
            total_devices: self.devices.len(),
            ..Default::default()
        };

        for r in &self.devices {
            match r.health {
                DeviceHealth::Healthy => summary.healthy += 1,
                DeviceHealth::Warning => summary.warning += 1,
                DeviceHealth::Degraded => summary.degraded += 1,
                DeviceHealth::Error => summary.failed += 1,
                DeviceHealth::Unknown => summary.unknown += 1,
            }
        }

        summary
    }

    /// Performance and event counters.
    pub fn counters(&self) -> ManagerCounters {
        self.counters
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pci::simulated::SimulatedPciBackend;

    #[test]
    fn manager_enumeration_and_auto_bind() {
        let mut backend = SimulatedPciBackend::new();
        let mut bus = PciBus::new(&mut backend);
        let mut mgr = DeviceManager::with_default_drivers(DeviceManagerConfig::default());

        let count = mgr.enumerate_bus(&mut bus).unwrap();
        assert_eq!(count, 5);
        assert_eq!(mgr.devices().len(), 5);

        // Verify NVMe device was bound to simulated nvme driver
        let nvme = mgr
            .devices()
            .iter()
            .find(|r| r.device.vendor_id() == 0x1B36)
            .unwrap();
        assert_eq!(nvme.driver_name.as_deref(), Some("qeos-nvme-driver"));
        assert_eq!(nvme.driver_state, DriverState::Bound);

        // Verify GPU device was bound to simulated gpu driver
        let gpu = mgr
            .devices()
            .iter()
            .find(|r| r.device.vendor_id() == 0x10DE)
            .unwrap();
        assert_eq!(gpu.driver_name.as_deref(), Some("qeos-gpu-driver"));
        assert_eq!(gpu.driver_state, DriverState::Bound);
    }

    #[test]
    fn manager_lifecycle_transitions() {
        let mut backend = SimulatedPciBackend::new();
        let mut bus = PciBus::new(&mut backend);
        let mut mgr = DeviceManager::with_default_drivers(DeviceManagerConfig::default());

        mgr.enumerate_bus(&mut bus).unwrap();
        let dev_id = mgr.devices()[1].id;

        // Initialize -> Start -> Stop -> Reset
        assert!(mgr.initialize_device(dev_id).is_ok());
        assert_eq!(
            mgr.device(dev_id).unwrap().driver_state,
            DriverState::Initialized
        );

        assert!(mgr.start_device(dev_id).is_ok());
        assert_eq!(
            mgr.device(dev_id).unwrap().driver_state,
            DriverState::Running
        );

        assert!(mgr.stop_device(dev_id).is_ok());
        assert_eq!(
            mgr.device(dev_id).unwrap().driver_state,
            DriverState::Stopped
        );

        mgr.grant_capability(dev_id, DeviceCapability::Reset)
            .unwrap();
        assert!(mgr.reset_device(dev_id).is_ok());
        assert_eq!(
            mgr.device(dev_id).unwrap().driver_state,
            DriverState::Initialized
        );
    }

    #[test]
    fn manager_dma_grant_checks_iommu_policy() {
        let mut backend = SimulatedPciBackend::new();
        let mut bus = PciBus::new(&mut backend);
        let mut mgr = DeviceManager::with_default_drivers(DeviceManagerConfig {
            iommu_policy: IommuPolicy::DenyUnmanagedDma,
            ..Default::default()
        });

        mgr.enumerate_bus(&mut bus).unwrap();
        let dev_id = mgr.devices()[1].id; // NVMe

        // Default DenyUnmanagedDma refuses DMA grant
        assert!(matches!(
            mgr.grant_capability(dev_id, DeviceCapability::DmaAccess),
            Err(DriverError::CapabilityDenied { .. })
        ));

        // Switch to AllowUnmanagedDma
        mgr.config_mut().iommu_policy = IommuPolicy::AllowUnmanagedDma;
        assert!(mgr
            .grant_capability(dev_id, DeviceCapability::DmaAccess)
            .is_ok());
        assert!(mgr
            .check_capability(dev_id, DeviceCapability::DmaAccess)
            .is_ok());
    }

    #[test]
    fn manager_hotplug_removal_checks_policy() {
        let mut backend = SimulatedPciBackend::new();
        let mut bus = PciBus::new(&mut backend);
        let mut mgr = DeviceManager::with_default_drivers(DeviceManagerConfig {
            hotplug_policy: HotplugPolicy::DenyRemoval,
            ..Default::default()
        });

        mgr.enumerate_bus(&mut bus).unwrap();
        let gpu_id = mgr
            .devices()
            .iter()
            .find(|r| r.device.vendor_id() == 0x10DE)
            .unwrap()
            .id;

        // Removal refused with DenyRemoval
        assert!(matches!(
            mgr.remove_device(gpu_id),
            Err(DriverError::HotplugDenied { .. })
        ));

        // Allowed with AllowReported (GPU fixture supports hotplug)
        mgr.config_mut().hotplug_policy = HotplugPolicy::AllowReported;
        assert!(mgr.remove_device(gpu_id).is_ok());
        assert!(mgr.device(gpu_id).is_none());
    }
}
