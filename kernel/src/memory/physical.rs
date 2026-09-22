//! Bitmap/frame physical memory manager.
//!
//! PhysAddr/PhysPage are strongly typed with checked arithmetic,
//! alignment helpers, and canonical address validation.

use super::PAGE_SIZE;
use std::alloc::{alloc, Layout};
use std::collections::BTreeSet;

/// Physical address width supported by x86_64 (52 bits = 4 PiB).
const PHYS_ADDR_WIDTH: usize = 52;
const PHYS_ADDR_MAX: usize = (1 << PHYS_ADDR_WIDTH) - 1;

/// Page-aligned backing storage for a single page frame.
struct PageFrame {
    ptr: *mut u8,
}

impl PageFrame {
    fn new() -> Self {
        let layout = Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap();
        let ptr = unsafe { alloc(layout) };
        if ptr.is_null() {
            std::alloc::handle_alloc_error(layout);
        }
        // Zero the page frame
        unsafe {
            std::ptr::write_bytes(ptr, 0, PAGE_SIZE);
        }
        Self { ptr }
    }

    fn as_ptr(&self) -> *const u8 {
        self.ptr
    }
}

impl Drop for PageFrame {
    fn drop(&mut self) {
        let layout = Layout::from_size_align(PAGE_SIZE, PAGE_SIZE).unwrap();
        unsafe {
            std::alloc::dealloc(self.ptr, layout);
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysAddr(pub usize);

impl PhysAddr {
    /// Create a new physical address, validating it is canonical.
    pub fn new(addr: usize) -> Result<Self, &'static str> {
        if addr > PHYS_ADDR_MAX {
            return Err("physical address exceeds 52-bit canonical range");
        }
        Ok(Self(addr))
    }

    /// Create without validation (for trusted internal use).
    /// # Safety
    /// Caller must ensure `addr <= PHYS_ADDR_MAX`.
    pub const unsafe fn new_unchecked(addr: usize) -> Self {
        Self(addr)
    }

    /// Get the raw address value.
    pub const fn as_usize(self) -> usize {
        self.0
    }

    /// Check if address is page-aligned (4 KiB).
    pub const fn is_aligned(self) -> bool {
        self.0.is_multiple_of(super::PAGE_SIZE)
    }

    /// Align up to the next page boundary.
    pub fn align_up(self) -> Result<Self, &'static str> {
        let aligned = self
            .0
            .checked_add(super::PAGE_SIZE - 1)
            .and_then(|v| v.checked_div(super::PAGE_SIZE))
            .and_then(|v| v.checked_mul(super::PAGE_SIZE))
            .ok_or("alignment overflow")?;
        Self::new(aligned)
    }

    /// Align down to the previous page boundary.
    pub fn align_down(self) -> Self {
        unsafe { Self::new_unchecked(self.0 & !(super::PAGE_SIZE - 1)) }
    }

    /// Checked addition.
    pub fn checked_add(self, rhs: usize) -> Option<Self> {
        self.0.checked_add(rhs).and_then(|v| Self::new(v).ok())
    }

    /// Checked subtraction.
    pub fn checked_sub(self, rhs: usize) -> Option<Self> {
        self.0.checked_sub(rhs).and_then(|v| Self::new(v).ok())
    }

    /// Checked multiplication.
    pub fn checked_mul(self, rhs: usize) -> Option<Self> {
        self.0.checked_mul(rhs).and_then(|v| Self::new(v).ok())
    }

    /// Check if this address is within the canonical physical range.
    pub const fn is_canonical(self) -> bool {
        self.0 <= PHYS_ADDR_MAX
    }

    /// Get the physical page containing this address.
    pub const fn page(self) -> PhysPage {
        PhysPage(self.0 / super::PAGE_SIZE)
    }

    /// Offset from the start of the containing page.
    pub const fn page_offset(self) -> usize {
        self.0 % super::PAGE_SIZE
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhysPage(pub usize);

impl PhysPage {
    /// Create a new physical page number, validating bounds.
    pub fn new(page: usize, max_pages: usize) -> Result<Self, &'static str> {
        if page >= max_pages {
            return Err("physical page number exceeds managed range");
        }
        Ok(Self(page))
    }

    /// Create without validation (for trusted internal use).
    /// # Safety
    /// Caller must ensure `page < max_pages`.
    pub const unsafe fn new_unchecked(page: usize) -> Self {
        Self(page)
    }

    /// Get the raw page number.
    pub const fn as_usize(self) -> usize {
        self.0
    }

    /// Convert to the base physical address of this page.
    pub fn addr(self) -> PhysAddr {
        unsafe { PhysAddr::new_unchecked(self.0 * super::PAGE_SIZE) }
    }

    /// Checked addition of page count.
    pub fn checked_add(self, rhs: usize) -> Option<Self> {
        self.0.checked_add(rhs).map(Self)
    }

    /// Checked subtraction of page count.
    pub fn checked_sub(self, rhs: usize) -> Option<Self> {
        self.0.checked_sub(rhs).map(Self)
    }

    /// Convert from a physical address (rounding down to page boundary).
    pub fn from_addr(addr: PhysAddr) -> Self {
        addr.page()
    }

    /// Convert to physical address at given offset within page.
    pub fn with_offset(self, offset: usize) -> Result<PhysAddr, &'static str> {
        if offset >= super::PAGE_SIZE {
            return Err("offset exceeds page size");
        }
        self.addr().checked_add(offset).ok_or("address overflow")
    }
}

pub struct PhysicalMemoryManager {
    total: usize,
    free: BTreeSet<usize>,
    allocated: usize,
    // Reserved regions (MMIO, ACPI, kernel image, bootloader) that must never be allocated
    reserved: BTreeSet<(usize, usize)>, // (start_page, end_page_exclusive)
    // Backing storage for page frames (used in tests; in production would use direct physical map)
    backing: Vec<Option<PageFrame>>,
}
impl PhysicalMemoryManager {
    pub fn new(total_pages: usize) -> Self {
        Self {
            total: total_pages,
            free: (0..total_pages).collect(),
            allocated: 0,
            reserved: BTreeSet::new(),
            backing: (0..total_pages).map(|_| None).collect(),
        }
    }

