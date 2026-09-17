//! Minimal 64-bit ELF loader with validation. Never trusts input.
pub const MAGIC: [u8; 4] = [0x7f, b'E', b'L', b'F'];
#[derive(Debug, PartialEq, Eq)]
pub enum ElfError {
    TooSmall,
    BadMagic,
    BadClass,
    BadEndian,
    BadMachine,
    BadSegments,
}
#[derive(Debug, PartialEq, Eq)]
pub struct ElfInfo {
    pub entry: u64,
    pub segments: u16,
}
pub fn load(data: &[u8]) -> Result<ElfInfo, ElfError> {
    if data.len() < 64 {
        return Err(ElfError::TooSmall);
    }
    if data[0..4] != MAGIC {
        return Err(ElfError::BadMagic);
    }
    if data[4] != 2 {
        return Err(ElfError::BadClass);
    }
    if data[5] != 1 {
        return Err(ElfError::BadEndian);
    }
    let machine = u16::from_le_bytes([data[18], data[19]]);
    if machine != 62 {
        return Err(ElfError::BadMachine);
    }
    // No unwrap() in kernel paths (§51): slice length already checked (>=64).
    let entry_bytes: [u8; 8] = data[24..32].try_into().map_err(|_| ElfError::TooSmall)?;
    let entry = u64::from_le_bytes(entry_bytes);
    if entry == 0 {
        return Err(ElfError::BadSegments);
    }
    Ok(ElfInfo { entry, segments: 1 })
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reject() {
        assert_eq!(load(&[0; 10]), Err(ElfError::TooSmall));
    }
    #[test]
    fn accept() {
        let mut b = vec![0u8; 64];
        b[0..4].copy_from_slice(&MAGIC);
        b[4] = 2;
        b[5] = 1;
        b[18] = 62;
        b[24] = 0x10;
        assert!(load(&b).is_ok());
    }
}
