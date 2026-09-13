# Service Framework (V.04 §7)

Canonical implementation: `crates/system-core` (`QuantumService` trait, `ServiceManager`, versioned IPC envelopes, health API, tracing).

Every major service implements `initialize/start/stop/status/health`. Local-first: Tokio + Unix Domain Sockets + versioned envelopes + auth. Replaceable transport, no distributed complexity.

See `docs/SERVICE_FRAMEWORK.md` and `docs/IPC_PROTOCOL.md`.