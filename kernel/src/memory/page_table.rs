//! Multi-level page table implementation for x86_64 (PML4 -> PDPT -> PD -> PT).
//!
//! This module provides a proper page table abstraction that owns its frames
//! and implements full 4-level page table walking.

use crate::arch::x86_64::paging::{EntryFlags, ENTRIES_PER_TABLE, p1_index, p2_index, p3_index, p4_index};
use crate::memory::{PhysicalMemoryManager, PhysAddr, PhysPage, VirtAddr, VirtPage};
use core::sync::atomic::{AtomicU64, Ordering};

const PAGE_TABLE_LEVELS: usize = 4;

#[repr(C, align(4096))]
struct PageTableFrame {
    entries: [AtomicU64; ENTRIES_PER_TABLE],
}

impl PageTableFrame {
    fn new() -> Self {
        Self { entries: core::array::from_fn(|_| AtomicU64::new(0)) }
    }
    fn entry(&self, idx: usize) -> &AtomicU64 { &self.entries[idx] }
}

pub struct PageTable {
    root: PhysPage,
    pmm: *mut PhysicalMemoryManager,
    switched: bool,
}
unsafe impl Send for PageTable {}

impl PageTable {
    pub fn new(pmm: &mut PhysicalMemoryManager) -> Result<Self, &'static str> {
        let pml4_page = pmm.allocate_page().ok_or("out of memory: cannot allocate PML4 frame")?;
        let pt = Self { root: pml4_page, pmm: pmm as *mut PhysicalMemoryManager, switched: false };
        pt.clear_frame(pml4_page); Ok(pt)
    }
    pub fn root(&self) -> PhysPage { self.root }
    pub fn root_addr(&self) -> PhysAddr { self.root.addr() }
    fn clear_frame(&self, page: PhysPage) -> Result<(), &'static str> {
        let pmm = unsafe { &mut *self.pmm };
        let frame_ptr = pmm.get_page_mut(page).ok_or("page table frame not allocated")?;
        let frame = unsafe { &mut *(frame_ptr as *mut PageTableFrame) };
        for entry in &frame.entries { entry.store(0, Ordering::Relaxed); }
        Ok(())
    }
    fn get_or_create_table(&self, parent_page: PhysPage, idx: usize, _level: usize) -> Result<PhysPage, &'static str> {
        let pmm = unsafe { &mut *self.pmm };
        let parent_frame_ptr = pmm.get_page_mut(parent_page).ok_or("parent page table frame not allocated")?;
        let parent_frame = unsafe { &mut *(parent_frame_ptr as *mut PageTableFrame) };
        let entry = parent_frame.entry(idx).load(Ordering::Acquire);
        if entry & 1 != 0 {
            let child_phys = unsafe { PhysAddr::new_unchecked(((entry >> 12) & 0x000F_FFFF_FFFF) as usize) };
            Ok(child_phys.page())
        } else {
            let child_page = pmm.allocate_page().ok_or("out of memory: cannot allocate page table frame")?;
            self.clear_frame(child_page)?;
            let flags = EntryFlags::PRESENT | EntryFlags::WRITABLE;
            let entry_bits = ((child_page.addr().as_usize() & 0x000F_FFFF_FFFF) << 12) | flags.bits() as usize;
            parent_frame.entry(idx).store(entry_bits as u64, Ordering::Release);
            Ok(child_page)
        }
    }