    /// Reserve a physical address range (inclusive start, exclusive end) from allocation.
    /// Used for MMIO, ACPI tables, kernel image, bootloader memory.
    pub fn reserve(&mut self, start: PhysAddr, end: PhysAddr) -> Result<(), &'static str> {
        let start_page = start.align_down().page().as_usize();
        let end_page = end.align_up()?.page().as_usize();
        if start_page >= end_page || end_page > self.total {
            return Err("invalid reserve range");
        }
        // Remove from free set
        for p in start_page..end_page {
            self.free.remove(&p);
        }
        self.reserved.insert((start_page, end_page));
        Ok(())
    }

    pub fn total_pages(&self) -> usize {
        self.total
    }

    pub fn free_pages(&self) -> usize {
        self.free.len()
    }

    pub fn allocated_pages(&self) -> usize {
        self.allocated
    }

    /// Allocate one 4KiB page frame.
    pub fn allocate_page(&mut self) -> Option<PhysPage> {
        let f = *self.free.iter().next()?;
        self.free.remove(&f);
        self.allocated += 1;
        if self.backing[f].is_none() {
            self.backing[f] = Some(PageFrame::new());
        }
        Some(PhysPage(f))
    }

    /// Allocate contiguous pages (for large pages or DMA buffers).
    pub fn allocate_contiguous(&mut self, count: usize) -> Option<PhysPage> {
        if count == 0 || count > self.free.len() {
            return None;
        }
        // Simple first-fit contiguous search
        let mut start = None;
        let mut streak = 0;
        for &p in &self.free {
            if start.is_none() || p == start.unwrap() + streak {
                if start.is_none() {
                    start = Some(p);
                }
                streak += 1;
                if streak == count {
                    break;
                }
            } else {
                start = Some(p);
                streak = 1;
            }
        }
        let start = start?;
        if streak < count {
            return None;
        }
        for p in start..start + count {
            self.free.remove(&p);
        }
        self.allocated += count;
        for p in start..start + count {
            if self.backing[p].is_none() {
                self.backing[p] = Some(PageFrame::new());
            }
        }
        Some(PhysPage(start))
    }

