use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};
use uuid::Uuid;

use crate::error::{IdentityError, Result};

/// Configuration for session management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionConfig {
    /// Session lifetime in seconds.
    pub session_lifetime_secs: u64,
    /// Maximum number of concurrent sessions per user.
    pub max_sessions_per_user: usize,
    /// Whether to extend session on activity.
    pub extend_on_activity: bool,
}

impl Default for SessionConfig {
    fn default() -> Self {
        Self {
            session_lifetime_secs: 86400,
            max_sessions_per_user: 5,
            extend_on_activity: true,
        }
    }
}

/// An authenticated session.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub id: String,
    pub user_id: String,
    pub created_at: u64,
    pub expires_at: u64,
    pub last_activity: u64,
    pub roles: Vec<String>,
    pub is_active: bool,
    pub metadata: HashMap<String, String>,
}

impl Session {
    /// Create a new session for a user.
    pub fn new(user_id: &str, roles: Vec<String>, config: &SessionConfig) -> Self {
        let now = current_timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            user_id: user_id.to_string(),
            created_at: now,
            expires_at: now + config.session_lifetime_secs,
            last_activity: now,
            roles,
            is_active: true,
            metadata: HashMap::new(),
        }
    }

    /// Check if the session is expired.
    pub fn is_expired(&self) -> bool {
        if !self.is_active {
            return true;
        }
        current_timestamp() >= self.expires_at
    }

    /// Check if the session is valid (active and not expired).
    pub fn is_valid(&self) -> bool {
        self.is_active && !self.is_expired()
    }

    /// Extend the session lifetime.
    pub fn extend(&mut self, config: &SessionConfig) {
        let now = current_timestamp();
        self.last_activity = now;
        self.expires_at = now + config.session_lifetime_secs;
    }

    /// Record activity on this session.
    pub fn record_activity(&mut self) {
        self.last_activity = current_timestamp();
    }

    /// Invalidate the session.
    pub fn invalidate(&mut self) {
        self.is_active = false;
    }

    /// Add metadata to the session.
    pub fn add_metadata(&mut self, key: &str, value: &str) {
        self.metadata.insert(key.to_string(), value.to_string());
    }
}

/// Manages user sessions.
pub struct SessionManager {
    config: SessionConfig,
    sessions: HashMap<String, Session>,
    user_sessions: HashMap<String, Vec<String>>,
}

impl SessionManager {
    pub fn new(config: SessionConfig) -> Self {
        Self {
            config,
            sessions: HashMap::new(),
            user_sessions: HashMap::new(),
        }
    }

    /// Create a new session for a user.
    pub fn create_session(&mut self, user_id: &str, roles: Vec<String>) -> Result<Session> {
        // Check session limit
        let user_session_count = self
            .user_sessions
            .get(user_id)
            .map(|s| s.len())
            .unwrap_or(0);

        if user_session_count >= self.config.max_sessions_per_user {
            return Err(IdentityError::SessionNotFound(format!(
                "maximum sessions ({}) reached for user {}",
                self.config.max_sessions_per_user, user_id
            )));
        }

        let session = Session::new(user_id, roles, &self.config);
        let session_id = session.id.clone();

        self.sessions.insert(session_id.clone(), session.clone());
        self.user_sessions
            .entry(user_id.to_string())
            .or_default()
            .push(session_id);

        Ok(session)
    }

    /// Get a session by ID.
    pub fn get_session(&self, session_id: &str) -> Option<&Session> {
        self.sessions.get(session_id)
    }

    /// Get a mutable reference to a session.
    pub fn get_session_mut(&mut self, session_id: &str) -> Option<&mut Session> {
        self.sessions.get_mut(session_id)
    }

    /// Validate a session (check existence and expiration).
    pub fn validate_session(&self, session_id: &str) -> Result<&Session> {
        let session = self
            .sessions
            .get(session_id)
            .ok_or_else(|| IdentityError::SessionNotFound(session_id.to_string()))?;

        if session.is_expired() {
            return Err(IdentityError::SessionExpired);
        }

        Ok(session)
    }

    /// Invalidate a session.
    pub fn invalidate_session(&mut self, session_id: &str) -> Result<()> {
        let session = self
            .sessions
            .get_mut(session_id)
            .ok_or_else(|| IdentityError::SessionNotFound(session_id.to_string()))?;

        session.invalidate();

        // Remove from user sessions
        if let Some(user_sessions) = self.user_sessions.get_mut(&session.user_id) {
            user_sessions.retain(|id| id != session_id);
        }

        Ok(())
    }

