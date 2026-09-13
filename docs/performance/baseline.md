# Performance baseline (§55-56, §86) — qualitative (no toolchain locally)

No claims without before/after benches. Target benches (`kernel/benches/`):
boot, context_switch, syscall, IPC, scheduler, heap alloc, IRQ, DMA throughput.
Current complexity: heap O(1) + 3 checked_add; allocator O(1); scheduler pop O(prio);
IPC O(1); DMA O(1). Next: add `criterion`-free `test::Bencher` or host timing +
`docs/performance/results.md` once cargo available. Never drop safety for speed (§68).