/// Quantum Scheduler: queue, priority, allocation, deadlines, cancellation,
/// hardware availability.
///
/// The scheduler is backend-agnostic. It orders `QuantumJob`s by priority
/// (Critical > High > Normal > Low, FIFO within a priority), enforces
/// deadlines/timeouts, supports cancellation, and tracks backend availability
/// so physical/remote targets are never assumed present.
///
/// Physical quantum hardware is never required; tests use Mock/Simulator paths.
use crate::backend::QuantumBackendType;
use crate::error::{QuantumError, Result};
use crate::job::{JobPriority, JobStatus, QuantumJob};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::time::{SystemTime, UNIX_EPOCH};

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

/// Availability state for a named backend.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum BackendAvailability {
    #[default]
    Available,
    Degraded,
    Unavailable,
}

/// Scheduler configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Maximum queued (non-terminal) jobs.
    pub max_queue_size: usize,
    /// Default timeout applied when a job sets 0 (0 = no timeout).
    pub default_timeout_seconds: u64,
    /// Whether physical backends may be scheduled (default: false).
    pub allow_physical: bool,
    /// Whether remote backends may be scheduled without explicit opt-in.
    pub allow_remote: bool,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_queue_size: 1024,
            default_timeout_seconds: 300,
            allow_physical: false,
            allow_remote: false,
        }
    }
}

/// Scheduler statistics.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SchedulerStats {
    pub submitted: u64,
    pub completed: u64,
    pub failed: u64,
    pub cancelled: u64,
    pub expired: u64,
    pub queued: usize,
}

/// Priority-aware quantum job scheduler.
#[derive(Debug)]
pub struct QuantumScheduler {
    config: SchedulerConfig,
    /// Queued jobs ordered by priority (highest first, FIFO within priority).
    queue: VecDeque<QuantumJob>,
    /// Terminal + active jobs by id (for status/cancel queries).
    jobs: HashMap<String, QuantumJob>,
    /// Backend availability map. Unknown backends default to Available for
    /// Simulation/Emulation, Unavailable for Physical/Remote unless registered.
    availability: HashMap<String, BackendAvailability>,
    stats: SchedulerStats,
}

