use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MajoranaZeroMode {
    pub id: String,
    pub parity: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TetronLikeLogicalQubit {
    pub id: String,
    pub majorana_modes: Vec<MajoranaZeroMode>,
    pub protected: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologicalErrorModel {
    pub model_name: String,
    pub error_rate: f64,
    pub correction_strategy: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraidingOperation {
    pub from: String,
    pub to: String,
    pub description: String,
}

pub fn describe_topology() -> &'static str {
    "Topological qubit model: Majorana-like logical degrees of freedom represented as a model abstraction; not a claim of physical hardware behavior."
}
