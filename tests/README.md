# Tests (V.04 §26)

Every subsystem requires: unit, integration, IPC, security, backend,
hardware-mock, quantum-simulation, UI, and end-to-end tests.

Physical QPU hardware is never required for CI. Use `MockQPU` /
`SimulatorQPU` (`SimulatorBackend`, `LocalEmulatorBackend`).

- Rust: `cargo test --workspace` (x86_64 + ARM64 in CI).
- Frontend: `pnpm --dir dashboard test` (vitest).
- Python: `pytest tools -q`.