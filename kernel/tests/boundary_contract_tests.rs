//! QEOS V.04 — Kernel ↔ User Space Contract & Boundary Verification Tests
//!
//! Milestone 5.1 Acceptance Tests:
//! 1. Memory boundary & pointer validation invariants
//! 2. Integer overflow protection on user buffers
//! 3. Capability-based access control & permission isolation
//! 4. Syscall dispatcher boundary & error handling
//! 5. Opaque handle model & resource isolation
//! 6. IPC channel boundary & backpressure isolation

use qeos_kernel::{
    fs::inode::InodeKind,
    fs::{FileHandle, OpenFlags, Vfs},
    ipc::{channel::ChannelError, Channel, Message},
    process::{
        process::{Pid, Process, ProcessState},
        thread::{Priority, Thread},
    },
    security::{check, Access, CapSet, Capability},
    syscall::{
        dispatcher::{dispatch_syscall, SyscallError},
        validate::{require, validate_range, ValidateError},
        SyscallNo,
    },
};

// ============================================================================
// 1. User Memory Boundary & Pointer Validation Tests
// ============================================================================

#[test]
fn test_boundary_valid_user_pointer_range() {
    let user_window_limit = 0x0000_7FFF_FFFF_0000usize;
    let ptr = 0x10000usize;
    let len = 4096usize;

    assert_eq!(validate_range(ptr, len, user_window_limit), Ok(()));
}

#[test]
fn test_boundary_null_pointer_rejected() {
    let user_window_limit = 0x100000usize;

    // Non-zero length with null pointer must be rejected
    assert_eq!(
        validate_range(0, 64, user_window_limit),
        Err(ValidateError::BadPointer)
    );
}

#[test]
fn test_boundary_out_of_bounds_length_rejected() {
    let user_window_limit = 0x1000usize;

    // Buffer extends beyond permissible user boundary
    assert_eq!(
        validate_range(0x800, 0x1000, user_window_limit),
        Err(ValidateError::BadLength)
    );
}

#[test]
fn test_boundary_integer_overflow_attack_rejected() {
    let user_window_limit = usize::MAX;

    // Integer overflow on address arithmetic must fail safely
    assert_eq!(
        validate_range(usize::MAX - 8, 16, user_window_limit),
        Err(ValidateError::Overflow)
    );
    assert_eq!(
        validate_range(usize::MAX, 1, user_window_limit),
        Err(ValidateError::Overflow)
    );
}

// ============================================================================
// 2. Capability Access Boundary & Isolation Tests
// ============================================================================

#[test]
fn test_boundary_unprivileged_process_denied_hardware_access() {
    let unprivileged_caps = CapSet::new();

    // All privileged operations must be rejected by default
    assert_eq!(
        require(&unprivileged_caps, Access::UseDma),
        Err(ValidateError::Denied)
    );
    assert_eq!(
        require(&unprivileged_caps, Access::ReadDevice),
        Err(ValidateError::Denied)
    );
    assert_eq!(
        require(&unprivileged_caps, Access::WriteDevice),
        Err(ValidateError::Denied)
    );
    assert_eq!(
        require(&unprivileged_caps, Access::UseQuantum),
        Err(ValidateError::Denied)
    );
}

#[test]
fn test_boundary_granular_capability_grant_isolation() {
    let mut caps = CapSet::new();
    caps.grant(Capability::Dma);

    // DMA access permitted
    assert_eq!(require(&caps, Access::UseDma), Ok(()));

    // Other hardware access still strictly denied
    assert_eq!(
        require(&caps, Access::ReadDevice),
        Err(ValidateError::Denied)
    );
    assert_eq!(
        require(&caps, Access::WriteDevice),
        Err(ValidateError::Denied)
    );
    assert_eq!(
        require(&caps, Access::UseQuantum),
        Err(ValidateError::Denied)
    );
}

#[test]
fn test_boundary_admin_capability_supercedes_all() {
    let mut caps = CapSet::new();
    caps.grant(Capability::Admin);

    assert!(check(&caps, Access::UseDma));
    assert!(check(&caps, Access::ReadDevice));
    assert!(check(&caps, Access::WriteDevice));
    assert!(check(&caps, Access::UseQuantum));
}

// ============================================================================
// 3. Syscall Dispatcher Boundary & Error Model Tests
// ============================================================================

#[test]
fn test_boundary_unknown_syscall_rejection() {
    assert_eq!(dispatch_syscall(999, 0, 0, 0), Err(SyscallError::Unknown));
    assert_eq!(
        dispatch_syscall(u64::MAX, 0, 0, 0),
        Err(SyscallError::Unknown)
    );
}

