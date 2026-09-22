//! P7.2-03 — Hardware Discovery.
//!
//! Discovers hardware where supported and normalizes it into [`Device`] records.
//!
//! The platform is explicitly honest about what it can observe:
//! - CPU topology (`std::thread::available_parallelism`) is REAL on the host.
//! - GPU/QPU/PCI/TPM/etc. are `UNAVAILABLE` from a host-source unless a real
//!   [`HardwareSource`] provides them.
//!
//! A [`HardwareSource`] trait allows real platform sources to be injected
//! without coupling this platform to any specific OS or vendor.
//! [`SimulatedDiscoverySource`] exists purely for tests and CI.

use std::thread;

use crate::device::{Device, DeviceCapabilities, DeviceClass, DeviceIdentity, DeviceTopology};

/// Discovery-time classification of a raw hardware entry, before normalization
/// into the richer [`DeviceClass`] carried by a [`Device`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceClassTag {
    Cpu,
    Memory,
    Storage,
    Network,
    Gpu,
    Accelerator,
    Qpu,
    Power,
    Sensor,
    Virtualization,
    Other,
}

impl From<DeviceClassTag> for DeviceClass {
    fn from(tag: DeviceClassTag) -> Self {
        match tag {
            DeviceClassTag::Cpu => DeviceClass::Cpu,
            DeviceClassTag::Memory => DeviceClass::Memory,
            DeviceClassTag::Storage => DeviceClass::Storage,
            DeviceClassTag::Network => DeviceClass::Network,
            DeviceClassTag::Gpu => DeviceClass::Gpu,
            DeviceClassTag::Accelerator => DeviceClass::Accelerator,
            DeviceClassTag::Qpu => DeviceClass::Qpu,
            DeviceClassTag::Power => DeviceClass::Power,
            DeviceClassTag::Sensor => DeviceClass::Sensor,
            DeviceClassTag::Virtualization => DeviceClass::Virtualization,
            DeviceClassTag::Other => DeviceClass::Other,
        }
    }
}

/// Error returned by a hardware source. A source that cannot observe a class
/// of hardware must report [`DiscoveryError::Unavailable`], never fabricate.
#[derive(Debug, thiserror::Error)]
pub enum DiscoveryError {
    #[error("hardware class unavailable from this source: {0}")]
    Unavailable(&'static str),
    #[error("discovery failed: {0}")]
    Failed(String),
}

/// A provider of raw hardware discoveries. Implementations are platform/vendor
/// specific; this platform never assumes a provider exists.
pub trait HardwareSource: Send + Sync {
    /// Return discovered devices, or [`DiscoveryError::Unavailable`] for
    /// classes this source cannot observe.
    fn discover(&self) -> std::result::Result<Vec<DiscoveryEntry>, DiscoveryError>;

    /// Human-readable description of this source (for classification).
    fn describe(&self) -> &'static str;
}

/// A single raw discovery entry produced by a source.
#[derive(Debug, Clone)]
pub struct DiscoveryEntry {
    pub tag: DeviceClassTag,
    pub id: String,
    pub vendor: String,
    pub model: String,
    pub serial_number: Option<String>,
    pub firmware_version: Option<String>,
    pub topology: DeviceTopology,
}

/// Real host source: observes CPU topology (REAL) and marks the rest unavailable.
pub struct HostDiscoverySource;

impl HardwareSource for HostDiscoverySource {
    fn discover(&self) -> std::result::Result<Vec<DiscoveryEntry>, DiscoveryError> {
        let mut entries = Vec::new();

        // CPU parallelism is genuinely observable on the host.
        if let Ok(count) = thread::available_parallelism() {
            entries.push(DiscoveryEntry {
                tag: DeviceClassTag::Cpu,
                id: "cpu0".to_string(),
                vendor: "host".to_string(),
                model: format!("{} logical processors", count.get()),
                serial_number: None,
                firmware_version: None,
                topology: DeviceTopology::default(),
            });
        }

        // Everything else is not directly observable by this generic host source.
        Ok(entries)
    }

