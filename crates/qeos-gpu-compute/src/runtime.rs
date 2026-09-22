//! P7.3-04/08 — GPU device runtime API and failure recovery.
//!
//! `GpuRuntime` is the concrete, host-implemented GPU device abstraction. It
//! provides discover, capabilities, allocate/free/read/write, submit/synchronize,
//! reset, telemetry and device-removal handling. Memory is owned by the
//! [`DeviceTable`] and all handles are generation-bound; a reset or removal
//! bumps the generation and invalidates every outstanding handle
//! (use-after-remove prevention).
//!
//! Reality classification:
//! - The *abstraction/runtime logic* is **REAL** (genuinely executes, validated
//!   by tests, over host memory).
//! - The compute *backend* is the CPU reference (**REAL**) or mock (**SIMULATED**).
//! - A physical *GPU driver/hardware* backend is **UNAVAILABLE**.
//!
//! These layers are reported separately via [`GpuDeviceInfo`].

use crate::backend::{ComputeBackend, CpuReferenceCompute};
use crate::caps::{GpuCaps, GpuDeviceInfo, GpuTelemetry};
use crate::error::{GpuError, Result};
use crate::memory::{check_size, DeviceTable, GpuMemoryHandle};

/// A synchronous submit/fence pair on a queue.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuFence {
    pub id: u64,
    pub done: bool,
}

/// A submit queue with monotonically increasing work counters.
#[derive(Debug, Default)]
pub struct GpuQueue {
    pub submitted: u64,
    pub completed: u64,
}

impl GpuQueue {
    pub fn submit(&mut self) -> GpuFence {
        self.submitted += 1;
        GpuFence {
            id: self.submitted,
            done: false,
        }
    }

    pub fn complete(&mut self, f: &mut GpuFence) {
        f.done = true;
        self.completed += 1;
    }
}

/// The GPU device runtime.
pub struct GpuRuntime {
    backend: Box<dyn ComputeBackend>,
    table: DeviceTable,
    caps: GpuCaps,
    /// Device generation; bumped on reset/removal to invalidate handles.
    generation: u64,
    /// Whether the device is currently present (not removed).
    present: bool,
    queue: GpuQueue,
    telemetry: GpuTelemetry,
}

impl GpuRuntime {
    /// Create a runtime over a host CPU-reference backend.
    pub fn cpu_reference() -> Self {
        Self::with_backend(Box::new(CpuReferenceCompute))
    }

    /// Create a runtime over a given compute backend.
    pub fn with_backend(backend: Box<dyn ComputeBackend>) -> Self {
        let kind = backend.kind();
        let caps = GpuCaps {
            kind,
            ..Default::default()
        };
        let generation = 1;
        Self {
            backend,
            table: DeviceTable::new(caps.vram_bytes),
            caps,
            generation,
            present: true,
            queue: GpuQueue::default(),
            telemetry: GpuTelemetry::default(),
        }
    }

    /// P7.3-04 discover — static info about this device instance.
    pub fn discover(&self) -> GpuDeviceInfo {
        GpuDeviceInfo {
            name: format!("QEOS {:?} backend", self.caps.kind),
            vendor: "QuantumEnergyOS".into(),
            kind: self.caps.kind,
            caps: self.caps.clone(),
        }
    }

    /// P7.3-04 capabilities.
    pub fn caps(&self) -> &GpuCaps {
        &self.caps
    }

    /// Current device generation (for handle validation).
    pub fn generation(&self) -> u64 {
        self.generation
    }

    fn ensure_present(&self) -> Result<()> {
        if !self.present {
            return Err(GpuError::DeviceRemoved);
        }
        Ok(())
    }

    /// P7.3-04/05 allocate device memory.
    pub fn allocate(&mut self, len: usize) -> Result<GpuMemoryHandle> {
        self.ensure_present()?;
        check_size(len, &self.caps)?;
        let h = self.table.allocate(len, self.generation)?;
        self.telemetry.allocations += 1;
        self.telemetry.bytes_allocated += (len * std::mem::size_of::<f32>()) as u64;
        Ok(h)
    }

    /// P7.3-04/05 free device memory.
    pub fn free(&mut self, handle: GpuMemoryHandle) -> Result<()> {
        self.ensure_present()?;
        let before = self.table.allocated_bytes;
        self.table.free(handle, self.generation)?;
        self.telemetry.frees += 1;
        self.telemetry.bytes_freed += before.saturating_sub(self.table.allocated_bytes);
        Ok(())
    }

    /// P7.3-04 read device memory into host bytes.
    pub fn read(&self, handle: GpuMemoryHandle) -> Result<Vec<f32>> {
        self.ensure_present()?;
        self.table.read(handle, self.generation)
    }

    /// P7.3-04 write host bytes into device memory.
    pub fn write(&mut self, handle: GpuMemoryHandle, data: &[f32]) -> Result<()> {
        self.ensure_present()?;
        self.table.write(handle, self.generation, data)
    }

    /// Map a buffer for direct access (single-mapping enforced by the table).
    pub fn map(&mut self, handle: GpuMemoryHandle) -> Result<()> {
        self.ensure_present()?;
        self.table.map(handle, self.generation)
    }

    pub fn unmap(&mut self, handle: GpuMemoryHandle) -> Result<()> {
        self.ensure_present()?;
        self.table.unmap(handle, self.generation)
    }

