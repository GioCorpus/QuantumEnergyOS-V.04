# Database Migrations (V.04 §10, Phase 3)

PostgreSQL/SQL-compatible authoritative store via `sqlx` + `tokio` + `tracing`.

Tables (planned): users, roles, permissions, sessions, audit_events, devices,
quantum_backends, quantum_jobs, telemetry, energy_nodes, system_profiles,
browser_profiles.

No embedded DB as authoritative identity store. CI validates this directory
exists and runs migrations against `postgres:16` when SQL files land.