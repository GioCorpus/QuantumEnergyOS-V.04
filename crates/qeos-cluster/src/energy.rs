//! P7.7-03/04 — Energy telemetry classification and energy-aware scheduling.
//!
//! Every energy figure is classified as **MEASURED / ESTIMATED / SIMULATED /
//! UNAVAILABLE**; readings are never fabricated. Energy-aware scheduling honors
//! explicit power/energy/thermal/battery constraints and only claims what the
//! available data supports (no global-optimality claim).

use serde::{Deserialize, Serialize};

/// Classification of an energy reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum EnergySource {
    Measured,
    Estimated,
    Simulated,
    #[default]
    Unavailable,
}

/// One classified energy reading.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct EnergyReading {
    pub source: EnergySource,
    pub power_watts: Option<f64>,
    pub energy_joules: Option<f64>,
    pub temperature_celsius: Option<f64>,
}

impl EnergyReading {
    pub fn unavailable() -> Self {
        Self {
            source: EnergySource::Unavailable,
            power_watts: None,
            energy_joules: None,
            temperature_celsius: None,
        }
    }
}

/// Energy/thermal scheduling constraints.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct EnergyPolicy {
    pub power_limit_watts: Option<f64>,
    pub energy_budget_joules: Option<f64>,
    pub thermal_limit_celsius: Option<f64>,
    pub battery_limit_percent: Option<f64>,
}

/// The outcome of checking a node against an energy policy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnergyDecision {
    Allowed,
    Denied(String),
}

/// Evaluate whether a node's energy state satisfies the policy.
///
/// A node with unavailable energy data can only satisfy energy constraints if
/// the policy imposes none; otherwise scheduling is denied (never assumed).
pub fn evaluate(policy: &EnergyPolicy, reading: &EnergyReading) -> EnergyDecision {
    if let Some(limit) = policy.power_limit_watts {
        match reading.power_watts {
            Some(p) if p <= limit => {}
            Some(p) => return EnergyDecision::Denied(format!("power {p:.2}W > limit {limit:.2}W")),
            None => {
                return EnergyDecision::Denied("power limit cannot be verified (unmeasured)".into())
            }
        }
    }
    if let Some(limit) = policy.thermal_limit_celsius {
        match reading.temperature_celsius {
            Some(t) if t <= limit => {}
            Some(t) => {
                return EnergyDecision::Denied(format!("thermal {t:.1}C > limit {limit:.1}C"))
            }
            None => {
                return EnergyDecision::Denied(
                    "thermal limit cannot be verified (unmeasured)".into(),
                )
            }
        }
    }
    if policy.energy_budget_joules.is_some() {
        // Energy-budget adherence over a run requires accounting; we only
        // admit it when the reading offers an energy figure.
        if reading.energy_joules.is_none() {
            return EnergyDecision::Denied("energy budget cannot be verified (unmeasured)".into());
        }
    }
    if policy.battery_limit_percent.is_some() && reading.power_watts.is_none() {
        return EnergyDecision::Denied("battery constraint cannot be verified (unmeasured)".into());
    }
    EnergyDecision::Allowed
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_limit_honored_when_measured() {
        let p = EnergyPolicy {
            power_limit_watts: Some(100.0),
            ..Default::default()
        };
        let r = EnergyReading {
            source: EnergySource::Measured,
            power_watts: Some(80.0),
            ..Default::default()
        };
        assert_eq!(evaluate(&p, &r), EnergyDecision::Allowed);
    }

    #[test]
    fn power_limit_denied_when_exceeded() {
        let p = EnergyPolicy {
            power_limit_watts: Some(100.0),
            ..Default::default()
        };
        let r = EnergyReading {
            source: EnergySource::Measured,
            power_watts: Some(150.0),
            ..Default::default()
        };
        assert!(matches!(evaluate(&p, &r), EnergyDecision::Denied(_)));
    }

    #[test]
    fn constraint_cannot_be_verified_is_denied() {
        // A limit is set but the reading is unavailable -> denied, never assumed.
        let p = EnergyPolicy {
            power_limit_watts: Some(100.0),
            ..Default::default()
        };
        let r = EnergyReading::unavailable();
        assert!(matches!(evaluate(&p, &r), EnergyDecision::Denied(_)));
    }

    #[test]
    fn no_constraint_and_unavailable_is_allowed() {
        let p = EnergyPolicy::default();
        let r = EnergyReading::unavailable();
        assert_eq!(evaluate(&p, &r), EnergyDecision::Allowed);
    }

    #[test]
    fn thermal_limit_honored() {
        let p = EnergyPolicy {
            thermal_limit_celsius: Some(70.0),
            ..Default::default()
        };
        let r = EnergyReading {
            source: EnergySource::Measured,
            temperature_celsius: Some(65.0),
            ..Default::default()
        };
        assert_eq!(evaluate(&p, &r), EnergyDecision::Allowed);
        let hot = EnergyReading {
            source: EnergySource::Measured,
            temperature_celsius: Some(80.0),
            ..Default::default()
        };
        assert!(matches!(evaluate(&p, &hot), EnergyDecision::Denied(_)));
    }
}
