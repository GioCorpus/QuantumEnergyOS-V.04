//! Experimental / future physical QPU backend (Phase 3, §7 and §29).
//!
//! CLASSIFICATION: FUTURE (interface) / ABSTRACT (no physical device present)
//!
//! This backend is disabled by default and never pretends to be connected to
//! hardware. Without an attached vendor adapter every operation returns
//! [`QuantumError::UnsupportedHardware`].
//!
//! There is no `Majorana2::connect()` style API here, because no public,
//! documented programming interface for such hardware exists. Instead the HAL
//! defines the *adapter contract* ([`QpuVendorAdapter`]) and the interface
//! families it may appear as ([`QpuInterface`]). When a real interface becomes
//! available, implementing the adapter is the only change required: the quantum
//! runtime, IR, job system, policy manager and dashboard stay untouched.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::device::{BackendClass, DeviceHealth, DeviceInfo, DeviceState, QuantumDevice};
use crate::error::{QuantumError, Result};
use crate::ir::QuantumIR;
use crate::job::{JobHandle, JobStatus, QuantumJob, QuantumResult, SimulationConfig};

/// Transport/interface family a future QPU may expose.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QpuInterface {
    /// PCIe card.
    Pcie,
    /// USB-attached controller.
    Usb,
    /// Network-addressable QPU service.
    NetworkQpu,
    /// Vendor SDK/runtime.
    VendorSdk(String),
    /// Custom FPGA control plane.
    CustomFpga,
    /// Cryogenic controller (for example a dilution-refrigerator control unit).
    CryogenicController,
}

impl QpuInterface {
    /// Stable label used in diagnostics and audit metadata.
    pub fn label(&self) -> String {
        match self {
            QpuInterface::Pcie => "pcie".to_string(),
            QpuInterface::Usb => "usb".to_string(),
            QpuInterface::NetworkQpu => "network-qpu".to_string(),
            QpuInterface::VendorSdk(name) => format!("vendor-sdk:{}", name),
            QpuInterface::CustomFpga => "custom-fpga".to_string(),
            QpuInterface::CryogenicController => "cryogenic-controller".to_string(),
        }
    }
}

/// Contract a real vendor integration must implement.
///
/// Implementations live outside the HAL core (vendor adapter crates) and are the
/// only place allowed to talk to hardware-specific APIs.
pub trait QpuVendorAdapter: Send + Sync {
    /// Interface family this adapter speaks.
    fn interface(&self) -> QpuInterface;

    /// Human readable adapter description (vendor, SDK version, docs link).
    fn describe(&self) -> String;

    /// Device metadata reported by the adapter.
    fn device_info(&self) -> DeviceInfo;

    /// Current device health as reported by the adapter.
    fn health(&self) -> DeviceHealth;

    /// Calibrate the physical device.
    fn calibrate(&mut self) -> Result<()>;

    /// Submit device-level IR to the physical device.
    fn submit_ir(
        &mut self,
        job_id: &str,
        ir: &QuantumIR,
        config: &SimulationConfig,
    ) -> Result<QuantumResult>;
}

/// Experimental QPU device: the future physical-hardware boundary.
#[derive(Default)]
pub struct ExperimentalQpuDevice {
    info: DeviceInfo,
    state: DeviceState,
    health: DeviceHealth,
    adapter: Option<Box<dyn QpuVendorAdapter>>,
    next_handle: u64,
    jobs: HashMap<u64, (JobStatus, Option<QuantumResult>, Option<String>)>,
}

impl ExperimentalQpuDevice {
    /// Disabled experimental device with no vendor adapter attached.
    pub fn new() -> Self {
        Self {
            info: DeviceInfo {
                id: "qpu0".to_string(),
                name: "Experimental QPU (no adapter)".to_string(),
                vendor: "unattached".to_string(),
                model: "experimental-qpu".to_string(),
                class: BackendClass::Experimental,
                qubits: 0,
                logical_qubits: 0,
                supported_gates: Vec::new(),
                firmware_version: None,
                simulation_only: false,
            },
            state: DeviceState::Disabled,
            health: DeviceHealth::Unavailable,
            adapter: None,
            next_handle: 1,
            jobs: HashMap::new(),
        }
    }

    /// Attach a vendor adapter. Only call this with a real, documented interface.
    pub fn with_adapter(adapter: Box<dyn QpuVendorAdapter>) -> Self {
        let info = adapter.device_info();
        let health = adapter.health();
        Self {
            info,
            state: DeviceState::Uninitialized,
            health,
            adapter: Some(adapter),
            next_handle: 1,
            jobs: HashMap::new(),
        }
    }

