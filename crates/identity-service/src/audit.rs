use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::error::{IdentityError, Result};

/// Severity level for audit events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// An audit event record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditEvent {
    pub id: String,
    pub timestamp: u64,
    pub severity: AuditSeverity,
    pub actor: String,
    pub action: String,
    pub resource: String,
    pub result: AuditResult,
    pub details: Option<String>,
    pub session_id: Option<String>,
    pub ip_address: Option<String>,
}

/// Result of an audited action.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AuditResult {
    Success,
    Failure,
    Denied,
    Error,
}

impl AuditEvent {
    pub fn new(actor: &str, action: &str, resource: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            timestamp: current_timestamp(),
            severity: AuditSeverity::Info,
            actor: actor.to_string(),
            action: action.to_string(),
            resource: resource.to_string(),
            result: AuditResult::Success,
            details: None,
            session_id: None,
            ip_address: None,
        }
    }

    pub fn with_severity(mut self, severity: AuditSeverity) -> Self {
        self.severity = severity;
        self
    }

    pub fn with_result(mut self, result: AuditResult) -> Self {
        self.result = result;
        self
    }

    pub fn with_details(mut self, details: &str) -> Self {
        self.details = Some(details.to_string());
        self
    }

    pub fn with_session(mut self, session_id: &str) -> Self {
        self.session_id = Some(session_id.to_string());
        self
    }

    pub fn with_ip(mut self, ip: &str) -> Self {
        self.ip_address = Some(ip.to_string());
        self
    }
}

/// Audit logger for security-relevant events.
pub struct AuditLogger {
    events: VecDeque<AuditEvent>,
    max_events: usize,
}

impl AuditLogger {
    pub fn new(max_events: usize) -> Self {
        Self {
            events: VecDeque::with_capacity(max_events),
            max_events,
        }
    }

    /// Log an audit event.
    pub fn log(&mut self, event: AuditEvent) {
        if self.events.len() >= self.max_events {
            self.events.pop_front();
        }
        self.events.push_back(event);
    }

    /// Log a successful action.
    pub fn log_success(&mut self, actor: &str, action: &str, resource: &str) {
        let event = AuditEvent::new(actor, action, resource).with_result(AuditResult::Success);
        self.log(event);
    }

    /// Log a failed action.
    pub fn log_failure(&mut self, actor: &str, action: &str, resource: &str, details: &str) {
        let event = AuditEvent::new(actor, action, resource)
            .with_result(AuditResult::Failure)
            .with_severity(AuditSeverity::Warning)
            .with_details(details);
        self.log(event);
    }

    /// Log a denied action.
    pub fn log_denied(&mut self, actor: &str, action: &str, resource: &str) {
        let event = AuditEvent::new(actor, action, resource)
            .with_result(AuditResult::Denied)
            .with_severity(AuditSeverity::Warning);
        self.log(event);
    }

    /// Get all events.
    pub fn events(&self) -> &VecDeque<AuditEvent> {
        &self.events
    }

    /// Get events for a specific actor.
    pub fn events_for_actor(&self, actor: &str) -> Vec<&AuditEvent> {
        self.events.iter().filter(|e| e.actor == actor).collect()
    }

    /// Get events with a specific severity or higher.
    pub fn events_with_severity(&self, min_severity: AuditSeverity) -> Vec<&AuditEvent> {
        self.events
            .iter()
            .filter(|e| e.severity as u8 >= min_severity as u8)
            .collect()
    }

    /// Get the number of events.
    pub fn len(&self) -> usize {
        self.events.len()
    }

    /// Check if there are no events.
    pub fn is_empty(&self) -> bool {
        self.events.is_empty()
    }

    /// Clear all events.
    pub fn clear(&mut self) {
        self.events.clear();
    }

    /// Serialize all events to JSON.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(&self.events).map_err(|e| IdentityError::AuditError(e.to_string()))
    }
}

impl Default for AuditLogger {
    fn default() -> Self {
        Self::new(10000)
    }
}

/// Get the current timestamp in seconds.
fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_audit_event_creation() {
        let event = AuditEvent::new("user1", "login", "auth");
        assert_eq!(event.actor, "user1");
        assert_eq!(event.action, "login");
        assert_eq!(event.resource, "auth");
        assert_eq!(event.result, AuditResult::Success);
    }

    #[test]
    fn test_audit_event_builder() {
        let event = AuditEvent::new("user1", "delete", "quantum")
            .with_severity(AuditSeverity::Critical)
            .with_result(AuditResult::Denied)
            .with_details("insufficient permissions")
            .with_session("session123")
            .with_ip("192.168.1.1");

        assert_eq!(event.severity, AuditSeverity::Critical);
        assert_eq!(event.result, AuditResult::Denied);
        assert_eq!(event.details, Some("insufficient permissions".to_string()));
        assert_eq!(event.session_id, Some("session123".to_string()));
        assert_eq!(event.ip_address, Some("192.168.1.1".to_string()));
    }

    #[test]
    fn test_audit_logger() {
        let mut logger = AuditLogger::new(100);

        logger.log_success("user1", "login", "auth");
        logger.log_failure("user2", "login", "auth", "wrong password");
        logger.log_denied("user3", "delete", "quantum");

        assert_eq!(logger.len(), 3);
    }

    #[test]
    fn test_events_for_actor() {
        let mut logger = AuditLogger::new(100);

        logger.log_success("user1", "login", "auth");
        logger.log_success("user2", "login", "auth");
        logger.log_success("user1", "logout", "auth");

        let user1_events = logger.events_for_actor("user1");
        assert_eq!(user1_events.len(), 2);
    }

    #[test]
    fn test_events_with_severity() {
        let mut logger = AuditLogger::new(100);

        logger.log(AuditEvent::new("user1", "login", "auth").with_severity(AuditSeverity::Info));
        logger
            .log(AuditEvent::new("user2", "delete", "data").with_severity(AuditSeverity::Warning));
        logger.log(
            AuditEvent::new("user3", "breach", "system").with_severity(AuditSeverity::Critical),
        );

        let warnings = logger.events_with_severity(AuditSeverity::Warning);
        assert_eq!(warnings.len(), 2);
    }

    #[test]
    fn test_max_events() {
        let mut logger = AuditLogger::new(3);

        for i in 0..5 {
            logger.log_success(&format!("user{}", i), "action", "resource");
        }

        assert_eq!(logger.len(), 3);
    }

    #[test]
    fn test_clear() {
        let mut logger = AuditLogger::new(100);
        logger.log_success("user1", "login", "auth");

        logger.clear();
        assert!(logger.is_empty());
    }

    #[test]
    fn test_serialization() {
        let mut logger = AuditLogger::new(100);
        logger.log_success("user1", "login", "auth");

        let json = logger.to_json().unwrap();
        assert!(json.contains("user1"));
        assert!(json.contains("login"));
    }

    #[test]
    fn test_audit_result_equality() {
        assert_eq!(AuditResult::Success, AuditResult::Success);
        assert_ne!(AuditResult::Success, AuditResult::Failure);
    }
}