    fn describe(&self) -> &'static str {
        "host (std) — CPU parallelism only; other classes unavailable"
    }
}

/// Simulated source used only for tests and CI. Never evidence of real hardware.
pub struct SimulatedDiscoverySource;

impl HardwareSource for SimulatedDiscoverySource {
    fn discover(&self) -> std::result::Result<Vec<DiscoveryEntry>, DiscoveryError> {
        Ok(vec![
            DiscoveryEntry {
                tag: DeviceClassTag::Cpu,
                id: "cpu0".to_string(),
                vendor: "SimVendor".to_string(),
                model: "SimCPU-8".to_string(),
                serial_number: Some("SIM-CPU-0001".to_string()),
                firmware_version: Some("1.0.0".to_string()),
                topology: DeviceTopology {
                    numa_node: Some(0),
                    ..Default::default()
                },
            },
            DiscoveryEntry {
                tag: DeviceClassTag::Gpu,
                id: "gpu0".to_string(),
                vendor: "SimVendor".to_string(),
                model: "SimGPU-1".to_string(),
                serial_number: Some("SIM-GPU-0001".to_string()),
                firmware_version: Some("2.1.0".to_string()),
                topology: DeviceTopology {
                    bus: Some("pci0000:00".to_string()),
                    slot: Some("01.0".to_string()),
                    ..Default::default()
                },
            },
            DiscoveryEntry {
                tag: DeviceClassTag::Qpu,
                id: "qpu0".to_string(),
                vendor: "SimVendor".to_string(),
                model: "SimQPU-tetron".to_string(),
                serial_number: Some("SIM-QPU-0001".to_string()),
                firmware_version: Some("0.9.0".to_string()),
                topology: DeviceTopology::default(),
            },
        ])
    }

    fn describe(&self) -> &'static str {
        "simulated (tests / CI only)"
    }
}

/// Normalizes raw discovery entries into [`Device`] records.
pub fn normalize(entry: DiscoveryEntry) -> Device {
    let class: DeviceClass = entry.tag.into();
    let caps = DeviceCapabilities::from(entry.tag);
    Device {
        identity: DeviceIdentity {
            device_id: entry.id,
            class,
            vendor: entry.vendor,
            model: entry.model,
            serial_number: entry.serial_number,
            firmware_version: entry.firmware_version,
        },
        capabilities: caps,
        topology: entry.topology,
        ..Default::default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn host_source_observes_cpu_or_none() {
        // On any host this must not panic; it returns CPU when parallelism is
        // available, otherwise an empty (honest) list — never fake GPU/QPU.
        let src = HostDiscoverySource;
        let devs = src.discover().unwrap();
        for d in devs {
            assert_eq!(d.tag, DeviceClassTag::Cpu);
        }
    }

    #[test]
    fn simulated_source_normalizes_to_devices() {
        let src = SimulatedDiscoverySource;
        let entries = src.discover().unwrap();
        let devices: Vec<Device> = entries.into_iter().map(normalize).collect();
        assert_eq!(devices.len(), 3);
        let gpu = devices
            .iter()
            .find(|d| d.identity.device_id == "gpu0")
            .unwrap();
        assert_eq!(gpu.identity.class, DeviceClass::Gpu);
        assert!(gpu.capabilities.gpu);
        assert!(gpu.capabilities.reset);
        assert!(!gpu.capabilities.qpu);
        let qpu = devices
            .iter()
            .find(|d| d.identity.device_id == "qpu0")
            .unwrap();
        assert!(qpu.capabilities.qpu);
        assert!(!qpu.capabilities.gpu);
    }

    #[test]
    fn tag_to_class_mapping() {
        assert_eq!(DeviceClass::from(DeviceClassTag::Cpu), DeviceClass::Cpu);
        assert_eq!(DeviceClass::from(DeviceClassTag::Gpu), DeviceClass::Gpu);
        assert_eq!(DeviceClass::from(DeviceClassTag::Qpu), DeviceClass::Qpu);
        assert_eq!(DeviceClass::from(DeviceClassTag::Power), DeviceClass::Power);
    }
}
