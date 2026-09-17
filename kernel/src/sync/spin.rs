//! Host-model spinlock (§33-34). Documents IRQ/preempt context explicitly.
//!
//! Context: non-sleepable, IRQ-unsafe unless interrupts disabled by caller.
//! Lock ordering (§34): Memory < Device < Process. Never sleep while holding.

use core::cell::UnsafeCell;
use core::sync::atomic::{AtomicBool, Ordering};

pub struct SpinLock<T> {
    locked: AtomicBool,
    data: UnsafeCell<T>,
}

// SAFETY: SpinLock provides mutual exclusion via AtomicBool test-and-set.
// Send/Sync iff T is Send. Locking uses Acquire, unlocking uses Release.
unsafe impl<T: Send> Send for SpinLock<T> {}
unsafe impl<T: Send> Sync for SpinLock<T> {}

impl<T> SpinLock<T> {
    pub const fn new(v: T) -> Self {
        Self {
            locked: AtomicBool::new(false),
            data: UnsafeCell::new(v),
        }
    }
    pub fn lock(&self) -> SpinGuard<'_, T> {
        while self.locked.swap(true, Ordering::Acquire) {
            core::hint::spin_loop();
        }
        SpinGuard { lock: self }
    }
    pub fn try_lock(&self) -> Option<SpinGuard<'_, T>> {
        if self.locked.swap(true, Ordering::Acquire) {
            None
        } else {
            Some(SpinGuard { lock: self })
        }
    }
    /// # Safety: caller must hold the lock (via guard).
    unsafe fn unlock(&self) {
        self.locked.store(false, Ordering::Release);
    }
}

pub struct SpinGuard<'a, T> {
    lock: &'a SpinLock<T>,
}

impl<T> core::ops::Deref for SpinGuard<'_, T> {
    type Target = T;
    fn deref(&self) -> &T {
        // SAFETY: guard proves exclusive access.
        unsafe { &*self.lock.data.get() }
    }
}
impl<T> core::ops::DerefMut for SpinGuard<'_, T> {
    fn deref_mut(&mut self) -> &mut T {
        // SAFETY: guard proves exclusive access.
        unsafe { &mut *self.lock.data.get() }
    }
}
impl<T> Drop for SpinGuard<'_, T> {
    fn drop(&mut self) {
        // SAFETY: guard held the lock.
        unsafe { self.lock.unlock() };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn spin_basic() {
        let l = SpinLock::new(1);
        {
            *l.lock() = 2;
        }
        assert_eq!(*l.lock(), 2);
        assert!(l.try_lock().is_some());
    }
}
