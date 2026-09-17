//! Phase 4.9 provenance: Measured vs Estimated vs Simulated vs Unavailable.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Provenance {
    Measured,
    Estimated,
    Simulated,
    Unavailable,
}
impl Provenance {
    pub fn label(self) -> &'static str {
        match self {
            Self::Measured => "measured",
            Self::Estimated => "estimated",
            Self::Simulated => "simulated",
            Self::Unavailable => "unavailable",
        }
    }
}
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ClassifiedSample {
    pub value: f64,
    pub provenance: Provenance,
}
impl ClassifiedSample {
    pub fn simulated(v: f64) -> Self {
        Self {
            value: v,
            provenance: Provenance::Simulated,
        }
    }
    pub fn is_measurement(self) -> bool {
        self.provenance == Provenance::Measured
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn labels() {
        assert_eq!(Provenance::Simulated.label(), "simulated");
        assert!(!ClassifiedSample::simulated(1.0).is_measurement());
    }
}
