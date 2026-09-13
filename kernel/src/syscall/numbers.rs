#[derive(Debug, Clone, Copy, PartialEq, Eq)] #[repr(u64)]
pub enum SyscallNo { Read = 0, Write = 1, Open = 2, Close = 3, Spawn = 10, Exit = 11, Sleep = 20, Send = 30, Recv = 31, Map = 40, Alloc = 41 }
impl TryFrom<u64> for SyscallNo {
    type Error = ();
    fn try_from(v: u64) -> Result<Self, ()> {
        Ok(match v { 0 => Self::Read, 1 => Self::Write, 2 => Self::Open, 3 => Self::Close, 10 => Self::Spawn, 11 => Self::Exit, 20 => Self::Sleep, 30 => Self::Send, 31 => Self::Recv, 40 => Self::Map, 41 => Self::Alloc, _ => return Err(()), })
    }
}
