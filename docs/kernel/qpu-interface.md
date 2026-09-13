# QPU boundary (§40-41) — kernel provides devices, runtime lives in userspace

Kernel: `qpu::QuantumDevice{initialize,submit,poll}` + `UnsupportedDevice` only.
No Majorana algebra / tetron / parity / QEC / ML in kernel. Job size bounded by
`QPU_MAX_JOB_BYTES` (enforcement in runtime). Path: QPU App → QPU Runtime
(userspace) → QPU Device API → Kernel Driver → HW. Simulator also userspace.