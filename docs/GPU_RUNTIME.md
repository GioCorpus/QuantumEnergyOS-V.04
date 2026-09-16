# GPU Runtime (Phase 4.5)

`crates/qeos-gpu-compute`: `GpuDevice` (vec_add, mat_vec), `GpuBuffer`,
`GpuKernel`, `GpuQueue`/`GpuFence`, `BackendKind::{CpuReference, Mock,
Vulkan, Cuda, Rocm}`. CPU reference is the correctness oracle; mock mirrors
it for equivalence tests. `probe()` reports vendor backends unavailable —
no CUDA/ROCm dependency, no fake GPU claims. Equivalence: CPU == mock
exactly (integer-exact float ops in tests); vendor backends must meet a
numeric tolerance + same seed/input before acceptance.
