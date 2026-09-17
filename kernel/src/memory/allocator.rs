use std::sync::Mutex;
#[derive(Debug, Default, Clone, Copy)]
pub struct AllocStats {
    pub allocs: u64,
    pub frees: u64,
    pub bytes: u64,
}
pub struct KernelAllocator {
    stats: Mutex<AllocStats>,
}
impl KernelAllocator {
    pub const fn new() -> Self {
        Self {
            stats: Mutex::new(AllocStats {
                allocs: 0,
                frees: 0,
                bytes: 0,
            }),
        }
    }
    pub fn record_alloc(&self, n: usize) {
        if let Ok(mut s) = self.stats.lock() {
            s.allocs += 1;
            s.bytes = s.bytes.saturating_add(n as u64);
        }
    }
    pub fn record_free(&self, n: usize) {
        if let Ok(mut s) = self.stats.lock() {
            s.frees += 1;
            s.bytes = s.bytes.saturating_sub(n as u64);
        }
    }
    pub fn stats(&self) -> AllocStats {
        self.stats.lock().map(|s| *s).unwrap_or_default()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stats() {
        let a = KernelAllocator::new();
        a.record_alloc(10);
        a.record_free(4);
        assert_eq!(a.stats().bytes, 6);
    }
}
