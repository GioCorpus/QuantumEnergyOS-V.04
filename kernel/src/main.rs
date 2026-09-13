//! QEOS Kernel V1 — kernel entry + host boot simulator.
//! On bare metal this would be `#![no_std] #![no_main]` with UEFI entry.
//! For host development (`cargo run`) it simulates the deterministic boot
//! sequence with explicit logging.

use qeos_kernel::{boot::BootSequence, logging::{Logger, Level}};

fn main() {
    let logger = Logger::new(Level::Info);
    let seq = BootSequence::new();
    for stage in seq.stages() {
        logger.log(Level::Info, stage.log_line());
    }
    println!("[INIT] userspace initialization -> /qeos/init (simulated)");
    println!("[DONE] QEOS Kernel V1 boot simulation complete");
}
