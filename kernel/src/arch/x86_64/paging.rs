//! Paging constants for x86_64 4KiB pages.
//!
//! Page Table Entry (PTE) with full x86_64 flag support including NX bit.

pub const PAGE_SIZE: usize = 4096;
pub const ENTRIES_PER_TABLE: usize = 512;

pub fn p4_index(v: usize) -> usize {
    (v >> 39) & 0x1ff
}
pub fn p3_index(v: usize) -> usize {
    (v >> 30) & 0x1ff
}
pub fn p2_index(v: usize) -> usize {
    (v >> 21) & 0x1ff
}
pub fn p1_index(v: usize) -> usize {
    (v >> 12) & 0x1ff
}

/// x86_64 Page Table Entry flags (bits 0-11, 52-62, 63).
/// Bit 63 is the NX (No-Execute) bit when NXE=1 in EFER.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct EntryFlags(pub u64);

impl EntryFlags {
    /// Present (P) - bit 0
    pub const PRESENT: Self = Self(1 << 0);
    /// Writable (RW) - bit 1
    pub const WRITABLE: Self = Self(1 << 1);
    /// User/Supervisor (US) - bit 2
    pub const USER: Self = Self(1 << 2);
    /// Page-level Write-Through (PWT) - bit 3
    pub const WRITE_THROUGH: Self = Self(1 << 3);
    /// Page-level Cache Disable (PCD) - bit 4
    pub const CACHE_DISABLED: Self = Self(1 << 4);
    /// Accessed (A) - bit 5
    pub const ACCESSED: Self = Self(1 << 5);
    /// Dirty (D) - bit 6 (only for leaf entries)
    pub const DIRTY: Self = Self(1 << 6);
    /// Page Attribute Table (PAT) - bit 7 (for huge pages) / bit 12 (for 4K)
    pub const PAT: Self = Self(1 << 7);
    /// Global (G) - bit 8
    pub const GLOBAL: Self = Self(1 << 8);
    /// No-Execute (NX) - bit 63 (requires EFER.NXE=1)
    pub const NO_EXECUTE: Self = Self(1 << 63);

    /// Empty flags
    pub const EMPTY: Self = Self(0);

    /// Create from raw bits.
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    /// Get raw bits.
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Check if flag is set.
    pub const fn contains(self, flag: Self) -> bool {
        (self.0 & flag.0) != 0
    }

    /// Insert a flag.
    pub fn insert(&mut self, flag: Self) {
        self.0 |= flag.0;
    }

    /// Remove a flag.
    pub fn remove(&mut self, flag: Self) {
        self.0 &= !flag.0;
    }

    /// Combine flags.
    pub const fn union(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }

    /// Check if writable and executable (W^X violation).
    pub const fn is_writable_and_executable(self) -> bool {
        self.contains(Self::WRITABLE) && !self.contains(Self::NO_EXECUTE)
    }
}

impl core::ops::BitOr for EntryFlags {
    type Output = Self;
    fn bitor(self, other: Self) -> Self {
        Self(self.0 | other.0)
    }
}

impl core::ops::BitOrAssign for EntryFlags {
    fn bitor_assign(&mut self, other: Self) {
        self.0 |= other.0;
    }
}

impl core::ops::BitAnd for EntryFlags {
    type Output = Self;
    fn bitand(self, other: Self) -> Self {
        Self(self.0 & other.0)
    }
}

/// x86_64 Page Table Entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PageTableEntry(pub u64);

impl PageTableEntry {
    /// Create an empty (non-present) entry.
    pub const fn new() -> Self {
        Self(0)
    }

    /// Create from raw bits.
    pub const fn from_bits(bits: u64) -> Self {
        Self(bits)
    }

    /// Get raw bits.
    pub const fn bits(self) -> u64 {
        self.0
    }

    /// Get the physical address from this entry (bits 12-51).
    pub const fn phys_addr(self) -> u64 {
        (self.0 >> 12) & 0x000F_FFFF_FFFF
    }

