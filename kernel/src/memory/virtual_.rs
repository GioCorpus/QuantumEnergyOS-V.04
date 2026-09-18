use std::collections::BTreeMap;
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct VirtAddr(pub usize);
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VirtPage(pub usize);
#[derive(Debug, Clone, Copy)]
pub struct MapFlags(pub u8);
impl MapFlags {
    pub const READ: Self = Self(1);
    pub const WRITE: Self = Self(2);
    pub const EXEC: Self = Self(4);
    pub const USER: Self = Self(8);
}
pub struct VirtualMemoryManager {
    map: BTreeMap<usize, (usize, MapFlags)>,
}
impl VirtualMemoryManager {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
        }
    }
    /// Maps a page. Enforces W^X (§12): WRITE+EXEC denied. Phys must be page-aligned.
    pub fn map_page(&mut self, v: VirtPage, p: usize, f: MapFlags) -> Result<(), &'static str> {
        if self.map.contains_key(&v.0) {
            return Err("already mapped");
        }
        if !p.is_multiple_of(crate::core::config::PAGE_SIZE) {
            return Err("phys misaligned");
        }
        if (f.0 & MapFlags::WRITE.0 != 0) && (f.0 & MapFlags::EXEC.0 != 0) {
            return Err("RWX denied: W^X");
        }
        self.map.insert(v.0, (p, f));
        Ok(())
    }
    pub fn unmap_page(&mut self, v: VirtPage) -> Result<(), &'static str> {
        self.map.remove(&v.0).map(|_| ()).ok_or("not mapped")
    }
    pub fn translate(&self, v: VirtPage) -> Option<usize> {
        self.map.get(&v.0).map(|(p, _)| *p)
    }
    pub fn is_user(&self, v: VirtPage) -> bool {
        self.map
            .get(&v.0)
            .map(|(_, f)| f.0 & MapFlags::USER.0 != 0)
            .unwrap_or(false)
    }
}
impl Default for VirtualMemoryManager {
    fn default() -> Self {
        Self::new()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn map() {
        let mut v = VirtualMemoryManager::new();
        v.map_page(VirtPage(1), 0x1000, MapFlags::READ).unwrap();
        assert_eq!(v.translate(VirtPage(1)), Some(0x1000));
        v.unmap_page(VirtPage(1)).unwrap();
        assert!(v.translate(VirtPage(1)).is_none());
    }
}
