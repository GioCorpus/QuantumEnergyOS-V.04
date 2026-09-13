 //! Bump kernel heap — host model. No free; OOM returns None (never panics, §14).
//! Validates alignment (non-zero power-of-two) and uses checked arithmetic.
pub struct KernelHeap { base: usize, size: usize, used: usize }
impl KernelHeap {
    pub fn new(base: usize, size: usize) -> Self { Self { base, size, used: 0 } }
    pub fn alloc(&mut self, n: usize, align: usize) -> Option<usize> {
        if !align.is_power_of_two() { return None; }
        let cur = self.base.checked_add(self.used)?;
        let aligned = cur.checked_add(align - 1)? & !(align - 1);
        let end = aligned.checked_add(n)?;
        let limit = self.base.checked_add(self.size)?;
        if end > limit { return None; }
        self.used = end - self.base; Some(aligned)
    }
    pub fn used(&self) -> usize { self.used }
}
#[cfg(test)] mod tests { use super::*; #[test] fn bump() { let mut h = KernelHeap::new(0x1000, 0x1000); assert!(h.alloc(16, 8).is_some()); assert!(h.alloc(0x2000, 8).is_none()); } #[test] fn rejects_bad_align() { let mut h = KernelHeap::new(0x1000, 0x1000); assert!(h.alloc(8, 0).is_none()); assert!(h.alloc(8, 3).is_none()); } #[test] fn overflow_safe() { let mut h = KernelHeap::new(usize::MAX - 16, 32); assert!(h.alloc(64, 8).is_none()); } }
