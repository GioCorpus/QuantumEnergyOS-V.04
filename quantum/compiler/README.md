# Quantum Compiler (V.04 §5)

Canonical implementation: `crates/quantum-runtime/src/compiler.rs`.

Pipeline: High-Level Circuit -> IR (`IrOperation`, `IntermediateRepresentation`) -> Backend artifact (`CompiledCircuit`) -> Execution.

Physical/Remote lowering is capability-gated (refused without documented interface). All artifacts record `backend` and `simulation_only`.