use qeos_kernel::{
    boot::BootSequence,
    dma::DmaRegion,
    elf,
    fs::inode::InodeKind,
    fs::{OpenFlags, Vfs},
    ipc::{Channel, Message},
    memory::{MapFlags, PhysicalMemoryManager, VirtPage, VirtualMemoryManager},
    process::thread::{Priority, Thread, Tid},
    qpu::QuantumDevice,
    qpu::UnsupportedDevice,
    ring::{OverflowPolicy, SpscRing},
    scheduler::{SchedClass, Scheduler},
    security::{CapSet, Capability},
    syscall::dispatcher::dispatch_syscall,
};

#[test]
fn boot_sequence_is_deterministic() {
    let s = BootSequence::new().stages();
    assert_eq!(s.len(), 11);
}

#[test]
fn full_stack_smoke() {
    // MEM
    let mut pm = PhysicalMemoryManager::new(8);
    let pg = pm.allocate_page().unwrap();
    let mut vm = VirtualMemoryManager::new();
    vm.map_page(VirtPage(0), pg.0 * 4096, MapFlags::READ)
        .unwrap();
    // SCHED
    let mut sched = Scheduler::new();
    sched.spawn(Tid(1), Priority::Normal);
    assert!(sched.schedule().is_some());
    let _ = SchedClass::Normal;
    let mut t = Thread::new(1, Priority::Normal);
    t.sleep(10);
    t.wake();
    // IPC
    let mut ch = Channel::new(4);
    ch.send(Message::new(1, 2, 0, vec![9])).unwrap();
    assert_eq!(ch.recv().unwrap().payload, vec![9]);
    // SYSCALL
    assert!(dispatch_syscall(10, 0, 0, 0).is_ok());
    // ELF
    assert!(elf::load(&[0u8; 8]).is_err());
    // RING + DMA
    let mut r = SpscRing::<u32, 4>::new(OverflowPolicy::DropNew);
    r.push(1).unwrap();
    assert!(DmaRegion::new(0x2000, 4096).is_ok());
    // SECURITY + VFS
    let mut caps = CapSet::new();
    caps.grant(Capability::DeviceRead);
    assert!(caps.has(Capability::DeviceRead));
    let mut vfs = Vfs::new();
    let id = vfs.create(InodeKind::File);
    let mut h = vfs.open(id, OpenFlags::WRITE).unwrap();
    vfs.write(&mut h, b"qeos").unwrap();
    // QPU must be unsupported without hardware
    let mut q = UnsupportedDevice;
    assert!(q.initialize().is_err());
}
