# Security (V.04 §24)

First-class subsystem: least privilege, secure IPC, RBAC, audit logs, secret
isolation, Argon2id hashing, token/key rotation, sandboxing, capability
restrictions, signed packages where practical.

Never: commit private keys/passwords, store plaintext passwords, hard-code JWT
secrets, expose privileged APIs without authorization.

CI enforces `cargo audit` + committed-secret scan. See `THREAT_MODEL.md`.