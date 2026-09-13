//! SPSC lock-free ring buffer (host model with atomics).
use std::sync::atomic::{AtomicUsize, Ordering};
#[derive(Debug, Clone, Copy, PartialEq, Eq)] pub enum OverflowPolicy { DropNew, OverwriteOld, Backpressure }
pub struct SpscRing<T: Copy, const N: usize> { buf: [Option<T>; N], head: AtomicUsize, tail: AtomicUsize, policy: OverflowPolicy }
impl<T: Copy, const N: usize> SpscRing<T, N> {
    pub fn new(policy: OverflowPolicy) -> Self { Self { buf: [None; N], head: AtomicUsize::new(0), tail: AtomicUsize::new(0), policy } }
    pub fn push(&mut self, v: T) -> Result<(), T> {
        let h = self.head.load(Ordering::Acquire); let t = self.tail.load(Ordering::Acquire);
        if h.wrapping_sub(t) >= N {
            match self.policy { OverflowPolicy::DropNew => return Err(v), OverflowPolicy::OverwriteOld => { self.buf[t % N] = Some(v); self.tail.store(t.wrapping_add(1), Ordering::Release); return Ok(()); } OverflowPolicy::Backpressure => return Err(v), }
        }
        self.buf[h % N] = Some(v); self.head.store(h.wrapping_add(1), Ordering::Release); Ok(())
    }
    pub fn pop(&mut self) -> Option<T> {
        let t = self.tail.load(Ordering::Acquire); let h = self.head.load(Ordering::Acquire);
        if t == h { return None; }
        let v = self.buf[t % N].take(); self.tail.store(t.wrapping_add(1), Ordering::Release); v
    }
    pub fn len(&self) -> usize { self.head.load(Ordering::Acquire).wrapping_sub(self.tail.load(Ordering::Acquire)) }
}
#[cfg(test)] mod tests { use super::*; #[test] fn ring() { let mut r = SpscRing::<u32, 4>::new(OverflowPolicy::DropNew); r.push(1).unwrap(); assert_eq!(r.pop(), Some(1)); } #[test] fn overwrite() { let mut r = SpscRing::<u32, 2>::new(OverflowPolicy::OverwriteOld); r.push(1).unwrap(); r.push(2).unwrap(); r.push(3).unwrap(); assert_eq!(r.len(), 2); } }
