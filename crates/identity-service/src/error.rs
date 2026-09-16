use thiserror::Error;

/// Result type for identity service operations.
pub type Result<T> = std::result::Result<T, IdentityError>;

/// Errors that can occur in the identity service.
#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("authorization failed: {0}")]
    AuthorizationFailed(String),

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("token validation failed: {0}")]
    TokenValidationFailed(String),

    #[error("token expired")]
    TokenExpired,

    #[error("session not found: {0}")]
    SessionNotFound(String),

    #[error("session expired")]
    SessionExpired,

    #[error("user not found: {0}")]
    UserNotFound(String),

    #[error("user already exists: {0}")]
    UserAlreadyExists(String),

    #[error("password hashing failed: {0}")]
    PasswordHashingFailed(String),

    #[error("JWT error: {0}")]
    JwtError(String),

    #[error("JWKS error: {0}")]
    JwksError(String),

    #[error("RBAC error: {0}")]
    RbacError(String),

    #[error("audit error: {0}")]
    AuditError(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    #[error("internal error: {0}")]
    InternalError(String),
}

impl From<jsonwebtoken::errors::Error> for IdentityError {
    fn from(err: jsonwebtoken::errors::Error) -> Self {
        IdentityError::JwtError(err.to_string())
    }
}

impl From<argon2::password_hash::Error> for IdentityError {
    fn from(err: argon2::password_hash::Error) -> Self {
        IdentityError::PasswordHashingFailed(err.to_string())
    }
}

impl From<std::io::Error> for IdentityError {
    fn from(err: std::io::Error) -> Self {
        IdentityError::InternalError(err.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = IdentityError::AuthenticationFailed("test".to_string());
        assert!(err.to_string().contains("test"));
    }

    #[test]
    fn test_error_from_jwt() {
        let jwt_err = jsonwebtoken::errors::Error::from(
            jsonwebtoken::errors::ErrorKind::InvalidToken,
        );
        let err: IdentityError = jwt_err.into();
        assert!(matches!(err, IdentityError::JwtError(_)));
    }
}
