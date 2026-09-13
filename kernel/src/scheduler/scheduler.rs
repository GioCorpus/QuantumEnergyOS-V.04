use super::runqueue::RunQueue;
use crate::process::thread::{Tid, Priority};
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum SchedClass { Normal, Realtime, Scientific, Telemetry, Quantum }
pub struct Scheduler { rq: RunQueue, current: Option<Tid>, ticks: u64 }
impl Scheduler {
    pub fn new() -> Self { Self { rq: RunQueue::new(), current: None, ticks: 0 } }
    pub fn spawn(&mut self, tid: Tid, p: Priority) { self.rq.push(tid, p); }
    pub fn schedule(&mut self) -> Option<Tid> {
        if let Some(c) = self.current.take() { /* in real kernel: save context */ let _ = c; }
        self.ticks += 1; self.current = self.rq.pop(); self.current
    }
    pub fn ticks(&self) -> u64 { self.ticks }
}
impl Default for Scheduler { fn default() -> Self { Self::new() } }
#[cfg(test)] mod tests { use super::*; #[test] fn sched() { let mut s = Scheduler::new(); s.spawn(Tid(1), Priority::Normal); assert_eq!(s.schedule(), Some(Tid(1))); assert_eq!(s.ticks(), 1); } }
