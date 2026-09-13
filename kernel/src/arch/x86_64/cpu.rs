#[derive(Debug, Clone, Copy)] pub struct CpuInfo { pub vendor: &'static str, pub cores: u32 }
static mut INIT: bool = false;
pub fn init() {
    // SAFETY: single-threaded boot, only place that sets INIT.
    unsafe { INIT = true; }
}
pub fn is_init() -> bool { unsafe { INIT } }
pub fn info() -> CpuInfo { CpuInfo { vendor: "GenuineIntel(emulated)", cores: 1 } }