    /// True when a vendor adapter is attached.
    pub fn has_adapter(&self) -> bool {
        self.adapter.is_some()
    }

    /// Interface family of the attached adapter, when present.
    pub fn interface(&self) -> Option<QpuInterface> {
        self.adapter.as_ref().map(|adapter| adapter.interface())
    }

    /// Description of the attached adapter, when present.
    pub fn adapter_description(&self) -> Option<String> {
        self.adapter.as_ref().map(|adapter| adapter.describe())
    }

    /// Error returned whenever the physical path is unavailable.
    fn no_hardware() -> QuantumError {
        QuantumError::UnsupportedHardware(
            "experimental QPU backend is disabled: no vendor adapter is attached and no public hardware interface is assumed"
                .to_string(),
        )
    }

    fn adapters_mut(&mut self) -> Result<&mut Box<dyn QpuVendorAdapter>> {
        self.adapter.as_mut().ok_or_else(Self::no_hardware)
    }
}

impl QuantumDevice for ExperimentalQpuDevice {
    fn device_info(&self) -> DeviceInfo {
        self.info.clone()
    }

    fn initialize(&mut self) -> Result<()> {
        let (health, interface) = {
            let adapter = self.adapters_mut()?;
            (adapter.health(), adapter.interface().label())
        };

        self.health = health;
        self.state = DeviceState::Ready;
        tracing::info!(
            device = %self.info.id,
            interface = %interface,
            "quantum-hal: experimental QPU initialised through a vendor adapter"
        );
        Ok(())
    }

    fn calibrate(&mut self) -> Result<()> {
        if self.state != DeviceState::Ready {
            return Err(QuantumError::DeviceNotInitialized(
                "call initialize() before calibrate()".to_string(),
            ));
        }
        let adapter = self.adapters_mut()?;
        adapter.calibrate()
    }

    fn reset(&mut self) -> Result<()> {
        if !self.has_adapter() {
            return Err(Self::no_hardware());
        }
        self.jobs.clear();
        self.state = DeviceState::Ready;
        Ok(())
    }

    fn submit(&mut self, job: QuantumJob) -> Result<JobHandle> {
        if self.state != DeviceState::Ready {
            return Err(QuantumError::DeviceNotInitialized(format!(
                "device '{}' is in state {:?}; call initialize() first",
                self.info.id, self.state
            )));
        }

        let handle = JobHandle(self.next_handle);
        self.next_handle += 1;
        self.jobs
            .insert(handle.0, (JobStatus::Running, None, None));

        let outcome = {
            let adapter = self.adapters_mut()?;
            adapter.submit_ir(&job.job_id, &job.circuit, &job.simulation)
        };

        match outcome {
            Ok(result) => {
                self.jobs
                    .insert(handle.0, (JobStatus::Completed, Some(result), None));
            }
            Err(err) => {
                tracing::warn!(
                    device = %self.info.id,
                    job_id = %job.job_id,
                    error = %err,
                    "quantum-hal: experimental QPU job failed"
                );
                self.jobs
                    .insert(handle.0, (JobStatus::Failed, None, Some(err.to_string())));
            }
        }

        Ok(handle)
    }

    fn poll(&mut self, job: JobHandle) -> Result<JobStatus> {
        self.jobs
            .get(&job.0)
            .map(|(status, _, _)| *status)
            .ok_or_else(|| QuantumError::JobNotFound(format!("handle {}", job.0)))
    }

    fn read_result(&mut self, job: JobHandle) -> Result<QuantumResult> {
        let (_, result, error) = self
            .jobs
            .get(&job.0)
            .ok_or_else(|| QuantumError::JobNotFound(format!("handle {}", job.0)))?;

        match result {
            Some(result) => Ok(result.clone()),
            None => match error {
                Some(message) => Err(QuantumError::BackendNotAvailable(message.clone())),
                None => Err(QuantumError::BackendNotAvailable(format!(
                    "job handle {} has no result yet",
                    job.0
                ))),
            },
        }
    }

