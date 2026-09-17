//! Experimental QPU device interface. Kernel provides access only.
#[derive(Debug, PartialEq, Eq)]
pub enum QuantumError {
    Unsupported,
    Busy,
    InvalidJob,
}
#[derive(Debug, Clone)]
pub struct QuantumJob {
    pub bytes: Vec<u8>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct JobHandle(pub u64);
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JobStatus {
    Queued,
    Running,
    Done,
    Failed,
}
pub trait QuantumDevice {
    fn initialize(&mut self) -> Result<(), QuantumError>;
    fn submit(&mut self, job: QuantumJob) -> Result<JobHandle, QuantumError>;
    fn poll(&mut self, job: JobHandle) -> Result<JobStatus, QuantumError>;
}
pub struct UnsupportedDevice;
impl QuantumDevice for UnsupportedDevice {
    fn initialize(&mut self) -> Result<(), QuantumError> {
        Err(QuantumError::Unsupported)
    }
    fn submit(&mut self, _j: QuantumJob) -> Result<JobHandle, QuantumError> {
        Err(QuantumError::Unsupported)
    }
    fn poll(&mut self, _j: JobHandle) -> Result<JobStatus, QuantumError> {
        Err(QuantumError::Unsupported)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unsupported() {
        let mut d = UnsupportedDevice;
        assert_eq!(d.initialize(), Err(QuantumError::Unsupported));
    }
}
