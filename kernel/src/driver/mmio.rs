//! Bounded MMIO window (host memory-backed model for tests).
use crate::dma::DmaError;
#[derive(Debug)]
pub struct MmioWindow {
    base: usize,
    size: usize,
    mem: Vec<u32>,
}
impl MmioWindow {
    pub fn new(base: usize, size: usize) -> Result<Self, DmaError> {
        if base % 4 != 0 || size % 4 != 0 || size == 0 {
            return Err(DmaError::Misaligned);
        }
        Ok(Self {
            base,
            size,
            mem: vec![0; size / 4],
        })
    }
    fn check(&self, off: usize, n: usize) -> Result<usize, DmaError> {
        if off % n != 0 {
            return Err(DmaError::UnalignedAccess);
        }
        if off.checked_add(n).map(|e| e > self.size).unwrap_or(true) {
            return Err(DmaError::OutOfRange);
        }
        Ok(off / 4)
    }
    pub fn read_u32(&self, off: usize) -> Result<u32, DmaError> {
        let i = self.check(off, 4)?;
        Ok(self.mem[i])
    }
    pub fn write_u32(&mut self, off: usize, v: u32) -> Result<(), DmaError> {
        let i = self.check(off, 4)?;
        self.mem[i] = v;
        Ok(())
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mmio() {
        let mut w = MmioWindow::new(0x1000, 16).unwrap();
        w.write_u32(0, 7).unwrap();
        assert_eq!(w.read_u32(0).unwrap(), 7);
        assert_eq!(w.write_u32(2, 1), Err(DmaError::UnalignedAccess));
        assert_eq!(w.read_u32(16), Err(DmaError::OutOfRange));
    }
}
