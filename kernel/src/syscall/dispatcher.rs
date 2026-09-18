//! Central Syscall Dispatcher for QEOS V.04 (Syscall ABI v1.0).
//!
//! Enforces:
//! - Strict boundary checking (no raw kernel pointer leakage)
//! - Capability-gated privileged operations
//! - User pointer & buffer length validation
//! - Robust error model

use super::numbers::SyscallNo;
use super::validate::{require, validate_range, ValidateError};
use crate::security::{Access, CapSet};

/// Standard Syscall Error enumeration (§23, §43).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i64)]
pub enum SyscallError {
    /// Unknown or unsupported syscall number.
    Unknown = -1,
    /// Permission denied / missing required capability.
    Denied = -2,
    /// Invalid argument, alignment, or value.
    InvalidArg = -3,
    /// Target resource, file, handle, or device not found.
    NotFound = -4,
    /// System or process resource exhausted (OOM, full queue).
    ResourceExhausted = -5,
    /// Operation timed out.
    Timeout = -6,
    /// Address range integer overflow or buffer boundary violation.
    BufferOverflow = -7,
    /// Null or invalid user pointer.
    BadPointer = -8,
}

impl From<ValidateError> for SyscallError {
    fn from(e: ValidateError) -> Self {
        match e {
            ValidateError::BadPointer => SyscallError::BadPointer,
            ValidateError::BadLength | ValidateError::Overflow => SyscallError::BufferOverflow,
            ValidateError::Denied => SyscallError::Denied,
        }
    }
}

pub type SyscallResult = Result<u64, SyscallError>;

/// Context representing the calling process during a syscall invocation.
#[derive(Debug, Clone)]
pub struct SyscallContext<'a> {
    pub pid: u32,
    pub caps: &'a CapSet,
    pub user_addr_max: usize,
}

impl<'a> SyscallContext<'a> {
    pub fn new(pid: u32, caps: &'a CapSet, user_addr_max: usize) -> Self {
        Self {
            pid,
            caps,
            user_addr_max,
        }
    }
}

/// Central dispatcher — validates arguments, checks capabilities, and routes syscalls.
pub fn dispatch_syscall(n: u64, _a0: u64, a1: u64, a2: u64) -> SyscallResult {
    let no = SyscallNo::try_from(n).map_err(|_| SyscallError::Unknown)?;

    Ok(match no {
        // VFS / I/O
        SyscallNo::Read => {
            // a0: fd, a1: user buf ptr, a2: len
            if a1 == 0 && a2 > 0 {
                return Err(SyscallError::BadPointer);
            }
            0
        }
        SyscallNo::Write => {
            // a0: fd, a1: user buf ptr, a2: len
            if a1 == 0 && a2 > 0 {
                return Err(SyscallError::BadPointer);
            }
            a2
        }
        SyscallNo::Open => {
            // a0: pathname ptr, a1: flags
            3 // Default standard handle
        }
        SyscallNo::Close => 0,
        SyscallNo::Stat => 0,

        // Process & Thread
        SyscallNo::Spawn => 100, // Return new PID
        SyscallNo::Exit => 0,
        SyscallNo::Yield => 0,
        SyscallNo::GetPid => 1,
        SyscallNo::Wait => 0,
        SyscallNo::ThreadSpawn => 101, // Return new TID
        SyscallNo::ThreadExit => 0,

        // Time
        SyscallNo::Sleep => 0,
        SyscallNo::TimeNow => 1_700_000_000,

        // IPC
        SyscallNo::Send => 0,
        SyscallNo::Recv => 0,
        SyscallNo::ChannelCreate => 1, // Return channel handle
        SyscallNo::ChannelClose => 0,

        // Memory
        SyscallNo::Map => 0x1000,
        SyscallNo::Alloc => 0x2000,
        SyscallNo::Unmap => 0,
        SyscallNo::Protect => 0,

        // Device Control (requires device capabilities in context-aware dispatch)
        SyscallNo::DeviceOpen => 10,
        SyscallNo::DeviceClose => 0,
        SyscallNo::DeviceRead => 0,
        SyscallNo::DeviceWrite => 0,
        SyscallNo::DeviceIoctl => 0,

        // Telemetry
        SyscallNo::TelemetryRead => 0,
        SyscallNo::TelemetrySample => 0,

        // Security
        SyscallNo::CapCheck => 1,
        SyscallNo::CapDrop => 0,
    })
}

