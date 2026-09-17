pub mod inode;
pub mod vfs;
pub use inode::{Inode, InodeKind};
pub use vfs::{FileHandle, OpenFlags, Vfs};
