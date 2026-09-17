use std::sync::{Mutex as StdMutex, MutexGuard, TryLockError};
// DEBT(AUDIT-2026-09-13): host-only std::Mutex. Sleepable, NOT IRQ-safe, NOT no_std.
// Valid contexts: process context, never hard-IRQ. For IRQ/non-sleepable use SpinLock.
// Lock ordering (§34): Memory < Device < Process.
pub struct KernelMutex<T> {
    inner: StdMutex<T>,
}
impl<T> KernelMutex<T> {
    pub fn new(v: T) -> Self {
        Self {
            inner: StdMutex::new(v),
        }
    }
    /// Locks; returns poisoned inner value instead of panicking where possible.
    pub fn lock(&self) -> MutexGuard<'_, T> {
        self.inner.lock().unwrap_or_else(|e| e.into_inner())
    }
    pub fn try_lock(&self) -> Option<MutexGuard<'_, T>> {
        match self.inner.try_lock() {
            Ok(g) => Some(g),
            Err(TryLockError::WouldBlock) => None,
            Err(TryLockError::Poisoned(e)) => Some(e.into_inner()),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn m() {
        let m = KernelMutex::new(1);
        *m.lock() = 2;
        assert_eq!(*m.lock(), 2);
    }
    #[test]
    fn try_m() {
        let m = KernelMutex::new(1);
        assert!(m.try_lock().is_some());
    }
}
