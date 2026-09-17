//! Phase 4.5 GPU compute layer (no vendor driver bundled).
//!
//! CLASSIFICATION: REAL (abstraction + CPU reference) / SIMULATED (mock).
//! No CUDA/ROCm dependency. Vendor backends are FUTURE behind `probe()`.
use serde::{Deserialize, Serialize};
use thiserror::Error;
#[derive(Debug, Error)]
pub enum GpuError {
    #[error("backend unavailable: {0}")]
    Unavailable(String),
    #[error("bad buffer: {0}")]
    BadBuffer(String),
    #[error("dispatch failed: {0}")]
    Dispatch(String),
}
pub type Result<T> = std::result::Result<T, GpuError>;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BackendKind {
    CpuReference,
    Mock,
    Vulkan,
    Cuda,
    Rocm,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuCaps {
    pub kind: BackendKind,
    pub vendor_available: bool,
    pub max_buffer_bytes: usize,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuBuffer {
    pub bytes: Vec<f32>,
}
impl GpuBuffer {
    pub fn new(n: usize) -> Result<Self> {
        if n == 0 || n > 1_048_576 {
            return Err(GpuError::BadBuffer("size".into()));
        }
        Ok(Self {
            bytes: vec![0.0; n],
        })
    }
    pub fn from_slice(s: &[f32]) -> Result<Self> {
        let mut b = Self::new(s.len())?;
        b.bytes.copy_from_slice(s);
        Ok(b)
    }
    pub fn len(&self) -> usize {
        self.bytes.len()
    }
    pub fn is_empty(&self) -> bool {
        self.bytes.is_empty()
    }
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuKernel {
    pub name: String,
}
impl GpuKernel {
    pub fn new(n: &str) -> Self {
        Self { name: n.into() }
    }
}
/// Queue + fence/event model (synchronous host model; real async is FUTURE).
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
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GpuFence {
    pub id: u64,
    pub done: bool,
}
pub trait GpuDevice: Send {
    fn kind(&self) -> BackendKind;
    fn caps(&self) -> GpuCaps;
    fn vec_add(&mut self, a: &GpuBuffer, b: &GpuBuffer) -> Result<GpuBuffer>;
    fn mat_vec(&mut self, m: &[Vec<f32>], v: &GpuBuffer) -> Result<GpuBuffer>;
}
/// CPU reference backend (REAL as software): correctness oracle.
pub struct CpuBackend;
impl GpuDevice for CpuBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::CpuReference
    }
    fn caps(&self) -> GpuCaps {
        GpuCaps {
            kind: BackendKind::CpuReference,
            vendor_available: false,
            max_buffer_bytes: 1_048_576,
        }
    }
    fn vec_add(&mut self, a: &GpuBuffer, b: &GpuBuffer) -> Result<GpuBuffer> {
        if a.len() != b.len() {
            return Err(GpuError::BadBuffer("len".into()));
        }
        Ok(GpuBuffer {
            bytes: a.bytes.iter().zip(&b.bytes).map(|(x, y)| x + y).collect(),
        })
    }
    fn mat_vec(&mut self, m: &[Vec<f32>], v: &GpuBuffer) -> Result<GpuBuffer> {
        let mut o = Vec::with_capacity(m.len());
        for row in m {
            if row.len() != v.len() {
                return Err(GpuError::BadBuffer("shape".into()));
            }
            o.push(row.iter().zip(&v.bytes).map(|(x, y)| x * y).sum());
        }
        Ok(GpuBuffer { bytes: o })
    }
}
/// Mock GPU backend (SIMULATED): same math, tagged mock for equivalence tests.
pub struct MockBackend;
impl GpuDevice for MockBackend {
    fn kind(&self) -> BackendKind {
        BackendKind::Mock
    }
    fn caps(&self) -> GpuCaps {
        GpuCaps {
            kind: BackendKind::Mock,
            vendor_available: false,
            max_buffer_bytes: 1_048_576,
        }
    }
    fn vec_add(&mut self, a: &GpuBuffer, b: &GpuBuffer) -> Result<GpuBuffer> {
        CpuBackend.vec_add(a, b)
    }
    fn mat_vec(&mut self, m: &[Vec<f32>], v: &GpuBuffer) -> Result<GpuBuffer> {
        CpuBackend.mat_vec(m, v)
    }
}
/// Vendor probe: always reports unavailable in this build (honest, no fake GPU).
pub fn probe(kind: BackendKind) -> GpuCaps {
    GpuCaps {
        kind,
        vendor_available: false,
        max_buffer_bytes: 0,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn cpu_mock_equivalent() {
        let a = GpuBuffer::from_slice(&[1.0, 2.0]).unwrap();
        let b = GpuBuffer::from_slice(&[3.0, 4.0]).unwrap();
        let r1 = CpuBackend.vec_add(&a, &b).unwrap();
        let r2 = MockBackend.vec_add(&a, &b).unwrap();
        assert_eq!(r1.bytes, r2.bytes);
        assert_eq!(r1.bytes, vec![4.0, 6.0]);
    }
    #[test]
    fn queue_fence() {
        let mut q = GpuQueue::default();
        let mut f = q.submit();
        assert!(!f.done);
        q.complete(&mut f);
        assert!(f.done);
    }
    #[test]
    fn probe_honest() {
        assert!(!probe(BackendKind::Cuda).vendor_available);
    }
    #[test]
    fn matvec() {
        let v = GpuBuffer::from_slice(&[1.0, 1.0]).unwrap();
        let r = CpuBackend
            .mat_vec(&[vec![1.0, 2.0], vec![3.0, 4.0]], &v)
            .unwrap();
        assert_eq!(r.bytes, vec![3.0, 7.0]);
    }
}