    /// Invalidate all sessions for a user.
    pub fn invalidate_user_sessions(&mut self, user_id: &str) -> usize {
        let session_ids: Vec<String> = self.user_sessions.get(user_id).cloned().unwrap_or_default();

        let mut count = 0;
        for session_id in session_ids {
            if let Some(session) = self.sessions.get_mut(&session_id) {
                session.invalidate();
                count += 1;
            }
        }

        self.user_sessions.remove(user_id);
        count
    }

    /// Get all active sessions for a user.
    pub fn get_user_sessions(&self, user_id: &str) -> Vec<&Session> {
        self.user_sessions
            .get(user_id)
            .map(|ids| {
                ids.iter()
                    .filter_map(|id| self.sessions.get(id))
                    .filter(|s| s.is_valid())
                    .collect()
            })
            .unwrap_or_default()
    }

    /// Clean up expired sessions.
    pub fn cleanup_expired(&mut self) -> usize {
        let expired_ids: Vec<String> = self
            .sessions
            .iter()
            .filter(|(_, s)| s.is_expired())
            .map(|(id, _)| id.clone())
            .collect();

        let count = expired_ids.len();
        for id in expired_ids {
            if let Some(session) = self.sessions.remove(&id) {
                if let Some(user_sessions) = self.user_sessions.get_mut(&session.user_id) {
                    user_sessions.retain(|sid| sid != &id);
                }
            }
        }

        count
    }

    /// Get the total number of sessions.
    pub fn session_count(&self) -> usize {
        self.sessions.len()
    }

    /// Get the number of active sessions.
    pub fn active_session_count(&self) -> usize {
        self.sessions.values().filter(|s| s.is_valid()).count()
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new(SessionConfig::default())
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
    fn test_session_creation() {
        let config = SessionConfig::default();
        let session = Session::new("user123", vec!["admin".to_string()], &config);

        assert_eq!(session.user_id, "user123");
        assert!(session.is_active);
        assert!(session.is_valid());
        assert!(!session.is_expired());
    }

    #[test]
    fn test_session_expiration() {
        let config = SessionConfig {
            session_lifetime_secs: 0,
            ..Default::default()
        };
        let session = Session::new("user", vec![], &config);
        assert!(session.is_expired());
    }

    #[test]
    fn test_session_invalidation() {
        let config = SessionConfig::default();
        let mut session = Session::new("user", vec![], &config);
        assert!(session.is_valid());

        session.invalidate();
        assert!(!session.is_valid());
        assert!(session.is_expired());
    }

    #[test]
    fn test_session_manager_create() {
        let mut manager = SessionManager::new(SessionConfig::default());
        let session = manager
            .create_session("user1", vec!["user".to_string()])
            .unwrap();

        assert_eq!(session.user_id, "user1");
        assert_eq!(manager.session_count(), 1);
    }

    #[test]
    fn test_session_manager_validate() {
        let mut manager = SessionManager::new(SessionConfig::default());
        let session = manager.create_session("user1", vec![]).unwrap();

        let validated = manager.validate_session(&session.id);
        assert!(validated.is_ok());
    }

    #[test]
    fn test_session_manager_invalidate() {
        let mut manager = SessionManager::new(SessionConfig::default());
        let session = manager.create_session("user1", vec![]).unwrap();

        assert!(manager.invalidate_session(&session.id).is_ok());
        assert!(manager.validate_session(&session.id).is_err());
    }

    #[test]
    fn test_session_limit() {
        let mut manager = SessionManager::new(SessionConfig {
            max_sessions_per_user: 2,
            ..Default::default()
        });

        manager.create_session("user1", vec![]).unwrap();
        manager.create_session("user1", vec![]).unwrap();

        let result = manager.create_session("user1", vec![]);
        assert!(result.is_err());
    }

    #[test]
    fn test_cleanup_expired() {
        let mut manager = SessionManager::new(SessionConfig {
            session_lifetime_secs: 0,
            ..Default::default()
        });

        manager.create_session("user1", vec![]).unwrap();
        manager.create_session("user2", vec![]).unwrap();

        let cleaned = manager.cleanup_expired();
        assert_eq!(cleaned, 2);
    }

    #[test]
    fn test_user_sessions() {
        let mut manager = SessionManager::new(SessionConfig::default());

        manager.create_session("user1", vec![]).unwrap();
        manager.create_session("user1", vec![]).unwrap();
        manager.create_session("user2", vec![]).unwrap();

        let user1_sessions = manager.get_user_sessions("user1");
        assert_eq!(user1_sessions.len(), 2);
    }

    #[test]
    fn test_invalidate_all_user_sessions() {
        let mut manager = SessionManager::new(SessionConfig::default());

        manager.create_session("user1", vec![]).unwrap();
        manager.create_session("user1", vec![]).unwrap();

        let count = manager.invalidate_user_sessions("user1");
        assert_eq!(count, 2);
        assert_eq!(manager.active_session_count(), 0);
    }
}
