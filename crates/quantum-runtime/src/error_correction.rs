/// Quantum error detection / correction models (MODEL, not HARDWARE).
///
/// Classical models of topological protection concepts: parity checks,
/// syndrome extraction, repetition-code logical encoding. All outputs are
/// explicitly labeled as models; no claim of physical Majorana behavior.
use crate::measurement::{Measurement, MeasurementValue};
use serde::{Deserialize, Serialize};

/// Correction strategy selector.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrectionStrategy {
    /// Detect only, no correction.
    DetectionOnly,
    /// Classical repetition-code majority vote (MODEL).
    RepetitionCode,
    /// Parity-check based detection (MODEL).
    ParityCheck,
}

impl Default for CorrectionStrategy {
    fn default() -> Self {
        Self::DetectionOnly
    }
}

/// A syndrome extracted from parity measurements.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Syndrome {
    /// Raw syndrome bits as string, e.g. "01".
    pub bits: String,
    /// Whether the syndrome indicates an error.
    pub error_detected: bool,
}

impl Syndrome {
    pub fn new(bits: impl Into<String>) -> Self {
        let bits = bits.into();
        let error_detected = bits.chars().any(|c| c == '1');
        Self {
            bits,
            error_detected,
        }
    }
}

/// Logical qubit encoding via repetition code (MODEL).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogicalQubit {
    /// Physical bit values (0/1).
    pub physical: Vec<u8>,
    /// Strategy used.
    pub strategy: CorrectionStrategy,
}

impl LogicalQubit {
    pub fn encode(bit: u8, replicas: usize, strategy: CorrectionStrategy) -> Self {
        let physical = vec![bit & 1; replicas.max(1)];
        Self { physical, strategy }
    }

    /// Majority-vote decode. Returns None on tie (even split).
    pub fn decode(&self) -> Option<u8> {
        if self.physical.is_empty() {
            return None;
        }
        let ones = self.physical.iter().filter(|&&b| b == 1).count();
        let zeros = self.physical.len() - ones;
        if ones > zeros {
            Some(1)
        } else if zeros > ones {
            Some(0)
        } else {
            None
        }
    }

    /// Extract syndrome: pairwise parity between neighbors.
    /// "0" = agree, "1" = disagree (possible error).
    pub fn syndrome(&self) -> Syndrome {
        if self.physical.len() < 2 {
            return Syndrome::new("");
        }
        let bits: String = self
            .physical
            .windows(2)
            .map(|w| if w[0] == w[1] { '0' } else { '1' })
            .collect();
        Syndrome::new(bits)
    }

    pub fn to_measurement(&self) -> Measurement {
        match self.decode() {
            Some(0) => Measurement {
                value: MeasurementValue::LogicalState("0".to_string()),
                shot_count: 1,
            },
            Some(1) => Measurement {
                value: MeasurementValue::LogicalState("1".to_string()),
                shot_count: 1,
            },
            _ => Measurement {
                value: MeasurementValue::Syndrome(self.syndrome().bits),
                shot_count: 1,
            },
        }
    }
}

/// Phase 4.8 error-correction contract (spec 4.8.1).
pub trait ErrorCorrectionCode: Send + Sync {
    fn name(&self) -> &str;
    fn encode_bit(&self, bit: u8) -> LogicalQubit;
    fn syndrome_of(&self, q: &LogicalQubit) -> Syndrome;
    fn logical_error_rate(&self, physical_rate: f64) -> f64;
}

/// Repetition code (MODEL): logical error approx p^ceil(n/2).
#[derive(Debug, Clone, Copy)]
pub struct RepetitionCode {
    pub replicas: usize,
}
impl RepetitionCode {
    pub fn new(replicas: usize) -> Self {
        Self {
            replicas: replicas.max(1),
        }
    }
}
impl ErrorCorrectionCode for RepetitionCode {
    fn name(&self) -> &str {
        "repetition-code"
    }
    fn encode_bit(&self, bit: u8) -> LogicalQubit {
        LogicalQubit::encode(bit, self.replicas, CorrectionStrategy::RepetitionCode)
    }
    fn syndrome_of(&self, q: &LogicalQubit) -> Syndrome {
        q.syndrome()
    }
    fn logical_error_rate(&self, p: f64) -> f64 {
        let t = (self.replicas + 1) / 2;
        p.powi(t as i32)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_encode_decode() {
        let lq = LogicalQubit::encode(1, 3, CorrectionStrategy::RepetitionCode);
        assert_eq!(lq.decode(), Some(1));
        assert!(!lq.syndrome().error_detected);
    }

    #[test]
    fn test_single_error_detected_but_correctable() {
        let mut lq = LogicalQubit::encode(0, 3, CorrectionStrategy::RepetitionCode);
        lq.physical[1] = 1; // single-bit error (MODEL)
        assert!(lq.syndrome().error_detected);
        assert_eq!(lq.decode(), Some(0)); // majority still correct
    }

    #[test]
    fn test_tie_returns_none() {
        let lq = LogicalQubit {
            physical: vec![0, 1],
            strategy: CorrectionStrategy::RepetitionCode,
        };
        assert_eq!(lq.decode(), None);
    }

    #[test]
    fn test_syndrome_bits() {
        let s = Syndrome::new("00");
        assert!(!s.error_detected);
        let s2 = Syndrome::new("01");
        assert!(s2.error_detected);
    }
}
