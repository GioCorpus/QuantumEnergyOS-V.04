//! Per-CPU data (§20). Host model: single CPU0, no false-sharing padding yet.

#[derive(Debug, Default)]
pub struct PerCpu {
    pub cpu_id: u32,
    pub current_tid: Option<crate::process::thread::Tid>,
    pub switches: u64,
    pub interrupts: u64,
}

impl PerCpu {
    pub fn new(cpu_id: u32) -> Self {
        Self {
            cpu_id,
            current_tid: None,
            switches: 0,
            interrupts: 0,
        }
    }
    pub fn on_switch(&mut self, tid: crate::process::thread::Tid) {
        self.current_tid = Some(tid);
        self.switches = self.switches.saturating_add(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::process::thread::Tid;
    #[test]
    fn percpu_switch() {
        let mut p = PerCpu::new(0);
        p.on_switch(Tid(1));
        assert_eq!(p.switches, 1);
    }
}
