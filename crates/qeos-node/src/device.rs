//! P7.2-03/P7.2-04 — Normalized hardware [`Device`] model and capability inventory.
//!
//! Hardware of many different origins (CPU, PCIe, GPU, storage, power, QPU...)
//! is normalized into a single [`Device`] record carrying identity, class,
//! capabilities, topology, state, health and telemetry. The capability set is
//! **per device**, because two devices of the same class need not share the
//! same capabilities.

use serde::{Deserialize, Serialize};

use crate::health::HealthStatus;

/// Broad hardware class of a device.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DeviceClass {
    #[default]
    Other,
    Cpu,
    Memory,
    Storage,
    Network,
    Gpu,
    Accelerator,
    Qpu,
    Power,
    Sensor,
    Tpm,
    Usb,
    Pci,
    Virtualization,
}

/// Stable identity of a hardware device.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceIdentity {
    pub device_id: String,
    pub class: DeviceClass,
    pub vendor: String,
    pub model: String,
    #[serde(default)]
    pub serial_number: Option<String>,
    #[serde(default)]
    pub firmware_version: Option<String>,
}

/// Capabilities that a device can advertise. Absence of a capability flag means
/// the device is *not* known to support that feature — never assume support.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    pub compute: bool,
    pub memory: bool,
    pub storage: bool,
    pub network: bool,
    pub gpu: bool,
    pub accelerator: bool,
    pub qpu: bool,
    pub virtualization: bool,
    pub energy: bool,
    pub telemetry: bool,
    pub reset: bool,
}

impl DeviceCapabilities {
    /// Number of advertised capability flags.
    pub fn count(self) -> u32 {
        [
            self.compute,
            self.memory,
            self.storage,
            self.network,
            self.gpu,
            self.accelerator,
            self.qpu,
            self.virtualization,
            self.energy,
            self.telemetry,
            self.reset,
        ]
        .iter()
        .filter(|b| **b)
        .count() as u32
    }
}

impl From<crate::discovery::DeviceClassTag> for DeviceCapabilities {
    fn from(class: crate::discovery::DeviceClassTag) -> Self {
        match class {
            crate::discovery::DeviceClassTag::Cpu => DeviceCapabilities {
                compute: true,
                telemetry: true,
                ..Default::default()
            },
            crate::discovery::DeviceClassTag::Memory => DeviceCapabilities {
                memory: true,
                telemetry: true,
                ..Default::default()
            },
            crate::discovery::DeviceClassTag::Gpu => DeviceCapabilities {
                compute: true,
                gpu: true,
                reset: true,
                telemetry: true,
                ..Default::default()
            },
            crate::discovery::DeviceClassTag::Storage => DeviceCapabilities {
                storage: true,
                ..Default::default()
            },
            crate::discovery::DeviceClassTag::Network => DeviceCapabilities {
                network: true,
                telemetry: true,
                ..Default::default()
            },
            crate::discovery::DeviceClassTag::Qpu => DeviceCapabilities {
                compute: true,
                qpu: true,
                reset: true,
                accelerator: true,
                ..Default::default()
            },
            crate::discovery::DeviceClassTag::Power => DeviceCapabilities {
                energy: true,
                telemetry: true,
                ..Default::default()
            },
            crate::discovery::DeviceClassTag::Sensor => DeviceCapabilities {
                telemetry: true,
                energy: true,
                ..Default::default()
            },
            crate::discovery::DeviceClassTag::Virtualization => DeviceCapabilities {
                virtualization: true,
                ..Default::default()
            },
            crate::discovery::DeviceClassTag::Accelerator => DeviceCapabilities {
                compute: true,
                accelerator: true,
                reset: true,
                telemetry: true,
                ..Default::default()
            },
            crate::discovery::DeviceClassTag::Other => DeviceCapabilities::default(),
        }
    }
}

/// Physical/logical placement of a device within the node.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceTopology {
    #[serde(default)]
    pub parent_device_id: Option<String>,
    #[serde(default)]
    pub bus: Option<String>,
    #[serde(default)]
    pub slot: Option<String>,
    #[serde(default)]
    pub numa_node: Option<u32>,
}

/// Operational state of a device at the node level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DeviceOperationalState {
    #[default]
    Discovered,
    Initializing,
    Ready,
    Active,
    Degraded,
    Resetting,
    Failed,
    Removing,
    Removed,
}

/// Telemetry summary attached to a device (values are only present when measured).
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct DeviceTelemetry {
    #[serde(default)]
    pub temperature_celsius: Option<f64>,
    #[serde(default)]
    pub utilization_percent: Option<f64>,
    #[serde(default)]
    pub power_watts: Option<f64>,
}

/// The normalized hardware device record.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Device {
    pub identity: DeviceIdentity,
    pub capabilities: DeviceCapabilities,
    pub topology: DeviceTopology,
    pub state: DeviceOperationalState,
    pub health: HealthStatus,
    pub telemetry: DeviceTelemetry,
}

impl Device {
    pub fn new(identity: DeviceIdentity, capabilities: DeviceCapabilities) -> Self {
        Self {
            identity,
            capabilities,
            ..Default::default()
        }
    }

    pub fn device_id(&self) -> &str {
        &self.identity.device_id
    }

    /// Whether the device is usable for work in its current state.
    pub fn is_usable(&self) -> bool {
        matches!(
            self.state,
            DeviceOperationalState::Ready | DeviceOperationalState::Active
        ) && matches!(self.health, HealthStatus::Healthy | HealthStatus::Degraded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cpu_identity() -> DeviceIdentity {
        DeviceIdentity {
            device_id: "cpu0".into(),
            class: DeviceClass::Cpu,
            vendor: "VendorA".into(),
            model: "X1".into(),
            ..Default::default()
        }
    }

    #[test]
    fn capability_count() {
        let c = DeviceCapabilities {
            compute: true,
            telemetry: true,
            ..Default::default()
        };
        assert_eq!(c.count(), 2);
        assert_eq!(DeviceCapabilities::default().count(), 0);
    }

    #[test]
    fn gpu_class_yields_gpu_capability() {
        use crate::discovery::DeviceClassTag;
        let caps = DeviceCapabilities::from(DeviceClassTag::Gpu);
        assert!(caps.gpu);
        assert!(caps.reset);
        assert!(caps.compute);
        assert!(!caps.qpu);
    }

    #[test]
    fn qpu_class_does_not_imply_gpu() {
        use crate::discovery::DeviceClassTag;
        let caps = DeviceCapabilities::from(DeviceClassTag::Qpu);
        assert!(caps.qpu);
        assert!(!caps.gpu);
    }

    #[test]
    fn device_usable_when_ready_and_healthy() {
        let d = Device {
            identity: cpu_identity(),
            state: DeviceOperationalState::Ready,
            health: HealthStatus::Healthy,
            ..Default::default()
        };
        assert!(d.is_usable());
    }

    #[test]
    fn device_not_usable_when_failed() {
        let d = Device {
            identity: cpu_identity(),
            state: DeviceOperationalState::Failed,
            health: HealthStatus::Failed,
            ..Default::default()
        };
        assert!(!d.is_usable());
    }

    #[test]
    fn device_not_usable_when_failed_health() {
        let d = Device {
            identity: cpu_identity(),
            state: DeviceOperationalState::Ready,
            health: HealthStatus::Failed,
            ..Default::default()
        };
        assert!(!d.is_usable());
    }
}
