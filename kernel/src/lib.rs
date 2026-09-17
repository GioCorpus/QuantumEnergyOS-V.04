//! QEOS Kernel V1 — library root.
//!
//! Host-testable kernel logic. Bare-metal entry lives in `main.rs` / `arch`.
//! All subsystems are pure-Rust, `no_std`-compatible in design (no heap
//! required except via explicit allocator APIs), but testable on std hosts.
//!
//! Priority order: CORRECTNESS > STABILITY > SECURITY > MODULARITY >
//! PERFORMANCE > SCIENTIFIC EXTENSIBILITY.

#![forbid(unsafe_op_in_unsafe_fn)]

pub mod arch;
pub mod boot;
pub mod core;
pub mod device;
pub mod dma;
pub mod driver;
pub mod elf;
pub mod fs;
pub mod hal;
pub mod ipc;
pub mod logging;
pub mod memory;
pub mod panic;
pub mod process;
pub mod qpu;
pub mod ring;
pub mod scheduler;
pub mod security;
pub mod sync;
pub mod syscall;
pub mod telemetry;
pub mod time;
pub mod tracing;

pub const VERSION: &str = "1.0.0";
pub const NAME: &str = "QEOS Kernel V1";
