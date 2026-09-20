use std::collections::BTreeMap;

/// x86_64 canonical address limits.
/// 48-bit canonical: bits 63:47 must be all 0 or all 1.
const CANONICAL_48_HIGH: usize = 0x0000_7FFF_FFFF_FFFF;
const CANONICAL_48_NEG_LOW: usize = 0xFFFF_8000_0000_0000;

/// 57-bit canonical (LA57): bits 63:56 must be all 0 or all 1.
const CANONICAL_57_HIGH: usize = 0x00FF_FFFF_FFFF_FFFF;
const CANONICAL_57_NEG_LOW: usize = 0xFF00_0000_0000_0000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtAddr(pub usize);

impl VirtAddr {
    /// Create a new virtual address (no validation - use `new_canonical` for validated).
    pub const fn new(addr: usize) -> Self {
        Self(addr)
    }

    /// Create a new virtual address with canonical validation (48-bit).
    pub fn new_canonical(addr: usize) -> Result<Self, &'static str> {
        if Self::is_canonical_48(addr) {
            Ok(Self(addr))
        } else {
            Err("virtual address is not canonical (48-bit)")
        }
    }

    /// Create a new virtual address with LA57 canonical validation (57-bit).
    pub fn new_canonical_57(addr: usize) -> Result<Self, &'static str> {
        if Self::is_canonical_57(addr) {
            Ok(Self(addr))
        } else {
            Err("virtual address is not canonical (57-bit)")
        }
    }

    /// Get the raw address value.
    pub const fn as_usize(self) -> usize {
        self.0
    }

    /// Check if address is canonical for standard 48-bit x86_64.
    pub const fn is_canonical_48(addr: usize) -> bool {
        (addr <= CANONICAL_48_HIGH) || (addr >= CANONICAL_48_NEG_LOW)
    }

    /// Check if address is canonical for 57-bit LA57 x86_64.
    pub const fn is_canonical_57(addr: usize) -> bool {
        (addr <= CANONICAL_57_HIGH) || (addr >= CANONICAL_57_NEG_LOW)
    }

    /// Check if this address is canonical (using default 48-bit).
    pub const fn is_canonical(self) -> bool {
        Self::is_canonical_48(self.0)
    }

    /// Get the virtual page containing this address.
    pub const fn page(self) -> VirtPage {
        VirtPage(self.0 / crate::memory::PAGE_SIZE)
    }

    /// Offset from the start of the containing page.
    pub const fn page_offset(self) -> usize {
        self.0 % crate::memory::PAGE_SIZE
    }

    /// Checked addition.
    pub fn checked_add(self, rhs: usize) -> Option<Self> {
        self.0.checked_add(rhs).map(Self)
    }

    /// Checked subtraction.
    pub fn checked_sub(self, rhs: usize) -> Option<Self> {
        self.0.checked_sub(rhs).map(Self)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtPage(pub usize);

impl VirtPage {
    /// Create a new virtual page number.
    pub const fn new(page: usize) -> Self {
        Self(page)
    }

    /// Create from a virtual address (rounding down to page boundary).
    pub fn from_addr(addr: VirtAddr) -> Self {
        addr.page()
    }

    /// Get the raw page number.
    pub const fn as_usize(self) -> usize {
        self.0
    }

    /// Convert to the base virtual address of this page.
    pub fn addr(self) -> VirtAddr {
        VirtAddr::new(self.0 * crate::memory::PAGE_SIZE)
    }

    /// Check if this page is canonical (using default 48-bit).
    pub const fn is_canonical(self) -> bool {
        VirtAddr::is_canonical_48(self.0 * crate::memory::PAGE_SIZE)
    }

    /// Check if this page is canonical for 57-bit LA57.
    pub const fn is_canonical_57(self) -> bool {
        VirtAddr::is_canonical_57(self.0 * crate::memory::PAGE_SIZE)
    }
}
pub struct VirtualMemoryManager {
    map: BTreeMap<usize, (usize, MapFlags)>,
}
impl VirtualMemoryManager {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
    /// Maps a page. Enforces W^X (§12): WRITE+EXEC denied. Phys must be page-aligned.
    /// Also validates virtual address is canonical.
    pub fn map_page(&mut self, v: VirtPage, p: usize, f: MapFlags) -> Result<(), &'static str> {
        if self.map.contains_key(&v.0) {
            return Err("already mapped");
        }
        if !v.is_canonical() {
            return Err("virtual page not canonical");
        }
        if !p.is_multiple_of(crate::core::config::PAGE_SIZE) {
            return Err("phys misaligned");
        }
        if (f.0 & MapFlags::WRITE.0 != 0) && (f.0 & MapFlags::EXEC.0 != 0) {
            return Err("RWX denied: W^X");
        }
        self.map.insert(v.0, (p, f));
        Ok(())
    }
    pub fn unmap_page(&mut self, v: VirtPage) -> Result<(), &'static str> {
        self.map.remove(&v.0).map(|_| ()).ok_or("not mapped")
    }
    pub fn translate(&self, v: VirtPage) -> Option<usize> {
        self.map.get(&v.0).map(|(p, _)| *p)
    }
    pub fn is_user(&self, v: VirtPage) -> bool {
        self.map
            .get(&v.0)
            .map(|(_, f)| f.0 & MapFlags::USER.0 != 0)
            .unwrap_or(false)
    }
}
impl Default for VirtualMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn map() {
        let mut v = VirtualMemoryManager::new();
        v.map_page(VirtPage(1), 0x1000, MapFlags::READ).unwrap();
        assert_eq!(v.translate(VirtPage(1)), Some(0x1000));
        v.unmap_page(VirtPage(1)).unwrap();
        assert!(v.translate(VirtPage(1)).is_none());
    }

    #[test]
    fn virtaddr_canonical_48() {
        // Valid low canonical
        assert!(VirtAddr::is_canonical_48(0x0000_0000_0000_0000));
        assert!(VirtAddr::is_canonical_48(0x0000_7FFF_FFFF_FFFF));
        // Valid high canonical (negative)
        assert!(VirtAddr::is_canonical_48(0xFFFF_8000_0000_0000));
        assert!(VirtAddr::is_canonical_48(0xFFFF_FFFF_FFFF_FFFF));
        // Invalid - hole in the middle
        assert!(!VirtAddr::is_canonical_48(0x0000_8000_0000_0000));
        assert!(!VirtAddr::is_canonical_48(0x7FFF_8000_0000_0000));
        assert!(!VirtAddr::is_canonical_48(0x8000_0000_0000_0000));
        assert!(!VirtAddr::is_canonical_48(0xFFFF_7FFF_FFFF_FFFF));
    }

    #[test]
    fn virtaddr_canonical_57() {
        // Valid low canonical
        assert!(VirtAddr::is_canonical_57(0x0000_0000_0000_0000));
        assert!(VirtAddr::is_canonical_57(0x00FF_FFFF_FFFF_FFFF));
        // Valid high canonical (negative)
        assert!(VirtAddr::is_canonical_57(0xFF00_0000_0000_0000));
        assert!(VirtAddr::is_canonical_57(0xFFFF_FFFF_FFFF_FFFF));
        // Invalid
        assert!(!VirtAddr::is_canonical_57(0x0100_0000_0000_0000));
        assert!(!VirtAddr::is_canonical_57(0xFEFF_FFFF_FFFF_FFFF));
    }

    #[test]
    fn virtpage_canonical() {
        let page = VirtPage(0x0000_7FFF_FFFF_FFFF / 4096);
        assert!(page.is_canonical());
        let page = VirtPage(0xFFFF_8000_0000_0000 / 4096);
        assert!(page.is_canonical());
        let page = VirtPage(0x0000_8000_0000_0000 / 4096);
        assert!(!page.is_canonical());
    }

    #[test]
    fn virtaddr_checked_arithmetic() {
        let a = VirtAddr::new(0x1000);
        assert_eq!(a.checked_add(0x1000).unwrap().as_usize(), 0x2000);
        assert_eq!(a.checked_sub(0x1000).unwrap().as_usize(), 0);
        assert!(a.checked_sub(0x2000).is_none());
    }

    #[test]
    fn map_non_canonical_fails() {
        let mut v = VirtualMemoryManager::new();
        // Non-canonical page (in the hole)
        let non_canonical = VirtPage(0x0000_8000_0000_0000 / 4096);
        assert!(!non_canonical.is_canonical());
        let err = v.map_page(non_canonical, 0x1000, MapFlags::READ);
        assert_eq!(err, Err("virtual page not canonical"));
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MapFlags(pub u8);
impl MapFlags {
    pub const READ: Self = Self(1);
    pub const WRITE: Self = Self(2);
    pub const EXEC: Self = Self(4);
    pub const USER: Self = Self(8);
    pub const GLOBAL: Self = Self(16);
    pub const CACHE_DISABLED: Self = Self(32); // PCD
    pub const WRITE_THROUGH: Self = Self(64); // PWT
}
