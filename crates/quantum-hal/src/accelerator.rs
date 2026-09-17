//! Accelerator device abstraction (Phase 3, §6).
//!
//! CLASSIFICATION: ABSTRACT (vendor backends) / SIMULATED (reference mode)
//!
//! A GPU, FPGA or NPU is NOT a quantum processor. This device exists to host
//! classical scientific computing (state-vector simulation, linear algebra,
//! Monte Carlo, optimisation, scientific models) on accelerators.
//!
//! No vendor runtime (CUDA, ROCm, oneAPI, Vulkan compute, OpenCL) is bundled or
//! assumed. When no vendor backend is present:
//!
//! - jobs still run, on the CPU reference engine, and every result is annotated
//!   as offload-unavailable and simulated;
//! - the device never claims to have executed a vendor kernel.

use serde::{Deserialize, Serialize};

use crate::device::{BackendClass, DeviceHealth, DeviceInfo, DeviceState, QuantumDevice};
use crate::error::{QuantumError, Result};
use crate::job::{JobHandle, JobStatus, QuantumJob, QuantumResult};
use crate::simulator::SimulatorDevice;

/// Accelerator API families the abstraction is prepared for.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcceleratorKind {
    /// NVIDIA CUDA.
    Cuda,
    /// AMD ROCm/HIP.
    Rocm,
    /// Intel oneAPI/SYCL.
    OneApi,
    /// Vulkan compute.
    VulkanCompute,
    /// OpenCL.
    OpenCl,
    /// Any other accelerator API (named explicitly, never guessed).
    Other(String),
}

impl AcceleratorKind {
    /// Stable label used in metadata and diagnostics.
    pub fn label(&self) -> String {
        match self {
            AcceleratorKind::Cuda => "cuda".to_string(),
            AcceleratorKind::Rocm => "rocm".to_string(),
            AcceleratorKind::OneApi => "oneapi".to_string(),
            AcceleratorKind::VulkanCompute => "vulkan-compute".to_string(),
            AcceleratorKind::OpenCl => "opencl".to_string(),
            AcceleratorKind::Other(name) => format!("other:{}", name),
        }
    }
}

/// Workload classes an accelerator may host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AcceleratorWorkload {
    /// Classical quantum-circuit simulation.
    QuantumSimulation,
    /// Dense/sparse linear algebra.
    LinearAlgebra,
    /// Monte Carlo sampling.
    MonteCarlo,
    /// Numerical optimisation.
    Optimization,
    /// Materials/energy scientific models.
    ScientificModel,
}

/// Accelerator-backed device abstraction.
#[derive(Debug)]
pub struct AcceleratorDevice {
    info: DeviceInfo,
    state: DeviceState,
    health: DeviceHealth,
    kind: AcceleratorKind,
    vendor_backend_available: bool,
    reference: SimulatorDevice,
}

impl AcceleratorDevice {
    /// Build an accelerator device, declaring whether a real vendor backend is
    /// available. Only pass `true` when a vendor runtime is actually integrated.
    pub fn with_vendor_backend(kind: AcceleratorKind, vendor_backend_available: bool) -> Self {
        let label = kind.label();
        Self {
            info: DeviceInfo {
                id: format!("accel-{}", label),
                name: format!("QuantumEnergyOS Accelerator ({})", label),
                vendor: "QuantumEnergyOS".to_string(),
                model: label,
                class: BackendClass::Accelerator,
                qubits: SimulatorDevice::DEFAULT_MAX_QUBITS,
                logical_qubits: 0,
                supported_gates: vec![
                    "H".to_string(),
                    "X".to_string(),
                    "Y".to_string(),
                    "Z".to_string(),
                    "CNOT".to_string(),
                    "Measure".to_string(),
                ],
                firmware_version: None,
                simulation_only: !vendor_backend_available,
            },
            state: DeviceState::Uninitialized,
            health: DeviceHealth::Unknown,
            kind,
            vendor_backend_available,
            reference: SimulatorDevice::new(),
        }
    }

    /// Reference mode: no vendor backend, jobs run on the CPU reference engine.
    pub fn reference(kind: AcceleratorKind) -> Self {
        Self::with_vendor_backend(kind, false)
    }

    /// Detect an accelerator. No vendor runtime ships with this build, so the
    /// detection result is always `false`; the call exists so that detection
    /// logic can be added without changing call sites.
    pub fn detect(kind: AcceleratorKind) -> Self {
        Self::with_vendor_backend(kind, false)
    }

    /// API family behind this device.
    pub fn kind(&self) -> &AcceleratorKind {
        &self.kind
    }

    /// True when a real vendor runtime is attached.
    pub fn has_vendor_backend(&self) -> bool {
        self.vendor_backend_available
    }

    /// Workload classes this abstraction may host.
    ///
    /// Declared capability only: no vendor kernel is implemented in this build.
    pub fn supported_workloads(&self) -> Vec<AcceleratorWorkload> {
        vec![
            AcceleratorWorkload::QuantumSimulation,
            AcceleratorWorkload::LinearAlgebra,
            AcceleratorWorkload::MonteCarlo,
            AcceleratorWorkload::Optimization,
            AcceleratorWorkload::ScientificModel,
        ]
    }

    /// Access the CPU reference engine used when offload is unavailable.
    pub fn reference_engine(&self) -> &SimulatorDevice {
        &self.reference
    }
}
