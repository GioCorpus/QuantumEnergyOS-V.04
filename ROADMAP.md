# QuantumEnergyOS V.04 Roadmap

## Phase 0 — Architecture
- Define system boundaries and responsibilities
- Document assumptions and non-goals
- Define hardware abstraction and quantum backend model
- Publish the operational threat model and architecture envelope

## Phase 1 — Bootable system
- Linux-compatible base environment
- init/systemd model
- filesystem, networking, shell, package manager
- logging and boot health tracing

## Phase 2 — Service framework
- service lifecycle interfaces
- IPC bus and message envelope
- telemetry and health APIs
- structured service manager

## Phase 3 — Identity and security
- PostgreSQL-backed persistence
- Argon2id password hashing
- RS256 / JWKS token system
- sessions, roles, permissions, audit logs

## Phase 4 — Quantum runtime
- circuit model and register abstractions
- backend adapters
- simulator, emulator, and mock QPU support
- measurement and parity semantics

## Phase 5 — Quartz 5D
- coordinate space model
- storage, projection, prediction, serialization
- UI projection from 5D data to 3D visualization

## Phase 6 — Energy telemetry
- sensor ingestion and ring buffers
- forecasting, optimization, and dashboards
- safety constraints and monitoring

## Phase 7 — Browser and dashboard
- Browser profiles and isolation
- web dashboard integrations and Tauri/WebView option
- service views for threat, energy, and quantum telemetry

## Phase 8 — Desktop integration
- minimal Wayland/tinywl variant
- full KDE Plasma variant
- clean separation from runtime services

## Phase 9 — QPU integration
- adapter-only integration behind a documented API
- simulator-first approach for CI and research
- physical QPU integration is deferred until real support exists
