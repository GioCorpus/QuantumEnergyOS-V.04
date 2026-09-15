//! Device-facing abstractions of the Quantum Hardware Abstraction Layer.
//!
//! CLASSIFICATION
//! - The `QuantumDevice` trait and the metadata types in this module: REAL
//!   (abstract interface, implemented and testable today).
//! - Concrete devices live in sibling modules and carry their own
//!   classification (`SIMULATED`, `ABSTRACT`, `EXPERIMENTAL`, `FUTURE`).
//!
//! Architectural rule (Phase 3, §31):
//!
//! ```text
//! Applications -> Services -> Runtime -> HAL -> Drivers -> Hardware
//! ```
//!
//! Applications never obtain a `dyn QuantumDevice`. Device handles are owned by
//! the quantum service, which authorises each call through the identity and
//! policy layer before the HAL is touched.

use serde::{Deserialize, Serialize};

use crate::error::Result;
use crate::job::{JobHandle, JobStatus, QuantumJob, QuantumResult};

/// Classification of the execution path behind a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum BackendClass {
    /// Classical simulation of quantum behaviour. Real as software, never a QPU.
    #[default]
    Simulator,
    /// Classical accelerator (GPU/FPGA) used for simulation and scientific compute.
    /// An accelerator is NOT a quantum processor.
    Accelerator,
    /// Software model of an experimental device: research code path.
    Experimental,
    /// Physical hardware. Requires a documented vendor interface before use.
    Physical,
}

impl std::fmt::Display for BackendClass {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let label = match self {
            BackendClass::Simulator => "SIMULATOR",
            BackendClass::Accelerator => "ACCELERATOR",
            BackendClass::Experimental => "EXPERIMENTAL",
            BackendClass::Physical => "PHYSICAL",
        };
        f.write_str(label)
    }
}

/// Static description of a quantum-capable device.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DeviceInfo {
    /// Stable device identifier used in telemetry and audit logs.
    pub id: String,
    /// Human readable device name.
    pub name: String,
    /// Vendor or owning subsystem. Never a hardware claim unless `class` is
    /// `Physical` and an adapter is attached.
    pub vendor: String,
    /// Model or engine name (for example `state-vector-reference`).
    pub model: String,
    /// Execution path classification.
    pub class: BackendClass,
    /// Physical or virtual qubit count.
    pub qubits: usize,
    /// Logical (error-corrected) qubit count, or 0 when not applicable.
    pub logical_qubits: usize,
    /// Gate names accepted by the device.
    pub supported_gates: Vec<String>,
    /// Firmware/engine version, when known.
    pub firmware_version: Option<String>,
    /// True when every result produced by this device is a classical simulation.
    /// Set to false only for a device with a real vendor backend attached.
    pub simulation_only: bool,
}

impl DeviceInfo {
    /// Check whether the device declares support for a gate name.
    pub fn supports_gate(&self, gate: &str) -> bool {
        self.supported_gates.iter().any(|g| g.eq_ignore_ascii_case(gate))
    }
}

/// Operational health of a device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DeviceHealth {
    /// Device is operational.
    Healthy,
    /// Device is operational with reduced capability.
    Degraded,
    /// Device is operational but reporting warnings.
    Warning,
    /// Device is not usable right now.
    Unavailable,
    /// Health could not be determined (device never initialised).
    #[default]
    Unknown,
}

/// Lifecycle state of a device inside the HAL.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum DeviceState {
    /// Constructed but never initialised.
    #[default]
    Uninitialized,
    /// `initialize()` in progress.
    Initializing,
    /// Ready to accept jobs.
    Ready,
    /// Executing a job.
    Busy,
    /// Faulted; requires reset or operator action.
    Error,
    /// Deliberately disabled (for example a physical backend without adapter).
    Disabled,
}