impl QuantumScheduler {
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            config,
            queue: VecDeque::new(),
            jobs: HashMap::new(),
            availability: HashMap::new(),
            stats: SchedulerStats::default(),
        }
    }

    pub fn config(&self) -> &SchedulerConfig {
        &self.config
    }

    pub fn stats(&self) -> &SchedulerStats {
        &self.stats
    }

    /// Register backend availability (e.g. "simulator" -> Available).
    pub fn set_availability(&mut self, backend: impl Into<String>, state: BackendAvailability) {
        self.availability.insert(backend.into(), state);
    }

    fn backend_available(&self, job: &QuantumJob) -> bool {
        let name = job.backend.as_deref().unwrap_or("simulator").to_lowercase();
        if let Some(state) = self.availability.get(&name) {
            return matches!(
                state,
                BackendAvailability::Available | BackendAvailability::Degraded
            );
        }
        // Defaults: simulator/emulation available; physical/remote gated.
        match name.as_str() {
            "simulator" | "simulation" | "emulator" | "emulation" | "mock" => true,
            "physical" | "majorana" | "qpu" => self.config.allow_physical,
            "remote" | "azure" | "azure-quantum" => self.config.allow_remote,
            _ => true,
        }
    }

    fn backend_type_of(&self, job: &QuantumJob) -> QuantumBackendType {
        let name = job.backend.as_deref().unwrap_or("simulator").to_lowercase();
        match name.as_str() {
            "emulator" | "emulation" | "mock" => QuantumBackendType::Emulation,
            "remote" | "azure" | "azure-quantum" => QuantumBackendType::Remote,
            "physical" | "majorana" | "qpu" => QuantumBackendType::Physical,
            _ => QuantumBackendType::Simulation,
        }
    }

    /// Submit a job: validates, applies defaults, enforces capacity.
    pub fn submit(&mut self, mut job: QuantumJob) -> Result<String> {
        if job.validate().is_err() {
            return Err(QuantumError::InvalidState("invalid job".to_string()));
        }
        if self.queue.len() >= self.config.max_queue_size {
            return Err(QuantumError::AllocationFailed(
                "scheduler queue full".to_string(),
            ));
        }
        // Capability gating: never silently queue physical/remote work.
        let btype = self.backend_type_of(&job);
        if btype == QuantumBackendType::Physical && !self.config.allow_physical {
            return Err(QuantumError::BackendNotAvailable(
                "physical QPU scheduling disabled without documented interface".to_string(),
            ));
        }
        if btype == QuantumBackendType::Remote && !self.config.allow_remote {
            // Allow explicit registration to enable remote.
            if !self.backend_available(&job) {
                return Err(QuantumError::BackendNotAvailable(
                    "remote QPU scheduling requires explicit availability registration".to_string(),
                ));
            }
        }
        if job.timeout_seconds == 0 {
            job.timeout_seconds = self.config.default_timeout_seconds;
        }
        job.mark_queued();
        let id = job.id.clone();
        self.insert_by_priority(job.clone());
        self.jobs.insert(id.clone(), job);
        self.stats.submitted += 1;
        self.stats.queued = self.queue.len();
        Ok(id)
    }

    fn insert_by_priority(&mut self, job: QuantumJob) {
        let pos = self
            .queue
            .iter()
            .position(|j| priority_rank(&j.priority) < priority_rank(&job.priority));
        match pos {
            Some(i) => self.queue.insert(i, job),
            None => self.queue.push_back(job),
        }
    }

    /// Peek at the next job without dequeuing.
    pub fn peek(&self) -> Option<&QuantumJob> {
        self.queue.front()
    }

    /// Dequeue the next schedulable job, skipping unavailable backends and
    /// expired deadlines (expired jobs are marked Failed).
    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<QuantumJob> {
        while let Some(mut job) = self.queue.pop_front() {
            // Deadline check.
            if job.is_timed_out() {
                job.status = JobStatus::Failed;
                job.error_message = Some("deadline exceeded before dispatch".to_string());
                job.completed_at = Some(now_secs());
                self.jobs.insert(job.id.clone(), job);
                self.stats.expired += 1;
                self.stats.failed += 1;
                self.stats.queued = self.queue.len();
                continue;
            }
            if !self.backend_available(&job) {
                // Re-queue at back; avoid busy loop by rotating once.
                self.queue.push_back(job);
                // If front hasn't changed to a schedulable job, stop.
                if let Some(front) = self.queue.front() {
                    if !self.backend_available(front) {
                        break;
                    }
                    continue;
                }
                break;
            }
            job.mark_running();
            self.jobs.insert(job.id.clone(), job.clone());
            self.stats.queued = self.queue.len();
            return Some(job);
        }
        self.stats.queued = self.queue.len();
        None
    }

    /// Mark a dispatched job completed.
    pub fn complete(&mut self, job_id: &str) -> Result<()> {
        let job = self
            .jobs
            .get_mut(job_id)
            .ok_or_else(|| QuantumError::InvalidState(format!("unknown job {job_id}")))?;
        job.mark_completed();
        self.stats.completed += 1;
        // Remove from queue if still present (e.g. completed without dispatch).
        self.queue.retain(|j| j.id != job_id);
        self.stats.queued = self.queue.len();
        Ok(())
    }

    /// Mark a dispatched job failed.
    pub fn fail(&mut self, job_id: &str, error: impl Into<String>) -> Result<()> {
        let job = self
            .jobs
            .get_mut(job_id)
            .ok_or_else(|| QuantumError::InvalidState(format!("unknown job {job_id}")))?;
        job.mark_failed(error.into());
        self.stats.failed += 1;
        self.queue.retain(|j| j.id != job_id);
        self.stats.queued = self.queue.len();
        Ok(())
    }

    /// Cancel a queued or tracked job.
    pub fn cancel(&mut self, job_id: &str) -> Result<()> {
        let mut found = false;
        self.queue.retain(|j| {
            if j.id == job_id {
                found = true;
                false
            } else {
                true
            }
        });
        if let Some(job) = self.jobs.get_mut(job_id) {
            if !is_terminal(&job.status) {
                job.mark_cancelled();
                self.stats.cancelled += 1;
                found = true;
            }
        }
        self.stats.queued = self.queue.len();
        if found {
            Ok(())
        } else {
            Err(QuantumError::InvalidState(format!("unknown job {job_id}")))
        }
    }

    /// Expire queued jobs past their deadlines. Returns expired ids.
    pub fn expire_deadlines(&mut self) -> Vec<String> {
        let mut expired = Vec::new();
        let mut remaining = VecDeque::new();
        while let Some(mut job) = self.queue.pop_front() {
            if job.is_timed_out() {
                job.status = JobStatus::Failed;
                job.error_message = Some("deadline exceeded".to_string());
                job.completed_at = Some(now_secs());
                expired.push(job.id.clone());
                self.jobs.insert(job.id.clone(), job);
                self.stats.expired += 1;
                self.stats.failed += 1;
            } else {
                remaining.push_back(job);
            }
        }
        self.queue = remaining;
        self.stats.queued = self.queue.len();
        expired
    }

    pub fn get(&self, job_id: &str) -> Option<&QuantumJob> {
        self.jobs.get(job_id)
    }

    pub fn queue_len(&self) -> usize {
        self.queue.len()
    }

    pub fn is_empty(&self) -> bool {
        self.queue.is_empty()
    }
}

