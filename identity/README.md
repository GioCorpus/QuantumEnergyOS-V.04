# Identity Service (V.04 §9)

Canonical implementation: `crates/identity-service`
(Argon2id password hashing, RS256 asymmetric JWT, JWKS, key rotation,
short-lived access + refresh tokens, RBAC, sessions, audit).

- Never stores plaintext passwords; hashing ≠ encryption.
- PostgreSQL/SQL-compatible for authoritative data (`sqlx`); embedded DB only for caches.
- Revocation strategy + audit events required.

See `THREAT_MODEL.md` for trust boundaries.