/// The single device contract of QuantumEnergyOS.
///
/// Every backend (simulator, accelerator, experimental QPU, future physical
/// hardware) implements this trait. The rest of the operating system never
/// depends on a vendor: it depends on `QuantumDevice`.
///
/// Implementations MUST NOT fabricate results. If the hardware behind the
/// implementation is absent, return
/// [`QuantumError::UnsupportedHardware`](crate::error::QuantumError::UnsupportedHardware).
pub trait QuantumDevice: Send + Sync {
    /// Static metadata for the device.
    fn device_info(&self) -> DeviceInfo;

    /// Bring the device to `Ready`, allocating whatever is needed.
    fn initialize(&mut self) -> Result<()>;

    /// Calibrate the device.
    ///
    /// For simulated devices this records a calibration event; it must never
    /// claim that physical calibration took place.
    fn calibrate(&mut self) -> Result<()>;

    /// Reset device state (register state and job history).
    fn reset(&mut self) -> Result<()>;

    /// Submit a job and receive a handle.
    fn submit(&mut self, job: QuantumJob) -> Result<JobHandle>;

    /// Query the status of a previously submitted job.
    fn poll(&mut self, job: JobHandle) -> Result<JobStatus>;

    /// Read the result of a completed job.
    fn read_result(&mut self, job: JobHandle) -> Result<QuantumResult>;

    /// Current health of the device.
    fn health(&self) -> DeviceHealth;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct StubDevice;

    impl QuantumDevice for StubDevice {
        fn device_info(&self) -> DeviceInfo {
            DeviceInfo {
                id: "stub-0".to_string(),
                name: "Stub".to_string(),
                class: BackendClass::Simulator,
                simulation_only: true,
                ..DeviceInfo::default()
            }
        }

        fn initialize(&mut self) -> Result<()> {
            Ok(())
        }

        fn calibrate(&mut self) -> Result<()> {
            Ok(())
        }

        fn reset(&mut self) -> Result<()> {
            Ok(())
        }

        fn submit(&mut self, _job: QuantumJob) -> Result<JobHandle> {
            Ok(JobHandle(1))
        }

        fn poll(&mut self, _job: JobHandle) -> Result<JobStatus> {
            Ok(JobStatus::Completed)
        }

        fn read_result(&mut self, _job: JobHandle) -> Result<QuantumResult> {
            let mut result = QuantumResult::new("stub-job", "stub-0", 1);
            result.status = JobStatus::Completed;
            Ok(result)
        }

        fn health(&self) -> DeviceHealth {
            DeviceHealth::Healthy
        }
    }

    #[test]
    fn test_backend_class_default_is_simulator() {
        assert_eq!(BackendClass::default(), BackendClass::Simulator);
        assert_eq!(BackendClass::Physical.to_string(), "PHYSICAL");
    }

    #[test]
    fn test_device_health_and_state_defaults() {
        assert_eq!(DeviceHealth::default(), DeviceHealth::Unknown);
        assert_eq!(DeviceState::default(), DeviceState::Uninitialized);
    }

    #[test]
    fn test_device_info_gate_lookup_is_case_insensitive() {
        let info = DeviceInfo {
            supported_gates: vec!["H".to_string(), "CNOT".to_string()],
            ..DeviceInfo::default()
        };
        assert!(info.supports_gate("h"));
        assert!(info.supports_gate("cnot"));
        assert!(!info.supports_gate("Toffoli"));
    }

    #[test]
    fn test_trait_is_object_safe_and_dispatchable() {
        let mut device: Box<dyn QuantumDevice> = Box::new(StubDevice);
        assert_eq!(device.device_info().id, "stub-0");
        assert_eq!(device.health(), DeviceHealth::Healthy);
        device.initialize().unwrap();
        device.calibrate().unwrap();

        let job = QuantumJob::new("stub", crate::ir::QuantumIR::new("empty", 1));
        let handle = device.submit(job).unwrap();
        assert_eq!(device.poll(handle).unwrap(), JobStatus::Completed);
        assert!(device.read_result(handle).unwrap().simulation_only);
        device.reset().unwrap();
    }
}