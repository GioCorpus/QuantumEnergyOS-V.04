# System Services (V.04 §7)

Services: identity, policy, quantum, qpu, telemetry, energy, browser, dashboard, storage, hardware.

Canonical implementations live in `crates/*` (system-core, identity-service, quantum-runtime, energy-telemetry, hardware-abstraction). This directory holds OS integration notes (units, sockets, permissions) per service.