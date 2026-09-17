use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::error::{IdentityError, Result};

/// Configuration for JWT token management.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    /// Issuer claim.
    pub issuer: String,
    /// Access token lifetime in seconds.
    pub access_token_lifetime_secs: u64,
    /// Refresh token lifetime in seconds.
    pub refresh_token_lifetime_secs: u64,
    /// Algorithm to use (RS256).
    pub algorithm: Algorithm,
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            issuer: "quantumenergyos".to_string(),
            access_token_lifetime_secs: 3600,
            refresh_token_lifetime_secs: 86400,
            algorithm: Algorithm::RS256,
        }
    }
}

/// JWT claims structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtClaims {
    /// Subject (user ID).
    pub sub: String,
    /// Issuer.
    pub iss: String,
    /// Issued at timestamp.
    pub iat: u64,
    /// Expiration timestamp.
    pub exp: u64,
    /// Token type: "access" or "refresh".
    pub token_type: String,
    /// User roles.
    pub roles: Vec<String>,
    /// Session ID.
    pub session_id: Option<String>,
}

impl JwtClaims {
    /// Create new claims for an access token.
    pub fn new_access(user_id: &str, roles: Vec<String>, config: &JwtConfig) -> Self {
        let now = current_timestamp();
        Self {
            sub: user_id.to_string(),
            iss: config.issuer.clone(),
            iat: now,
            exp: now + config.access_token_lifetime_secs,
            token_type: "access".to_string(),
            roles,
            session_id: None,
        }
    }

    /// Create new claims for a refresh token.
    pub fn new_refresh(user_id: &str, session_id: &str, config: &JwtConfig) -> Self {
        let now = current_timestamp();
        Self {
            sub: user_id.to_string(),
            iss: config.issuer.clone(),
            iat: now,
            exp: now + config.refresh_token_lifetime_secs,
            token_type: "refresh".to_string(),
            roles: Vec::new(),
            session_id: Some(session_id.to_string()),
        }
    }

    /// Check if the token is expired.
    pub fn is_expired(&self) -> bool {
        current_timestamp() > self.exp
    }

    /// Check if this is an access token.
    pub fn is_access_token(&self) -> bool {
        self.token_type == "access"
    }

    /// Check if this is a refresh token.
    pub fn is_refresh_token(&self) -> bool {
        self.token_type == "refresh"
    }
}

/// A pair of access and refresh tokens.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenPair {
    pub access_token: String,
    pub refresh_token: String,
    pub access_expires_at: u64,
    pub refresh_expires_at: u64,
    pub token_type: String,
}

impl TokenPair {
    pub fn new(access_token: String, refresh_token: String, config: &JwtConfig) -> Self {
        let now = current_timestamp();
        Self {
            access_token,
            refresh_token,
            access_expires_at: now + config.access_token_lifetime_secs,
            refresh_expires_at: now + config.refresh_token_lifetime_secs,
            token_type: "Bearer".to_string(),
        }
    }
}

/// JWT token manager using RS256.
pub struct JwtManager {
    config: JwtConfig,
    encoding_key: EncodingKey,
    decoding_key: DecodingKey,
    validation: Validation,
}

impl JwtManager {
    /// Create a new JWT manager with RSA key pair.
    pub fn new(config: JwtConfig, private_key_pem: &[u8], public_key_pem: &[u8]) -> Result<Self> {
        let encoding_key = EncodingKey::from_rsa_pem(private_key_pem)
            .map_err(|e| IdentityError::JwtError(format!("invalid private key: {}", e)))?;

        let decoding_key = DecodingKey::from_rsa_pem(public_key_pem)
            .map_err(|e| IdentityError::JwtError(format!("invalid public key: {}", e)))?;

        let mut validation = Validation::new(config.algorithm);
        validation.set_issuer(&[&config.issuer]);
        validation.validate_exp = true;

        Ok(Self {
            config,
            encoding_key,
            decoding_key,
            validation,
        })
    }

    /// Generate a token pair (access + refresh) for a user.
    pub fn generate_token_pair(
        &self,
        user_id: &str,
        roles: Vec<String>,
        session_id: &str,
    ) -> Result<TokenPair> {
        let access_claims = JwtClaims::new_access(user_id, roles, &self.config);
        let refresh_claims = JwtClaims::new_refresh(user_id, session_id, &self.config);

        let access_token = self.encode(&access_claims)?;
        let refresh_token = self.encode(&refresh_claims)?;

        Ok(TokenPair::new(access_token, refresh_token, &self.config))
    }

    /// Encode claims into a JWT token.
    pub fn encode(&self, claims: &JwtClaims) -> Result<String> {
        let header = Header::new(self.config.algorithm);
        encode(&header, claims, &self.encoding_key)
            .map_err(|e| IdentityError::JwtError(e.to_string()))
    }

    /// Decode and validate a JWT token.
    pub fn decode(&self, token: &str) -> Result<JwtClaims> {
        let token_data = decode::<JwtClaims>(token, &self.decoding_key, &self.validation).map_err(
            |e| match e.kind() {
                jsonwebtoken::errors::ErrorKind::ExpiredSignature => IdentityError::TokenExpired,
                _ => IdentityError::TokenValidationFailed(e.to_string()),
            },
        )?;

        Ok(token_data.claims)
    }

    /// Validate a token and return its claims if valid.
    pub fn validate(&self, token: &str) -> Result<JwtClaims> {
        let claims = self.decode(token)?;

        if claims.is_expired() {
            return Err(IdentityError::TokenExpired);
        }

        Ok(claims)
    }

    /// Get the configuration.
    pub fn config(&self) -> &JwtConfig {
        &self.config
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
    fn test_jwt_config_default() {
        let config = JwtConfig::default();
        assert_eq!(config.algorithm, Algorithm::RS256);
        assert_eq!(config.access_token_lifetime_secs, 3600);
    }

    #[test]
    fn test_jwt_claims_access() {
        let config = JwtConfig::default();
        let claims = JwtClaims::new_access("user123", vec!["admin".to_string()], &config);

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.token_type, "access");
        assert!(claims.is_access_token());
        assert!(!claims.is_expired());
        assert!(claims.roles.contains(&"admin".to_string()));
    }

    #[test]
    fn test_jwt_claims_refresh() {
        let config = JwtConfig::default();
        let claims = JwtClaims::new_refresh("user123", "session456", &config);

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.token_type, "refresh");
        assert!(claims.is_refresh_token());
        assert_eq!(claims.session_id, Some("session456".to_string()));
    }

    #[test]
    fn test_token_pair() {
        let config = JwtConfig::default();
        let pair = TokenPair::new("access".to_string(), "refresh".to_string(), &config);

        assert_eq!(pair.access_token, "access");
        assert_eq!(pair.refresh_token, "refresh");
        assert_eq!(pair.token_type, "Bearer");
    }

    #[test]
    fn test_claims_expiration() {
        let mut config = JwtConfig::default();
        config.access_token_lifetime_secs = 0;

        let claims = JwtClaims::new_access("user", vec![], &config);
        assert!(claims.is_expired() || claims.exp <= current_timestamp());
    }
}
