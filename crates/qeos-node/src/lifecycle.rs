//! P7.2-01 — Production Operations Model.
//!
//! Defines the normalized node lifecycle, service lifecycle, readiness/liveness,
//! dependency ordering, graceful shutdown, restart policy and maintenance mode.
//!
//! The lifecycle is a *state machine* with explicitly validated transitions.
//! Transitions that are not allowed are rejected with [`crate::NodeError::IllegalTransition`]
//! rather than being silently accepted. This guarantees that a node can never
//! jump between unrelated operational states (e.g. `FAILED -> SHUTDOWN` is
//! allowed, `FAILED -> READY` is not — a node must pass through `RECOVERING`).

use serde::{Deserialize, Serialize};

use crate::error::{NodeError, Result};

/// The operational phase of a QEOS node.
///
/// `BOOTING      ` kernel/runtime bring-up, before services start.
/// `STARTING  `   services and dependencies are being started.
/// `READY      `  node fully operational, serving workloads.
/// `DEGRADED  `   node operational but with reduced capability (e.g. a GPU lost).
/// `MAINTENANCE` node drained or paused for operator work.
/// `RECOVERING`  node recovering after a failure.
/// `FAILED     ` node failed and cannot serve workloads without recovery.
/// `SHUTDOWN  `  node is shutting down or offline.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum NodeLifecycleState {
    #[default]
    Booting,
    Starting,
    Ready,
    Degraded,
    Maintenance,
    Recovering,
    Failed,
    Shutdown,
}

impl NodeLifecycleState {
    /// Whether a node in this state can accept (execute) workloads.
    pub fn can_accept_workloads(self) -> bool {
        matches!(
            self,
            NodeLifecycleState::Ready | NodeLifecycleState::Degraded
        )
    }

    /// Whether a node in this state is considered "alive" (has a running
    /// runtime) for liveness purposes.
    pub fn is_alive(self) -> bool {
        !matches!(
            self,
            NodeLifecycleState::Failed | NodeLifecycleState::Shutdown
        )
    }

    /// Whether the node is fully healthy (not degraded, not recovering).
    pub fn is_fully_healthy(self) -> bool {
        self == NodeLifecycleState::Ready
    }

    /// Validate a transition `self -> next` against the allowed transition table.
    ///
    /// Returns [`NodeError::IllegalTransition`] when the transition is not
    /// permitted. This prevents invalid state jumps.
    pub fn can_transition_to(self, next: NodeLifecycleState) -> Result<()> {
        // Explicit transition table. Only these edges are permitted.
        let allowed = match self {
            NodeLifecycleState::Booting => {
                vec![
                    NodeLifecycleState::Starting,
                    NodeLifecycleState::Shutdown,
                    NodeLifecycleState::Failed,
                ]
            }
            NodeLifecycleState::Starting => {
                vec![
                    NodeLifecycleState::Ready,
                    NodeLifecycleState::Degraded,
                    NodeLifecycleState::Failed,
                    NodeLifecycleState::Recovering,
                    NodeLifecycleState::Shutdown,
                ]
            }
            NodeLifecycleState::Ready => {
                vec![
                    NodeLifecycleState::Degraded,
                    NodeLifecycleState::Maintenance,
                    NodeLifecycleState::Shutdown,
                ]
            }
            NodeLifecycleState::Degraded => {
                vec![
                    NodeLifecycleState::Ready,
                    NodeLifecycleState::Recovering,
                    NodeLifecycleState::Maintenance,
                    NodeLifecycleState::Failed,
                    NodeLifecycleState::Shutdown,
                ]
            }
            NodeLifecycleState::Maintenance => {
                vec![
                    NodeLifecycleState::Ready,
                    NodeLifecycleState::Degraded,
                    NodeLifecycleState::Shutdown,
                ]
            }
            NodeLifecycleState::Recovering => {
                vec![
                    NodeLifecycleState::Ready,
                    NodeLifecycleState::Degraded,
                    NodeLifecycleState::Failed,
                ]
            }
            NodeLifecycleState::Failed => {
                vec![NodeLifecycleState::Recovering, NodeLifecycleState::Shutdown]
            }
            NodeLifecycleState::Shutdown => vec![NodeLifecycleState::Booting],
        };

        if allowed.contains(&next) {
            Ok(())
        } else {
            Err(NodeError::IllegalTransition {
                from: format!("{self:?}"),
                to: format!("{next:?}"),
            })
        }
    }
}

/// Restart policy for a service or node component.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RestartPolicy {
    /// Never restart automatically.
    Never,
    /// Restart only when the component fails (non-zero/error exit).
    #[default]
    OnFailure,
    /// Always restart, even after clean termination.
    Always,
}

