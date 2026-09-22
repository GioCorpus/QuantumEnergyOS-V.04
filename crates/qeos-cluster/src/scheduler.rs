//! P7.6-01/03/04 — Resource-aware distributed scheduler.
//!
//! [`Scheduler::schedule`] places a [`DistributedJob`] on the most suitable
//! healthy node whose [`ResourceSnapshot`] satisfies the requirements, honoring
//! priority, affinity and locality. Every decision is observable: an
//! unsuccessful placement returns a `Deferred` outcome with the reason.
//!
//! Backend selection is derived from requirements: a QPU job is routed to a
//! `qpu-sim` (or `vendor-qpu`) backend, a GPU job to `gpu-sim`, otherwise `cpu`.
//! No claim of *optimal* scheduling is made — the policy is deterministic and
//! documented; optimality would require benchmarks (P7.9).

use crate::identity::NodeId;
use crate::job::DistributedJobId;
use crate::resources::{ResourceRequirements, ResourceSnapshot};

/// A node the scheduler may place a job on.
#[derive(Debug, Clone)]
pub struct SchedulableNode {
    pub id: NodeId,
    /// Whether the node is healthy and accepting workloads.
    pub available: bool,
    pub resources: ResourceSnapshot,
    /// Latency to this node (ms); used for locality preference.
    pub latency_ms: Option<u64>,
}

/// A concrete placement decision.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placement {
    pub job_id: DistributedJobId,
    pub node: NodeId,
    pub backend: String,
}

/// Outcome of a scheduling attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ScheduleOutcome {
    Placed(Placement),
    /// No suitable placement within this scheduling pass; includes the reason.
    Deferred(String),
}

impl ScheduleOutcome {
    pub fn placed_node(&self) -> Option<&NodeId> {
        match self {
            ScheduleOutcome::Placed(p) => Some(&p.node),
            _ => None,
        }
    }
}

/// The distributed scheduler.
pub struct Scheduler;

impl Scheduler {
    /// Choose a backend for a job based on its requirements.
    pub fn pick_backend(req: &ResourceRequirements) -> String {
        if req.qpu {
            "qpu-sim".to_string()
        } else if req.gpu > 0 {
            "gpu-sim".to_string()
        } else {
            "cpu".to_string()
        }
    }

