//! DMA abstraction with ownership tracking.
#[derive(Debug, PartialEq, Eq)] pub enum DmaError { Misaligned, TooLarge, NotMapped, UnalignedAccess, OutOfRange, Full, Empty }
pub trait DmaBuffer { fn physical_address(&self) -> usize; fn size(&self) -> usize; }
#[derive(Debug)] pub struct DmaRegion { phys: usize, size: usize, mapped: bool }
impl DmaRegion {
    pub fn new(phys: usize, size: usize) -> Result<Self, DmaError> {
        if phys % 4096 != 0 { return Err(DmaError::Misaligned); }
        if size == 0 || size > (1 << 20) * 64 { return Err(DmaError::TooLarge); }
        Ok(Self { phys, size, mapped: false })
    }
    pub fn map(&mut self) { self.mapped = true; }
    pub fn unmap(&mut self) { self.mapped = false; }
    pub fn is_mapped(&self) -> bool { self.mapped }
}
impl Drop for DmaRegion { fn drop(&mut self) { self.mapped = false; } }
impl DmaBuffer for DmaRegion { fn physical_address(&self) -> usize { self.phys } fn size(&self) -> usize { self.size } }
#[derive(Debug)] pub struct DmaMapping { iova: u64, size: usize, mapped: bool }
impl DmaMapping {
    pub fn new(iova: u64, size: usize) -> Result<Self, DmaError> {
        if iova % 4096 != 0 { return Err(DmaError::Misaligned); }
        if size == 0 || size > (1 << 20) * 64 { return Err(DmaError::TooLarge); }
        Ok(Self { iova, size, mapped: true })
    }
    pub fn iova(&self) -> u64 { self.iova }
    pub fn size(&self) -> usize { self.size }
    pub fn is_mapped(&self) -> bool { self.mapped }
    pub fn unmap(&mut self) { self.mapped = false; }
}
impl Drop for DmaMapping { fn drop(&mut self) { self.mapped = false; } }
#[derive(Debug)] pub struct DmaRing { desc: Vec<u64>, head: usize, tail: usize }
impl DmaRing {
    pub fn new(depth: usize) -> Result<Self, DmaError> { if depth < 2 || depth > 1024 || !depth.is_power_of_two() { return Err(DmaError::TooLarge); } Ok(Self { desc: vec![0; depth], head: 0, tail: 0 }) }
    pub fn capacity(&self) -> usize { self.desc.len() }
    pub fn len(&self) -> usize { self.tail.wrapping_sub(self.head) % self.capacity() }
    pub fn is_empty(&self) -> bool { self.head == self.tail }
    pub fn push(&mut self, d: u64) -> Result<(), DmaError> { let n = (self.tail + 1) % self.capacity(); if n == self.head { return Err(DmaError::Full); } self.desc[self.tail] = d; self.tail = n; Ok(()) }
    pub fn pop(&mut self) -> Result<u64, DmaError> { if self.is_empty() { return Err(DmaError::Empty); } let d = self.desc[self.head]; self.head = (self.head + 1) % self.capacity(); Ok(d) }
}
#[cfg(test)] mod tests { use super::*; #[test] fn dma() { assert!(DmaRegion::new(1, 4096).is_err()); let mut r = DmaRegion::new(0x1000, 4096).unwrap(); r.map(); assert!(r.is_mapped()); } #[test] fn ring_spsc() { let mut r = DmaRing::new(4).unwrap(); r.push(1).unwrap(); assert_eq!(r.pop().unwrap(), 1); } }
