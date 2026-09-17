use core::sync::atomic::{AtomicBool, Ordering};

// DEBT(AUDIT-2026-09-13): host stub. Atomic flag replaces `static mut`.
// Contract (§15): handler must stay minimal — no block, no complex I/O,
// no arbitrary alloc, no heavy logic. Deferred work goes to driver.
static INIT: AtomicBool = AtomicBool::new(false);
static ENABLED: AtomicBool = AtomicBool::new(false);
pub fn init() {
    INIT.store(true, Ordering::Release);
}
pub fn is_init() -> bool {
    INIT.load(Ordering::Acquire)
}
pub fn enable() {
    ENABLED.store(true, Ordering::Release);
}
pub fn disable() {
    ENABLED.store(false, Ordering::Release);
}
pub fn is_enabled() -> bool {
    ENABLED.load(Ordering::Acquire)
}