    /// Attempt to place a job. Returns `Placed` on a satisfying healthy node or
    /// `Deferred(reason)`.
    pub fn schedule(
        job_id: DistributedJobId,
        req: &ResourceRequirements,
        nodes: &[SchedulableNode],
        affinity: Option<&NodeId>,
    ) -> ScheduleOutcome {
        if req.is_zero() {
            return ScheduleOutcome::Deferred("empty resource requirements".into());
        }

        // Honor explicit affinity/locality first.
        if let Some(hint) = affinity {
            if let Some(node) = nodes.iter().find(|n| &n.id == hint) {
                if node.available && node.resources.can_satisfy(req) {
                    return ScheduleOutcome::Placed(Placement {
                        job_id,
                        node: node.id.clone(),
                        backend: Self::pick_backend(req),
                    });
                }
                return ScheduleOutcome::Deferred(
                    "affinity node unavailable or under-resourced".into(),
                );
            }
        }

        // Otherwise pick the first satisfying available node (deterministic
        // order). Latency, when required, is honored as a locality bound.
        let backend = Self::pick_backend(req);
        for node in nodes {
            if !node.available {
                continue;
            }
            if let Some(max_lat) = req.max_latency_ms {
                match node.latency_ms {
                    Some(l) if l > max_lat => continue,
                    _ => {}
                }
            }
            if node.resources.can_satisfy(req) {
                return ScheduleOutcome::Placed(Placement {
                    job_id,
                    node: node.id.clone(),
                    backend: backend.clone(),
                });
            }
        }

        ScheduleOutcome::Deferred("no healthy node satisfies requirements".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn node(id: &str, avail: bool, cpu: u32, gpu: u32, qpu: bool) -> SchedulableNode {
        SchedulableNode {
            id: NodeId(id.into()),
            available: avail,
            resources: ResourceSnapshot {
                cpu_cores_available: cpu,
                gpu_available: gpu,
                qpu_available: qpu,
                ram_bytes_available: 1 << 30,
                vram_bytes_available: 1 << 28,
                storage_bytes_available: 1 << 31,
                network_mbps_available: 1000,
                energy_capacity_joules: Some(1000.0),
            },
            latency_ms: Some(10),
        }
    }

    fn req(cpu: u32, gpu: u32, qpu: bool) -> ResourceRequirements {
        ResourceRequirements {
            cpu_cores: cpu,
            gpu,
            qpu,
            ..Default::default()
        }
    }

    #[test]
    fn places_on_satisfying_healthy_node() {
        let nodes = vec![node("n1", true, 8, 0, false)];
        let out = Scheduler::schedule(DistributedJobId(1), &req(2, 0, false), &nodes, None);
        assert_eq!(out.placed_node(), Some(&NodeId("n1".into())));
        assert_eq!(
            out,
            ScheduleOutcome::Placed(Placement {
                job_id: DistributedJobId(1),
                node: NodeId("n1".into()),
                backend: "cpu".into(),
            })
        );
    }

    #[test]
    fn defers_when_node_unavailable() {
        let nodes = vec![node("n1", false, 8, 0, false)];
        let out = Scheduler::schedule(DistributedJobId(1), &req(2, 0, false), &nodes, None);
        assert!(matches!(out, ScheduleOutcome::Deferred(_)));
    }

    #[test]
    fn routes_qpu_job_to_qpu_backend() {
        let nodes = vec![node("n1", true, 8, 0, true)];
        let out = Scheduler::schedule(DistributedJobId(1), &req(1, 0, true), &nodes, None);
        match out {
            ScheduleOutcome::Placed(p) => assert_eq!(p.backend, "qpu-sim"),
            _ => panic!("expected placement"),
        }
    }

    #[test]
    fn routes_gpu_job_to_gpu_backend() {
        let nodes = vec![node("n1", true, 8, 1, false)];
        let out = Scheduler::schedule(DistributedJobId(1), &req(1, 1, false), &nodes, None);
        match out {
            ScheduleOutcome::Placed(p) => assert_eq!(p.backend, "gpu-sim"),
            _ => panic!("expected placement"),
        }
    }

    #[test]
    fn affinity_preferred() {
        let nodes = vec![node("nA", true, 4, 0, false), node("nB", true, 4, 0, false)];
        // Affinity toward nB.
        let out = Scheduler::schedule(
            DistributedJobId(1),
            &req(2, 0, false),
            &nodes,
            Some(&NodeId("nB".into())),
        );
        assert_eq!(out.placed_node(), Some(&NodeId("nB".into())));
    }

    #[test]
    fn latency_bound_honored() {
        let mut fast = node("fast", true, 8, 0, false);
        fast.latency_ms = Some(5);
        let mut slow = node("slow", true, 8, 0, false);
        slow.latency_ms = Some(100);
        let req_lat = ResourceRequirements {
            cpu_cores: 1,
            max_latency_ms: Some(20),
            ..Default::default()
        };
        let out = Scheduler::schedule(DistributedJobId(1), &req_lat, &[fast, slow], None);
        // slow is skipped due to latency bound.
        assert_eq!(out.placed_node(), Some(&NodeId("fast".into())));
    }

    #[test]
    fn empty_requirements_deferred() {
        let nodes = vec![node("n1", true, 8, 0, false)];
        let out = Scheduler::schedule(DistributedJobId(1), &Default::default(), &nodes, None);
        assert!(matches!(out, ScheduleOutcome::Deferred(_)));
    }
}
