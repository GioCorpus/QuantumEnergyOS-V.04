//! Phase 4.6 QPU device API (spec 4.6.1/4.6.2): capabilities + job handles.
use crate::error::Result;
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct QpuCapabilities {
    pub qubit_count: usize,
    pub connectivity: String,
    pub native_ops: Vec<String>,
    pub measurement: bool,
    pub reset: bool,
    pub max_job_shots: u32,
}
impl Default for QpuCapabilities {
    fn default() -> Self {
        Self {
            qubit_count: 2,
            connectivity: "simulator:all-to-all".into(),
            native_ops: vec!["H".into(), "CNOT".into(), "Measure".into()],
            measurement: true,
            reset: true,
            max_job_shots: 100000,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct QpuJobHandle(pub u64);
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QpuDeviceError {
    Busy,
    BadJob,
    Timeout,
    Down(String),
}
impl std::fmt::Display for QpuDeviceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}
impl std::error::Error for QpuDeviceError {}
/// Generic QPU contract: simulator + vendor adapters implement this.
pub trait QpuDevice: Send {
    fn id(&self) -> &str;
    fn capabilities(&self) -> QpuCapabilities;
    fn submit_ir(
        &mut self,
        ir: &str,
        shots: u32,
        seed: u64,
    ) -> std::result::Result<QpuJobHandle, QpuDeviceError>;
    fn poll(&self, h: QpuJobHandle) -> QpuJobStatus;
    fn cancel(&mut self, h: QpuJobHandle) -> Result<()>;
    fn reset(&mut self) -> Result<()>;
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum QpuJobStatus {
    Created,
    Queued,
    Compiling,
    Running,
    Measuring,
    PostProcessing,
    Completed,
    Failed,
    Cancelled,
    Timeout,
}
/// In-memory mock QPU (SIMULATION): completes jobs synchronously.
pub struct MockQpu {
    id: String,
    caps: QpuCapabilities,
    next: u64,
}
impl MockQpu {
    pub fn new(id: &str) -> Self {
        Self {
            id: id.into(),
            caps: QpuCapabilities::default(),
            next: 1,
        }
    }
}
impl QpuDevice for MockQpu {
    fn id(&self) -> &str {
        &self.id
    }
    fn capabilities(&self) -> QpuCapabilities {
        self.caps.clone()
    }
    fn submit_ir(
        &mut self,
        _ir: &str,
        _s: u32,
        _seed: u64,
    ) -> std::result::Result<QpuJobHandle, QpuDeviceError> {
        let h = QpuJobHandle(self.next);
        self.next += 1;
        Ok(h)
    }
    fn poll(&self, _h: QpuJobHandle) -> QpuJobStatus {
        QpuJobStatus::Completed
    }
    fn cancel(&mut self, _h: QpuJobHandle) -> Result<()> {
        Ok(())
    }
    fn reset(&mut self) -> Result<()> {
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mock_qpu() {
        let mut q = MockQpu::new("mock0");
        assert!(q.capabilities().measurement);
        let h = q.submit_ir("H0", 8, 1).unwrap();
        assert_eq!(q.poll(h), QpuJobStatus::Completed);
    }
}
