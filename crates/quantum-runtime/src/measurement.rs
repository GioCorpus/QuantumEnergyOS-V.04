use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MeasurementValue {
    Bit0,
    Bit1,
    ProbabilityDistribution(Vec<f64>),
    Parity(bool),
    Syndrome(String),
    LogicalState(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Measurement {
    pub value: MeasurementValue,
    pub shot_count: u64,
}

impl Default for Measurement {
    fn default() -> Self {
        Self {
            value: MeasurementValue::Bit0,
            shot_count: 1,
        }
    }
}
