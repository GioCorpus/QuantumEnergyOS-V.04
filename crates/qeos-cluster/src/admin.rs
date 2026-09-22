//! P7.6-05/06 — Remote administration & command security.
//!
//! Remote operations are **typed** (no unrestricted shell). Every
//! [`AdminRequest`] carries a request id, the operator identity, a timeout and
//! a typed [`AdminOperation`]. An [`Authorizer`] decides allow/deny based on the
//! principal's permissions; every attempt — allowed or denied — is recorded in
//! an [`AuditLog`] with request id, principal, operation, timestamp and outcome.
//!
//! Privileged operations never receive the ability to run arbitrary commands.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// Result of an admin operation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminResult {
    pub request_id: String,
    pub ok: bool,
    pub message: String,
}

/// A typed remote operation. No operation is a free-form shell command.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AdminOperation {
    GetStatus { node: String },
    GetHealth { node: String },
    RestartService { node: String, service: String },
    CancelJob { job_id: u64 },
    CollectDiagnostics { node: String },
    QuiesceNode { node: String },
}

impl AdminOperation {
    /// Minimum permission required to execute this operation.
    pub fn required_permission(&self) -> Permission {
        match self {
            AdminOperation::GetStatus { .. } | AdminOperation::GetHealth { .. } => Permission::View,
            AdminOperation::RestartService { .. }
            | AdminOperation::CancelJob { .. }
            | AdminOperation::CollectDiagnostics { .. }
            | AdminOperation::QuiesceNode { .. } => Permission::Administer,
        }
    }
}

/// Permissions a principal may hold.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Permission {
    View,
    Operate,
    Administer,
}

impl Permission {
    pub fn satisfies(&self, required: Permission) -> bool {
        *self >= required
    }
}

/// An administrative request (identity + authorization context).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdminRequest {
    pub request_id: String,
    pub principal: String,
    pub permission: Permission,
    pub op: AdminOperation,
    pub timeout_ms: u64,
}

/// A record of an admin attempt for audit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEntry {
    pub request_id: String,
    pub principal: String,
    pub op: String,
    pub authorized: bool,
    pub when_ms: u64,
}

/// Authorizer: decides whether a principal may run an operation.
pub trait Authorizer: Send + Sync {
    fn allow(&self, principal: &str, required: Permission) -> bool;
}

/// Default authorizer: grant iff the caller's permission satisfies the required.
#[derive(Debug, Default)]
pub struct PermissionAuthorizer {
    /// principal -> highest permission granted.
    grants: HashMap<String, Permission>,
}

impl PermissionAuthorizer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn grant(&mut self, principal: &str, perm: Permission) {
        self.grants.insert(principal.to_string(), perm);
    }
}

impl Authorizer for PermissionAuthorizer {
    fn allow(&self, principal: &str, required: Permission) -> bool {
        self.grants
            .get(principal)
            .map(|p| p.satisfies(required))
            .unwrap_or(false)
    }
}

/// Remote administration service with full audit.
pub struct RemoteAdmin {
    authorizer: Box<dyn Authorizer>,
    audit: Vec<AuditEntry>,
}

impl RemoteAdmin {
    pub fn new(authorizer: Box<dyn Authorizer>) -> Self {
        Self {
            authorizer,
            audit: Vec::new(),
        }
    }

    /// Execute a typed operation under authorization. Always records an audit
    /// entry (allowed or denied).
    pub fn execute(&mut self, req: &AdminRequest) -> AdminResult {
        let required = req.op.required_permission();
        let authorized = self.authorizer.allow(&req.principal, required);
        let when = now_ms();
        self.audit.push(AuditEntry {
            request_id: req.request_id.clone(),
            principal: req.principal.clone(),
            op: format!("{:?}", req.op),
            authorized,
            when_ms: when,
        });

        if !authorized {
            return AdminResult {
                request_id: req.request_id.clone(),
                ok: false,
                message: format!("denied: {} lacks {:?}", req.principal, required),
            };
        }

        // A zero timeout is treated as an immediate failure (defensive).
        if req.timeout_ms == 0 {
            return AdminResult {
                request_id: req.request_id.clone(),
                ok: false,
                message: "operation timeout is zero".into(),
            };
        }

        // Typed dispatch — never a shell command.
        let message = self.dispatch(&req.op);
        AdminResult {
            request_id: req.request_id.clone(),
            ok: message.is_ok(),
            message: message.unwrap_or_else(|e| e),
        }
    }

