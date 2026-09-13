pub mod dispatcher; pub mod numbers;
pub use dispatcher::{dispatch_syscall, SyscallError, SyscallResult};
pub use numbers::SyscallNo;
