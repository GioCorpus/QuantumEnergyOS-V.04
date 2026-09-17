use super::numbers::SyscallNo;
#[derive(Debug, PartialEq, Eq)]
pub enum SyscallError {
    Unknown,
    Denied,
    InvalidArg,
}
pub type SyscallResult = Result<u64, SyscallError>;
/// Central dispatcher — never exposes internal kernel structs to userspace.
pub fn dispatch_syscall(n: u64, _a0: u64, _a1: u64, _a2: u64) -> SyscallResult {
    let no = SyscallNo::try_from(n).map_err(|_| SyscallError::Unknown)?;
    Ok(match no {
        SyscallNo::Read => 0,
        SyscallNo::Write => 0,
        SyscallNo::Open => 3,
        SyscallNo::Close => 0,
        SyscallNo::Spawn => 100,
        SyscallNo::Exit => 0,
        SyscallNo::Sleep => 0,
        SyscallNo::Send => 0,
        SyscallNo::Recv => 0,
        SyscallNo::Map => 0x1000,
        SyscallNo::Alloc => 0x2000,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn dispatch() {
        assert!(dispatch_syscall(999, 0, 0, 0).is_err());
        assert_eq!(dispatch_syscall(10, 0, 0, 0), Ok(100));
    }
}
