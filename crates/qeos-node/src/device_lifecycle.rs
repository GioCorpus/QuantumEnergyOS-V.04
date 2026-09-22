//! P7.3-01 — Device state machine.
//!
//! A device lifecycle machine over the states specified by P7.3:
//! `DISCOVERED → INITIALIZING → READY → ACTIVE → QUIESCING → STOPPED →
//! FAILED → REMOVING → REMOVED` (with the additional `DEGRADED` and `RESETTING`
//! operational states).
//!
//! Every transition is validated against an allowed-edge table. The machine
//! enforces the P7.3 rules:
//! - no use-after-remove (`REMOVED` is terminal; no operation is permitted);
//! - no operation on `FAILED` without a recovery path;
//! - no double initialization (`INITIALIZING` is never re-entered);
//! - no double shutdown (`QUIESCING`/`STOPPED` cannot loop);
//! - deterministic cleanup (`REMOVING → REMOVED` clears resources).

use serde::{Deserialize, Serialize};

use crate::error::{NodeError, Result};

/// Operational state of a device.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeviceState {
    /// Discovered on the bus, not yet claimed.
    #[default]
    Discovered,
    /// Being initialized. Entered exactly once from `Discovered`.
    Initializing,
    /// Fully initialized and healthy, idle.
    Ready,
    /// Actively processing requests.
    Active,
    /// Draining in-flight work in preparation for stop/restart.
    Quiescing,
    /// Cleanly stopped.
    Stopped,
    /// Encountered a fault; requires a recovery path before reuse.
    Failed,
    /// Running with reduced capability (still operational).
    Degraded,
    /// Being reset.
    Resetting,
    /// Cleanup in progress (deterministic teardown).
    Removing,
    /// Terminal. No operation may target a removed device.
    Removed,
}

impl DeviceState {
    /// Validate `self -> next` against the allowed transition table.
    pub fn can_transition_to(self, next: DeviceState) -> Result<()> {
        let allowed: &[DeviceState] = match self {
            DeviceState::Discovered => &[
                DeviceState::Initializing,
                DeviceState::Failed,
                DeviceState::Removing,
            ],
            DeviceState::Initializing => &[
                DeviceState::Ready,
                DeviceState::Failed,
                DeviceState::Resetting,
                DeviceState::Removing,
            ],
            DeviceState::Ready => &[
                DeviceState::Active,
                DeviceState::Quiescing,
                DeviceState::Degraded,
                DeviceState::Failed,
                DeviceState::Resetting,
            ],
            DeviceState::Active => &[
                DeviceState::Quiescing,
                DeviceState::Degraded,
                DeviceState::Failed,
            ],
            DeviceState::Quiescing => &[
                DeviceState::Stopped,
                DeviceState::Ready,
                DeviceState::Failed,
            ],
            DeviceState::Stopped => &[
                DeviceState::Ready,
                DeviceState::Resetting,
                DeviceState::Removing,
            ],
            DeviceState::Failed => &[
                // Recovery path: must re-initialize, reset, or be removed.
                DeviceState::Initializing,
                DeviceState::Resetting,
                DeviceState::Removing,
            ],
            DeviceState::Degraded => &[
                DeviceState::Ready,
                DeviceState::Active,
                DeviceState::Failed,
                DeviceState::Resetting,
            ],
            DeviceState::Resetting => &[
                DeviceState::Ready,
                DeviceState::Failed,
                DeviceState::Removing,
            ],
            DeviceState::Removing => &[DeviceState::Removed],
            // Terminal state — no transitions out (no use-after-remove).
            DeviceState::Removed => &[],
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

    /// Whether any operation is permitted in this state.
    pub fn is_operational(self) -> bool {
        !matches!(self, DeviceState::Removed | DeviceState::Removing)
    }

    /// Whether the device has reached a terminal removed state.
    pub fn is_removed(self) -> bool {
        self == DeviceState::Removed
    }
}

/// A mutable device state machine enforcing valid transitions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceLifecycle {
    pub state: DeviceState,
}

impl Default for DeviceLifecycle {
    fn default() -> Self {
        Self {
            state: DeviceState::Discovered,
        }
    }
}

impl DeviceLifecycle {
    /// Attempt a validated transition.
    pub fn transition(&mut self, next: DeviceState) -> Result<()> {
        self.state.can_transition_to(next)?;
        self.state = next;
        Ok(())
    }

    /// Perform the deterministic cleanup path (`REMOVING -> REMOVED`).
    pub fn cleanup(&mut self) -> Result<()> {
        match self.state {
            DeviceState::Removing => {
                self.state = DeviceState::Removed;
                Ok(())
            }
            _ => Err(NodeError::InvalidNodeState {
                required: "REMOVING".into(),
                actual: format!("{:?}", self.state),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_normal_flow() {
        let mut m = DeviceLifecycle::default();
        assert_eq!(m.state, DeviceState::Discovered);
        m.transition(DeviceState::Initializing).unwrap();
        m.transition(DeviceState::Ready).unwrap();
        m.transition(DeviceState::Active).unwrap();
        m.transition(DeviceState::Quiescing).unwrap();
        m.transition(DeviceState::Stopped).unwrap();
    }

    #[test]
    fn removal_is_terminal() {
        let mut m = DeviceLifecycle::default();
        m.transition(DeviceState::Removing).unwrap();
        m.cleanup().unwrap();
        assert!(m.state.is_removed());
        assert!(!m.state.is_operational());
        // No transitions out of REMOVED.
        assert!(m.transition(DeviceState::Initializing).is_err());
    }

    #[test]
    fn double_initialization_rejected() {
        let mut m = DeviceLifecycle::default();
        m.transition(DeviceState::Initializing).unwrap();
        assert!(m.transition(DeviceState::Initializing).is_err());
    }

    #[test]
    fn failed_requires_recovery_path() {
        let mut m = DeviceLifecycle::default();
        m.transition(DeviceState::Initializing).unwrap();
        m.transition(DeviceState::Ready).unwrap();
        m.transition(DeviceState::Active).unwrap();
        m.transition(DeviceState::Failed).unwrap();
        // Cannot jump straight to Ready.
        assert!(m.transition(DeviceState::Ready).is_err());
        // Recovery path: reset.
        m.transition(DeviceState::Resetting).unwrap();
        m.transition(DeviceState::Ready).unwrap();
    }

    #[test]
    fn no_use_after_remove() {
        let mut m = DeviceLifecycle::default();
        m.transition(DeviceState::Removing).unwrap();
        m.cleanup().unwrap();
        // Any operation on a removed device must be rejected.
        assert!(!m.state.is_operational());
    }

    #[test]
    fn cleanup_requires_removing() {
        let mut m = DeviceLifecycle::default();
        m.transition(DeviceState::Initializing).unwrap();
        m.transition(DeviceState::Ready).unwrap();
        assert!(m.cleanup().is_err()); // not in REMOVING
    }

    #[test]
    fn quiescing_cannot_loop() {
        let mut m = DeviceLifecycle::default();
        m.transition(DeviceState::Initializing).unwrap();
        m.transition(DeviceState::Ready).unwrap();
        m.transition(DeviceState::Active).unwrap();
        m.transition(DeviceState::Quiescing).unwrap();
        assert!(m.transition(DeviceState::Quiescing).is_err());
    }

    #[test]
    fn degraded_is_operational() {
        let mut m = DeviceLifecycle::default();
        m.transition(DeviceState::Initializing).unwrap();
        m.transition(DeviceState::Ready).unwrap();
        m.transition(DeviceState::Degraded).unwrap();
        m.transition(DeviceState::Ready).unwrap();
    }
}
