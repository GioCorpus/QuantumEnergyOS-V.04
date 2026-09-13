# Telemetry (V.04 §14, §25)

Uses `tracing`, Prometheus/OpenTelemetry, structured JSON logs.

Metrics: CPU, memory, temperature, power, telemetry rate, dropped frames,
IPC latency, service health, quantum job latency, QPU availability,
simulation fidelity.

Implementation: `crates/energy-telemetry` ring buffers + `crates/system-core`
service bus. Never claims zero-copy unless actually implemented.