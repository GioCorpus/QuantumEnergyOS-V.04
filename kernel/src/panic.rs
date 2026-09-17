//! Deterministic panic: capture diagnostics, halt safely.
pub fn panic_info(file: &str, line: u32, msg: &str) -> ! {
    eprintln!("[PANIC] {}:{} {}", file, line, msg);
    loop {
        core::hint::spin_loop();
    }
}
