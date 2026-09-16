use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash as ArgonPasswordHash, PasswordHasher, PasswordVerifier, SaltString,
    },
    Argon2,
};
use serde::{Deserialize, Serialize};

use crate::error::{IdentityError, Result};

/// Configuration for the authentication service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    /// Argon2id memory cost in KiB.
    pub memory_cost: u32,
    /// Argon2id time cost (iterations).
    pub time_cost: u32,
    /// Argon2id parallelism degree.
    pub parallelism: u32,
    /// Maximum password length.
    pub max_password_length: usize,
    /// Minimum password length.
    pub min_password_length: usize,
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            memory_cost: 65536,
            time_cost: 3,
            parallelism: 4,
            max_password_length: 128,
            min_password_length: 8,
        }
    }
}

/// A hashed password with its algorithm metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StoredPasswordHash {
    pub hash: String,
    pub algorithm: String,
}

/// Authentication service using Argon2id for password hashing.
pub struct AuthService {
    config: AuthConfig,
    argon2: Argon2<'static>,
}

impl AuthService {
    pub fn new(config: AuthConfig) -> Result<Self> {
        let params = argon2::Params::new(
            config.memory_cost,
            config.time_cost,
            config.parallelism,
            None,
        )
        .map_err(|e| IdentityError::PasswordHashingFailed(format!("invalid Argon2 params: {}", e)))?;

        let argon2 = Argon2::new(
            argon2::Algorithm::Argon2id,
            argon2::Version::V0x13,
            params,
        );

        Ok(Self { config, argon2 })
    }

    /// Hash a password using Argon2id.
    pub fn hash_password(&self, password: &str) -> Result<StoredPasswordHash> {
        self.validate_password(password)?;

        let salt = SaltString::generate(&mut OsRng);
        let hash = self
            .argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| IdentityError::PasswordHashingFailed(e.to_string()))?;

        Ok(StoredPasswordHash {
            hash: hash.to_string(),
            algorithm: "Argon2id".to_string(),
        })
    }

    /// Verify a password against a stored hash.
    pub fn verify_password(&self, password: &str, stored_hash: &str) -> Result<bool> {
        let parsed_hash = ArgonPasswordHash::new(stored_hash)
            .map_err(|e| IdentityError::PasswordHashingFailed(e.to_string()))?;

        match self.argon2.verify_password(password.as_bytes(), &parsed_hash) {
            Ok(()) => Ok(true),
            Err(argon2::password_hash::Error::Password) => Ok(false),
            Err(e) => Err(IdentityError::PasswordHashingFailed(e.to_string())),
        }
    }

    /// Validate password meets policy requirements.
    pub fn validate_password(&self, password: &str) -> Result<()> {
        if password.len() < self.config.min_password_length {
            return Err(IdentityError::PasswordHashingFailed(format!(
                "password must be at least {} characters",
                self.config.min_password_length
            )));
        }

        if password.len() > self.config.max_password_length {
            return Err(IdentityError::PasswordHashingFailed(format!(
                "password must not exceed {} characters",
                self.config.max_password_length
            )));
        }

        Ok(())
    }

    /// Check if a password hash needs rehashing (e.g., parameters changed).
    pub fn needs_rehash(&self, stored_hash: &str) -> Result<bool> {
        let parsed_hash = ArgonPasswordHash::new(stored_hash)
            .map_err(|e| IdentityError::PasswordHashingFailed(e.to_string()))?;

        Ok(parsed_hash.hash.is_none() || parsed_hash.salt.is_none())
    }
}

impl Default for AuthService {
    fn default() -> Self {
        Self::new(AuthConfig::default()).expect("default config is valid")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_password() {
        let service = AuthService::new(AuthConfig::default()).unwrap();
        let hash = service.hash_password("secure_password_123").unwrap();

        assert_eq!(hash.algorithm, "Argon2id");
        assert!(!hash.hash.is_empty());
        assert!(hash.hash.starts_with("$argon2id$"));
    }

    #[test]
    fn test_verify_password_correct() {
        let service = AuthService::new(AuthConfig::default()).unwrap();
        let hash = service.hash_password("secure_password_123").unwrap();

        assert!(service.verify_password("secure_password_123", &hash.hash).unwrap());
    }

    #[test]
    fn test_verify_password_incorrect() {
        let service = AuthService::new(AuthConfig::default()).unwrap();
        let hash = service.hash_password("secure_password_123").unwrap();

        assert!(!service.verify_password("wrong_password", &hash.hash).unwrap());
    }

    #[test]
    fn test_password_validation() {
        let service = AuthService::new(AuthConfig::default()).unwrap();

        assert!(service.validate_password("short").is_err());
        assert!(service.validate_password("valid_password").is_ok());
    }

    #[test]
    fn test_password_too_long() {
        let service = AuthService::new(AuthConfig {
            max_password_length: 20,
            ..Default::default()
        }).unwrap();

        assert!(service.validate_password("this_password_is_way_too_long").is_err());
    }

    #[test]
    fn test_different_passwords_different_hashes() {
        let service = AuthService::new(AuthConfig::default()).unwrap();
        let hash1 = service.hash_password("password123").unwrap();
        let hash2 = service.hash_password("password123").unwrap();

        assert_ne!(hash1.hash, hash2.hash);
    }
}
