//! GDT stub.
#[derive(Debug, Default)]
pub struct Gdt;
impl Gdt {
    pub fn new() -> Self {
        Gdt
    }
    pub fn load(&self) {}
}
