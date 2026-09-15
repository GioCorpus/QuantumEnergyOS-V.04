# GPU Quantum Simulation (Phase 4.3)

CPU is the reference. GPU entries (`wgpu`/`cuda`/`rocm`) are capability-gated:

- `probe_backend(kind)` reports availability (only CPU guaranteed).
- `execute_on_backend` falls back to CPU and sets `gpu_used=false`.
- Results always carry `simulation_only=true`.
- Host/device transfer model documented in `quantum-hal::accelerator`
  (no zero-copy claims; no vendor kernel ships in this build).
