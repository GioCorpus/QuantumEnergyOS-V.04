//! Bump + free-list kernel heap (host model using Vec).
pub struct KernelHeap { base: usize, size: usize, used: usize }
impl KernelHeap {
    pub fn new(base: usize, size: usize) -> Self { Self { base, size, used: 0 } }
    pub fn alloc(&mut self, n: usize, align: usize) -> Option<usize> {
        let a = (self.base + self.used + align - 1) & !(align - 1);
        if a + n > self.base + self.size { return None; }
        self.used = a + n - self.base; Some(a)
    }
    pub fn used(&self) -> usize { self.used }
}
#[cfg(test)] mod tests { use super::*; #[test] fn bump() { let mut h = KernelHeap::new(0x1000, 0x1000); assert!(h.alloc(16, 8).is_some()); assert!(h.alloc(0x2000, 8).is_none()); } }
