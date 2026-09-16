# Quantum IR (Phase 4.6)

Hardware-independent ops (Allocate/Reset/X/Y/Z/H/CNOT/Measure/Barrier/Delay/
Conditional) in `quantum-hal::ir` + `quantum-runtime::{circuit, compiler}`.
Lowering maps logical ops → backend-native ops; no backend is assumed to
execute every op natively (Majorana devices lower to parity measurements).
`QuantumJob` states extend the IR lifecycle with Compiling/Measuring/
PostProcessing/Timeout alongside Submitted/Queued/Running/ Completed/Failed/
Cancelled/Unsupported.
