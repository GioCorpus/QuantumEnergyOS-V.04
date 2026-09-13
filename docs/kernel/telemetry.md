# Kernel Telemetry (Host Model)

`kernel/src/telemetry/` (`Sample`, `PowerTelemetry`, `DevicePowerState`, `EnergyCounter`) + `kernel/src/tracing/` (`TraceCtx`) + `kernel/src/logging/`.

- `Sample { ts_mono_ns, source, value }`; energy counters saturating, never panic.
- `TraceCtx { cpu, thread, process, device, job, trace, ts_mono }` for correlation with user-space `tracing`/Prometheus/OpenTelemetry.
- No fake hardware telemetry; host `println!` logger only. Metrics (ctx switches, IRQs, faults, allocs) are future work (S20).