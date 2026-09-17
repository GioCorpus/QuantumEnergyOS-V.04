//! Kernel timer abstraction (§17) with arch backend.

/// Backend contract: monotonic nanoseconds, never wall-clock.
pub trait KernelTimer {
    fn now_ns(&self) -> u64;
    fn sleep_until(&self, deadline_ns: u64);
}

/// Host backend reusing MonotonicClock.
pub struct HostTimer {
    clock: super::clock::MonotonicClock,
}

impl HostTimer {
    pub fn new() -> Self {
        Self {
            clock: super::clock::MonotonicClock::new(),
        }
    }
}

impl Default for HostTimer {
    fn default() -> Self {
        Self::new()
    }
}

impl KernelTimer for HostTimer {
    fn now_ns(&self) -> u64 {
        self.clock.now_ns()
    }
    fn sleep_until(&self, _deadline_ns: u64) {}
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn host_mono() {
        let t = HostTimer::new();
        assert!(t.now_ns() < 1_000_000_000_000);
    }
}
