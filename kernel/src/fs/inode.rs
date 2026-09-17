#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InodeKind {
    File,
    Dir,
}
#[derive(Debug)]
pub struct Inode {
    pub id: u64,
    pub kind: InodeKind,
    pub size: usize,
    pub data: Vec<u8>,
}
impl Inode {
    pub fn new(id: u64, kind: InodeKind) -> Self {
        Self {
            id,
            kind,
            size: 0,
            data: Vec::new(),
        }
    }
}
