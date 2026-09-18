//! QEOS V.04 — Syscall ABI & Dispatcher Comprehensive Test Suite (P5-02).
//!
//! Validates:
//! 1. Syscall number conversion & exhaustiveness
//! 2. Basic non-contextual syscall dispatching
//! 3. Context-aware capability enforcement across all privileged domains
//! 4. User pointer and buffer range validation
//! 5. Integer overflow protection in memory and I/O syscalls
//! 6. Typed SyscallError code mapping and stability

use qeos_kernel::{
    security::{CapSet, Capability},
    syscall::{dispatch_syscall, dispatch_syscall_ctx, SyscallContext, SyscallError, SyscallNo},
};

// ============================================================================
// 1. Syscall Number Encoding & Conversion Tests
// ============================================================================

#[test]
fn test_syscall_no_try_from_all_variants() {
    // VFS / IO
    assert_eq!(SyscallNo::try_from(0), Ok(SyscallNo::Read));
    assert_eq!(SyscallNo::try_from(1), Ok(SyscallNo::Write));
    assert_eq!(SyscallNo::try_from(2), Ok(SyscallNo::Open));
    assert_eq!(SyscallNo::try_from(3), Ok(SyscallNo::Close));
    assert_eq!(SyscallNo::try_from(4), Ok(SyscallNo::Stat));

    // Process & Thread
    assert_eq!(SyscallNo::try_from(10), Ok(SyscallNo::Spawn));
    assert_eq!(SyscallNo::try_from(11), Ok(SyscallNo::Exit));
    assert_eq!(SyscallNo::try_from(12), Ok(SyscallNo::Yield));
    assert_eq!(SyscallNo::try_from(13), Ok(SyscallNo::GetPid));
    assert_eq!(SyscallNo::try_from(14), Ok(SyscallNo::Wait));
    assert_eq!(SyscallNo::try_from(15), Ok(SyscallNo::ThreadSpawn));
    assert_eq!(SyscallNo::try_from(16), Ok(SyscallNo::ThreadExit));

    // Time
    assert_eq!(SyscallNo::try_from(20), Ok(SyscallNo::Sleep));
    assert_eq!(SyscallNo::try_from(21), Ok(SyscallNo::TimeNow));

    // IPC
    assert_eq!(SyscallNo::try_from(30), Ok(SyscallNo::Send));
    assert_eq!(SyscallNo::try_from(31), Ok(SyscallNo::Recv));
    assert_eq!(SyscallNo::try_from(32), Ok(SyscallNo::ChannelCreate));
    assert_eq!(SyscallNo::try_from(33), Ok(SyscallNo::ChannelClose));

    // Memory
    assert_eq!(SyscallNo::try_from(40), Ok(SyscallNo::Map));
    assert_eq!(SyscallNo::try_from(41), Ok(SyscallNo::Alloc));
    assert_eq!(SyscallNo::try_from(42), Ok(SyscallNo::Unmap));
    assert_eq!(SyscallNo::try_from(43), Ok(SyscallNo::Protect));

    // Device
    assert_eq!(SyscallNo::try_from(50), Ok(SyscallNo::DeviceOpen));
    assert_eq!(SyscallNo::try_from(51), Ok(SyscallNo::DeviceClose));
    assert_eq!(SyscallNo::try_from(52), Ok(SyscallNo::DeviceRead));
    assert_eq!(SyscallNo::try_from(53), Ok(SyscallNo::DeviceWrite));
    assert_eq!(SyscallNo::try_from(54), Ok(SyscallNo::DeviceIoctl));

    // Telemetry
    assert_eq!(SyscallNo::try_from(60), Ok(SyscallNo::TelemetryRead));
    assert_eq!(SyscallNo::try_from(61), Ok(SyscallNo::TelemetrySample));

    // Security
    assert_eq!(SyscallNo::try_from(70), Ok(SyscallNo::CapCheck));
    assert_eq!(SyscallNo::try_from(71), Ok(SyscallNo::CapDrop));

    // Invalid numbers
    assert!(SyscallNo::try_from(5).is_err());
    assert!(SyscallNo::try_from(999).is_err());
    assert!(SyscallNo::try_from(u64::MAX).is_err());
}

// ============================================================================
// 2. Syscall Dispatcher Basic Validation Tests
// ============================================================================

#[test]
fn test_syscall_dispatch_unknown_rejected() {
    assert_eq!(dispatch_syscall(999, 0, 0, 0), Err(SyscallError::Unknown));
    assert_eq!(dispatch_syscall(1000, 0, 0, 0), Err(SyscallError::Unknown));
    assert_eq!(
        dispatch_syscall(u64::MAX, 0, 0, 0),
        Err(SyscallError::Unknown)
    );
}

