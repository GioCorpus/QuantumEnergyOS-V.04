//! Explicit OOM policy (§14). Never `panic!("out of memory")` as sole strategy.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OomAction {
    Deny,
    LogAndDeny,
    ReclaimAndRetry,
}

#[derive(Debug, Clone, Copy)]
pub struct OomPolicy {
    pub action: OomAction,
}

impl Default for OomPolicy {
    fn default() -> Self {
        Self {
            action: OomAction::LogAndDeny,
        }
    }
}

impl OomPolicy {
    /// Decide on allocation failure. Returns Err always (deny), caller logs if needed.
    pub fn on_oom(&self, requested: usize) -> Result<(), &'static str> {
        let _ = requested;
        Err("out of memory: allocation denied by policy")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn denies() {
        let p = OomPolicy::default();
        assert!(p.on_oom(4096).is_err());
    }
}