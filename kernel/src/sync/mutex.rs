use std::sync::{Mutex as StdMutex, MutexGuard};
pub struct KernelMutex<T> { inner: StdMutex<T> }
impl<T> KernelMutex<T> { pub fn new(v: T) -> Self { Self { inner: StdMutex::new(v) } } pub fn lock(&self) -> MutexGuard<'_, T> { self.inner.lock().unwrap() } }
#[cfg(test)] mod tests { use super::*; #[test] fn m() { let m = KernelMutex::new(1); *m.lock() = 2; assert_eq!(*m.lock(), 2); } }
