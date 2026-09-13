# System — Service Framework (V.04 §7–§8)

Arch-compatible, Rust-first service layer.

Canonical implementation: `crates/system-core` (`QuantumService` trait,
`ServiceManager`, Unix-Domain-Socket IPC envelopes, health API, tracing).

This directory documents OS integration points (systemd units, D-Bus policy,
filesystem layout) without duplicating the Rust implementation.

- `service-framework/`: systemd unit templates, socket activation policy.
- `services/`: per-service integration notes mapping to `crates/*`.

No distributed-system complexity: local IPC first, replaceable transport.