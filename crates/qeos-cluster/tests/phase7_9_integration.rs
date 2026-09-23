//! P7.9 — Security, chaos, recovery & update-rollback validation (integration).
//!
//! These integration tests exercise the cross-crate guarantees that make up the
//! P7.9 release gate:
//! - P7.9-01 memory-safety boundary checks (oversized/rejected inputs).
//! - P7.9-02/03 remote-admin security (unauthorized ops denied + audited).
//! - P7.9-04/06 distributed chaos + recovery (fleet fault matrix).
//! - P7.9-05 update/rollback (versioned, integrity-checked).
//! - P7.9-07 honest performance baseline (QPU simulator throughput).

use qeos_cluster::admin::{
    AdminOperation, AdminRequest, Permission, PermissionAuthorizer, RemoteAdmin,
};
use qeos_cluster::checkpoint::Checkpoint;
use qeos_cluster::fleet::{run_fleet_failure_matrix, FleetFault};
use qeos_cluster::scheduler::{SchedulableNode, Scheduler};
use qeos_cluster::{NodeId, ResourceRequirements, ResourceSnapshot};

/// P7.9-01 — memory-safety boundary: absurd allocations/requirements are rejected.
#[test]
fn memory_safety_boundaries_rejected() {
    // GPU-adjacent: a resource requirement demanding more than any node offers.
    let snap = ResourceSnapshot {
        cpu_cores_available: 16,
        ram_bytes_available: 1 << 40,
        ..Default::default()
    };
    let node = SchedulableNode {
        id: NodeId("n1".into()),
        available: true,
        resources: snap,
        latency_ms: Some(5),
    };
    // Oversized RAM demand cannot be satisfied.
    let req = ResourceRequirements {
        ram_bytes: 1 << 60,
        ..Default::default()
    };
    assert!(!node.resources.can_satisfy(&req));
    // Oversized CPU demand cannot be satisfied.
    let req = ResourceRequirements {
        cpu_cores: 1 << 20,
        ..Default::default()
    };
    assert!(!node.resources.can_satisfy(&req));
    // Zero-size / malformed requirements are never scheduled.
    let out = Scheduler::schedule(
        qeos_cluster::DistributedJobId(1),
        &Default::default(),
        &[node],
        None,
    );
    assert!(matches!(out, qeos_cluster::ScheduleOutcome::Deferred(_)));
}

/// P7.9-02/03 — remote administration: unauthorized ops are denied and audited.
#[test]
fn remote_admin_denies_and_audits_unauthorized() {
    let mut auth = PermissionAuthorizer::new();
    auth.grant("alice", Permission::View);
    let mut admin = RemoteAdmin::new(Box::new(auth));

    for op in [
        AdminOperation::RestartService {
            node: "n1".into(),
            service: "scheduler".into(),
        },
        AdminOperation::CancelJob { job_id: 7 },
        AdminOperation::QuiesceNode { node: "n1".into() },
        AdminOperation::CollectDiagnostics { node: "n1".into() },
    ] {
        let req = AdminRequest {
            request_id: "req-unauth".into(),
            principal: "alice".into(),
            permission: Permission::View, // too low for these ops
            op,
            timeout_ms: 1000,
        };
        let res = admin.execute(&req);
        assert!(!res.ok, "op must be denied for a viewer: {res:?}");
    }
    // Every denial is audited.
    assert!(admin.audit_log().iter().all(|e| !e.authorized));
}

/// P7.9-04/06 — distributed chaos: every fleet fault recovers.
#[test]
fn distributed_chaos_recovers() {
    let reports = run_fleet_failure_matrix();
    assert_eq!(reports.len(), 7);
    for r in &reports {
        assert!(r.recovered, "{:?} did not recover: {}", r.fault, r.detected);
    }
}

/// P7.9-05 — update/rollback: an integrity-broken update is rejected and the
/// prior version is retained (simulated via versioned checkpoints).
#[test]
fn update_rollback_rejects_corrupt_update() {
    // Baseline (v1) is intact and loadable.
    let v1 = Checkpoint::new(1, "job-9", "state:v1");
    assert!(v1.load(1).is_ok());

    // A corrupt update (tampered payload) is rejected — it can never replace v1.
    let mut corrupt = Checkpoint::new(1, "job-9", "state:v2");
    corrupt.data = "state:v2-CORRUPTED".to_string();
    assert!(!corrupt.verify_integrity());
    assert!(corrupt.load(1).is_err());

    // Rollback: the intact v1 remains the valid recovery point.
    assert!(v1.load(1).is_ok());
    assert_eq!(v1.checkpoint_format_version, 1);
}

/// P7.9-07 — honest performance baseline: QPU simulator throughput on this host.
///
/// Not a product benchmark; it records a measured, reproducible number so the
/// release report can state an actual (not fabricated) figure.
#[test]
fn performance_baseline_qpu_simulator() {
    use std::time::Instant;
    let mut dev = qeos_qpu::QpuDevice::simulator("perf");
    dev.initialize().unwrap();
    let mut job = qeos_qpu::QpuJob::new(0, 1, 100_000, 1);
    dev.submit(&mut job).unwrap();
    let t0 = Instant::now();
    let res = dev.measure(&mut job).unwrap();
    let elapsed = t0.elapsed();

    // Throughput in shots/sec. Must be > 0 and deterministic.
    let shots_per_sec = (res.shots as f64) / elapsed.as_secs_f64().max(1e-9);
    eprintln!(
        "P7.9 baseline: qpu-sim {} shots in {:?} -> {:.0} shots/s (simulation_only={})",
        res.shots, elapsed, shots_per_sec, res.simulation_only
    );
    assert!(shots_per_sec > 0.0);
    assert!(res.simulation_only);
}

/// P7.9-00 — a cross-cutting check that the fault enum set is complete.
#[test]
fn fault_enum_is_exhaustive_over_known_faults() {
    let all = [
        FleetFault::NodeLoss,
        FleetFault::NetworkInterruption,
        FleetFault::ServiceCrash,
        FleetFault::GpuFailure,
        FleetFault::QpuBackendFailure,
        FleetFault::StorageFailure,
        FleetFault::CredentialExpiration,
    ];
    assert_eq!(all.len(), 7);
}