/// Exponential backoff configuration for restart loops, preventing restart storms.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct Backoff {
    /// Initial delay between restart attempts (in milliseconds).
    pub initial_ms: u64,
    /// Multiplier applied to the delay after each failed attempt.
    pub factor: f64,
    /// Maximum delay between attempts (in milliseconds).
    pub max_ms: u64,
}

impl Default for Backoff {
    fn default() -> Self {
        Backoff {
            initial_ms: 500,
            factor: 2.0,
            max_ms: 30_000,
        }
    }
}

impl Backoff {
    /// Compute the delay for the `attempt`-th restart attempt (0-indexed).
    pub fn delay_ms(&self, attempt: u32) -> u64 {
        let attempt = attempt.saturating_add(1);
        let scaled = (self.initial_ms as f64) * self.factor.powi(attempt as i32 - 1);
        (scaled as u64).min(self.max_ms)
    }
}

/// Lifecycle state of an individual service.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum ServiceLifecycleState {
    #[default]
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed,
    Restarting,
}

impl ServiceLifecycleState {
    pub fn can_transition_to(self, next: ServiceLifecycleState) -> Result<()> {
        let allowed = match self {
            ServiceLifecycleState::Starting => vec![
                ServiceLifecycleState::Running,
                ServiceLifecycleState::Failed,
                ServiceLifecycleState::Stopping,
            ],
            ServiceLifecycleState::Running => vec![
                ServiceLifecycleState::Stopping,
                ServiceLifecycleState::Failed,
                ServiceLifecycleState::Restarting,
            ],
            ServiceLifecycleState::Stopping => vec![
                ServiceLifecycleState::Stopped,
                ServiceLifecycleState::Failed,
            ],
            ServiceLifecycleState::Stopped => vec![
                ServiceLifecycleState::Starting,
                ServiceLifecycleState::Restarting,
            ],
            ServiceLifecycleState::Failed => vec![
                ServiceLifecycleState::Starting,
                ServiceLifecycleState::Restarting,
                ServiceLifecycleState::Stopped,
            ],
            ServiceLifecycleState::Restarting => vec![
                ServiceLifecycleState::Starting,
                ServiceLifecycleState::Stopped,
                ServiceLifecycleState::Failed,
            ],
        };

        if allowed.contains(&next) {
            Ok(())
        } else {
            Err(NodeError::IllegalTransition {
                from: format!("{self:?}"),
                to: format!("{next:?}"),
            })
        }
    }
}

/// Declares whether a component is ready to receive the portion of traffic it
/// is responsible for. Readiness and liveness are distinct concepts:
/// - **Liveness**: the process/component is alive and making progress.
/// - **Readiness**: the component is fully initialized and can serve requests.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct ServiceStatus {
    pub liveness: bool,
    #[serde(default)]
    pub ready: bool,
}

impl ServiceStatus {
    pub fn alive(self) -> bool {
        self.liveness
    }

    pub fn ready(self) -> bool {
        self.ready
    }
}

/// A service declaration with dependency ordering used during start/stop.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSpec {
    pub name: String,
    /// Names of services that must be running before this one starts.
    pub depends_on: Vec<String>,
    pub restart_policy: RestartPolicy,
    pub backoff: Backoff,
}

impl ServiceSpec {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            depends_on: Vec::new(),
            restart_policy: RestartPolicy::default(),
            backoff: Backoff::default(),
        }
    }

    pub fn with_dependency(mut self, dep: &str) -> Self {
        self.depends_on.push(dep.to_string());
        self
    }
}

/// Tracks whether a node is in maintenance mode, during which it must not
/// receive new workloads.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct MaintenanceMode {
    pub active: bool,
}

/// A concrete, mutable node runtime that carries the current lifecycle state
/// and enforces valid transitions, readiness/liveness and maintenance mode.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NodeRuntime {
    pub state: NodeLifecycleState,
    pub services: Vec<ServiceStatus>,
    pub maintenance: MaintenanceMode,
}

impl NodeRuntime {
    /// Attempt a lifecycle transition. Rejects illegal transitions.
    pub fn transition(&mut self, next: NodeLifecycleState) -> Result<()> {
        self.state.can_transition_to(next)?;
        self.state = next;
        Ok(())
    }

    /// Enter maintenance mode (only allowed from READY/DEGRADED).
    pub fn enter_maintenance(&mut self) -> Result<()> {
        if !matches!(
            self.state,
            NodeLifecycleState::Ready | NodeLifecycleState::Degraded
        ) {
            return Err(NodeError::InvalidNodeState {
                required: "READY or DEGRADED".into(),
                actual: format!("{:?}", self.state),
            });
        }
        self.transition(NodeLifecycleState::Maintenance)?;
        self.maintenance.active = true;
        Ok(())
    }

