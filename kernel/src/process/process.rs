use std::collections::BTreeMap;
use super::thread::Thread;
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)] pub struct Pid(pub u32);
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum ProcessState { Created, Ready, Running, Blocked, Sleeping, Terminated }
#[derive(Debug)] pub struct Credentials { pub uid: u32, pub gid: u32 }
#[derive(Debug)] pub struct Process {
    pub pid: Pid, pub state: ProcessState, pub creds: Credentials,
    pub threads: BTreeMap<u32, Thread>, pub handles: u32,
}
impl Process {
    pub fn new(pid: u32, uid: u32) -> Self {
        Self { pid: Pid(pid), state: ProcessState::Created, creds: Credentials { uid, gid: uid }, threads: BTreeMap::new(), handles: 0 }
    }
    pub fn add_thread(&mut self, t: Thread) { self.threads.insert(t.tid.0, t); }
    pub fn transition(&mut self, s: ProcessState) { self.state = s; }
}
#[cfg(test)] mod tests { use super::*; use crate::process::thread::{Thread, Priority}; #[test] fn lifecycle() { let mut p = Process::new(1, 0); p.transition(ProcessState::Ready); assert_eq!(p.state, ProcessState::Ready); p.add_thread(Thread::new(1, Priority::Normal)); assert_eq!(p.threads.len(), 1); } }
