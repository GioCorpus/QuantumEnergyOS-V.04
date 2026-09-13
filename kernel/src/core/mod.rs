//! Kernel core: config + lifecycle + health (§7, §69, §77).
pub mod config;
pub mod health;
pub mod state;
pub use config::{KernelConfig, PAGE_SIZE, DMA_MAX_BYTES, IPC_MAX_PAYLOAD, QPU_MAX_JOB_BYTES};
pub use health::{KernelHealth, SubsystemHealth};
pub use state::{KernelPhase, KernelState};