use core::sync::atomic::{AtomicBool, Ordering};

#[derive(Debug, Clone, Copy)]
pub struct CpuInfo {
    pub vendor: &'static str,
    pub cores: u32,
}
// DEBT(AUDIT-2026-09-13): host model, single CPU. Replaces `static mut` (data-race/UB).
// No `unsafe` needed: AtomicBool is data-race-free. Ordering Relaxed suffices for
// a boot-once flag read after init; Acquire/Release used for clarity.
static INIT: AtomicBool = AtomicBool::new(false);
pub fn init() {
    INIT.store(true, Ordering::Release);
}
pub fn is_init() -> bool {
    INIT.load(Ordering::Acquire)
}
pub fn info() -> CpuInfo {
    CpuInfo {
        vendor: "GenuineIntel(emulated)",
        cores: 1,
    }
}
