//! IOMMU abstraction (Phase 4.4): domain + attach + map/unmap.
//! Software model only; real IOMMU stays FUTURE.
use crate::dma::{DmaError, DmaMapping};
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct IommuPerm {
    pub read: bool,
    pub write: bool,
}
impl IommuPerm {
    pub const RW: Self = Self {
        read: true,
        write: true,
    };
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DomainId(pub u32);
pub trait Iommu {
    type Error;
    fn create_domain(&mut self) -> Result<DomainId, Self::Error>;
    fn attach(&mut self, d: DomainId, dev: &str) -> Result<(), Self::Error>;
    fn map(
        &mut self,
        d: DomainId,
        iova: u64,
        size: usize,
        perm: IommuPerm,
    ) -> Result<DmaMapping, Self::Error>;
    fn unmap(&mut self, m: &mut DmaMapping) -> Result<(), Self::Error>;
}
#[derive(Debug, Default)]
pub struct MockIommu {
    next: u32,
    attached: Vec<(DomainId, String)>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum MockIommuError {
    NoDomain,
    Unmapped,
}
impl Iommu for MockIommu {
    type Error = MockIommuError;
    fn create_domain(&mut self) -> Result<DomainId, Self::Error> {
        self.next += 1;
        Ok(DomainId(self.next))
    }
    fn attach(&mut self, d: DomainId, dev: &str) -> Result<(), Self::Error> {
        if d.0 == 0 || d.0 > self.next {
            return Err(MockIommuError::NoDomain);
        }
        self.attached.push((d, dev.into()));
        Ok(())
    }
    fn map(
        &mut self,
        d: DomainId,
        iova: u64,
        size: usize,
        _p: IommuPerm,
    ) -> Result<DmaMapping, Self::Error> {
        if d.0 == 0 || d.0 > self.next {
            return Err(MockIommuError::NoDomain);
        }
        DmaMapping::new(iova, size).map_err(|_| MockIommuError::Unmapped)
    }
    fn unmap(&mut self, m: &mut DmaMapping) -> Result<(), Self::Error> {
        if !m.is_mapped() {
            return Err(MockIommuError::Unmapped);
        }
        m.unmap();
        Ok(())
    }
}
impl From<DmaError> for MockIommuError {
    fn from(_: DmaError) -> Self {
        MockIommuError::Unmapped
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mock() {
        let mut i = MockIommu::default();
        let d = i.create_domain().unwrap();
        i.attach(d, "0000:00:01.0").unwrap();
        let mut m = i.map(d, 0x3000, 4096, IommuPerm::RW).unwrap();
        i.unmap(&mut m).unwrap();
    }
}
