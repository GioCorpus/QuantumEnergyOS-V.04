/// Quantum Job System
///
/// This module manages the lifecycle of quantum circuit execution jobs.
/// All quantum operations must be submitted as jobs, enabling:
/// - Priority-based scheduling
/// - Timeout management
/// - Job tracking and monitoring
/// - Audit logging
/// - Resource allocation
///
/// Job lifecycle:
///
/// ```text
/// Submitted -> Queued -> Running -> Completed/Failed
///                              -> Cancelled
/// ```
use crate::circuit::QuantumCircuit;
use crate::error::{QuantumError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Job priority level
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum JobPriority {
    /// Lowest priority - background jobs
    Low = 0,
    /// Normal priority
    Normal = 1,
    /// High priority - interactive jobs
    High = 2,
    /// Critical priority - reserved for system operations
    Critical = 3,
}

impl std::fmt::Display for JobPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Low => write!(f, "Low"),
            Self::Normal => write!(f, "Normal"),
            Self::High => write!(f, "High"),
            Self::Critical => write!(f, "Critical"),
        }
    }
}

impl Default for JobPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Job execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum JobStatus {
    /// Job has been submitted but not started
    Submitted,
    /// Job is queued for execution
    Queued,
    /// Job is currently executing
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed
    Failed,
    /// Job was cancelled
    Cancelled,
    /// Backend does not support this job
    Unsupported,
}

impl std::fmt::Display for JobStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Submitted => write!(f, "Submitted"),
            Self::Queued => write!(f, "Queued"),
            Self::Running => write!(f, "Running"),
            Self::Completed => write!(f, "Completed"),
            Self::Failed => write!(f, "Failed"),
            Self::Cancelled => write!(f, "Cancelled"),
            Self::Unsupported => write!(f, "Unsupported"),
        }
    }
}

/// Quantum job - represents a circuit execution request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumJob {
    /// Unique job identifier (UUID)
    pub id: String,
    /// Job name
    pub name: String,
    /// Quantum circuit to execute
    pub circuit: QuantumCircuit,
    /// Number of measurement shots
    pub shots: u32,
    /// Execution priority
    pub priority: JobPriority,
    /// Backend to use (if specified)
    pub backend: Option<String>,
    /// Timeout in seconds (0 = no timeout)
    pub timeout_seconds: u64,
    /// Job metadata
    pub metadata: HashMap<String, String>,
    /// Submitted timestamp (Unix seconds)
    pub submitted_at: u64,
    /// Started timestamp (Unix seconds)
    pub started_at: Option<u64>,
    /// Completed timestamp (Unix seconds)
    pub completed_at: Option<u64>,
    /// Current status
    pub status: JobStatus,
    /// Error message if failed
    pub error_message: Option<String>,
    /// User/service identifier
    pub user_id: Option<String>,
    /// Session identifier
    pub session_id: Option<String>,
}

