//! QEOS GPU compute — P7.3 GPU device API, memory safety, CPU reference and
//! failure recovery.
//!
//! ## Reality classification (distinct layers)
//!
//! | Layer | Status |
//! |---|---|
//! | GPU abstraction / runtime logic | **REAL** (executes and is tested over host memory) |
//! | Compute backend — CPU reference | **REAL** (correctness oracle) |
//! | Compute backend — mock | **SIMULATED** (stand-in, tagged, never hardware evidence) |
//! | GPU driver / vendor hardware (Vulkan/CUDA/ROCm) | **UNAVAILABLE** (no vendor runtime bundled) |
//!
//! A working GPU *abstraction* does not imply hardware acceleration. The exact
//! state of each layer is reported through [`GpuDeviceInfo`] / `BackendKind`.

#![forbid(unsafe_code)]

pub mod backend;
pub mod caps;
pub mod error;
pub mod memory;
pub mod runtime;

pub use backend::{verify_against_reference, ComputeBackend, CpuReferenceCompute, MockCompute};
pub use caps::{BackendKind, GpuCaps, GpuDeviceInfo, GpuTelemetry};
pub use error::{GpuError, Result};
pub use memory::{check_size, DeviceMemory, DeviceTable, GpuMemoryHandle, GpuMemoryId};
pub use runtime::{GpuFence, GpuQueue, GpuRuntime};

/// Honest vendor probe: always reports a vendor backend as unavailable in this
/// build. Never fabricates GPU availability.
pub fn probe(kind: BackendKind) -> GpuCaps {
    GpuCaps {
        kind,
        vendor_available: kind.vendor_available(),
        max_buffer_bytes: 0,
        ..Default::default()
    }
}

/// QEOS GPU compute crate version.
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn probe_is_honest() {
        assert!(!probe(BackendKind::Cuda).vendor_available);
        assert!(!probe(BackendKind::Rocm).vendor_available);
        assert_eq!(probe(BackendKind::Vulkan).max_buffer_bytes, 0);
    }
}
