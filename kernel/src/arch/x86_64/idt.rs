//! IDT stub: 256 entries.
pub const ENTRIES: usize = 256;
#[derive(Debug)]
pub struct Idt {
    pub entries: [u64; ENTRIES],
}
impl Default for Idt {
    fn default() -> Self {
        Self::new()
    }
}
impl Idt {
    pub fn new() -> Self {
        Self {
            entries: [0; ENTRIES],
        }
    }
    pub fn load(&self) {}
}
