//! P7.3-06 — Compute backends and the CPU reference.
//!
//! The CPU reference (`CpuReferenceCompute`) is the **correctness oracle** for
//! the GPU runtime. Every accelerated path is compared against it with a
//! documented numerical tolerance; divergent results are never silently
//! accepted.
//!
//! `MockCompute` presents the same math but is tagged **SIMULATED** (a stand-in
//! for a device, never evidence of vendor hardware).

use crate::caps::BackendKind;
use crate::error::{GpuError, Result};

/// A compute backend that performs numeric kernels on device memory.
pub trait ComputeBackend: Send {
    fn kind(&self) -> BackendKind;

    /// Element-wise `a + b`.
    fn vec_add(&self, a: &[f32], b: &[f32]) -> Result<Vec<f32>>;

    /// Matrix-vector product `m * v`.
    fn mat_vec(&self, m: &[Vec<f32>], v: &[f32]) -> Result<Vec<f32>>;
}

/// CPU reference backend — REAL. Numerically correct implementation used as the
/// oracle for all accelerated results.
#[derive(Debug, Clone, Copy, Default)]
pub struct CpuReferenceCompute;

impl ComputeBackend for CpuReferenceCompute {
    fn kind(&self) -> BackendKind {
        BackendKind::CpuReference
    }

    fn vec_add(&self, a: &[f32], b: &[f32]) -> Result<Vec<f32>> {
        if a.len() != b.len() {
            return Err(GpuError::BadBuffer("length mismatch".into()));
        }
        Ok(a.iter().zip(b).map(|(x, y)| x + y).collect())
    }

    fn mat_vec(&self, m: &[Vec<f32>], v: &[f32]) -> Result<Vec<f32>> {
        let mut out = Vec::with_capacity(m.len());
        for row in m {
            if row.len() != v.len() {
                return Err(GpuError::BadBuffer("shape mismatch".into()));
            }
            out.push(row.iter().zip(v).map(|(x, y)| x * y).sum());
        }
        Ok(out)
    }
}

/// Mock GPU backend — SIMULATED. Same math as the CPU reference, explicitly
/// tagged as a stand-in (never evidence of a physical device).
#[derive(Debug, Clone, Copy, Default)]
pub struct MockCompute;

impl ComputeBackend for MockCompute {
    fn kind(&self) -> BackendKind {
        BackendKind::Mock
    }

    fn vec_add(&self, a: &[f32], b: &[f32]) -> Result<Vec<f32>> {
        CpuReferenceCompute.vec_add(a, b)
    }

    fn mat_vec(&self, m: &[Vec<f32>], v: &[f32]) -> Result<Vec<f32>> {
        CpuReferenceCompute.mat_vec(m, v)
    }
}

/// Compare an accelerated result against the CPU reference with a tolerance.
///
/// Returns `Ok(())` when every element is within `tol` of the reference.
/// Divergent results are reported, never silently accepted.
pub fn verify_against_reference(actual: &[f32], reference: &[f32], tol: f32) -> Result<()> {
    if actual.len() != reference.len() {
        return Err(GpuError::Dispatch("result length mismatch".into()));
    }
    for (a, r) in actual.iter().zip(reference) {
        if (a - r).abs() > tol {
            return Err(GpuError::Dispatch(format!(
                "result diverges from CPU reference: {a} vs {r}"
            )));
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cpu_reference_vec_add() {
        assert_eq!(
            CpuReferenceCompute
                .vec_add(&[1.0, 2.0], &[3.0, 4.0])
                .unwrap(),
            vec![4.0, 6.0]
        );
    }

    #[test]
    fn mock_matches_reference() {
        let a = vec![1.5, -2.0, 3.0];
        let b = vec![0.5, 2.0, -1.0];
        let ref_out = CpuReferenceCompute.vec_add(&a, &b).unwrap();
        let mock_out = MockCompute.vec_add(&a, &b).unwrap();
        assert_eq!(ref_out, mock_out);
    }

    #[test]
    fn verify_accepts_within_tolerance() {
        assert!(verify_against_reference(&[1.0, 2.0], &[1.000001, 2.0], 1e-3).is_ok());
    }

    #[test]
    fn verify_rejects_divergence() {
        assert!(verify_against_reference(&[1.0, 2.0], &[5.0, 2.0], 1e-3).is_err());
    }

    #[test]
    fn verify_rejects_length_mismatch() {
        assert!(verify_against_reference(&[1.0], &[1.0, 2.0], 1e-3).is_err());
    }

    #[test]
    fn kind_classification() {
        assert_eq!(CpuReferenceCompute.kind(), BackendKind::CpuReference);
        assert_eq!(MockCompute.kind(), BackendKind::Mock);
        // Vendor kinds report unavailable.
        assert!(!BackendKind::Cuda.vendor_available());
    }
}
