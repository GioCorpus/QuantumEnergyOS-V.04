//! Phase 4.3 optimizer: validation + simple peephole rules.
//!
//! CLASSIFICATION: MODEL
//!
//! Rules are limited to verifiable identities: X-X, H-H, Barrier removal.
//! No complex rewriting without tests.

use crate::gates::QuantumGate;

/// Optimization report.
#[derive(Debug, Clone)]
pub struct OptimizationReport {
    pub removed: usize,
    pub rules_applied: Vec<String>,
}

/// Remove redundant pairs (X-X, H-H) and barriers.
pub fn optimize_circuit(gates: &mut Vec<QuantumGate>) -> OptimizationReport {
    let mut removed = 0;
    let mut rules = Vec::new();
    // Barrier removal.
    let before = gates.len();
    gates.retain(|g| !matches!(g, QuantumGate::Barrier));
    if gates.len() != before {
        removed += before - gates.len();
        rules.push("remove-barrier".to_string());
    }
    // Adjacent inverse cancellation for X and H.
    let mut out: Vec<QuantumGate> = Vec::with_capacity(gates.len());
    for g in gates.drain(..) {
        if let Some(last) = out.last() {
            let cancel = matches!(
                (last, &g),
                (QuantumGate::PauliX, QuantumGate::PauliX)
                    | (QuantumGate::Hadamard, QuantumGate::Hadamard)
            );
            if cancel {
                out.pop();
                removed += 2;
                rules.push("cancel-inverse-pair".to_string());
                continue;
            }
        }
        out.push(g);
    }
    *gates = out;
    OptimizationReport {
        removed,
        rules_applied: rules,
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_cancel_hh() {
        let mut gates = vec![QuantumGate::Hadamard, QuantumGate::Hadamard];
        let r = optimize_circuit(&mut gates);
        assert!(gates.is_empty());
        assert_eq!(r.removed, 2);
    }
    #[test]
    fn test_remove_barrier() {
        let mut gates = vec![QuantumGate::Barrier, QuantumGate::PauliX];
        let r = optimize_circuit(&mut gates);
        assert_eq!(gates.len(), 1);
        assert_eq!(r.removed, 1);
    }
}