    pub fn free_page(&mut self, p: PhysPage) {
        if p.0 < self.total && !self.is_reserved(p) && self.free.insert(p.0) {
            self.allocated -= 1;
        }
        self.backing[p.0] = None;
    }

    /// Free contiguous pages.
    pub fn free_contiguous(&mut self, start: PhysPage, count: usize) {
        for p in start.0..start.0 + count {
            let page = PhysPage(p);
            if page.0 < self.total && !self.is_reserved(page) && self.free.insert(p) {
                self.allocated -= 1;
            }
        }
        for p in start.0..start.0 + count {
            self.backing[p] = None;
        }
    }

    /// Get a mutable pointer to the physical page's memory (for test environment).
    /// Returns None if the page is not allocated or has no backing.
    pub fn get_page_mut(&self, page: PhysPage) -> Option<*mut u8> {
        self.backing[page.0].as_ref().map(|pf| pf.ptr)
    }

    /// Get an immutable pointer to the physical page's memory (for test environment).
    pub fn get_page(&self, page: PhysPage) -> Option<*const u8> {
        self.backing[page.0].as_ref().map(|pf| pf.as_ptr())
    }

    fn is_reserved(&self, p: PhysPage) -> bool {
        self.reserved.iter().any(|(s, e)| p.0 >= *s && p.0 < *e)
    }

    /// Check if a page is free (not allocated, not reserved).
    pub fn is_free(&self, p: PhysPage) -> bool {
        p.0 < self.total && self.free.contains(&p.0) && !self.is_reserved(p)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physaddr_canonical() {
        assert!(PhysAddr::new(0).is_ok());
        assert!(PhysAddr::new(PHYS_ADDR_MAX).is_ok());
        assert!(PhysAddr::new(PHYS_ADDR_MAX + 1).is_err());
    }

    #[test]
    fn physaddr_alignment() {
        let a = PhysAddr::new(0x1000).unwrap();
        assert!(a.is_aligned());
        let a = PhysAddr::new(0x1001).unwrap();
        assert!(!a.is_aligned());
        assert_eq!(a.align_down().as_usize(), 0x1000);
        assert_eq!(a.align_up().unwrap().as_usize(), 0x2000);
    }

    #[test]
    fn physaddr_checked_arithmetic() {
        let a = PhysAddr::new(0x1000).unwrap();
        assert_eq!(a.checked_add(0x1000).unwrap().as_usize(), 0x2000);
        assert_eq!(a.checked_sub(0x1000).unwrap().as_usize(), 0);
        assert!(a.checked_sub(0x2000).is_none());
        assert!(PhysAddr::new(PHYS_ADDR_MAX)
            .unwrap()
            .checked_add(1)
            .is_none());
    }

    #[test]
    fn physpage_conversion() {
        let page = PhysPage(5);
        let addr = page.addr();
        assert_eq!(addr.as_usize(), 5 * 4096);
        assert_eq!(PhysPage::from_addr(addr).as_usize(), 5);
        assert_eq!(page.with_offset(100).unwrap().as_usize(), 5 * 4096 + 100);
        assert!(page.with_offset(4096).is_err());
    }

    #[test]
    fn physpage_checked_arithmetic() {
        let p = PhysPage(10);
        assert_eq!(p.checked_add(5).unwrap().as_usize(), 15);
        assert_eq!(p.checked_sub(3).unwrap().as_usize(), 7);
        assert!(p.checked_sub(20).is_none());
    }

    #[test]
    fn pmm_reserve() {
        let mut pmm = PhysicalMemoryManager::new(100);
        // Reserve pages 10-19 (MMIO region)
        pmm.reserve(
            PhysAddr::new(10 * 4096).unwrap(),
            PhysAddr::new(20 * 4096).unwrap(),
        )
        .unwrap();
        assert_eq!(pmm.free_pages(), 90);
        // Allocate should skip reserved
        let p = pmm.allocate_page().unwrap();
        assert!(p.0 < 10 || p.0 >= 20);
    }

    #[test]
    fn pmm_contiguous() {
        let mut pmm = PhysicalMemoryManager::new(20);
        let p = pmm.allocate_contiguous(4).unwrap();
        assert!(pmm.is_free(PhysPage(p.0 + 4)));
        pmm.free_contiguous(p, 4);
        assert_eq!(pmm.free_pages(), 20);
    }

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
