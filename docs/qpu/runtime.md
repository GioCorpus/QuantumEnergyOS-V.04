# QEOS QPU Runtime (Phase 4.3)

SIMULATION ONLY. No hardware is contacted.

## Backends

- `cpu-simulator`: state-vector reference (correctness baseline).
- `majorana-simulator`: tetron/parity model on the CPU reference.
- `gpu-simulator`: capability-gated; falls back to CPU with `gpu_used=false`.
- Future QPU: `quantum-hal::ExperimentalQpuDevice` adapter contract only.

## Run

```bash
cargo run -p qeos-qpu -- backends
cargo run -p qeos-qpu -- simulate --shots 64 --seed 42
cargo run -p qeos-qpu -- majorana --shots 1000 --seed 42
cargo run -p qeos-qpu -- benchmark
```

## Current Limitations

- Max 20 qubits (state-vector 2^n memory).
- Single-qubit gates in `QuantumCircuit` apply to qubit 0; full target
  addressing flows through `quantum-hal` device IR.
- Noise is an effective classical model, not device physics.
- GPU entries report fallback honestly; no vendor kernel ships.
