//! Central kernel configuration — no magic constants scattered.
//!
//! Host-testable. All values are explicit so `no_std` bring-up can reuse them.

/// Page size in bytes (x86_64 4KiB baseline).
pub const PAGE_SIZE: usize = 4096;
/// Maximum DMA region (64 MiB) — see `dma::DmaRegion`.
pub const DMA_MAX_BYTES: usize = 64 * 1024 * 1024;
/// Maximum IPC payload per message (64 KiB) — DoS bound.
pub const IPC_MAX_PAYLOAD: usize = 64 * 1024;
/// Maximum QPU job bytes (1 MiB) — enforced by runtime, checked here as bound.
pub const QPU_MAX_JOB_BYTES: usize = 1024 * 1024;
/// Kernel link base for x86_64 UEFI placeholder.
pub const KERNEL_LINK_BASE: usize = 0x100000;

#[derive(Debug, Clone, Copy)]
pub struct KernelConfig {
    pub page_size: usize,
    pub dma_max_bytes: usize,
    pub ipc_max_payload: usize,
    pub qpu_max_job_bytes: usize,
}

impl Default for KernelConfig {
    fn default() -> Self {
        Self {
            page_size: PAGE_SIZE,
            dma_max_bytes: DMA_MAX_BYTES,
            ipc_max_payload: IPC_MAX_PAYLOAD,
            qpu_max_job_bytes: QPU_MAX_JOB_BYTES,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn defaults_sane() {
        let c = KernelConfig::default();
        assert_eq!(c.page_size, 4096);
        assert!(c.dma_max_bytes >= 4096);
    }
}