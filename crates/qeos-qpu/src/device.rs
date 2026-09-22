//! P7.4-01 — Generic QPU device interface.
//!
//! A hardware-independent device abstraction with the lifecycle
//! `discover -> initialize -> allocate -> submit -> measure -> reset -> shutdown`.
//! The [`QpuDevice`] owns a [`QpuBackend`], drives job isolation, and reports
//! telemetry. It never claims a physical device: backend capabilities are
//! authoritative about availability.

use serde::{Deserialize, Serialize};

use crate::backend::{QpuBackend, QpuBackendKind, QpuCapabilities, SimulatorBackend};
use crate::error::QpuError;
use crate::job::{QpuJob, QpuJobId, QpuResult};

/// QPU device lifecycle state.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum QpuDeviceState {
    Discovered,
    Initializing,
    #[default]
    Ready,
    Busy,
    Failed,
    Shutdown,
}

/// Telemetry counters for a QPU device.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct QpuTelemetry {
    pub jobs_submitted: u64,
    pub jobs_completed: u64,
    pub jobs_failed: u64,
    pub jobs_cancelled: u64,
    pub jobs_timed_out: u64,
    pub measures: u64,
    pub resets: u64,
}

/// A generic QPU device.
pub struct QpuDevice {
    pub id: String,
    backend: Box<dyn QpuBackend>,
    state: QpuDeviceState,
    telemetry: QpuTelemetry,
    next_job_id: u64,
}

impl QpuDevice {
    /// Create a device over a simulator backend.
    pub fn simulator(id: &str) -> Self {
        Self::with_backend(id, Box::new(SimulatorBackend::new()))
    }

    pub fn with_backend(id: &str, backend: Box<dyn QpuBackend>) -> Self {
        Self {
            id: id.to_string(),
            backend,
            state: QpuDeviceState::Discovered,
            telemetry: QpuTelemetry::default(),
            next_job_id: 1,
        }
    }

    /// discover — report device info + backend capabilities.
    pub fn discover(&self) -> QpuDeviceInfoView {
        QpuDeviceInfoView {
            id: self.id.clone(),
            state: self.state,
            kind: self.backend.kind(),
            caps: self.backend.capabilities().clone(),
        }
    }

    /// initialize — move from Discovered to Ready.
    pub fn initialize(&mut self) -> Result<(), QpuError> {
        if self.state != QpuDeviceState::Discovered {
            return Err(QpuError::InvalidInput("device not in Discovered".into()));
        }
        self.state = QpuDeviceState::Ready;
        Ok(())
    }

    /// Backend capabilities (validates hardware availability).
    pub fn capabilities(&self) -> &QpuCapabilities {
        self.backend.capabilities()
    }

    /// allocate/validate a job against backend limits.
    pub fn validate_job(&self, job: &QpuJob) -> Result<(), QpuError> {
        let caps = self.backend.capabilities();
        job.validate(caps.qubit_limit, caps.max_shots)
    }

    /// submit — place a job on the device queue. Validates limits first and
    /// transitions the caller's job to `Queued` (so it can be measured/cancelled).
    pub fn submit(&mut self, job: &mut QpuJob) -> Result<QpuJobId, QpuError> {
        self.ensure_ready()?;
        job.id = QpuJobId(self.next_job_id);
        self.next_job_id += 1;
        self.validate_job(job)?;
        job.transition(crate::job::QpuJobState::Queued);
        self.telemetry.jobs_submitted += 1;
        Ok(job.id)
    }

    /// measure — execute a submitted job and return its result.
    pub fn measure(&mut self, job: &mut QpuJob) -> Result<QpuResult, QpuError> {
        self.ensure_ready()?;
        if job.state != crate::job::QpuJobState::Queued {
            return Err(QpuError::InvalidInput("job is not queued".into()));
        }
        job.transition(crate::job::QpuJobState::Running);
        job.transition(crate::job::QpuJobState::Measuring);
        let result = self.backend.execute(job)?;
        job.transition(crate::job::QpuJobState::Completed);
        self.telemetry.measures += 1;
        self.telemetry.jobs_completed += 1;
        Ok(result)
    }