    /// Exit maintenance mode back to READY.
    pub fn exit_maintenance(&mut self) -> Result<()> {
        if self.state != NodeLifecycleState::Maintenance {
            return Err(NodeError::InvalidNodeState {
                required: "MAINTENANCE".into(),
                actual: format!("{:?}", self.state),
            });
        }
        self.transition(NodeLifecycleState::Ready)?;
        self.maintenance.active = false;
        Ok(())
    }

    /// Gracefully shut down: transition through SHUTDOWN.
    pub fn shutdown(&mut self) -> Result<()> {
        self.transition(NodeLifecycleState::Shutdown)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lifecycle_default_is_booting() {
        assert_eq!(NodeLifecycleState::default(), NodeLifecycleState::Booting);
    }

    #[test]
    fn valid_transition_accepted() {
        assert!(NodeLifecycleState::Ready
            .can_transition_to(NodeLifecycleState::Maintenance)
            .is_ok());
        assert!(NodeLifecycleState::Failed
            .can_transition_to(NodeLifecycleState::Recovering)
            .is_ok());
        assert!(NodeLifecycleState::Recovering
            .can_transition_to(NodeLifecycleState::Ready)
            .is_ok());
    }

    #[test]
    fn invalid_transition_rejected() {
        assert!(NodeLifecycleState::Failed
            .can_transition_to(NodeLifecycleState::Ready)
            .is_err());
        assert!(NodeLifecycleState::Ready
            .can_transition_to(NodeLifecycleState::Booting)
            .is_err());
        // READY -> RECOVERING is not an allowed direct jump
        assert!(NodeLifecycleState::Ready
            .can_transition_to(NodeLifecycleState::Recovering)
            .is_err());
    }

    #[test]
    fn full_boot_sequence() {
        let mut rt = NodeRuntime::default();
        rt.transition(NodeLifecycleState::Starting).unwrap();
        rt.transition(NodeLifecycleState::Ready).unwrap();
        assert!(rt.state.can_accept_workloads());
    }

    #[test]
    fn failed_requires_recovery() {
        let mut rt = NodeRuntime {
            state: NodeLifecycleState::Ready,
            ..Default::default()
        };
        rt.transition(NodeLifecycleState::Degraded).unwrap();
        rt.transition(NodeLifecycleState::Failed).unwrap();
        // Can't go straight to READY
        assert!(rt.transition(NodeLifecycleState::Ready).is_err());
        rt.transition(NodeLifecycleState::Recovering).unwrap();
        rt.transition(NodeLifecycleState::Ready).unwrap();
        assert!(rt.state.can_accept_workloads());
    }

    #[test]
    fn maintenance_requires_ready() {
        let mut rt = NodeRuntime::default();
        assert!(rt.enter_maintenance().is_err()); // not ready
        rt.transition(NodeLifecycleState::Starting).unwrap();
        rt.transition(NodeLifecycleState::Ready).unwrap();
        rt.enter_maintenance().unwrap();
        assert!(rt.maintenance.active);
        assert!(!rt.state.can_accept_workloads());
        rt.exit_maintenance().unwrap();
        assert!(!rt.maintenance.active);
        assert_eq!(rt.state, NodeLifecycleState::Ready);
    }

    #[test]
    fn shutdown_transition() {
        let mut rt = NodeRuntime {
            state: NodeLifecycleState::Ready,
            ..Default::default()
        };
        rt.shutdown().unwrap();
        assert_eq!(rt.state, NodeLifecycleState::Shutdown);
    }

    #[test]
    fn backoff_grows_bounded() {
        let b = Backoff {
            initial_ms: 100,
            factor: 2.0,
            max_ms: 400,
        };
        assert_eq!(b.delay_ms(0), 100);
        assert_eq!(b.delay_ms(1), 200);
        // bounded by max
        assert_eq!(b.delay_ms(10), 400);
    }

    #[test]
    fn service_lifecycle_transitions() {
        assert!(ServiceLifecycleState::Starting
            .can_transition_to(ServiceLifecycleState::Running)
            .is_ok());
        assert!(ServiceLifecycleState::Running
            .can_transition_to(ServiceLifecycleState::Failed)
            .is_ok());
        assert!(ServiceLifecycleState::Failed
            .can_transition_to(ServiceLifecycleState::Starting)
            .is_ok());
    }

    #[test]
    fn service_status_ready_and_liveness() {
        let s = ServiceStatus {
            liveness: true,
            ready: true,
        };
        assert!(s.alive());
        assert!(s.ready());
        assert!(!ServiceStatus::default().ready());
    }

    #[test]
    fn service_spec_dependencies() {
        let spec = ServiceSpec::new("gpu")
            .with_dependency("kernel")
            .with_dependency("runtime");
        assert_eq!(spec.depends_on, vec!["kernel", "runtime"]);
    }
}