#[test]
fn test_syscall_dispatch_null_buffer_read_write_rejected() {
    // Read with NULL buffer pointer and non-zero length
    assert_eq!(
        dispatch_syscall(SyscallNo::Read as u64, 0, 0, 128),
        Err(SyscallError::BadPointer)
    );

    // Write with NULL buffer pointer and non-zero length
    assert_eq!(
        dispatch_syscall(SyscallNo::Write as u64, 0, 0, 128),
        Err(SyscallError::BadPointer)
    );

    // Read with NULL pointer and zero length is safe no-op
    assert_eq!(dispatch_syscall(SyscallNo::Read as u64, 0, 0, 0), Ok(0));
}

// ============================================================================
// 3. Context-Aware Capability Enforcement Tests
// ============================================================================

#[test]
fn test_syscall_ctx_unprivileged_denied_device_and_telemetry() {
    let unprivileged = CapSet::new();
    let ctx = SyscallContext::new(42, &unprivileged, 0x1000_0000);

    // Device operations must be denied
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::DeviceOpen as u64, 1, 0, 0),
        Err(SyscallError::Denied)
    );
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::DeviceRead as u64, 1, 0, 0),
        Err(SyscallError::Denied)
    );
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::DeviceWrite as u64, 1, 0, 0),
        Err(SyscallError::Denied)
    );
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::DeviceIoctl as u64, 1, 0, 0),
        Err(SyscallError::Denied)
    );

    // Telemetry operations must be denied
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::TelemetryRead as u64, 0, 0, 0),
        Err(SyscallError::Denied)
    );
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::TelemetrySample as u64, 0, 0, 0),
        Err(SyscallError::Denied)
    );
}

#[test]
fn test_syscall_ctx_granular_capability_grant() {
    let mut caps = CapSet::new();
    caps.grant(Capability::DeviceRead);
    caps.grant(Capability::Telemetry);

    let ctx = SyscallContext::new(42, &caps, 0x1000_0000);

    // Read device is allowed
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::DeviceOpen as u64, 7, 0, 0),
        Ok(7)
    );
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::DeviceRead as u64, 7, 0, 0),
        Ok(7)
    );

    // Write device is still denied
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::DeviceWrite as u64, 7, 0, 0),
        Err(SyscallError::Denied)
    );

    // Telemetry is allowed
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::TelemetryRead as u64, 0, 0, 0),
        Ok(0)
    );
}

#[test]
fn test_syscall_ctx_admin_capability_allows_all() {
    let mut caps = CapSet::new();
    caps.grant(Capability::Admin);

    let ctx = SyscallContext::new(1, &caps, 0x1000_0000);

    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::DeviceOpen as u64, 1, 0, 0),
        Ok(1)
    );
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::DeviceWrite as u64, 1, 0, 0),
        Ok(1)
    );
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::TelemetryRead as u64, 0, 0, 0),
        Ok(0)
    );
}

// ============================================================================
// 4. Memory & Buffer Boundary Tests
// ============================================================================

#[test]
fn test_syscall_ctx_buffer_range_checks() {
    let caps = CapSet::new();
    let max_user = 0x8000_0000usize;
    let ctx = SyscallContext::new(10, &caps, max_user);

    // Valid buffer inside address space
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::Read as u64, 3, 0x1000, 512),
        Ok(512)
    );

    // Out of bounds buffer exceeding max_user
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::Read as u64, 3, 0x7FFF_FF00, 1024),
        Err(SyscallError::BufferOverflow)
    );

    // Integer overflow attack on buffer length
    assert_eq!(
        dispatch_syscall_ctx(
            &ctx,
            SyscallNo::Read as u64,
            3,
            (usize::MAX - 32) as u64,
            64
        ),
        Err(SyscallError::BufferOverflow)
    );
}

#[test]
fn test_syscall_ctx_memory_allocation_validation() {
    let caps = CapSet::new();
    let ctx = SyscallContext::new(10, &caps, 0x1000_0000);

    // Alloc with 0 size is rejected
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::Alloc as u64, 0, 0, 0),
        Err(SyscallError::InvalidArg)
    );

    // Alloc with non-zero size succeeds
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::Alloc as u64, 0, 4096, 0),
        Ok(0x0000_7000_0000_0000)
    );

    // Unmap with NULL address is rejected
    assert_eq!(
        dispatch_syscall_ctx(&ctx, SyscallNo::Unmap as u64, 0, 0, 0),
        Err(SyscallError::BadPointer)
    );
}
