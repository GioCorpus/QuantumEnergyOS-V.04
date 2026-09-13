pub mod dispatcher; pub mod numbers; pub mod validate;
pub use dispatcher::{dispatch_syscall, SyscallError, SyscallResult};
pub use numbers::SyscallNo;
pub use validate::{validate_range, ValidateError};
