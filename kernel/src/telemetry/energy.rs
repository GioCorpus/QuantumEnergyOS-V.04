//! Energy/power telemetry primitives (§37-38). No prediction in kernel.

#[derive(Debug, Clone, Copy, Default)]
pub struct PowerTelemetry {
    pub microwatts: u64,
    pub millidegree_c: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DevicePowerState {
    On,
    Idle,
    Suspended,
    Off,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct EnergyCounter {
    pub microjoules: u64,
}

impl EnergyCounter {
    pub fn add(&mut self, uj: u64) {
        self.microjoules = self.microjoules.saturating_add(uj);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn saturates() {
        let mut c = EnergyCounter {
            microjoules: u64::MAX,
        };
        c.add(1);
        assert_eq!(c.microjoules, u64::MAX);
    }
}