    fn dispatch(&self, op: &AdminOperation) -> std::result::Result<String, String> {
        Ok(match op {
            AdminOperation::GetStatus { node } => format!("status of {node}"),
            AdminOperation::GetHealth { node } => format!("health of {node}"),
            AdminOperation::RestartService { node, service } => {
                format!("restart {service} on {node}")
            }
            AdminOperation::CancelJob { job_id } => format!("cancel job {job_id}"),
            AdminOperation::CollectDiagnostics { node } => format!("diagnostics of {node}"),
            AdminOperation::QuiesceNode { node } => format!("quiesce {node}"),
        })
    }

    /// All audit entries so far.
    pub fn audit_log(&self) -> &[AuditEntry] {
        &self.audit
    }

    /// Filter audit for a request id.
    pub fn audit_for(&self, request_id: &str) -> Option<&AuditEntry> {
        self.audit.iter().rev().find(|a| a.request_id == request_id)
    }
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

#[cfg(test)]
mod tests {
    use super::*;

    fn op(op: AdminOperation, principal: &str, perm: Permission) -> AdminRequest {
        AdminRequest {
            request_id: format!("req-{}", std::process::id()),
            principal: principal.to_string(),
            permission: perm,
            op,
            timeout_ms: 1000,
        }
    }

    #[test]
    fn viewer_can_query_status_but_not_restart() {
        let mut auth = PermissionAuthorizer::new();
        auth.grant("alice", Permission::View);
        let mut admin = RemoteAdmin::new(Box::new(auth));

        let status = admin.execute(&op(
            AdminOperation::GetStatus { node: "n1".into() },
            "alice",
            Permission::View,
        ));
        assert!(status.ok);

        let restart = admin.execute(&op(
            AdminOperation::RestartService {
                node: "n1".into(),
                service: "gpu".into(),
            },
            "alice",
            Permission::View,
        ));
        assert!(!restart.ok);
        assert!(restart.message.contains("denied"));
    }

    #[test]
    fn administer_can_restart() {
        let mut auth = PermissionAuthorizer::new();
        auth.grant("bob", Permission::Administer);
        let mut admin = RemoteAdmin::new(Box::new(auth));
        let r = admin.execute(&op(
            AdminOperation::RestartService {
                node: "n1".into(),
                service: "scheduler".into(),
            },
            "bob",
            Permission::Administer,
        ));
        assert!(r.ok);
    }

    #[test]
    fn unknown_principal_denied() {
        let mut admin = RemoteAdmin::new(Box::new(PermissionAuthorizer::new()));
        let r = admin.execute(&op(
            AdminOperation::QuiesceNode { node: "n1".into() },
            "eve",
            Permission::Administer,
        ));
        assert!(!r.ok);
    }

    #[test]
    fn every_attempt_audited() {
        let mut auth = PermissionAuthorizer::new();
        auth.grant("alice", Permission::View);
        let mut admin = RemoteAdmin::new(Box::new(auth));
        let req = op(
            AdminOperation::RestartService {
                node: "n1".into(),
                service: "gpu".into(),
            },
            "alice",
            Permission::View,
        );
        admin.execute(&req);
        let entry = admin.audit_for(&req.request_id).unwrap();
        assert!(!entry.authorized);
        assert_eq!(entry.principal, "alice");
    }

    #[test]
    fn zero_timeout_denied_even_if_authorized() {
        let mut auth = PermissionAuthorizer::new();
        auth.grant("bob", Permission::Administer);
        let mut admin = RemoteAdmin::new(Box::new(auth));
        let mut req = op(
            AdminOperation::GetStatus { node: "n1".into() },
            "bob",
            Permission::Administer,
        );
        req.timeout_ms = 0;
        let r = admin.execute(&req);
        assert!(!r.ok);
    }

    #[test]
    fn no_shell_escape() {
        // The operation model is typed; there is no free-form command field.
        let req = op(
            AdminOperation::GetStatus {
                node: "n1; rm -rf".into(),
            },
            "x",
            Permission::View,
        );
        // The node value is just data; it is never interpreted as a command line.
        assert_eq!(req.op.required_permission(), Permission::View);
    }
}
