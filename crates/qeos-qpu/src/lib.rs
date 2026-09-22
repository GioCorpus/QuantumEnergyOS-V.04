//! # QEOS QPU — P7.4 generic QPU interface
//!
//! A hardware-independent QPU interface with a backend architecture
//! (Simulator / Mock / Vendor) and strict job isolation, plus the Majorana
//! hardware-readiness contract.
//!
//! ## Reality classification
//!
//! | Layer | Status |
//! |---|---|
//! | Generic QPU interface (device lifecycle, telemetry) | **REAL** |
//! | Simulator backend (delegates to `quantum-runtime` Majorana simulator) | **SIMULATED** |
//! | Mock backend | **MOCK** |
//! | Vendor/physical QPU backend | **UNAVAILABLE** |
//! | Majorana math model (anticommutation, tetron, parity) | lives in `quantum-runtime` — **SIMULATED** |
//!
//! A working QPU *abstraction* does not imply physical hardware access. Vendor
//! availability is always reported via `QpuCapabilities::vendor_available`.

#![forbid(unsafe_code)]

pub mod backend;
pub mod device;
pub mod error;
pub mod job;
pub mod readiness;

pub use backend::{
    MockBackend, QpuBackend, QpuBackendKind, QpuCapabilities, SimulatorBackend, VendorBackend,
};
pub use device::{QpuDevice, QpuDeviceInfoView, QpuDeviceState, QpuTelemetry};
pub use error::{QpuError, Result as QpuResultType};
pub use job::{JobLimits, QpuJob, QpuJobId, QpuJobState, QpuResult};
pub use readiness::{HardwareAvailabilityReport, HardwareReadinessContract};

/// QEOS QPU crate version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
