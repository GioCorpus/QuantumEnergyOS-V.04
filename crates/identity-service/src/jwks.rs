use serde::{Deserialize, Serialize};

use crate::error::{IdentityError, Result};

/// A single JSON Web Key.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JsonWebKey {
    /// Key ID.
    pub kid: String,
    /// Key type (e.g., "RSA").
    pub kty: String,
    /// Algorithm (e.g., "RS256").
    pub alg: String,
    /// Usage ("sig" for signature, "enc" for encryption).
    #[serde(rename = "use")]
    pub key_use: String,
    /// RSA public key modulus (base64url).
    pub n: String,
    /// RSA public key exponent (base64url).
    pub e: String,
}

/// JSON Web Key Set.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Jwks {
    pub keys: Vec<JsonWebKey>,
}

impl Jwks {
    pub fn new() -> Self {
        Self { keys: Vec::new() }
    }

    pub fn add_key(&mut self, key: JsonWebKey) {
        self.keys.push(key);
    }

    pub fn find_key(&self, kid: &str) -> Option<&JsonWebKey> {
        self.keys.iter().find(|k| k.kid == kid)
    }

    pub fn is_empty(&self) -> bool {
        self.keys.is_empty()
    }

    pub fn len(&self) -> usize {
        self.keys.len()
    }

    /// Serialize to JSON.
    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| IdentityError::JwksError(e.to_string()))
    }

    /// Deserialize from JSON.
    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| IdentityError::JwksError(e.to_string()))
    }
}

/// Manages a JSON Web Key Set with key rotation support.
pub struct JwksManager {
    current_jwks: Jwks,
    current_key_id: String,
}

impl JwksManager {
    pub fn new() -> Self {
        Self {
            current_jwks: Jwks::new(),
            current_key_id: String::new(),
        }
    }

    /// Add a new key and set it as current.
    pub fn add_key(&mut self, kid: &str, n: &str, e: &str) {
        let key = JsonWebKey {
            kid: kid.to_string(),
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            key_use: "sig".to_string(),
            n: n.to_string(),
            e: e.to_string(),
        };

        self.current_jwks.add_key(key);
        self.current_key_id = kid.to_string();
    }

    /// Get the current JWKS.
    pub fn jwks(&self) -> &Jwks {
        &self.current_jwks
    }

    /// Get the current key ID.
    pub fn current_key_id(&self) -> &str {
        &self.current_key_id
    }

    /// Find a key by ID.
    pub fn find_key(&self, kid: &str) -> Option<&JsonWebKey> {
        self.current_jwks.find_key(kid)
    }

    /// Rotate keys: add a new key, keep old ones for verification.
    pub fn rotate_key(&mut self, new_kid: &str, n: &str, e: &str) {
        self.add_key(new_kid, n, e);
    }

    /// Remove a key (e.g., after rotation period expires).
    pub fn remove_key(&mut self, kid: &str) -> bool {
        let len_before = self.current_jwks.keys.len();
        self.current_jwks.keys.retain(|k| k.kid != kid);
        self.current_jwks.keys.len() < len_before
    }

    /// Get the JWKS as a JSON string for publishing.
    pub fn to_json(&self) -> Result<String> {
        self.current_jwks.to_json()
    }
}

impl Default for JwksManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_jwks_creation() {
        let jwks = Jwks::new();
        assert!(jwks.is_empty());
        assert_eq!(jwks.len(), 0);
    }

    #[test]
    fn test_add_key() {
        let mut jwks = Jwks::new();
        jwks.add_key(JsonWebKey {
            kid: "key1".to_string(),
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            key_use: "sig".to_string(),
            n: "modulus".to_string(),
            e: "AQAB".to_string(),
        });

        assert_eq!(jwks.len(), 1);
        assert!(jwks.find_key("key1").is_some());
    }

    #[test]
    fn test_find_key() {
        let mut jwks = Jwks::new();
        jwks.add_key(JsonWebKey {
            kid: "key1".to_string(),
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            key_use: "sig".to_string(),
            n: "modulus".to_string(),
            e: "AQAB".to_string(),
        });

        let key = jwks.find_key("key1");
        assert!(key.is_some());
        assert_eq!(key.unwrap().kid, "key1");

        assert!(jwks.find_key("nonexistent").is_none());
    }

    #[test]
    fn test_jwks_manager() {
        let mut manager = JwksManager::new();
        manager.add_key("key1", "modulus1", "AQAB");

        assert_eq!(manager.current_key_id(), "key1");
        assert_eq!(manager.jwks().len(), 1);
    }

    #[test]
    fn test_key_rotation() {
        let mut manager = JwksManager::new();
        manager.add_key("key1", "modulus1", "AQAB");
        manager.rotate_key("key2", "modulus2", "AQAB");

        assert_eq!(manager.current_key_id(), "key2");
        assert_eq!(manager.jwks().len(), 2);
    }

    #[test]
    fn test_remove_key() {
        let mut manager = JwksManager::new();
        manager.add_key("key1", "modulus1", "AQAB");
        manager.add_key("key2", "modulus2", "AQAB");

        assert!(manager.remove_key("key1"));
        assert_eq!(manager.jwks().len(), 1);
        assert!(!manager.remove_key("nonexistent"));
    }

    #[test]
    fn test_jwks_serialization() {
        let mut jwks = Jwks::new();
        jwks.add_key(JsonWebKey {
            kid: "key1".to_string(),
            kty: "RSA".to_string(),
            alg: "RS256".to_string(),
            key_use: "sig".to_string(),
            n: "modulus".to_string(),
            e: "AQAB".to_string(),
        });

        let json = jwks.to_json().unwrap();
        let deserialized = Jwks::from_json(&json).unwrap();

        assert_eq!(deserialized.len(), 1);
        assert_eq!(deserialized.keys[0].kid, "key1");
    }
}