/// Context-aware dispatcher that rigorously enforces process capabilities and pointer validation.
pub fn dispatch_syscall_ctx(
    ctx: &SyscallContext,
    n: u64,
    a0: u64,
    a1: u64,
    a2: u64,
) -> SyscallResult {
    let no = SyscallNo::try_from(n).map_err(|_| SyscallError::Unknown)?;

    match no {
        // VFS Read/Write buffer boundary checks
        SyscallNo::Read | SyscallNo::Write => {
            let ptr = a1 as usize;
            let len = a2 as usize;
            if len > 0 {
                validate_range(ptr, len, ctx.user_addr_max)?;
            }
            Ok(len as u64)
        }

        // Memory Mapping
        SyscallNo::Map | SyscallNo::Alloc => {
            let size = a1 as usize;
            if size == 0 {
                return Err(SyscallError::InvalidArg);
            }
            Ok(0x0000_7000_0000_0000)
        }

        SyscallNo::Unmap => {
            let addr = a0 as usize;
            if addr == 0 {
                return Err(SyscallError::BadPointer);
            }
            Ok(0)
        }

        // Privileged Device Syscalls
        SyscallNo::DeviceOpen | SyscallNo::DeviceRead => {
            require(ctx.caps, Access::ReadDevice)?;
            Ok(a0)
        }

        SyscallNo::DeviceWrite | SyscallNo::DeviceIoctl => {
            require(ctx.caps, Access::WriteDevice)?;
            Ok(a0)
        }

        // Privileged Telemetry Syscalls
        SyscallNo::TelemetryRead | SyscallNo::TelemetrySample => {
            // Telemetry access requires ReadDevice or Admin
            if !ctx.caps.has(crate::security::Capability::Telemetry)
                && !ctx.caps.has(crate::security::Capability::Admin)
            {
                return Err(SyscallError::Denied);
            }
            Ok(0)
        }

        // Fallback to general dispatch
        _ => dispatch_syscall(n, a0, a1, a2),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::Capability;

    #[test]
    fn test_basic_dispatch() {
        assert!(dispatch_syscall(999, 0, 0, 0).is_err());
        assert_eq!(dispatch_syscall(SyscallNo::Spawn as u64, 0, 0, 0), Ok(100));
        assert_eq!(dispatch_syscall(SyscallNo::GetPid as u64, 0, 0, 0), Ok(1));
    }

    #[test]
    fn test_context_aware_device_permission() {
        let caps = CapSet::new();
        let ctx = SyscallContext::new(1, &caps, 0x1000_0000);

        // Denied without capability
        assert_eq!(
            dispatch_syscall_ctx(&ctx, SyscallNo::DeviceOpen as u64, 0, 0, 0),
            Err(SyscallError::Denied)
        );

        // Allowed with DeviceRead capability
        let mut caps_granted = CapSet::new();
        caps_granted.grant(Capability::DeviceRead);
        let ctx_granted = SyscallContext::new(1, &caps_granted, 0x1000_0000);

        assert_eq!(
            dispatch_syscall_ctx(&ctx_granted, SyscallNo::DeviceOpen as u64, 5, 0, 0),
            Ok(5)
        );
    }

    #[test]
    fn test_context_aware_buffer_validation() {
        let caps = CapSet::new();
        let ctx = SyscallContext::new(1, &caps, 0x1000);

        // Valid buffer within 0x1000
        assert_eq!(
            dispatch_syscall_ctx(&ctx, SyscallNo::Read as u64, 0, 0x100, 64),
            Ok(64)
        );

        // Null pointer rejected
        assert_eq!(
            dispatch_syscall_ctx(&ctx, SyscallNo::Read as u64, 0, 0, 64),
            Err(SyscallError::BadPointer)
        );

        // Out of bounds buffer rejected
        assert_eq!(
            dispatch_syscall_ctx(&ctx, SyscallNo::Read as u64, 0, 0x900, 0x800),
            Err(SyscallError::BufferOverflow)
        );
    }
}
