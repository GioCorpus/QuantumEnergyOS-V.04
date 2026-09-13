//! Deterministic boot sequence.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Stage { Entry, Cpu, PhysMem, VirtMem, Irq, Sched, Devices, Vfs, Ipc, Services, Userspace }
impl Stage {
    pub fn log_line(self) -> &'static str {
        match self {
            Stage::Entry => "[BOOT] QEOS Kernel V1",
            Stage::Cpu => "[CPU ] x86_64 initialized",
            Stage::PhysMem => "[MEM ] physical memory initialized",
            Stage::VirtMem => "[MEM ] virtual memory initialized",
            Stage::Irq => "[IRQ ] interrupt subsystem initialized",
            Stage::Sched => "[SCH ] scheduler initialized",
            Stage::Devices => "[PCI ] device enumeration complete",
            Stage::Vfs => "[VFS ] filesystem initialized",
            Stage::Ipc => "[IPC ] IPC subsystem initialized",
            Stage::Services => "[SVC ] system services ready",
            Stage::Userspace => "[INIT] userspace initialization",
        }
    }
}
pub struct BootSequence;
impl BootSequence {
    pub fn new() -> Self { Self }
    pub fn stages(&self) -> Vec<Stage> {
        vec![Stage::Entry, Stage::Cpu, Stage::PhysMem, Stage::VirtMem, Stage::Irq, Stage::Sched, Stage::Devices, Stage::Vfs, Stage::Ipc, Stage::Services, Stage::Userspace]
    }
}
#[cfg(test)] mod tests { use super::*; #[test] fn order() { let s = BootSequence::new().stages(); assert_eq!(s[0], Stage::Entry); assert_eq!(s.len(), 11); } }
