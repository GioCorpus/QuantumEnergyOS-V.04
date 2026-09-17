use super::inode::{Inode, InodeKind};
use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy)]
pub struct OpenFlags(pub u8);
impl OpenFlags {
    pub const READ: Self = Self(1);
    pub const WRITE: Self = Self(2);
}
#[derive(Debug)]
pub struct FileHandle {
    pub ino: u64,
    pub off: usize,
}
pub struct Vfs {
    next: u64,
    inodes: BTreeMap<u64, Inode>,
}
impl Vfs {
    pub fn new() -> Self {
        let mut v = Self {
            next: 1,
            inodes: BTreeMap::new(),
        };
        v.inodes.insert(0, Inode::new(0, InodeKind::Dir));
        v
    }
    pub fn create(&mut self, kind: InodeKind) -> u64 {
        let id = self.next;
        self.next += 1;
        self.inodes.insert(id, Inode::new(id, kind));
        id
    }
    pub fn open(&self, ino: u64, _f: OpenFlags) -> Result<FileHandle, &'static str> {
        self.inodes
            .get(&ino)
            .ok_or("noent")
            .map(|_| FileHandle { ino, off: 0 })
    }
    pub fn write(&mut self, h: &mut FileHandle, buf: &[u8]) -> Result<usize, &'static str> {
        let i = self.inodes.get_mut(&h.ino).ok_or("noent")?;
        if h.off + buf.len() > i.data.len() {
            i.data.resize(h.off + buf.len(), 0);
        }
        i.data[h.off..h.off + buf.len()].copy_from_slice(buf);
        h.off += buf.len();
        i.size = i.data.len();
        Ok(buf.len())
    }
    pub fn read(&self, h: &mut FileHandle, buf: &mut [u8]) -> Result<usize, &'static str> {
        let i = self.inodes.get(&h.ino).ok_or("noent")?;
        let n = buf
            .len()
            .min(i.data.len().saturating_sub(h.off.min(i.data.len())));
        buf[..n].copy_from_slice(&i.data[h.off..h.off + n]);
        h.off += n;
        Ok(n)
    }
}
impl Default for Vfs {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn rw() {
        let mut v = Vfs::new();
        let id = v.create(InodeKind::File);
        let mut h = v.open(id, OpenFlags::WRITE).unwrap();
        v.write(&mut h, b"hi").unwrap();
        let mut h2 = v.open(id, OpenFlags::READ).unwrap();
        let mut b = [0u8; 2];
        v.read(&mut h2, &mut b).unwrap();
        assert_eq!(&b, b"hi");
    }
}