impl QuantumJob {
    /// Create a new quantum job
    pub fn new(name: impl Into<String>, circuit: QuantumCircuit, shots: u32) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        Self {
            id: uuid::Uuid::new_v4().to_string(),
            name: name.into(),
            circuit,
            shots,
            priority: JobPriority::Normal,
            backend: None,
            timeout_seconds: 300, // 5 minute default timeout
            metadata: HashMap::new(),
            submitted_at: now,
            started_at: None,
            completed_at: None,
            status: JobStatus::Submitted,
            error_message: None,
            user_id: None,
            session_id: None,
        }
    }

    /// Set job priority
    pub fn with_priority(mut self, priority: JobPriority) -> Self {
        self.priority = priority;
        self
    }

    /// Set backend preference
    pub fn with_backend(mut self, backend: impl Into<String>) -> Self {
        self.backend = Some(backend.into());
        self
    }

    /// Set timeout
    pub fn with_timeout(mut self, seconds: u64) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Set user identifier
    pub fn with_user(mut self, user_id: impl Into<String>) -> Self {
        self.user_id = Some(user_id.into());
        self
    }

    /// Set session identifier
    pub fn with_session(mut self, session_id: impl Into<String>) -> Self {
        self.session_id = Some(session_id.into());
        self
    }

    /// Add metadata
    pub fn add_metadata(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.metadata.insert(key.into(), value.into());
        self
    }

    /// Check if job has timed out
    pub fn is_timed_out(&self) -> bool {
        if self.timeout_seconds == 0 {
            return false; // No timeout
        }

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let elapsed = now.saturating_sub(self.submitted_at);
        elapsed > self.timeout_seconds
    }

    /// Mark job as queued
    pub fn mark_queued(&mut self) {
        self.status = JobStatus::Queued;
    }

    /// Mark job as running
    pub fn mark_running(&mut self) {
        self.status = JobStatus::Running;
        self.started_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }

    /// Mark job as completed
    pub fn mark_completed(&mut self) {
        self.status = JobStatus::Completed;
        self.completed_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }

    /// Mark job as failed
    pub fn mark_failed(&mut self, error: impl Into<String>) {
        self.status = JobStatus::Failed;
        self.error_message = Some(error.into());
        self.completed_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }

    /// Mark job as cancelled
    pub fn mark_cancelled(&mut self) {
        self.status = JobStatus::Cancelled;
        self.completed_at = Some(
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs(),
        );
    }

    /// Get execution duration in seconds
    pub fn execution_duration_seconds(&self) -> Option<f64> {
        match (self.started_at, self.completed_at) {
            (Some(start), Some(end)) => Some((end as f64) - (start as f64)),
            _ => None,
        }
    }

    /// Check if job is in a terminal state
    pub fn is_terminal(&self) -> bool {
        matches!(
            self.status,
            JobStatus::Completed
                | JobStatus::Failed
                | JobStatus::Cancelled
                | JobStatus::Unsupported
        )
    }

    /// Validate job
    pub fn validate(&self) -> Result<()> {
        // Validate circuit
        self.circuit.validate()?;

        // Validate shots
        if self.shots == 0 {
            return Err(QuantumError::InvalidGateParameters(
                "shots must be > 0".to_string(),
            ));
        }

        if self.shots > 1_000_000 {
            return Err(QuantumError::InvalidGateParameters(
                "shots must be <= 1,000,000".to_string(),
            ));
        }

        Ok(())
    }
}

/// Job statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobStatistics {
    /// Total jobs processed
    pub total_jobs: u64,
    /// Successfully completed jobs
    pub completed_jobs: u64,
    /// Failed jobs
    pub failed_jobs: u64,
    /// Cancelled jobs
    pub cancelled_jobs: u64,
    /// Average execution time in seconds
    pub avg_execution_time_seconds: f64,
    /// Total execution time in seconds
    pub total_execution_time_seconds: f64,
    /// Average queue wait time in seconds
    pub avg_queue_wait_seconds: f64,
}

impl Default for JobStatistics {
    fn default() -> Self {
        Self {
            total_jobs: 0,
            completed_jobs: 0,
            failed_jobs: 0,
            cancelled_jobs: 0,
            avg_execution_time_seconds: 0.0,
            total_execution_time_seconds: 0.0,
            avg_queue_wait_seconds: 0.0,
        }
    }
}

/// Job queue for managing quantum jobs
pub struct JobQueue {
    /// Queue capacity (max pending jobs)
    max_queue_size: usize,
    /// Pending jobs, grouped by priority
    queues: [Vec<QuantumJob>; 4], // 4 priority levels
}

impl JobQueue {
    /// Create new job queue
    pub fn new(max_queue_size: usize) -> Self {
        Self {
            max_queue_size,
            queues: [Vec::new(), Vec::new(), Vec::new(), Vec::new()],
        }
    }

