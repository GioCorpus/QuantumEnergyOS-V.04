# Kernel SMP

Single-CPU host model. `cores = 1` explicit in `CpuInfo`; `PerCpu` tracks
per-CPU `current_tid`, saturating `switches`/`interrupts` counters.

No SMP claimed. Future SMP requires: per-CPU runqueues, `SpinLock` ordering,
IPI, TLB shootdown, and QEMU multi-CPU validation. No `static mut` allowed.