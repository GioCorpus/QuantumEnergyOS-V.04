//! P7.6-00/01 — Resource-aware scheduling: resource model.
//!
//! A job declares [`ResourceRequirements`]; a node publishes a
//! [`ResourceSnapshot`] describing what it can currently offer. The scheduler
//! places a job only on a node whose snapshot satisfies the requirements, and
//! only on nodes that are healthy and not in maintenance.

use serde::{Deserialize, Serialize};

/// Resources a job requests.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu_cores: u32,
    pub ram_bytes: u64,
    pub gpu: u32,
    pub vram_bytes: u64,
    /// Whether a QPU (simulator) is required and how many shots.
    pub qpu: bool,
    pub qpu_shots: Option<u64>,
    pub storage_bytes: u64,
    pub network_mbps: u64,
    /// Optional energy budget (joules); the job is not schedulable on a node
    /// unable to honor it.
    pub energy_budget_joules: Option<f64>,
    /// Optional maximum acceptable latency (ms) — locality hint.
    pub max_latency_ms: Option<u64>,
}

impl ResourceRequirements {
    pub fn is_zero(&self) -> bool {
        self.cpu_cores == 0
            && self.ram_bytes == 0
            && self.gpu == 0
            && self.vram_bytes == 0
            && !self.qpu
            && self.storage_bytes == 0
            && self.network_mbps == 0
            && self.energy_budget_joules.is_none()
    }
}

/// Current resource availability of a node.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ResourceSnapshot {
    pub cpu_cores_available: u32,
    pub ram_bytes_available: u64,
    pub gpu_available: u32,
    pub vram_bytes_available: u64,
    pub qpu_available: bool,
    pub storage_bytes_available: u64,
    pub network_mbps_available: u64,
    pub energy_capacity_joules: Option<f64>,
}

impl ResourceSnapshot {
    /// Whether this snapshot can satisfy the job's requirements.
    pub fn can_satisfy(&self, req: &ResourceRequirements) -> bool {
        if req.cpu_cores > self.cpu_cores_available {
            return false;
        }
        if req.ram_bytes > self.ram_bytes_available {
            return false;
        }
        if req.gpu > self.gpu_available {
            return false;
        }
        if req.vram_bytes > self.vram_bytes_available {
            return false;
        }
        if req.qpu && !self.qpu_available {
            return false;
        }
        if req.storage_bytes > self.storage_bytes_available {
            return false;
        }
        if req.network_mbps > self.network_mbps_available {
            return false;
        }
        if let Some(budget) = req.energy_budget_joules {
            match self.energy_capacity_joules {
                Some(cap) if cap >= budget => {}
                _ => return false,
            }
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn snapshot_satisfies_basic() {
        let snap = ResourceSnapshot {
            cpu_cores_available: 8,
            ram_bytes_available: 1 << 30,
            gpu_available: 1,
            vram_bytes_available: 1 << 28,
            qpu_available: true,
            storage_bytes_available: 1 << 31,
            network_mbps_available: 1000,
            energy_capacity_joules: Some(500.0),
        };
        let req = ResourceRequirements {
            cpu_cores: 4,
            ram_bytes: 1 << 29,
            gpu: 1,
            vram_bytes: 1 << 27,
            qpu: true,
            qpu_shots: Some(1000),
            storage_bytes: 1 << 30,
            network_mbps: 500,
            energy_budget_joules: Some(100.0),
            max_latency_ms: None,
        };
        assert!(snap.can_satisfy(&req));
    }

    #[test]
    fn rejects_insufficient_cpu() {
        let snap = ResourceSnapshot {
            cpu_cores_available: 2,
            ..Default::default()
        };
        let req = ResourceRequirements {
            cpu_cores: 4,
            ..Default::default()
        };
        assert!(!snap.can_satisfy(&req));
    }

    #[test]
    fn rejects_insufficient_gpu() {
        let snap = ResourceSnapshot {
            gpu_available: 0,
            ..Default::default()
        };
        let req = ResourceRequirements {
            gpu: 1,
            ..Default::default()
        };
        assert!(!snap.can_satisfy(&req));
    }

    #[test]
    fn rejects_qpu_requirement_without_qpu() {
        let snap = ResourceSnapshot {
            qpu_available: false,
            ..Default::default()
        };
        let req = ResourceRequirements {
            qpu: true,
            ..Default::default()
        };
        assert!(!snap.can_satisfy(&req));
    }

    #[test]
    fn rejects_energy_budget_not_honorable() {
        let snap = ResourceSnapshot {
            energy_capacity_joules: Some(50.0),
            ..Default::default()
        };
        let req = ResourceRequirements {
            energy_budget_joules: Some(200.0),
            ..Default::default()
        };
        assert!(!snap.can_satisfy(&req));
    }
}