#[test]
fn test_boundary_known_syscall_dispatching() {
    assert_eq!(dispatch_syscall(SyscallNo::Read as u64, 0, 0, 0), Ok(0));
    assert_eq!(dispatch_syscall(SyscallNo::Write as u64, 0, 0, 0), Ok(0));
    assert_eq!(dispatch_syscall(SyscallNo::Open as u64, 0, 0, 0), Ok(3));
    assert_eq!(dispatch_syscall(SyscallNo::Close as u64, 0, 0, 0), Ok(0));
    assert_eq!(dispatch_syscall(SyscallNo::Spawn as u64, 0, 0, 0), Ok(100));
    assert_eq!(dispatch_syscall(SyscallNo::Exit as u64, 0, 0, 0), Ok(0));
    assert_eq!(dispatch_syscall(SyscallNo::Sleep as u64, 0, 0, 0), Ok(0));
    assert_eq!(dispatch_syscall(SyscallNo::Send as u64, 0, 0, 0), Ok(0));
    assert_eq!(dispatch_syscall(SyscallNo::Recv as u64, 0, 0, 0), Ok(0));
    assert_eq!(dispatch_syscall(SyscallNo::Map as u64, 0, 0, 0), Ok(0x1000));
    assert_eq!(
        dispatch_syscall(SyscallNo::Alloc as u64, 0, 0, 0),
        Ok(0x2000)
    );
}

// ============================================================================
// 4. Opaque Handle & Resource Isolation Tests
// ============================================================================

#[test]
fn test_boundary_vfs_handle_isolation() {
    let mut vfs = Vfs::new();
    let file_id = vfs.create(InodeKind::File);

    // Opening non-existent handle fails cleanly
    assert!(vfs.open(9999, OpenFlags::READ).is_err());

    // Valid handle read/write
    let mut handle = vfs.open(file_id, OpenFlags::WRITE).unwrap();
    let written = vfs.write(&mut handle, b"quantum-energy-os").unwrap();
    assert_eq!(written, 17);

    let mut read_handle = vfs.open(file_id, OpenFlags::READ).unwrap();
    let mut buffer = [0u8; 17];
    let read_count = vfs.read(&mut read_handle, &mut buffer).unwrap();
    assert_eq!(read_count, 17);
    assert_eq!(&buffer, b"quantum-energy-os");

    // Attempting to read invalid handle returns error
    let mut fake_handle = FileHandle { ino: 8888, off: 0 };
    let mut fake_buf = [0u8; 4];
    assert!(vfs.read(&mut fake_handle, &mut fake_buf).is_err());
    assert!(vfs.write(&mut fake_handle, &fake_buf).is_err());
}

// ============================================================================
// 5. IPC Channel Boundary & Backpressure Isolation Tests
// ============================================================================

#[test]
fn test_boundary_ipc_channel_backpressure_and_closure() {
    let mut channel = Channel::new(2);

    // Send up to capacity
    assert!(channel.send(Message::new(1, 2, 0, vec![0xAA])).is_ok());
    assert!(channel.send(Message::new(1, 2, 0, vec![0xBB])).is_ok());

    // Backpressure: channel full rejection
    match channel.send(Message::new(1, 2, 0, vec![0xCC])) {
        Err(ChannelError::Full) => {}
        other => panic!("Expected ChannelError::Full, got {:?}", other),
    }

    // Read messages
    assert_eq!(channel.recv().unwrap().payload, vec![0xAA]);
    assert_eq!(channel.recv().unwrap().payload, vec![0xBB]);

    // Empty channel rejection
    match channel.recv() {
        Err(ChannelError::Empty) => {}
        other => panic!("Expected ChannelError::Empty, got {:?}", other),
    }

    // Channel closure rejection
    channel.close();
    match channel.send(Message::new(1, 2, 0, vec![0xDD])) {
        Err(ChannelError::Closed) => {}
        other => panic!("Expected ChannelError::Closed, got {:?}", other),
    }
}

// ============================================================================
// 6. Process Table & State Isolation Tests
// ============================================================================

#[test]
fn test_boundary_process_state_and_thread_isolation() {
    let mut proc1 = Process::new(1, 1000);
    let mut proc2 = Process::new(2, 1001);

    assert_eq!(proc1.pid, Pid(1));
    assert_eq!(proc2.pid, Pid(2));
    assert_eq!(proc1.creds.uid, 1000);
    assert_eq!(proc2.creds.uid, 1001);

    proc1.add_thread(Thread::new(101, Priority::High));
    proc2.add_thread(Thread::new(201, Priority::Normal));

    assert_eq!(proc1.threads.len(), 1);
    assert!(proc1.threads.contains_key(&101));
    assert!(!proc1.threads.contains_key(&201));

    assert_eq!(proc2.threads.len(), 1);
    assert!(proc2.threads.contains_key(&201));
    assert!(!proc2.threads.contains_key(&101));

    proc1.transition(ProcessState::Running);
    assert_eq!(proc1.state, ProcessState::Running);
    assert_eq!(proc2.state, ProcessState::Created);
}