    /// Set the physical address (bits 12-51).
    pub fn set_phys_addr(&mut self, addr: u64) {
        self.0 = (self.0 & 0xFFF0_0000_0000_0FFF) | ((addr & 0x000F_FFFF_FFFF) << 12);
    }

    /// Get flags.
    pub const fn flags(self) -> EntryFlags {
        EntryFlags(self.0 & 0xFFF)
    }

    /// Set flags.
    pub fn set_flags(&mut self, flags: EntryFlags) {
        self.0 = (self.0 & !0xFFF) | (flags.bits() & 0xFFF);
    }

    /// Check if entry is present.
    pub const fn is_present(self) -> bool {
        (self.0 & 1) != 0
    }

    /// Check if entry is writable.
    pub const fn is_writable(self) -> bool {
        (self.0 & 2) != 0
    }

    /// Check if entry is user-accessible.
    pub const fn is_user(self) -> bool {
        (self.0 & 4) != 0
    }

    /// Check if NX bit is set.
    pub const fn is_nx(self) -> bool {
        (self.0 & (1 << 63)) != 0
    }

    /// Set present bit.
    pub fn set_present(&mut self, present: bool) {
        if present {
            self.0 |= 1;
        } else {
            self.0 &= !1;
        }
    }

    /// Set writable bit.
    pub fn set_writable(&mut self, writable: bool) {
        if writable {
            self.0 |= 2;
        } else {
            self.0 &= !2;
        }
    }

    /// Set user bit.
    pub fn set_user(&mut self, user: bool) {
        if user {
            self.0 |= 4;
        } else {
            self.0 &= !4;
        }
    }

    /// Set NX bit.
    pub fn set_nx(&mut self, nx: bool) {
        if nx {
            self.0 |= 1 << 63;
        } else {
            self.0 &= !(1 << 63);
        }
    }
}

impl Default for PageTableEntry {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pte_basic() {
        let mut pte = PageTableEntry::new();
        assert!(!pte.is_present());
        pte.set_present(true);
        assert!(pte.is_present());
    }

    #[test]
    fn pte_flags() {
        let mut pte = PageTableEntry::new();
        pte.set_present(true);
        pte.set_writable(true);
        pte.set_user(true);
        assert!(pte.is_present());
        assert!(pte.is_writable());
        assert!(pte.is_user());
    }

    #[test]
    fn pte_nx() {
        let mut pte = PageTableEntry::new();
        pte.set_present(true);
        pte.set_nx(true);
        assert!(pte.is_nx());
        pte.set_nx(false);
        assert!(!pte.is_nx());
    }

    #[test]
    fn pte_phys_addr() {
        let mut pte = PageTableEntry::new();
        pte.set_phys_addr(0x1000_0000);
        assert_eq!(pte.phys_addr(), 0x1000_0000);
    }

    #[test]
    fn entry_flags() {
        let flags = EntryFlags::PRESENT | EntryFlags::WRITABLE | EntryFlags::USER;
        assert!(flags.contains(EntryFlags::PRESENT));
        assert!(flags.contains(EntryFlags::WRITABLE));
        assert!(flags.contains(EntryFlags::USER));
        assert!(!flags.contains(EntryFlags::NO_EXECUTE));
    }

    #[test]
    fn entry_flags_nx() {
        let flags = EntryFlags::PRESENT | EntryFlags::WRITABLE;
        assert!(!flags.contains(EntryFlags::NO_EXECUTE));
        let flags = flags | EntryFlags::NO_EXECUTE;
        assert!(flags.contains(EntryFlags::NO_EXECUTE));
    }

    #[test]
    fn w_x_detection() {
        let flags = EntryFlags::PRESENT | EntryFlags::WRITABLE;
        assert!(flags.is_writable_and_executable());
        let flags = flags | EntryFlags::NO_EXECUTE;
        assert!(!flags.is_writable_and_executable());
    }
}
