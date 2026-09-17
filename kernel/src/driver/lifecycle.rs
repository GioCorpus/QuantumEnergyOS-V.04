//! 9-state device lifecycle (spec 4.4.6), kernel-side tracker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DeviceLifecycle {
    Discovered,
    Probing,
    Initialized,
    Ready,
    Running,
    Suspended,
    Stopping,
    Removed,
    Failed,
}
impl DeviceLifecycle {
    pub fn is_terminal(self) -> bool {
        matches!(self, Self::Removed | Self::Failed)
    }
    pub fn can_transition(self, n: Self) -> bool {
        use DeviceLifecycle::*;
        matches!(
            (self, n),
            (Discovered, Probing)
                | (Probing, Initialized)
                | (Probing, Failed)
                | (Initialized, Ready)
                | (Initialized, Failed)
                | (Ready, Running)
                | (Ready, Stopping)
                | (Running, Suspended)
                | (Running, Stopping)
                | (Running, Failed)
                | (Suspended, Ready)
                | (Suspended, Stopping)
                | (Stopping, Removed)
                | (Stopping, Failed)
        )
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn flow() {
        assert!(DeviceLifecycle::Discovered.can_transition(DeviceLifecycle::Probing));
        assert!(!DeviceLifecycle::Ready.can_transition(DeviceLifecycle::Discovered));
        assert!(DeviceLifecycle::Failed.is_terminal());
    }
}
