//! Paging constants for x86_64 4KiB pages.
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
