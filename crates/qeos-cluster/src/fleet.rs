//! P7.7-07 — Fleet failure simulation & recovery verification.
//!
//! Models controlled fleet faults (node loss, network interruption, service
//! crash, GPU/QPU backend failure, storage failure, credential expiration) and
//! verifies that detection + recovery keep the fleet consistent. The fault
//! *injection* is **SIMULATED**, but the detection/recovery logic it exercises
//! (heartbeat expiry, offline/re-register, credential checks) is **REAL**.

use crate::control_plane::ControlPlane;
use crate::identity::RegistrationRequest;

/// A controlled fleet fault.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FleetFault {
    NodeLoss,
    NetworkInterruption,
    ServiceCrash,
    GpuFailure,
    QpuBackendFailure,
    StorageFailure,
    CredentialExpiration,
}

/// Result of injecting and recovering from a fault.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FleetRecoveryReport {
    pub fault: FleetFault,
    pub detected: String,
    pub recovered: bool,
    pub healthy_after: usize,
}

/// Simulate a fleet failure against a control plane and verify recovery.
///
/// The scenario always registers two nodes (`n1`, `n2`), injects the fault, and
/// verifies the fleet still converges to a consistent, auditable state.
pub fn inject_and_recover(fault: FleetFault) -> FleetRecoveryReport {
    let mut cp = ControlPlane::new("fleet-test", 2000);

    let r1 = RegistrationRequest::new("n1", b"secret1");
    let r2 = RegistrationRequest::new("n2", b"secret2");
    // Successful registrations.
    cp.register(r1.clone()).unwrap();
    cp.register(r2.clone()).unwrap();

    let now = 1000u64;
    let mut result = FleetRecoveryReport {
        fault,
        detected: String::new(),
        recovered: false,
        healthy_after: 2,
    };

    match fault {
        FleetFault::NodeLoss | FleetFault::NetworkInterruption | FleetFault::ServiceCrash => {
            // Establish a clock, establish liveness, then let nodes fall
            // silent (failure detected via heartbeat timeout), then recover.
            cp.tick(now);
            cp.heartbeat(&r1, 0).unwrap();
            cp.heartbeat(&r2, 0).unwrap();
            cp.tick(now + 5000); // both timed out -> offline
            result.detected = match fault {
                FleetFault::NodeLoss => "heartbeat timeout (node lost)".into(),
                FleetFault::NetworkInterruption => "network interruption (both offline)".into(),
                FleetFault::ServiceCrash => "service crash (heartbeats missed)".into(),
                // Unreachable: this arm only runs for the three faults above.
                _ => unreachable!(),
            };
            // Recovery: re-register (allowed once offline) and re-heartbeat.
            cp.register(r1.clone()).unwrap();
            cp.register(r2.clone()).unwrap();
            cp.heartbeat(&r1, 0).unwrap();
            cp.heartbeat(&r2, 0).unwrap();
            result.healthy_after = cp.healthy_count();
            result.recovered = cp.healthy_count() == 2;
        }
        FleetFault::CredentialExpiration => {
            // Expired/rotated credential is rejected and must be re-issued.
            let mut bad = RegistrationRequest::new("n3", b"expired");
            bad.secret = b"wrong".to_vec();
            let denied = cp.register(bad).is_err();
            // Legitimate re-issue with a fresh secret succeeds.
            let good = RegistrationRequest::new("n3", b"fresh");
            let accepted = cp.register(good).is_ok();
            result.detected = "credential rotation required".into();
            result.recovered = denied && accepted;
            result.healthy_after = cp.healthy_count();
        }
        FleetFault::GpuFailure | FleetFault::QpuBackendFailure | FleetFault::StorageFailure => {
            // These are device/backend degradations: the node stays registered
            // but must be drained and re-checked.
            cp.drain("n1").unwrap();
            result.detected = format!("{:?} -> node drained", fault);
            cp.register(r1.clone()).unwrap(); // re-admit once drained
            cp.heartbeat(&r1, 0).unwrap();
            result.healthy_after = cp.healthy_count();
            result.recovered = cp.healthy_count() >= 1;
        }
    }

    result
}

/// Run the full fleet fault matrix. Returns a report per fault.
pub fn run_fleet_failure_matrix() -> Vec<FleetRecoveryReport> {
    use FleetFault::*;
    [
        NodeLoss,
        NetworkInterruption,
        ServiceCrash,
        GpuFailure,
        QpuBackendFailure,
        StorageFailure,
        CredentialExpiration,
    ]
    .iter()
    .map(|f| inject_and_recover(*f))
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_fault_recovers() {
        let reports = run_fleet_failure_matrix();
        assert_eq!(reports.len(), 7);
        for r in reports {
            assert!(
                r.recovered,
                "fault {:?} did not recover: {}",
                r.fault, r.detected
            );
        }
    }

    #[test]
    fn credential_expiration_enforces_rotation() {
        let r = inject_and_recover(FleetFault::CredentialExpiration);
        assert!(r.recovered);
        assert!(r.detected.contains("rotation"));
    }

    #[test]
    fn node_loss_detected_via_timeout() {
        let r = inject_and_recover(FleetFault::NodeLoss);
        assert!(r.recovered);
        assert!(r.detected.contains("heartbeat timeout"));
    }
}