fn walk(&self, vpage: VirtPage) -> Result<(PhysPage, usize), &'static str> {
        let vaddr = vpage.addr().as_usize();
        let mut current_page = self.root;
        current_page = self.get_or_create_table(current_page, p4_index(vaddr), 4)?;
        current_page = self.get_or_create_table(current_page, p3_index(vaddr), 3)?;
        current_page = self.get_or_create_table(current_page, p2_index(vaddr), 2)?;
        Ok((current_page, p1_index(vaddr)))
    }
    fn walk_existing(&self, vpage: VirtPage) -> Option<(PhysPage, usize)> {
        let vaddr = vpage.addr().as_usize();
        let pmm = unsafe { &*self.pmm };
        let mut current_page = self.root;
        let root_frame_ptr = pmm.get_page(current_page)?;
        let root_frame = unsafe { &*(root_frame_ptr as *const PageTableFrame) };
        let p4_entry = root_frame.entry(p4_index(vaddr)).load(Ordering::Acquire);
        if p4_entry & 1 == 0 { return None; }
        current_page = unsafe { PhysAddr::new_unchecked(((p4_entry >> 12) & 0x000F_FFFF_FFFF) as usize) }.page();
        let p3_frame_ptr = pmm.get_page(current_page)?;
        let p3_frame = unsafe { &*(p3_frame_ptr as *const PageTableFrame) };
        let p3_entry = p3_frame.entry(p3_index(vaddr)).load(Ordering::Acquire);
        if p3_entry & 1 == 0 { return None; }
        current_page = unsafe { PhysAddr::new_unchecked(((p3_entry >> 12) & 0x000F_FFFF_FFFF) as usize) }.page();
        let p2_frame_ptr = pmm.get_page(current_page)?;
        let p2_frame = unsafe { &*(p2_frame_ptr as *const PageTableFrame) };
        let p2_entry = p2_frame.entry(p2_index(vaddr)).load(Ordering::Acquire);
        if p2_entry & 1 == 0 { return None; }
        current_page = unsafe { PhysAddr::new_unchecked(((p2_entry >> 12) & 0x000F_FFFF_FFFF) as usize) }.page();
        Some((current_page, p1_index(vaddr)))
    }
    pub fn map_page(&mut self, vpage: VirtPage, paddr: PhysAddr, flags: EntryFlags) -> Result<(), &'static str> {
        if !vpage.is_canonical() { return Err("virtual page not canonical"); }
        if !paddr.is_aligned() { return Err("physical address not page-aligned"); }
        if flags.is_writable_and_executable() { return Err("W^X violation: writable and executable"); }
        let (pt_page, p1_idx) = self.walk(vpage)?;
        let pmm = unsafe { &mut *self.pmm };
        let pt_frame_ptr = pmm.get_page_mut(pt_page).ok_or("page table frame not allocated")?;
        let pt_frame = unsafe { &mut *(pt_frame_ptr as *mut PageTableFrame) };
        let entry_val = pt_frame.entry(p1_idx).load(Ordering::Acquire);
        if entry_val & 1 != 0 { return Err("page already mapped"); }
        let mut entry_bits = ((paddr.as_usize() & 0x000F_FFFF_FFFF) << 12) | flags.bits() as usize;
        if flags.contains(EntryFlags::NO_EXECUTE) { entry_bits |= 1usize << 63; }
        pt_frame.entry(p1_idx).store(entry_bits as u64, Ordering::Release); Ok(())
    }
