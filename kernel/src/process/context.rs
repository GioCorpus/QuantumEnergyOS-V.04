#[derive(Debug, Default, Clone, Copy)]
pub struct ThreadContext {
    pub entry: u64,
    pub stack: u64,
}
impl ThreadContext {
    pub fn new(entry: u64, stack: u64) -> Self {
        Self { entry, stack }
    }
}
