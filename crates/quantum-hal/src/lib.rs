//! Quantum Hardware Abstraction Layer (Phase 3, §6–§11).
//!
//! ```text
//! Applications -> Services -> Runtime -> HAL -> Drivers -> Hardware
//! ```
//!
//! The HAL owns the device-facing contracts of the quantum stack and the devices
//! that implement them. Classification of every path is explicit:
//!
//! - device traits, IR and the job system: REAL (implemented, host-testable);
//! - [`simulator`] and the reference execution path: SIMULATED (classical);
//! - [`accelerator`]: ABSTRACT vendor backends, CPU reference fallback;
//! - [`qpu`]: EXPERIMENTAL/FUTURE — the physical-hardware boundary, disabled
//!   unless a documented vendor adapter is attached.
//!
//! No module in this crate claims control of hardware that does not exist. When a
//! capability is missing, the canonical `quantum_runtime::QuantumError::UnsupportedHardware`
//! is returned instead of a fabricated result.
//!
//! NOTE(PHASE-4.1): this file was added so the declared workspace member resolves
//! (`no targets specified in the manifest`); it only wires up the modules that
//! already exist in `src/`. Content is owned by the quantum-HAL workstream.

pub mod accelerator;
pub mod device;
pub mod error;
pub mod ir;
pub mod job;
pub mod qpu;
pub mod simulator;

pub use device::{BackendClass, DeviceHealth, DeviceInfo, DeviceState, QuantumDevice};
pub use error::{QuantumError, Result};
pub use ir::{QuantumIR, QuantumOperation, QubitId};
pub use job::{JobHandle, JobStatus, QuantumJob, QuantumResult, SimulationConfig};
pub use qpu::{ExperimentalQpuDevice, QpuInterface, QpuVendorAdapter};
pub use simulator::SimulatorDevice;

/// Crate version, reported by diagnostics.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