pub fn unmap_page(&mut self, vpage: VirtPage) -> Result<(), &'static str> {
        let Some((pt_page, p1_idx)) = self.walk_existing(vpage) else { return Err("page not mapped"); };
        let pmm = unsafe { &mut *self.pmm };
        let pt_frame_ptr = pmm.get_page_mut(pt_page).ok_or("page table frame not allocated")?;
        let pt_frame = unsafe { &mut *(pt_frame_ptr as *mut PageTableFrame) };
        if pt_frame.entry(p1_idx).load(Ordering::Acquire) & 1 == 0 { return Err("page not mapped"); }
        pt_frame.entry(p1_idx).store(0, Ordering::Release); Ok(())
    }
    pub fn translate(&self, vpage: VirtPage) -> Option<PhysAddr> {
        let (pt_page, p1_idx) = self.walk_existing(vpage)?;
        let pmm = unsafe { &*self.pmm };
        let pt_frame_ptr = pmm.get_page(pt_page)?;
        let pt_frame = unsafe { &*(pt_frame_ptr as *const PageTableFrame) };
        let entry = pt_frame.entry(p1_idx).load(Ordering::Acquire);
        if entry & 1 == 0 { return None; }
        Some(unsafe { PhysAddr::new_unchecked(((entry >> 12) & 0x000F_FFFF_FFFF) as usize) })
    }
    pub fn is_user(&self, vpage: VirtPage) -> bool {
        let Some((pt_page, p1_idx)) = self.walk_existing(vpage) else { return false; };
        let pmm = unsafe { &*self.pmm };
        let Some(pt_frame_ptr) = pmm.get_page(pt_page) else { return false; };
        let pt_frame = unsafe { &*(pt_frame_ptr as *const PageTableFrame) };
        (pt_frame.entry(p1_idx).load(Ordering::Acquire) & 4) != 0
    }
    pub fn switch(&mut self) { self.switched = true; crate::memory::page_table::tlb::flush_all(); }
    fn free_all_frames(&mut self, page: PhysPage, level: usize) {
        let pmm = unsafe { &mut *self.pmm };
        let frame_ptr = pmm.get_page(page).ok_or("page table frame not allocated").ok();
        if let Some(frame_ptr) = frame_ptr {
            let frame = unsafe { &*(frame_ptr as *const PageTableFrame) };
            if level > 1 {
                for entry in &frame.entries {
                    let bits = entry.load(Ordering::Acquire);
                    if bits & 1 != 0 {
                        let child_phys = unsafe { PhysAddr::new_unchecked(((bits >> 12) & 0x000F_FFFF_FFFF) as usize) };
                        self.free_all_frames(child_phys.page(), level - 1);
                    }
                }
            }
        }
        pmm.free_page(page);
    }
}
impl Drop for PageTable { fn drop(&mut self) { self.free_all_frames(self.root, PAGE_TABLE_LEVELS); } }
pub mod tlb {
    use crate::memory::VirtAddr; use core::sync::atomic::{AtomicU64, Ordering};
    static FLUSH_PAGE_COUNT: AtomicU64 = AtomicU64::new(0);
    static FLUSH_RANGE_COUNT: AtomicU64 = AtomicU64::new(0);
    static FLUSH_ALL_COUNT: AtomicU64 = AtomicU64::new(0);
    pub fn flush_page(_vaddr: VirtAddr) { FLUSH_PAGE_COUNT.fetch_add(1, Ordering::Relaxed); #[cfg(target_os = "none")] unsafe { core::arch::asm!("invlpg [{}]", in(reg) _vaddr.as_usize(), options(nostack, preserves_flags)); } }
    pub fn flush_range(start: VirtAddr, end: VirtAddr) { FLUSH_RANGE_COUNT.fetch_add(1, Ordering::Relaxed); let mut addr = start.as_usize(); while addr < end.as_usize() { flush_page(VirtAddr::new(addr)); addr += 4096; } }
    pub fn flush_all() { FLUSH_ALL_COUNT.fetch_add(1, Ordering::Relaxed); #[cfg(target_os = "none")] unsafe { let cr3: u64; core::arch::asm!("mov {}, cr3", out(reg) cr3, options(nostack, preserves_flags)); core::arch::asm!("mov cr3, {}", in(reg) cr3, options(nostack, preserves_flags)); } }
    pub fn stats() -> (u64, u64, u64) { (FLUSH_PAGE_COUNT.load(Ordering::Relaxed), FLUSH_RANGE_COUNT.load(Ordering::Relaxed), FLUSH_ALL_COUNT.load(Ordering::Relaxed)) }
    pub fn reset_stats() { FLUSH_PAGE_COUNT.store(0, Ordering::Relaxed); FLUSH_RANGE_COUNT.store(0, Ordering::Relaxed); FLUSH_ALL_COUNT.store(0, Ordering::Relaxed); }
}

#[cfg(test)] mod tests {
    use super::*; use crate::memory::{PhysicalMemoryManager, PhysAddr, VirtPage, PAGE_SIZE}; use crate::arch::x86_64::paging::EntryFlags;
    #[test] fn page_table_basic() { let mut pmm = PhysicalMemoryManager::new(100); let mut pt = PageTable::new(&mut pmm).unwrap(); assert!(pt.root().as_usize() < 100); }
    #[test] fn map_translate_unmap() { let mut pmm = PhysicalMemoryManager::new(100); let mut pt = PageTable::new(&mut pmm).unwrap(); let vpage = VirtPage(1); let paddr = PhysAddr::new(0x1000).unwrap(); let flags = EntryFlags::PRESENT | EntryFlags::WRITABLE | EntryFlags::USER | EntryFlags::NO_EXECUTE; pt.map_page(vpage, paddr, flags).unwrap(); let translated = pt.translate(vpage).unwrap(); assert_eq!(translated.as_usize(), 0x1000); assert!(pt.is_user(vpage)); pt.unmap_page(vpage).unwrap(); assert!(pt.translate(vpage).is_none()); }
    #[test] fn map_non_canonical_fails() { let mut pmm = PhysicalMemoryManager::new(100); let mut pt = PageTable::new(&mut pmm).unwrap(); let non_canonical = VirtPage(0x0000_8000_0000_0000 / PAGE_SIZE); let paddr = PhysAddr::new(0x1000).unwrap(); let flags = EntryFlags::PRESENT; assert_eq!(pt.map_page(non_canonical, paddr, flags), Err("virtual page not canonical")); }
    #[test] fn map_wx_fails() { let mut pmm = PhysicalMemoryManager::new(100); let mut pt = PageTable::new(&mut pmm).unwrap(); let vpage = VirtPage(1); let paddr = PhysAddr::new(0x1000).unwrap(); let flags = EntryFlags::PRESENT | EntryFlags::WRITABLE; assert_eq!(pt.map_page(vpage, paddr, flags), Err("W^X violation: writable and executable")); }
    #[test] fn map_misaligned_fails() { let mut pmm = PhysicalMemoryManager::new(100); let mut pt = PageTable::new(&mut pmm).unwrap(); let vpage = VirtPage(1); let paddr = PhysAddr::new(0x1001).unwrap(); let flags = EntryFlags::PRESENT; assert_eq!(pt.map_page(vpage, paddr, flags), Err("physical address not page-aligned")); }
    #[test] fn tlb_flush_tracking() { tlb::reset_stats(); let vaddr = VirtAddr::new(0x1000); tlb::flush_page(vaddr); tlb::flush_page(vaddr); let (page, range, all) = tlb::stats(); assert_eq!(page, 2); assert_eq!(range, 0); assert_eq!(all, 0); }
    #[test] fn tlb_flush_range() { tlb::reset_stats(); let start = VirtAddr::new(0x1000); let end = VirtAddr::new(0x3000); tlb::flush_range(start, end); let (page, range, all) = tlb::stats(); assert_eq!(range, 1); assert_eq!(page, 2); }
    #[test] fn tlb_flush_all() { tlb::reset_stats(); tlb::flush_all(); let (page, range, all) = tlb::stats(); assert_eq!(all, 1); }
    #[test] fn multiple_pages() { let mut pmm = PhysicalMemoryManager::new(200); let mut pt = PageTable::new(&mut pmm).unwrap(); for i in 1..10 { let vpage = VirtPage(i); let paddr = PhysAddr::new((i as usize) * PAGE_SIZE).unwrap(); let flags = EntryFlags::PRESENT; pt.map_page(vpage, paddr, flags).unwrap(); } for i in 1..10 { let vpage = VirtPage(i); let translated = pt.translate(vpage).unwrap(); assert_eq!(translated.as_usize(), (i as usize) * PAGE_SIZE); } }
    #[test] fn page_table_switch() { let mut pmm = PhysicalMemoryManager::new(100); let mut pt = PageTable::new(&mut pmm).unwrap(); tlb::reset_stats(); pt.switch(); let (_, _, all) = tlb::stats(); assert_eq!(all, 1); }
}
