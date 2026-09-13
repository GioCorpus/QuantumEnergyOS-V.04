#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct Tid(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)] pub enum Priority { Idle = 0, Low = 1, Normal = 2, High = 3, Realtime = 4 }
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum ThreadState { Ready, Running, Blocked, Sleeping(u64), Terminated }
#[derive(Debug, Clone)] pub struct Thread { pub tid: Tid, pub prio: Priority, pub state: ThreadState, pub affinity: u32 }
impl Thread {
    pub fn new(tid: u32, prio: Priority) -> Self { Self { tid: Tid(tid), prio, state: ThreadState::Ready, affinity: 0 } }
    pub fn sleep(&mut self, until: u64) { self.state = ThreadState::Sleeping(until); }
    pub fn wake(&mut self) { if matches!(self.state, ThreadState::Sleeping(_) | ThreadState::Blocked) { self.state = ThreadState::Ready; } }
}
#[cfg(test)] mod tests { use super::*; #[test] fn sleep_wake() { let mut t = Thread::new(1, Priority::Normal); t.sleep(99); t.wake(); assert_eq!(t.state, ThreadState::Ready); } }
