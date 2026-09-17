//! x86_64 backend (UEFI first target).
pub mod boot;
pub mod context;
pub mod cpu;
pub mod gdt;
pub mod idt;
pub mod interrupts;
pub mod paging;
pub struct X86_64;
impl crate::arch::ArchHal for X86_64 {
    fn name() -> &'static str {
        "x86_64/UEFI"
    }
    fn init_cpu() {
        cpu::init();
        interrupts::init();
    }
    fn halt() -> ! {
        loop {
            core::hint::spin_loop();
        }
    }
}
