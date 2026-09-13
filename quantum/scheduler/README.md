# Quantum Scheduler (V.04 §5)

Canonical implementation: `crates/quantum-runtime/src/scheduler.rs` + `job.rs`.

Manages: queue, priority (Critical>High>Normal>Low FIFO), resource allocation, deadlines, cancellation, hardware availability.

Physical scheduling disabled by default (`allow_physical=false`); remote requires explicit availability. No physical QPU required for CI (Mock/Simulator only).