    /// P7.3-04 submit an element-wise add of two buffers (`dst = a + b`).
    pub fn submit_add(
        &mut self,
        a: GpuMemoryHandle,
        b: GpuMemoryHandle,
        dst: GpuMemoryHandle,
    ) -> Result<GpuFence> {
        self.ensure_present()?;
        let fence = self.queue.submit();
        let gen = self.generation;
        let (av, bv) = {
            // Validate a and b against the current generation before borrowing.
            let av = self.table.read(a, gen)?;
            let bv = self.table.read(b, gen)?;
            (av, bv)
        };
        let result = self.backend.vec_add(&av, &bv)?;
        {
            let dst_slice = self.table.data_mut(dst, gen)?;
            if dst_slice.len() != result.len() {
                return Err(GpuError::BadBuffer("destination size mismatch".into()));
            }
            dst_slice.copy_from_slice(&result);
        }
        self.telemetry.submits += 1;
        let mut fence = fence;
        self.queue.complete(&mut fence);
        self.telemetry.completions += 1;
        Ok(fence)
    }

    /// P7.3-04 synchronize on a fence. Simulates a bounded wait; a fence that
    /// is not complete within the nominal timeout reports [`GpuError::Timeout`].
    pub fn synchronize(&mut self, fence: GpuFence, timeout_ms: u64) -> Result<()> {
        self.ensure_present()?;
        // In the synchronous host model a submitted fence completes immediately,
        // but we honor a timeout for the recovery path.
        if fence.done {
            Ok(())
        } else if timeout_ms == 0 {
            Err(GpuError::Timeout)
        } else {
            // Emulate completion.
            let mut f = fence;
            self.queue.complete(&mut f);
            Ok(())
        }
    }

    /// P7.3-04/08 reset the device: invalidate all memory handles and counters.
    pub fn reset(&mut self) -> Result<()> {
        self.ensure_present()?;
        self.table.clear();
        self.generation += 1;
        self.telemetry.resets += 1;
        Ok(())
    }

    /// P7.3-08 simulate device removal/loss: all handles become stale and the
    /// device reports unavailable until the runtime is recreated.
    pub fn device_removed(&mut self) {
        self.table.clear();
        self.generation += 1;
        self.present = false;
    }

    /// P7.3-04 telemetry.
    pub fn telemetry(&self) -> &GpuTelemetry {
        &self.telemetry
    }

    /// Verify that a computed result equals the CPU reference within tolerance.
    /// Used to cross-check `submit_add` output against the oracle.
    pub fn verify_result(&self, computed: &[f32], reference: &[f32], tol: f32) -> Result<()> {
        crate::backend::verify_against_reference(computed, reference, tol)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::MockCompute;

    #[test]
    fn full_alloc_submit_read_cycle() {
        let mut rt = GpuRuntime::cpu_reference();
        let a = rt.allocate(4).unwrap();
        let b = rt.allocate(4).unwrap();
        let dst = rt.allocate(4).unwrap();
        rt.write(a, &[1.0, 2.0, 3.0, 4.0]).unwrap();
        rt.write(b, &[10.0, 20.0, 30.0, 40.0]).unwrap();
        let f = rt.submit_add(a, b, dst).unwrap();
        rt.synchronize(f, 100).unwrap();
        assert_eq!(rt.read(dst).unwrap(), vec![11.0, 22.0, 33.0, 44.0]);
        assert_eq!(rt.telemetry().submits, 1);
        assert_eq!(rt.telemetry().completions, 1);
    }

    #[test]
    fn reset_invalidates_handles() {
        let mut rt = GpuRuntime::cpu_reference();
        let h = rt.allocate(4).unwrap();
        rt.reset().unwrap();
        assert_eq!(rt.read(h).unwrap_err(), GpuError::StaleHandle);
    }

    #[test]
    fn removal_marks_unavailable() {
        let mut rt = GpuRuntime::cpu_reference();
        let h = rt.allocate(4).unwrap();
        rt.device_removed();
        // All operations report the device as removed (honesty over stale).
        assert_eq!(rt.read(h).unwrap_err(), GpuError::DeviceRemoved);
        assert_eq!(rt.allocate(4).unwrap_err(), GpuError::DeviceRemoved);
    }

    #[test]
    fn oom_reported_honestly() {
        let mut rt = GpuRuntime::with_backend(Box::new(CpuReferenceCompute));
        rt.caps = GpuCaps {
            vram_bytes: 0,
            ..rt.caps.clone()
        };
        rt.table = DeviceTable::new(0);
        assert!(matches!(rt.allocate(8), Err(GpuError::AllocationFailed(_))));
    }

    #[test]
    fn timeoute_zero_wait() {
        let mut rt = GpuRuntime::cpu_reference();
        let f = GpuFence { id: 1, done: false };
        assert_eq!(rt.synchronize(f, 0).unwrap_err(), GpuError::Timeout);
    }

    #[test]
    fn mock_backend_is_simulated() {
        let rt = GpuRuntime::with_backend(Box::new(MockCompute));
        assert!(!rt.discover().caps.vendor_available);
        let info = rt.discover();
        assert_eq!(info.kind, crate::caps::BackendKind::Mock);
    }

    #[test]
    fn cpu_reference_classified_real_kind() {
        let rt = GpuRuntime::cpu_reference();
        assert_eq!(rt.discover().kind, crate::caps::BackendKind::CpuReference);
    }

    #[test]
    fn verify_against_reference() {
        let rt = GpuRuntime::cpu_reference();
        assert!(rt.verify_result(&[1.0, 2.0], &[1.0, 2.0], 1e-3).is_ok());
        assert!(rt.verify_result(&[1.0, 9.0], &[1.0, 2.0], 1e-3).is_err());
    }
}
