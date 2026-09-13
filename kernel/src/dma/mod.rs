//! DMA abstraction with ownership tracking.
#[derive(Debug, PartialEq, Eq)] pub enum DmaError { Misaligned, TooLarge, NotMapped }
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
// Invariant I1 (§65): mapping belongs to an active device context and is
// released on Drop. Host model: unmap on drop (no IOMMU yet — deny-by-default).
impl Drop for DmaRegion {
    fn drop(&mut self) { self.mapped = false; }
}
impl DmaBuffer for DmaRegion { fn physical_address(&self) -> usize { self.phys } fn size(&self) -> usize { self.size } }
#[cfg(test)] mod tests { use super::*; #[test] fn dma() { assert!(DmaRegion::new(1, 4096).is_err()); let mut r = DmaRegion::new(0x1000, 4096).unwrap(); r.map(); assert!(r.is_mapped()); } }
