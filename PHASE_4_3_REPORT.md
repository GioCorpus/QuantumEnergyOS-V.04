# Phase 4.3 Report — Majorana Quantum Runtime (SIMULATION/MODEL)

## Architecture

`QuantumCircuit -> QuantumCompiler (IR) -> ExperimentLimits validation ->
ComputeBackend (CPU ref / honest GPU fallback) -> ExperimentResult`.
Majorana path: `Tetron -> parity sampling -> RepetitionDecoder-style
correction -> MajoranaExperimentResult`. HAL unchanged (simulator reference,
accelerator abstract, experimental QPU adapter-gated).

## Implemented

Quantum core IDs; S/T/Reset/Barrier/ConditionalX gates; CZ/SWAP/Toffoli
engines (CNOT double-swap fixed); `reset_qubit`; resource guard
(`SimulationResource`); compiler lowering for new ops; optimizer (X-X/H-H,
barrier); `RuntimeNoiseModel`; `Decoder` + repetition/lookup; capabilities;
`run_experiment` shot engine; `run_majorana_experiment`; `qeos-qpu` CLI;
golden tests; docs.

## Quantum Model

State-vector `Complex<f64>`, norm tolerance 1e-10, Born-rule sampling +
collapse, seeded RNG reproducibility.

## Majorana Model

`{gamma_i,gamma_j}=2δij` verified in 2-mode Pauli sector; tetron 4-mode
abstraction; parity Even(+1/0)/Odd(-1/1); symbolic exchange only.
NOT device physics; NOT hardware control.

## Simulator

CPU reference, max 20 qubits, S/T/CNOT/CZ/SWAP/Toffoli/measurement/reset;
`check_resources` pre-alloc guard.

## Error Correction

Syndrome -> Decoder trait -> Correction; repetition + lookup verified.

## GPU

CPU reference; wgpu/cuda/rocm probe + honest CPU fallback (`gpu_used=false`).

## Benchmarks

`qeos-qpu benchmark` prints 2/4/8-qubit elapsed ms (host-dependent, not
claimed as fixed results).

## Tests

`cargo test -p quantum-runtime`: 116 lib + golden (6) + integration (20)
green. Full workspace `cargo check` green for new crates (pre-existing
kernel warnings untouched).

## Security

`ExperimentLimits` (qubits/shots/depth), noise bounds [0,1], syndrome/input
validation returning errors (no panics on invalid input paths tested).

## Hardware

Supported: none (simulators only). Future: `ExperimentalQpuDevice` +
`QpuVendorAdapter` contract; no fake protocol, no invented registers.

## Known Limitations

Circuit-level single-qubit gates target qubit 0 (HAL device IR carries full
targets); dephasing rate reserved; MWPM/neural decoders future; 20-qubit cap.

## Next Phase

Target-carrying circuit gates; HAL Reset/Custom lowering; property/fuzz
harness; telemetry hookup; scheduler<->experiment async runner.
