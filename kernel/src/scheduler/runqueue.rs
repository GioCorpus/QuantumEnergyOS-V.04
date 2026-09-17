use crate::process::thread::{Priority, Tid};
use std::collections::VecDeque;
pub struct RunQueue {
    queues: [VecDeque<Tid>; 5],
}
impl RunQueue {
    pub fn new() -> Self {
        Self {
            queues: Default::default(),
        }
    }
    pub fn push(&mut self, tid: Tid, p: Priority) {
        self.queues[p as usize].push_back(tid);
    }
    pub fn pop(&mut self) -> Option<Tid> {
        for q in self.queues.iter_mut().rev() {
            if let Some(t) = q.pop_front() {
                return Some(t);
            }
        }
        None
    }
    pub fn len(&self) -> usize {
        self.queues.iter().map(|q| q.len()).sum()
    }
}
impl Default for RunQueue {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn prio() {
        let mut q = RunQueue::new();
        q.push(Tid(1), Priority::Low);
        q.push(Tid(2), Priority::High);
        assert_eq!(q.pop(), Some(Tid(2)));
    }
}
