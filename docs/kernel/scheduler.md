# Kernel Scheduler

Cooperative host model (`kernel/src/scheduler/`, `kernel/src/process/`).

- `RunQueue`: 5 priority queues (Idle/Low/Normal/High/Realtime), highest-first pop.
- `Scheduler`: `spawn` + `schedule` (ticks++), `current: Option<Tid>`.
- `SchedClass` (Normal/Realtime/Scientific/Telemetry/Quantum) is reserved, not scheduled.
- No preemption, no SMP balancing, no aging — strict priority can starve Low (documented, tested consciously).
- `PerCpu { cpu, current_tid, switches, interrupts }` with saturating `on_switch`.
- `affinity: u32` stored but not enforced (single-CPU host).

Contract: scheduler never blocks; sleep is state-only (no timer wakeup yet).
Future SMP requires per-CPU runqueues + lock ordering + `SpinLock`.