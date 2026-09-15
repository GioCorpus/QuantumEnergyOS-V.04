//! Phase 4.3 decoder abstraction + repetition-code decoder (MODEL).
//!
//! CLASSIFICATION: MODEL
//!
//! Decoders map syndromes to corrections. This file provides the trait plus a
//! verifiable repetition-code implementation. MWPM/neural decoders are future
//! extension points and are NOT implemented here.

use serde::{Deserialize, Serialize};

use crate::error::{QuantumError, Result};
use crate::error_correction::Syndrome;

/// Correction proposed by a decoder.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Correction {
    pub flipped_positions: Vec<usize>,
    pub logical_flip: bool,
    pub confidence: f64,
    pub decoder: String,
}

impl Correction {
    pub fn none(decoder: impl Into<String>) -> Self {
        Self {
            flipped_positions: Vec::new(),
            logical_flip: false,
            confidence: 1.0,
            decoder: decoder.into(),
        }
    }
}

/// Decoder contract shared by every error-correction code.
pub trait Decoder: Send + Sync {
    fn name(&self) -> &str;
    fn decode(&self, syndrome: &Syndrome) -> Result<Correction>;
}

/// Simple repetition-code decoder: any `1` in the pairwise syndrome marks a
/// disagreement between neighbours; correct the right member of each pair.
#[derive(Debug, Clone, Default)]
pub struct RepetitionDecoder {
    pub replicas: usize,
}

impl RepetitionDecoder {
    pub fn new(replicas: usize) -> Self {
        Self {
            replicas: replicas.max(1),
        }
    }
}

impl Decoder for RepetitionDecoder {
    fn name(&self) -> &str {
        "repetition-decoder"
    }

    fn decode(&self, syndrome: &Syndrome) -> Result<Correction> {
        for c in syndrome.bits.chars() {
            if c != '0' && c != '1' {
                return Err(QuantumError::DecoderError(format!(
                    "invalid syndrome bit '{c}' in {:?}",
                    syndrome.bits
                )));
            }
        }
        let flipped: Vec<usize> = syndrome
            .bits
            .chars()
            .enumerate()
            .filter(|(_, c)| *c == '1')
            .map(|(i, _)| i + 1)
            .collect();
        Ok(Correction {
            flipped_positions: flipped,
            logical_flip: false,
            confidence: if syndrome.error_detected { 0.5 } else { 1.0 },
            decoder: self.name().to_string(),
        })
    }
}

/// Lookup-table decoder over explicit syndrome -> correction entries.
#[derive(Debug, Clone, Default)]
pub struct LookupDecoder {
    pub table: std::collections::HashMap<String, Correction>,
}

impl LookupDecoder {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, syndrome_bits: impl Into<String>, correction: Correction) {
        self.table.insert(syndrome_bits.into(), correction);
    }
}

impl Decoder for LookupDecoder {
    fn name(&self) -> &str {
        "lookup-decoder"
    }

    fn decode(&self, syndrome: &Syndrome) -> Result<Correction> {
        self.table.get(&syndrome.bits).cloned().ok_or_else(|| {
            QuantumError::DecoderError(format!("no table entry for syndrome {:?}", syndrome.bits))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error_correction::Syndrome;

    #[test]
    fn test_repetition_decoder_clean() {
        let d = RepetitionDecoder::new(3);
        let c = d.decode(&Syndrome::new("00")).unwrap();
        assert!(c.flipped_positions.is_empty());
    }

    #[test]
    fn test_repetition_decoder_flags_pair() {
        let d = RepetitionDecoder::new(3);
        let c = d.decode(&Syndrome::new("10")).unwrap();
        assert_eq!(c.flipped_positions, vec![1]);
    }

    #[test]
    fn test_lookup_decoder_hit_and_miss() {
        let mut d = LookupDecoder::new();
        d.insert("01", Correction::none("lookup-decoder"));
        assert!(d.decode(&Syndrome::new("01")).is_ok());
        assert!(d.decode(&Syndrome::new("11")).is_err());
    }

    #[test]
    fn test_invalid_syndrome_rejected() {
        let d = RepetitionDecoder::new(3);
        assert!(d.decode(&Syndrome::new("0x")).is_err());
    }
}
