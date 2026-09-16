# QPU Runtime (Phase 4.6)

Stack: app → `QpuDevice` → Quantum IR → compiler/lowering → scheduler →
simulator/vendor adapter. `quantum-runtime::qpu_device::{QpuCapabilities,
QpuDevice, QpuJobStatus, MockQpu}` covers capabilities/allocate/configure/
submit/measure/reset/cancel/telemetry shapes. Job states: Created, Queued,
Compiling, Running, Measuring, PostProcessing, Completed, Failed, Cancelled,
Timeout. Scheduler: priority + fairness + limits + timeouts + cancellation +
backpressure + telemetry. All RNG seeded for reproducibility.
