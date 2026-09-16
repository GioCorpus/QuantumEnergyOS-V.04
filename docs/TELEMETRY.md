# Telemetry (Phase 4.9)

Pipeline: hardware → kernel driver → SPSC ring collector → IPC →
`energy-telemetry::{TelemetryService, EnergyService}` → dashboard/DB.
Top half (IRQ): ack + capture + enqueue only; bottom half: deferred parse/
aggregate — never heavy work in interrupt context. Provenance per sample:
Measured / Estimated / Simulated / Unavailable — estimates are never
presented as physical measurements. Energy: power/energy/voltage/current/
temperature/frequency/utilization via `EnergySample` + `forecast_energy`
(moving-average ESTIMATE, labeled). Faults: ring overflow counters,
telemetry-overflow + device/GPU/QPU failure injection points (host tests).
