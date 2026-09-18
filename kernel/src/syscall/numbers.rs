//! Syscall Number Definitions for QEOS V.04 (Syscall ABI v1.0).

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u64)]
pub enum SyscallNo {
    // VFS / I/O (0..9)
    Read = 0,
    Write = 1,
    Open = 2,
    Close = 3,
    Stat = 4,

    // Process & Thread Management (10..19)
    Spawn = 10,
    Exit = 11,
    Yield = 12,
    GetPid = 13,
    Wait = 14,
    ThreadSpawn = 15,
    ThreadExit = 16,

    // Time & Clocks (20..29)
    Sleep = 20,
    TimeNow = 21,

    // Inter-Process Communication (30..39)
    Send = 30,
    Recv = 31,
    ChannelCreate = 32,
    ChannelClose = 33,

    // Memory Management (40..49)
    Map = 40,
    Alloc = 41,
    Unmap = 42,
    Protect = 43,

    // Device Management & Control (50..59)
    DeviceOpen = 50,
    DeviceClose = 51,
    DeviceRead = 52,
    DeviceWrite = 53,
    DeviceIoctl = 54,

    // Telemetry & Observability (60..69)
    TelemetryRead = 60,
    TelemetrySample = 61,

    // Security & Capabilities (70..79)
    CapCheck = 70,
    CapDrop = 71,
}

impl TryFrom<u64> for SyscallNo {
    type Error = ();

    fn try_from(v: u64) -> Result<Self, ()> {
        Ok(match v {
            // VFS / I/O
            0 => Self::Read,
            1 => Self::Write,
            2 => Self::Open,
            3 => Self::Close,
            4 => Self::Stat,

            // Process & Thread
            10 => Self::Spawn,
            11 => Self::Exit,
            12 => Self::Yield,
            13 => Self::GetPid,
            14 => Self::Wait,
            15 => Self::ThreadSpawn,
            16 => Self::ThreadExit,

            // Time
            20 => Self::Sleep,
            21 => Self::TimeNow,

            // IPC
            30 => Self::Send,
            31 => Self::Recv,
            32 => Self::ChannelCreate,
            33 => Self::ChannelClose,

            // Memory
            40 => Self::Map,
            41 => Self::Alloc,
            42 => Self::Unmap,
            43 => Self::Protect,

            // Device
            50 => Self::DeviceOpen,
            51 => Self::DeviceClose,
            52 => Self::DeviceRead,
            53 => Self::DeviceWrite,
            54 => Self::DeviceIoctl,

            // Telemetry
            60 => Self::TelemetryRead,
            61 => Self::TelemetrySample,

            // Security
            70 => Self::CapCheck,
            71 => Self::CapDrop,

            _ => return Err(()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_syscall_number_conversion() {
        assert_eq!(SyscallNo::try_from(0), Ok(SyscallNo::Read));
        assert_eq!(SyscallNo::try_from(10), Ok(SyscallNo::Spawn));
        assert_eq!(SyscallNo::try_from(20), Ok(SyscallNo::Sleep));
        assert_eq!(SyscallNo::try_from(30), Ok(SyscallNo::Send));
        assert_eq!(SyscallNo::try_from(40), Ok(SyscallNo::Map));
        assert_eq!(SyscallNo::try_from(50), Ok(SyscallNo::DeviceOpen));
        assert_eq!(SyscallNo::try_from(60), Ok(SyscallNo::TelemetryRead));
        assert_eq!(SyscallNo::try_from(70), Ok(SyscallNo::CapCheck));
        assert!(SyscallNo::try_from(999).is_err());
    }
}
