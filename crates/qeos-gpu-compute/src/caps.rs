//! Backend classification and capability/telemetry models for the QEOS GPU crate.
//!
//! Reality discipline: `BackendKind::CpuReference` is **REAL** (a correctness
//! oracle that genuinely computes), `BackendKind::Mock` is **SIMULATED**, and
//! vendor kinds (Vulkan/Cuda/Rocm) are **UNAVAILABLE** unless a runtime exists.

use serde::{Deserialize, Serialize};

/// Which compute backend a GPU runtime is backed by.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendKind {
    /// Host CPU reference — REAL correctness oracle.
    CpuReference,
    /// Host mock — SIMULATED device, delegates to the CPU reference.
    Mock,
    /// Vendor runtimes — UNAVAILABLE in this build (no vendor SDK bundled).
    Vulkan,
    Cuda,
    Rocm,
}

impl BackendKind {
    /// Whether a vendor runtime is actually available in this build.
    pub fn vendor_available(self) -> bool {
        false
    }
}

/// Capabilities of a discovered GPU runtime/device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuCaps {
    pub kind: BackendKind,
    /// Whether a real vendor device/driver is present.
    pub vendor_available: bool,
    /// Maximum addressable buffer size in bytes for the runtime.
    pub max_buffer_bytes: usize,
    /// Nominal compute units (host threads for CPU/mock backends).
    pub compute_units: u32,
    /// Nominal device memory in bytes (host heap reserved for CPU/mock).
    pub vram_bytes: u64,
    /// Whether the runtime supports reset.
    pub supports_reset: bool,
    /// Whether the runtime reports telemetry.
    pub supports_telemetry: bool,
}

impl Default for GpuCaps {
    fn default() -> Self {
        Self {
            kind: BackendKind::CpuReference,
            vendor_available: false,
            max_buffer_bytes: 1_048_576,
            compute_units: 1,
            vram_bytes: 64 * 1024 * 1024,
            supports_reset: true,
            supports_telemetry: true,
        }
    }
}

/// Static information about a discovered GPU runtime.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuDeviceInfo {
    pub name: String,
    pub vendor: String,
    pub kind: BackendKind,
    pub caps: GpuCaps,
}

/// Runtime counters reported by the GPU runtime.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct GpuTelemetry {
    pub allocations: u64,
    pub frees: u64,
    pub submits: u64,
    pub completions: u64,
    pub resets: u64,
    pub bytes_allocated: u64,
    pub bytes_freed: u64,
    pub errors: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn vendor_backends_never_available() {
        assert!(!BackendKind::Cuda.vendor_available());
        assert!(!BackendKind::Rocm.vendor_available());
        assert!(!BackendKind::Vulkan.vendor_available());
    }

    #[test]
    fn caps_default() {
        let c = GpuCaps::default();
        assert!(!c.vendor_available);
        assert!(c.supports_reset);
    }
}