    /// Enqueue a job
    pub fn enqueue(&mut self, job: QuantumJob) -> Result<()> {
        let queue_idx = job.priority as usize;
        let total_size: usize = self.queues.iter().map(|q| q.len()).sum();

        if total_size >= self.max_queue_size {
            return Err(QuantumError::AllocationFailed("Queue is full".to_string()));
        }

        self.queues[queue_idx].push(job);
        Ok(())
    }

    /// Dequeue next job (highest priority first)
    pub fn dequeue(&mut self) -> Option<QuantumJob> {
        // Check from highest to lowest priority
        for queue in self.queues.iter_mut().rev() {
            if !queue.is_empty() {
                return Some(queue.remove(0));
            }
        }
        None
    }

    /// Get current queue size
    pub fn size(&self) -> usize {
        self.queues.iter().map(|q| q.len()).sum()
    }

    /// Get queue size for specific priority
    pub fn size_for_priority(&self, priority: JobPriority) -> usize {
        self.queues[priority as usize].len()
    }

    /// Clear all queues
    pub fn clear(&mut self) {
        for queue in &mut self.queues {
            queue.clear();
        }
    }

    /// Peek at next job without removing it
    pub fn peek(&self) -> Option<&QuantumJob> {
        for queue in self.queues.iter().rev() {
            if let Some(job) = queue.first() {
                return Some(job);
            }
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_job_priority_ordering() {
        assert!(JobPriority::High > JobPriority::Normal);
        assert!(JobPriority::Critical > JobPriority::High);
    }

    #[test]
    fn test_job_creation() {
        let circuit = QuantumCircuit::new("test", 2).unwrap();
        let job = QuantumJob::new("test_job", circuit, 1000);
        assert_eq!(job.shots, 1000);
        assert_eq!(job.status, JobStatus::Submitted);
        assert_eq!(job.priority, JobPriority::Normal);
    }

    #[test]
    fn test_job_builder() {
        let circuit = QuantumCircuit::new("test", 2).unwrap();
        let job = QuantumJob::new("test_job", circuit, 1000)
            .with_priority(JobPriority::High)
            .with_timeout(600)
            .with_user("user123");

        assert_eq!(job.priority, JobPriority::High);
        assert_eq!(job.timeout_seconds, 600);
        assert_eq!(job.user_id, Some("user123".to_string()));
    }

    #[test]
    fn test_job_status_transitions() {
        let circuit = QuantumCircuit::new("test", 2).unwrap();
        let mut job = QuantumJob::new("test_job", circuit, 1000);

        assert_eq!(job.status, JobStatus::Submitted);
        job.mark_queued();
        assert_eq!(job.status, JobStatus::Queued);
        job.mark_running();
        assert_eq!(job.status, JobStatus::Running);
        job.mark_completed();
        assert_eq!(job.status, JobStatus::Completed);
        assert!(job.is_terminal());
    }

    #[test]
    fn test_job_queue() {
        let mut queue = JobQueue::new(100);

        let circuit = QuantumCircuit::new("test", 2).unwrap();
        let job1 = QuantumJob::new("job1", circuit.clone(), 100);
        let job2 = QuantumJob::new("job2", circuit, 100).with_priority(JobPriority::High);

        queue.enqueue(job1).unwrap();
        queue.enqueue(job2).unwrap();

        // Higher priority job should be dequeued first
        let next = queue.dequeue().unwrap();
        assert_eq!(next.priority, JobPriority::High);
    }

    #[test]
    fn test_job_queue_full() {
        let mut queue = JobQueue::new(1);
        let circuit = QuantumCircuit::new("test", 2).unwrap();
        let job1 = QuantumJob::new("job1", circuit.clone(), 100);
        let job2 = QuantumJob::new("job2", circuit, 100);

        queue.enqueue(job1).unwrap();
        assert!(queue.enqueue(job2).is_err());
    }
}
