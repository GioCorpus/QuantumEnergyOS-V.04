//! Bitmap/frame physical memory manager.
use std::collections::BTreeSet;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysAddr(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysPage(pub usize);
impl PhysPage {
    pub fn addr(self) -> PhysAddr {
        PhysAddr(self.0 * super::PAGE_SIZE)
    }
}
pub struct PhysicalMemoryManager {
    total: usize,
    free: BTreeSet<usize>,
    allocated: usize,
}
impl PhysicalMemoryManager {
    pub fn new(total_pages: usize) -> Self {
        Self {
            total: total_pages,
            free: (0..total_pages).collect(),
            allocated: 0,
        }
    }
    pub fn total_pages(&self) -> usize {
        self.total
    }
    pub fn free_pages(&self) -> usize {
        self.free.len()
    }
    /// Allocate one 4KiB page frame.
    pub fn allocate_page(&mut self) -> Option<PhysPage> {
        let f = *self.free.iter().next()?;
        self.free.remove(&f);
        self.allocated += 1;
        Some(PhysPage(f))
    }
    pub fn free_page(&mut self, p: PhysPage) {
        if p.0 < self.total && self.free.insert(p.0) {
            self.allocated -= 1;
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn alloc_free() {
        let mut m = PhysicalMemoryManager::new(4);
        let p = m.allocate_page().unwrap();
        assert_eq!(m.free_pages(), 3);
        m.free_page(p);
        assert_eq!(m.free_pages(), 4);
    }
    #[test]
    fn oom() {
        let mut m = PhysicalMemoryManager::new(1);
        m.allocate_page().unwrap();
        assert!(m.allocate_page().is_none());
    }
}
