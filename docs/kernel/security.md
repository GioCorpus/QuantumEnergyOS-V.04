# Security (§42-45, §87 acceptance)

Boundary: `syscall::validate::{validate_range,require}` — null/overflow/bounds +
caps. Never deref user ptr without validation (I3). Handles, not raw pointers,
for kernel objects (pending full handle table).

Answers today (honest):
* Cross-process memory? Host BTreeMap isolation only; no HW enforcement yet.
* Arbitrary phys map? `map_page` checks align + W^X but no priv check yet — must add caps.
* Arbitrary DMA? `DmaRegion` checks align/size + Drop unmap; no IOMMU yet — deny-by-default.
* Privileged op from user? `dispatch_syscall` ignores args — MUST enforce caps before use.
* Corrupt driver? No isolation yet — lifecycle + ownership pending.
* QPU job exhaust memory? `QPU_MAX_JOB_BYTES` const exists; enforcement pending in runtime.
* Device access without IOMMU? Yes possible in model — documented, IOMMU future.

Unsafe: `cpu/interrupts` now 0 unsafe; `spin` 4 unsafe with SAFETY; `lib.rs`
`forbid(unsafe_op_in_unsafe_fn)`. No `unwrap/expect/panic!` in kernel paths
(except tests).