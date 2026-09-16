# Security Model (Phase 4.9)

Boundaries: user/kernel via handles + capabilities + validated descriptors
(no kernel pointers, no arbitrary phys mappings, no unvalidated user
pointers, no unrestricted DMA). DMA gated by capability + bus-master facts
+ IOMMU policy (`device-manager`, `kernel::driver::iommu`). MMIO bounded
windows only. Audited: IPC perms, memory safety, int/buffer overflow,
UAF/double-free, races, TOCTOU, privesc — see audit + unsafe review.
Unsafe: `energy-telemetry::ring_buffer` UnsafeCell+atomics (SPSC invariants
documented); `zeroed()` init valid for `Copy` samples only. No transmute /
from_raw_parts / raw MMIO deref in workspace.
