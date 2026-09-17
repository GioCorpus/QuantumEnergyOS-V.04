//! Phase 4.8 dataset export: JSON + CSV (reproducibility artifacts).
use crate::error::{QuantumError, Result};
use serde::Serialize;
pub fn export_json<T: Serialize>(v: &T) -> Result<String> {
    serde_json::to_string_pretty(v).map_err(|e| QuantumError::InvalidGateParameters(e.to_string()))
}
pub fn export_csv(headers: &[&str], rows: &[Vec<String>]) -> Result<String> {
    for r in rows {
        if r.len() != headers.len() {
            return Err(QuantumError::InvalidGateParameters(
                "row width mismatch".into(),
            ));
        }
    }
    let mut o = headers.join(",");
    o.push('\n');
    for r in rows {
        o.push_str(&r.join(","));
        o.push('\n');
    }
    Ok(o)
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn json_csv() {
        assert!(export_json(&serde_json::json!({"a": 1}))
            .unwrap()
            .contains('1'));
        let c = export_csv(&["s", "p"], &[vec!["00".into(), "0.5".into()]]).unwrap();
        assert!(c.starts_with("s,p"));
        assert!(export_csv(&["a"], &[vec!["x".into(), "y".into()]]).is_err());
    }
}
