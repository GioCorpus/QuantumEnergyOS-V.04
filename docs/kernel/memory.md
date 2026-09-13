# Memory (§10-14) — hardened this round

* `PhysAddr/PhysPage/PhysicalMemoryManager` — bump model, OOM → None.
* `VirtAddr/VirtPage/VirtualMemoryManager` — `map_page` enforces: no double-map,
  phys page-aligned, W^X deny (WRITE+EXEC → Err). `is_user` via USER flag.
* `KernelHeap::alloc` — rejects non-pow2 align (incl. 0), checked_add chain.
* `KernelAllocator` — saturating stats, no unwrap.
* `OomPolicy` — Deny/LogAndDeny/ReclaimAndRetry; `on_oom` always Err, never panic.
* Pending: real page tables, AddressSpace per process, canonical-addr checks,
  kernel/user split enforcement, IOMMU-backed DMA.