# Threat Model

## Assets
- identity credentials and session tokens
- system configuration and service metadata
- telemetry and energy data
- quantum job definitions and backend metadata
- browser profiles and certificate stores
- signed package metadata and CI artifacts

## Trust boundaries
- local services communicating over IPC
- database-backed identity and system metadata
- browser profiles and session data
- external API or remote QPU adapters
- hardware telemetry and sensor interfaces

## Threats
- unauthorized service access to IPC endpoints
- credential leakage or replay
- token forgery or session hijacking
- unsafe execution of untrusted code or packages
- telemetry poisoning or fake sensor input
- unvalidated remote QPU adapter responses

## Controls
- authenticated and authorized IPC
- RBAC and least-privilege enforcement
- short-lived signed tokens and JWKS rotation
- Argon2id for password storage
- audit logging for security-relevant actions
- strict typed interfaces between service, hardware, and quantum layers
- simulation-only behavior in CI, never physical-hardware-only assumptions

## Security posture

QuantumEnergyOS is designed to be secure by default, with explicit trust boundaries and swap-in adapters for future QPU integrations. No sensitive material should be committed to source control.
