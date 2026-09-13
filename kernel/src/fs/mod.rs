pub mod vfs; pub mod inode;
pub use vfs::{Vfs, FileHandle, OpenFlags};
pub use inode::{Inode, InodeKind};
