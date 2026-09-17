//! IDT stub: 256 entries.
pub const ENTRIES: usize = 256;
pub struct Idt {
    pub entries: [u64; ENTRIES],
}
impl Idt {
    pub fn new() -> Self {
        Self {
            entries: [0; ENTRIES],
        }
    }
    pub fn load(&self) {}
}
