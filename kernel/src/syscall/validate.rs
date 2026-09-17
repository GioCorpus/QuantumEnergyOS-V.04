//! User/kernel boundary validation (§23, §43). Never trust user pointer/len.
//!
//! Host model: user addresses are indices into a bounded window. Real kernel
//! would check canonical form, page permissions, and object ownership.

use crate::security::{check, Access, CapSet};

#[derive(Debug, PartialEq, Eq)]
pub enum ValidateError {
    BadPointer,
    BadLength,
    Denied,
    Overflow,
}

pub fn validate_range(ptr: usize, len: usize, max: usize) -> Result<(), ValidateError> {
    let end = ptr.checked_add(len).ok_or(ValidateError::Overflow)?;
    if end > max {
        return Err(ValidateError::BadLength);
    }
    // Null pointer is never valid as user buffer.
    if len > 0 && ptr == 0 {
        return Err(ValidateError::BadPointer);
    }
    Ok(())
}

pub fn require(caps: &CapSet, a: Access) -> Result<(), ValidateError> {
    if check(caps, a) {
        Ok(())
    } else {
        Err(ValidateError::Denied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::security::Capability;
    #[test]
    fn range() {
        assert!(validate_range(0x100, 16, 0x1000).is_ok());
        assert_eq!(validate_range(0, 8, 0x1000), Err(ValidateError::BadPointer));
        assert!(validate_range(0xFFF, 8, 0x1000).is_err());
    }
    #[test]
    fn caps() {
        let mut c = CapSet::new();
        assert_eq!(require(&c, Access::UseDma), Err(ValidateError::Denied));
        c.grant(Capability::Dma);
        assert!(require(&c, Access::UseDma).is_ok());
    }
}