    fn health(&self) -> DeviceHealth {
        if !self.has_adapter() {
            return DeviceHealth::Unavailable;
        }
        match self.state {
            DeviceState::Ready => self.health,
            DeviceState::Uninitialized => DeviceHealth::Unknown,
            _ => DeviceHealth::Unavailable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Test-only vendor adapter. Proves the adapter interface works end to end
    /// without claiming that any physical device exists.
    struct ScriptedAdapter {
        calibrated: bool,
    }

    impl QpuVendorAdapter for ScriptedAdapter {
        fn interface(&self) -> QpuInterface {
            QpuInterface::VendorSdk("test-sdk".to_string())
        }

        fn describe(&self) -> String {
            "test-only adapter (no hardware access)".to_string()
        }

        fn device_info(&self) -> DeviceInfo {
            DeviceInfo {
                id: "qpu-test".to_string(),
                name: "Scripted Test QPU".to_string(),
                vendor: "test".to_string(),
                model: "scripted".to_string(),
                class: BackendClass::Experimental,
                qubits: 4,
                logical_qubits: 0,
                supported_gates: vec!["H".to_string(), "CNOT".to_string(), "Measure".to_string()],
                firmware_version: None,
                simulation_only: false,
            }
        }

        fn health(&self) -> DeviceHealth {
            if self.calibrated {
                DeviceHealth::Healthy
            } else {
                DeviceHealth::Warning
            }
        }

        fn calibrate(&mut self) -> Result<()> {
            self.calibrated = true;
            Ok(())
        }

        fn submit_ir(
            &mut self,
            job_id: &str,
            _ir: &QuantumIR,
            config: &SimulationConfig,
        ) -> Result<QuantumResult> {
            if !self.calibrated {
                return Err(QuantumError::CalibrationFailed(
                    "adapter requires calibration before execution".to_string(),
                ));
            }
            let mut result = QuantumResult::new(job_id, "qpu-test", config.shots);
            result.status = JobStatus::Completed;
            result.simulation_only = false;
            result.push_note("TEST ADAPTER: scripted result, not physical hardware");
            Ok(result)
        }
    }

    fn scripted_device() -> ExperimentalQpuDevice {
        ExperimentalQpuDevice::with_adapter(Box::new(ScriptedAdapter { calibrated: false }))
    }

    #[test]
    fn test_disabled_device_returns_unsupported_hardware() {
        let mut device = ExperimentalQpuDevice::new();
        let info = device.device_info();

        assert_eq!(info.class, BackendClass::Experimental);
        assert!(!device.has_adapter());
        assert_eq!(device.health(), DeviceHealth::Unavailable);
        assert!(matches!(
            device.initialize(),
            Err(QuantumError::UnsupportedHardware(_))
        ));
        assert!(matches!(
            device.calibrate(),
            Err(QuantumError::DeviceNotInitialized(_))
        ));
        assert!(matches!(
            device.submit(QuantumJob::new("qpu0", QuantumIR::new("empty", 1))),
            Err(QuantumError::DeviceNotInitialized(_))
        ));
        assert!(matches!(
            device.poll(JobHandle(1)),
            Err(QuantumError::JobNotFound(_))
        ));
        assert_eq!(device.interface(), None);
    }

    #[test]
    fn test_interface_labels() {
        assert_eq!(QpuInterface::Pcie.label(), "pcie");
        assert_eq!(
            QpuInterface::CryogenicController.label(),
            "cryogenic-controller"
        );
        assert_eq!(
            QpuInterface::VendorSdk("v".to_string()).label(),
            "vendor-sdk:v"
        );
    }

    #[test]
    fn test_adapter_path_works_end_to_end() {
        let mut device = scripted_device();
        assert!(device.has_adapter());
        assert_eq!(
            device.interface(),
            Some(QpuInterface::VendorSdk("test-sdk".to_string()))
        );
        assert!(device
            .adapter_description()
            .unwrap()
            .contains("no hardware access"));

        device.initialize().unwrap();
        assert_eq!(device.health(), DeviceHealth::Warning);

        // Execution before calibration fails, and the failure is reported.
        let handle = device
            .submit(QuantumJob::new("qpu-test", QuantumIR::new("bell", 2)).with_shots(16))
            .unwrap();
        assert_eq!(device.poll(handle).unwrap(), JobStatus::Failed);
        assert!(device.read_result(handle).is_err());

        // After calibration the adapter completes the job.
        device.calibrate().unwrap();
        assert_eq!(device.health(), DeviceHealth::Healthy);

        let handle = device
            .submit(QuantumJob::new("qpu-test", QuantumIR::new("bell", 2)).with_shots(16))
            .unwrap();
        assert_eq!(device.poll(handle).unwrap(), JobStatus::Completed);
        let result = device.read_result(handle).unwrap();
        assert!(!result.simulation_only);
        assert!(result.notes.iter().any(|note| note.contains("TEST ADAPTER")));

        device.reset().unwrap();
        assert!(matches!(device.poll(handle), Err(QuantumError::JobNotFound(_))));
    }

    #[test]
    fn test_reset_requires_adapter() {
        let mut device = ExperimentalQpuDevice::new();
        assert!(matches!(
            device.reset(),
            Err(QuantumError::UnsupportedHardware(_))
        ));
    }
}