    /// cancel — allow a queued/job to be cancelled.
    pub fn cancel(&mut self, job: &mut QpuJob) -> Result<(), QpuError> {
        self.ensure_ready()?;
        job.transition(crate::job::QpuJobState::Cancelled);
        self.telemetry.jobs_cancelled += 1;
        Ok(())
    }

    /// reset — reset device telemetry counters (and, in a real device, the
    /// hardware). Does not change backend availability.
    pub fn reset(&mut self) -> Result<(), QpuError> {
        self.ensure_ready()?;
        self.telemetry = QpuTelemetry::default();
        self.telemetry.resets += 1;
        Ok(())
    }

    /// shutdown — move device to Shutdown state.
    pub fn shutdown(&mut self) -> Result<(), QpuError> {
        self.state = QpuDeviceState::Shutdown;
        Ok(())
    }

    /// Device telemetry.
    pub fn telemetry(&self) -> &QpuTelemetry {
        &self.telemetry
    }

    /// Whether the underlying backend has a physical/vendor device.
    pub fn hardware_available(&self) -> bool {
        self.backend.capabilities().vendor_available
    }

    fn ensure_ready(&self) -> Result<(), QpuError> {
        if self.state != QpuDeviceState::Ready {
            return Err(QpuError::InvalidInput("device not Ready".into()));
        }
        Ok(())
    }
}

/// Snapshot of device discovery info.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QpuDeviceInfoView {
    pub id: String,
    pub state: QpuDeviceState,
    pub kind: QpuBackendKind,
    pub caps: QpuCapabilities,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn initialize_then_submit_and_measure() {
        let mut dev = QpuDevice::simulator("q0");
        dev.initialize().unwrap();
        let mut job = QpuJob::new(0, 1, 64, 42);
        let _id = dev.submit(&mut job).unwrap();
        let res = dev.measure(&mut job).unwrap();
        assert!(res.simulation_only);
        assert_eq!(res.shots, 64);
        assert_eq!(dev.telemetry().jobs_completed, 1);
    }

    #[test]
    fn submit_rejects_over_limit() {
        let mut dev = QpuDevice::simulator("q0");
        dev.initialize().unwrap();
        let mut job = QpuJob::new(0, 40, 10, 7);
        assert!(matches!(
            dev.submit(&mut job),
            Err(QpuError::ResourceLimit(_))
        ));
    }

    #[test]
    fn device_not_ready_rejects_ops() {
        // Device still Discovered.
        let mut dev = QpuDevice::simulator("q0");
        let mut job = QpuJob::new(0, 1, 10, 7);
        assert!(dev.submit(&mut job).is_err());
        assert!(dev.reset().is_err());
    }

    #[test]
    fn hardware_never_claimed_when_unavailable() {
        let dev = QpuDevice::simulator("q0");
        assert!(!dev.hardware_available());
        assert!(!dev.discover().caps.vendor_available);
    }

    #[test]
    fn shutdown_blocks_ops() {
        let mut dev = QpuDevice::simulator("q0");
        dev.initialize().unwrap();
        dev.shutdown().unwrap();
        let mut job = QpuJob::new(0, 1, 10, 7);
        assert!(dev.submit(&mut job).is_err());
    }

    #[test]
    fn telemetry_tracks_cancel_and_reset() {
        let mut dev = QpuDevice::simulator("q0");
        dev.initialize().unwrap();
        let mut job = QpuJob::new(0, 1, 10, 7);
        let _id = dev.submit(&mut job).unwrap();
        dev.cancel(&mut job).unwrap();
        assert_eq!(dev.telemetry().jobs_cancelled, 1);
        dev.reset().unwrap();
        assert_eq!(dev.telemetry().jobs_cancelled, 0);
        assert_eq!(dev.telemetry().resets, 1);
    }
}
