//! P7.4-07 — QPU hardware readiness contract.
//!
//! Defines the exact requirements a physical QPU integration must satisfy
//! before it can be used. No undocumented register map, PCIe protocol, firmware
//! API, or vendor command is invented. Physical (Majorana/vendor) hardware is
//! **UNAVAILABLE** until a validated adapter with all required capabilities
//! exists.

use serde::{Deserialize, Serialize};

/// The requirements a vendor/physical QPU adapter must satisfy to be usable.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HardwareReadinessContract {
    pub device_discovery: bool,
    pub transport: bool,
    pub authentication: bool,
    pub command_interface: bool,
    pub measurement_interface: bool,
    pub telemetry: bool,
    pub calibration: bool,
    pub error_reporting: bool,
    pub firmware_provenance: bool,
    pub security: bool,
}

impl HardwareReadinessContract {
    /// Whether every required capability is satisfied.
    pub fn is_satisfied(&self) -> bool {
        self.device_discovery
            && self.transport
            && self.authentication
            && self.command_interface
            && self.measurement_interface
            && self.telemetry
            && self.calibration
            && self.error_reporting
            && self.firmware_provenance
            && self.security
    }
}

/// A documented, authoritative statement of the hardware-availability state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HardwareAvailabilityReport {
    /// e.g. "quantum-runtime Majorana simulator"
    pub simulator: String,
    /// e.g. "none"
    pub vendor_adapter: String,
    /// Always `false` until a validated adapter exists.
    pub physical_hardware_available: bool,
    /// List of documented interfaces required but not yet provided.
    pub outstanding_requirements: Vec<String>,
}

impl HardwareAvailabilityReport {
    /// Build the truthful current-state report.
    pub fn current() -> Self {
        Self {
            simulator: "quantum-runtime majorana-sim (SIMULATED)".into(),
            vendor_adapter: "none (no validated QpuVendorAdapter bundled)".into(),
            physical_hardware_available: false,
            outstanding_requirements: vec![
                "device discovery".into(),
                "transport".into(),
                "authentication".into(),
                "command interface".into(),
                "measurement interface".into(),
                "telemetry".into(),
                "calibration".into(),
                "error reporting".into(),
                "firmware provenance".into(),
                "security".into(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn contract_unsatisfied_by_default() {
        assert!(!HardwareReadinessContract::default().is_satisfied());
    }

    #[test]
    fn contract_satisfied_when_all_present() {
        let c = HardwareReadinessContract {
            device_discovery: true,
            transport: true,
            authentication: true,
            command_interface: true,
            measurement_interface: true,
            telemetry: true,
            calibration: true,
            error_reporting: true,
            firmware_provenance: true,
            security: true,
        };
        assert!(c.is_satisfied());
    }

    #[test]
    fn current_report_is_honest() {
        let r = HardwareAvailabilityReport::current();
        assert!(!r.physical_hardware_available);
        assert_eq!(r.outstanding_requirements.len(), 10);
    }
}