fn priority_rank(p: &JobPriority) -> u8 {
    match p {
        JobPriority::Low => 0,
        JobPriority::Normal => 1,
        JobPriority::High => 2,
        JobPriority::Critical => 3,
    }
}

fn is_terminal(status: &JobStatus) -> bool {
    matches!(
        status,
        JobStatus::Completed | JobStatus::Failed | JobStatus::Cancelled | JobStatus::Timeout
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::circuit::QuantumCircuit;
    use crate::gates::QuantumGate;

    fn test_job(name: &str, priority: JobPriority) -> QuantumJob {
        let mut c = QuantumCircuit::new(name, 2).unwrap();
        c.add_gate(QuantumGate::Hadamard).unwrap();
        QuantumJob::new(name, c, 100).with_priority(priority)
    }

    #[test]
    fn test_priority_ordering() {
        let mut s = QuantumScheduler::new(SchedulerConfig::default());
        s.submit(test_job("low", JobPriority::Low)).unwrap();
        s.submit(test_job("critical", JobPriority::Critical))
            .unwrap();
        s.submit(test_job("normal", JobPriority::Normal)).unwrap();
        let first = s.next().unwrap();
        assert_eq!(first.name, "critical");
        let second = s.next().unwrap();
        assert_eq!(second.name, "normal");
    }

    #[test]
    fn test_physical_gated_by_default() {
        let mut s = QuantumScheduler::new(SchedulerConfig::default());
        let mut c = QuantumCircuit::new("p", 2).unwrap();
        c.add_gate(QuantumGate::Hadamard).unwrap();
        let job = QuantumJob::new("p", c, 10).with_backend("physical");
        assert!(s.submit(job).is_err());
    }

    #[test]
    fn test_cancel() {
        let mut s = QuantumScheduler::new(SchedulerConfig::default());
        let id = s.submit(test_job("a", JobPriority::Normal)).unwrap();
        s.cancel(&id).unwrap();
        assert!(s.is_empty());
        assert_eq!(s.get(&id).unwrap().status, JobStatus::Cancelled);
    }

    #[test]
    fn test_complete_and_fail() {
        let mut s = QuantumScheduler::new(SchedulerConfig::default());
        let id = s.submit(test_job("a", JobPriority::Normal)).unwrap();
        let dispatched = s.next().unwrap();
        assert_eq!(dispatched.id, id);
        s.complete(&id).unwrap();
        assert_eq!(s.get(&id).unwrap().status, JobStatus::Completed);

        let id2 = s.submit(test_job("b", JobPriority::Normal)).unwrap();
        let _ = s.next().unwrap();
        s.fail(&id2, "simulated failure").unwrap();
        assert_eq!(s.get(&id2).unwrap().status, JobStatus::Failed);
    }

    #[test]
    fn test_queue_full() {
        let mut s = QuantumScheduler::new(SchedulerConfig {
            max_queue_size: 1,
            ..Default::default()
        });
        s.submit(test_job("a", JobPriority::Normal)).unwrap();
        assert!(s.submit(test_job("b", JobPriority::Normal)).is_err());
    }

    #[test]
    fn test_expire_deadlines() {
        let mut s = QuantumScheduler::new(SchedulerConfig::default());
        let mut c = QuantumCircuit::new("e", 1).unwrap();
        c.add_gate(QuantumGate::Hadamard).unwrap();
        let mut job = QuantumJob::new("e", c, 10);
        job.timeout_seconds = 1;
        job.submitted_at = now_secs().saturating_sub(100);
        job.mark_queued();
        let id = job.id.clone();
        s.queue.push_back(job.clone());
        s.jobs.insert(id.clone(), job);
        let expired = s.expire_deadlines();
        assert!(expired.contains(&id));
    }
}
