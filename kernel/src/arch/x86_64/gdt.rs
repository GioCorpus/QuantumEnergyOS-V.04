//! GDT stub.
#[derive(Debug)] pub struct Gdt;
impl Gdt { pub fn new() -> Self { Gdt } pub fn load(&self) {} }
