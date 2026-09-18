//! Explicit kernel lifecycle phases (§7). Transition is explicit, no silent continuation.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum KernelPhase {
    #[default]
    Boot,
    EarlyInit,
    MemoryInit,
    InterruptInit,
    SchedulerInit,
    DeviceInit,
    SmpInit,
    ServiceInit,
    Running,
    Shutdown,
}

#[derive(Debug, Default)]
pub struct KernelState {
    phase: KernelPhase,
}

impl KernelState {
    pub const fn new() -> Self {
        Self {
            phase: KernelPhase::Boot,
        }
    }
    pub fn phase(&self) -> KernelPhase {
        self.phase
    }
    /// Advance exactly one step forward; returns Err on out-of-order jump.
    pub fn advance(&mut self, next: KernelPhase) -> Result<(), &'static str> {
        let order = [
            KernelPhase::Boot,
            KernelPhase::EarlyInit,
            KernelPhase::MemoryInit,
            KernelPhase::InterruptInit,
            KernelPhase::SchedulerInit,
            KernelPhase::DeviceInit,
            KernelPhase::SmpInit,
            KernelPhase::ServiceInit,
            KernelPhase::Running,
            KernelPhase::Shutdown,
        ];
        let cur = order.iter().position(|p| *p == self.phase).unwrap_or(0);
        let nxt = order
            .iter()
            .position(|p| *p == next)
            .ok_or("unknown phase")?;
        if nxt == cur + 1 {
            self.phase = next;
            Ok(())
        } else {
            Err("phase must advance exactly one step")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn ordered() {
        let mut s = KernelState::new();
        assert!(s.advance(KernelPhase::EarlyInit).is_ok());
        assert!(s.advance(KernelPhase::Running).is_err());
    }
}
