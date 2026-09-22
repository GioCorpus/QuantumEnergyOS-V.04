//! P7.2-02 — Normalized Node Health model.
//!
//! A single normalized view of a node's health across CPU, memory, storage,
//! network, GPU, QPU, services, temperature and energy. Subsystems that have
//! no measured or reported signal are left absent (`None`) — the platform does
//! **not** fabricate a "healthy" value for a subsystem it cannot observe.

use serde::{Deserialize, Serialize};

/// Availability of an energy reading. Every energy figure must be classified;
/// fabricated/assumed readings are never presented as measured.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum MeasurementSource {
    Measured,
    Estimated,
    Simulated,
    #[default]
    Unavailable,
}

/// Overall health status of a component or the node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Failed,
    #[default]
    Unknown,
}

/// Health of a single node component with a normalized status.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct ComponentHealth {
    pub status: HealthStatus,
    #[serde(default)]
    pub utilization_percent: f64,
    /// Free resource measure (bytes for memory/storage).
    #[serde(default)]
    pub available: u64,
    /// Total resource measure (bytes for memory/storage).
    #[serde(default)]
    pub total: u64,
    /// Temperature in degrees Celsius, if measured.
    #[serde(default)]
    pub temperature_celsius: Option<f64>,
}

impl ComponentHealth {
    pub fn healthy() -> Self {
        Self {
            status: HealthStatus::Healthy,
            ..Default::default()
        }
    }

    pub fn free_percent(&self) -> f64 {
        if self.total == 0 {
            0.0
        } else {
            (self.available as f64) / (self.total as f64) * 100.0
        }
    }
}

/// Aggregate health of running services: number alive/ready and a summary status.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceHealthSummary {
    pub total: u32,
    pub running: u32,
    pub ready: u32,
    pub failed: u32,
    pub status: HealthStatus,
}

/// Energy health of a node, with explicit measurement classification.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EnergyHealth {
    pub source: MeasurementSource,
    #[serde(default)]
    pub power_watts: Option<f64>,
    #[serde(default)]
    pub thermal_state: Option<String>,
}

/// The normalized node health report.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeHealth {
    pub cpu: Option<ComponentHealth>,
    pub memory: Option<ComponentHealth>,
    pub storage: Option<ComponentHealth>,
    pub network: Option<ComponentHealth>,
    pub gpu: Option<ComponentHealth>,
    pub qpu: Option<ComponentHealth>,
    pub services: ServiceHealthSummary,
    #[serde(default)]
    pub temperature_celsius: Option<f64>,
    pub energy: EnergyHealth,
    pub overall: HealthStatus,
}

impl NodeHealth {
    /// Compute the overall node health from the sub-component statuses.
    ///
    /// - Any `Failed` subsystem makes the node `Failed`.
    /// - Any `Degraded` subsystem (with no failures) makes the node `Degraded`.
    /// - Otherwise `Healthy` if at least one subsystem is observed, else `Unknown`.
    pub fn recompute_overall(&mut self) {
        let mut has_any = false;
        let mut saw_failure = false;
        let mut saw_degraded = false;

        for c in [
            self.cpu,
            self.memory,
            self.storage,
            self.network,
            self.gpu,
            self.qpu,
        ]
        .into_iter()
        .flatten()
        {
            has_any = true;
            match c.status {
                HealthStatus::Failed => saw_failure = true,
                HealthStatus::Degraded => saw_degraded = true,
                _ => {}
            }
        }

        if self.services.status == HealthStatus::Failed {
            saw_failure = true;
        } else if self.services.status == HealthStatus::Degraded {
            saw_degraded = true;
        }

        self.overall = if saw_failure {
            HealthStatus::Failed
        } else if saw_degraded {
            HealthStatus::Degraded
        } else if has_any {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        };
    }

    /// Whether the node is healthy enough to accept workloads.
    pub fn can_accept_workloads(&self) -> bool {
        matches!(self.overall, HealthStatus::Healthy | HealthStatus::Degraded)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn measurement_source_default_unavailable() {
        // We never default to measured/estimated.
        assert_eq!(MeasurementSource::default(), MeasurementSource::Unavailable);
    }

    #[test]
    fn free_percent_computation() {
        let c = ComponentHealth {
            available: 4,
            total: 8,
            ..Default::default()
        };
        assert_eq!(c.free_percent(), 50.0);
        assert_eq!(ComponentHealth::default().free_percent(), 0.0);
    }

    #[test]
    fn overall_healthy_when_all_ok() {
        let mut h = NodeHealth {
            cpu: Some(ComponentHealth::healthy()),
            memory: Some(ComponentHealth::healthy()),
            ..Default::default()
        };
        h.recompute_overall();
        assert_eq!(h.overall, HealthStatus::Healthy);
        assert!(h.can_accept_workloads());
    }

    #[test]
    fn failure_flips_overall() {
        let mut h = NodeHealth {
            cpu: Some(ComponentHealth::healthy()),
            gpu: Some(ComponentHealth {
                status: HealthStatus::Failed,
                ..Default::default()
            }),
            ..Default::default()
        };
        h.recompute_overall();
        assert_eq!(h.overall, HealthStatus::Failed);
        assert!(!h.can_accept_workloads());
    }

    #[test]
    fn degraded_when_component_degraded() {
        let mut h = NodeHealth {
            memory: Some(ComponentHealth {
                status: HealthStatus::Degraded,
                ..Default::default()
            }),
            ..Default::default()
        };
        h.recompute_overall();
        assert_eq!(h.overall, HealthStatus::Degraded);
        assert!(h.can_accept_workloads());
    }

    #[test]
    fn unknown_when_nothing_observed() {
        let mut h = NodeHealth::default();
        h.recompute_overall();
        assert_eq!(h.overall, HealthStatus::Unknown);
    }

    #[test]
    fn unavailable_energy_not_reported_as_measured() {
        let e = EnergyHealth::default();
        assert_eq!(e.source, MeasurementSource::Unavailable);
        assert!(e.power_watts.is_none());
    }

    #[test]
    fn services_failure_degrades_node() {
        let mut h = NodeHealth {
            cpu: Some(ComponentHealth::healthy()),
            services: ServiceHealthSummary {
                total: 2,
                running: 1,
                ready: 1,
                failed: 1,
                status: HealthStatus::Degraded,
            },
            ..Default::default()
        };
        h.recompute_overall();
        assert_eq!(h.overall, HealthStatus::Degraded);
    }
}
