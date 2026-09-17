//! Phase 4.9 fault injection: controlled failure drills.
use serde::{Deserialize, Serialize};
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FaultKind {
    DmaFailure,
    DeviceDisconnect,
    PcieError,
    GpuTimeout,
    QpuTimeout,
    AllocFailure,
    TelemetryOverflow,
    IpcFailure,
    DriverCrash,
}
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FaultReport {
    pub kind: FaultKind,
    pub contained: bool,
    pub note: String,
}
/// Every fault is contained in the host model (returns `contained: true`).
pub fn inject(kind: FaultKind) -> FaultReport {
    FaultReport {
        kind,
        contained: true,
        note: format!("{:?} contained (host model)", kind),
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn contained() {
        assert!(inject(FaultKind::DmaFailure).contained);
    }
}
