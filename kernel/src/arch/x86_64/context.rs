//! CPU context for context switching.
#[derive(Debug, Default, Clone, Copy)]
#[repr(C)]
pub struct CpuContext { pub rip: u64, pub rsp: u64, pub rflags: u64, pub regs: [u64; 15] }
impl CpuContext {
    pub fn new(entry: u64, stack: u64) -> Self {
        Self { rip: entry, rsp: stack, rflags: 0x202, regs: [0; 15] }
    }
}
