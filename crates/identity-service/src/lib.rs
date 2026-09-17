pub mod audit;
pub mod auth;
pub mod error;
pub mod jwks;
pub mod jwt;
pub mod rbac;
pub mod sessions;

pub use audit::{AuditEvent, AuditLogger, AuditSeverity};
pub use auth::{AuthConfig, AuthService, StoredPasswordHash};
pub use error::{IdentityError, Result};
pub use jwks::{JsonWebKey, Jwks, JwksManager};
pub use jwt::{JwtClaims, JwtConfig, JwtManager, TokenPair};
pub use rbac::{Permission, RbacManager, Role, RoleAssignment};
pub use sessions::{Session, SessionConfig, SessionManager};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
