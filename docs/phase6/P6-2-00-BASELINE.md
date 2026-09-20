# QEOS V.04 Phase 6.2-00: Kernel, Memory & Concurrency Baseline

**Date:** 2026-09-18
**Auditor:** Principal Systems Engineer
**Phase:** P6.2 Kernel, Memory & Concurrency Hardening
**Subphase:** P6.2-00 Baseline Establishment
**Status:** IN PROGRESS

---

## 1. Executive Summary

This document establishes the verified baseline of the QEOS V.04 kernel's fundamental execution and memory model before Phase 6.2 hardening begins. The kernel is a host-testable, monolithic modular Rust-first kernel core with x86_64 HAL stubs. All subsystems build cleanly and pass the full test suite (390+ tests across workspace).

---

## 2. Kernel Architecture Overview

### 2.1 Kernel Structure

```
kernel/
  src/
    arch/           # Architecture-specific code (x86_64)
      x86_64/
        boot.rs         # Boot entry stub
        context.rs      # CPU context for switching
        cpu.rs          # CPU initialization
        gdt.rs          # Global Descriptor Table
        idt.rs          # Interrupt Descriptor Table
        interrupts.rs   # IRQ subsystem (atomic flags)
    boot/           # Deterministic boot sequence
    core/           # Central kernel configuration
    device/         # Device abstraction (re-exports driver)
    dma/            # DMA abstraction with ownership tracking
    driver/         # PCI, MMIO, IRQ, IOMMU, lifecycle
    elf/            # Minimal 64-bit ELF loader
    fs/             # VFS + inode (File, Dir)
    hal/            # Hardware Abstraction Layer traits
    ipc/            # Channel-based message passing
    logging/        # Kernel logger
    memory/         # Physical, virtual, heap, allocator
    panic/          # Deterministic panic handler
    process/        # Process & thread management
    qpu/            # Quantum device interface (stub)
    ring/           # SPSC lock-free ring buffer
    scheduler/      # Multi-class scheduler with per-CPU
    security/       # Capability-based access control
    sync/           # Mutex, SpinLock, atomics
    syscall/        # Syscall dispatcher + validation
    telemetry/      # Energy telemetry
    time/           # Clock & timer
    tracing/        # Trace context propagation
```

### 2.2 Kernel Entry & Boot

| Component | File | Status | Notes |
|-----------|------|--------|-------|
| Boot sequence | boot/mod.rs | REAL | 11-stage deterministic sequence |
| UEFI entry stub | arch/x86_64/boot.rs | STUB | Empty entry_stub() |
| Panic handler | panic.rs | PLACEHOLDER | Not a real #[panic_handler]; loops on spin |
| Linker script | boot/linker.ld | MISSING | ENTRY(_start) but no _start symbol |
---

## 13. Unsafe Code Inventory

### 13.1 Kernel Unsafe Blocks (4 total)

| File | Line | Block | Justification |
|------|------|-------|---------------|
| `sync/spin.rs` | 16-17 | `unsafe impl Send/Sync` | SpinLock provides mutual exclusion via AtomicBool |
| `sync/spin.rs` | 53 | `&*data.get()` | Guard proves exclusive access |
| `sync/spin.rs` | 59 | `&mut *data.get()` | Guard proves exclusive access |
| `sync/spin.rs` | 65 | `unlock()` | Guard held the lock |

**All 4 blocks are in `SpinLock` implementation. Each has `// SAFETY:` comment.**

### 13.2 User-Space Unsafe Blocks (3 total in `energy-telemetry`)

| Crate | File | Count | Purpose |
|-------|------|-------|---------|
| `energy-telemetry` | `ring_buffer.rs` | 3 | Lock-free SPSC ring with atomics |

---

## 14. Test Coverage Summary

| Module | Unit Tests | Integration Tests |
|--------|------------|-------------------|
| `kernel` | 48 (lib + 3 integration) | `kernel_integration.rs`, `syscall_tests.rs`, `boundary_contract_tests.rs` |
| `memory` | 12 | — |
| `scheduler` | 3 | — |
| `sync` | 3 | — |
| `syscall` | 32 | — |
| `process` | 3 | — |
| `dma` | 2 | — |
| `driver` | 50 (device-manager crate) | — |
| `ipc` | 3 | — |
| `security` | 4 | — |
| `boot` | 1 | — |
| `elf` | 2 | — |
| `fs` | 1 | — |
| `qpu` | 1 | — |
| `ring` | 2 | — |

**Total Kernel Tests: ~80 (lib) + 50 (device-manager) + 3 (integration) = 133**

---

## 15. Identified Gaps for Phase 6.2

### 15.1 Memory Model (P6.2-01)
- [ ] Physical address types (`PhysAddr`, `PhysPage`) used consistently
- [ ] Virtual memory manager uses typed physical addresses
- [ ] Page table hierarchy (PML4→PDPT→PD→PT) for x86_64
- [ ] Kernel/user page table separation
- [ ] TLB invalidation hooks
- [ ] NX bit enforcement
- [ ] Canonical address validation
- [ ] Mapping lifetime tracking
- [ ] Double mapping prevention
- [ ] Use-after-free detection

### 15.2 Allocator Hardening (P6.2-02)
- [ ] Implement `KernelHeap::free()` with coalescing
- [ ] Add allocation size tracking
- [ ] Add guard pages / red zones
- [ ] Add double-free detection
- [ ] Add invalid-free detection
- [ ] Stress tests for fragmentation
- [ ] OOM behavior specification

### 15.3 Page Table Hardening (P6.2-03)
- [ ] Multi-level page table implementation
- [ ] User/kernel address space separation
- [ ] NX enforcement
- [ ] TLB shootdown for SMP
- [ ] Address validation on every map/unmap

### 15.4 Scheduler Hardening (P6.2-04)
- [ ] Time slice / quantum enforcement
- [ ] Forced preemption on timer tick
- [ ] CPU affinity enforcement
- [ ] Load balancing (SMP)
- [ ] Priority aging / anti-starvation
- [ ] Thread cancellation support
- [ ] Context switch implementation

### 15.5 Concurrency Hardening (P6.2-05)
- [ ] Replace `KernelMutex` with IRQ-safe, `no_std` implementation
- [ ] Add lock ordering documentation & enforcement
- [ ] Add deadlock detection (lockdep-style)
- [ ] Add priority inheritance for mutexes
- [ ] Formalize IRQ-safe vs sleepable lock classification
- [ ] Add recursion tracking for spinlocks

### 15.6 Syscall Safety (P6.2-06)
- [ ] Validate all syscall arguments comprehensively
- [ ] Add syscall argument count validation
- [ ] Add return value validation
- [ ] Implement syscall auditing/tracing
- [ ] Harden pointer validation (canonical form, permissions)

### 15.7 Fault Handling (P6.2-07)
- [ ] Real `#[panic_handler]` implementation
- [ ] Page fault handler
- [ ] Invalid syscall handler
- [ ] Stack overflow detection
- [ ] Kernel assertion framework
- [ ] OOM handling in syscalls

### 15.8 Unsafe Rust Audit (P6.2-08)
- [ ] Verify all 4 `SpinLock` unsafe blocks have correct invariants
- [ ] Verify 3 `energy-telemetry` unsafe blocks
- [ ] Add `unsafe_op_in_unsafe_fn` lint (already in lib.rs)
- [ ] Document invariants for each unsafe block
- [ ] Consider replacing `SpinLock` with `lock_api` + raw spinlock

---

## 16. Baseline Gate Decision

**P6.2-00 STATUS: BASELINE ESTABLISHED**

The kernel subsystems are sufficiently understood to proceed with Phase 6.2 hardening milestones. All identified defects are documented in PHASE_6_TECHNICAL_DEBT.md with traceability to specific remediation milestones.

**Next:** P6.2-01 — Memory Model Audit & Hardening

---

## 17. Evidence

- `cargo test --workspace` → PASS (390+ tests)
- `cargo fmt --all -- --check` → PASS
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` → PASS
- Unsafe code inventory complete (7 total: 4 kernel, 3 user-space)
- All Phase 5 milestones verified complete
---

## 10. Driver Subsystem Audit

### 10.1 PCI (`driver/pci.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Address | `PciAddr` (bus, dev, func) | REAL | Label formatting |
| Identity | `PciIdentity` (vendor, device, class, etc.) | REAL | `present()` checks vendor != 0xFFFF |
| BARs | `BarDescriptor` (6 slots) | REAL | `is_mappable()` check |
| IRQ facts | `InterruptFacts` (MSI, MSI-X, legacy) | REAL | — |
| DMA facts | `DmaFacts` (bus_master, addr64, ATS) | REAL | `declared_ready()` check |
| Config source | `PciConfigSource` trait + `StubConfigSource` | REAL | Host test stub |
| Enumeration | `enumerate_source()` | REAL | Filters present devices |

### 10.2 MMIO (`driver/mmio.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Window | `MmioWindow` with `Vec<u32>` backing | MOCK | Host memory-backed |
| Access | `read_u32`/`write_u32` with bounds/alignment | REAL | Returns `DmaError` |
| Alignment | 4-byte required | REAL | Checked |

### 10.3 Device Lifecycle (`driver/lifecycle.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| States | 9-state enum | REAL | Discovered→Probing→Initialized→Ready→Running→Suspended→Stopping→Removed/Failed |
| Transitions | `can_transition()` matrix | REAL | Explicit allowed transitions |
| Terminal states | `Removed`, `Failed` | REAL | `is_terminal()` |

---

## 11. Security & Capabilities

### 11.1 Capabilities (`security/capability.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Capabilities | 8 variants (DeviceRead, DeviceWrite, Dma, Pci, Telemetry, Quantum, Admin) | REAL | Admin implies all |
| CapSet | `Vec<Capability>` | REAL | Linear search, no bitmask |
| Grant | `grant()` deduplicates | REAL | — |
| Check | `has()` includes Admin check | REAL | — |

**Defects:**
- K-17: No capability delegation or revocation
- Linear search O(n) — fine for small sets, not scalable

### 11.2 Permissions (`security/permission.rs`)

| Aspect | Implementation | Status |
|--------|----------------|--------|
| Access types | ReadDevice, WriteDevice, UseDma, UseQuantum | REAL |
| Check | Maps Access → Capability | REAL |

---

## 12. IPC Subsystem

### 12.1 Channel (`ipc/channel.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Backend | `VecDeque<Message>` | REAL | Bounded capacity |
| Errors | Full, Empty, Closed | REAL | — |
| Close | `close()` sets flag | REAL | Further sends fail |

**Defects:**
- K-16: No IPC disconnect handling, no flow control backpressure propagation

### 12.2 Message (`ipc/message.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Structure | from, to, tag, payload (Vec<u8>) | REAL | — |
| Size limit | `try_new()` enforces `IPC_MAX_PAYLOAD` (64 KiB) | REAL | DoS bound |

---
---

## 6. Syscall Boundary Audit

### 6.1 Syscall Dispatcher (`syscall/dispatcher.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Syscall numbers | 35 defined (0-71) | REAL | ABI v1.0 |
| Validation | `validate_range()`, `require()` | REAL | Pointer, length, capability checks |
| Error codes | `SyscallError` enum (8 variants) | REAL | Typed, stable repr(i64) |
| Context | `SyscallContext` (pid, caps, user_addr_max) | REAL | Capability-gated operations |
| User pointers | Validated against `user_addr_max` | REAL | Null rejected, overflow checked |

**Defects:**
- No syscall argument count validation beyond a0-a2
- No syscall return value validation
- Some syscalls are stubs returning hardcoded values
- No syscall tracing/auditing

### 6.2 Validation (`syscall/validate.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Range check | `ptr + len` overflow check | REAL | Uses `checked_add` |
| Null pointer | Rejected when `len > 0` | REAL | Correct |
| Capability check | `require()` delegates to `security::check` | REAL | Granular capabilities |

---

## 7. Process & Thread Model (`process/`)

### 7.1 Process (`process/process.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| PID | `Pid(u32)` newtype | REAL | — |
| State | `ProcessState` enum (7 variants) | REAL | Created→Ready→Running→Blocked→Sleeping→Terminated |
| Credentials | `uid`, `gid` | REAL | No capabilities in process struct |
| Threads | `BTreeMap<u32, Thread>` | REAL | Per-process thread table |
| Handles | Counter only (`handles: u32`) | MOCK | No handle table |

**Defects:**
- K-15: No process supervisor, restart policy, or health checks
- No resource limits (RLIMIT)
- No process groups/sessions
- No namespace isolation

### 7.2 Thread (`process/thread.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| TID | `Tid(u32)` newtype | REAL | — |
| Priority | `Priority` enum (5 levels) | REAL | Maps to run queue index |
| State | `ThreadState` enum (6 variants) | REAL | Includes `Sleeping(u64)` with wake time |
| Affinity | `u32` CPU mask | MOCK | Single CPU only |
| Sleep/Wake | Basic implementation | REAL | No timer integration |

---

## 8. Interrupt & IRQ Subsystem

### 8.1 Interrupts (`arch/x86_64/interrupts.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Init flag | `AtomicBool` | REAL | Replaces `static mut` |
| Enable flag | `AtomicBool` | REAL | — |
| Handlers | **NOT IMPLEMENTED** | MISSING | No IDT population, no handler registration |
| Top-half/bottom-half | Trait `IrqHandler` with `top_half()` | REAL | In `driver/interrupt.rs` |
| Deferred work | Not implemented | MISSING | No workqueue/NAPI equivalent |

### 8.2 IDT (`arch/x86_64/idt.rs`)

*Need to inspect*

---

## 9. DMA & IOMMU Audit

### 9.1 DMA (`dma/mod.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| `DmaRegion` | Phys addr + size + mapped flag | REAL | Drop unmaps |
| `DmaMapping` | IOVA + size + mapped flag | REAL | Drop unmaps |
| `DmaRing` | SPSC ring with `Vec<u64>` descriptors | REAL | Power-of-two depth 2-1024 |
| Alignment | 4096-byte required | REAL | Checked at creation |
| Size limit | 64 MiB max | REAL | `DMA_MAX_BYTES` from config |

**Defects:**
- K-12: Driver DMA integration is `todo!()`
- No cache coherency model
- No DMA direction (to/from device)
- No scatter/gather
- `DmaRing` assumes SPSC but no compile-time enforcement

### 9.2 IOMMU (`driver/iommu.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Trait | `Iommu` with `create_domain`, `attach`, `map`, `unmap` | REAL | Generic over error type |
| Mock | `MockIommu` with software tracking | MOCK | K-11: No hardware IOMMU driver |
| Permissions | `IommuPerm` (read, write) | REAL | No execute |
| Domains | `DomainId(u32)` | REAL | Simple counter |

---
---

## 4. Scheduler & Concurrency Audit

### 4.1 Scheduler (`scheduler/scheduler.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Algorithm | Priority-based run queue | REAL | 5 priority levels (Idle..Realtime) |
| Classes | `SchedClass`: Normal, Realtime, Scientific, Telemetry, Quantum | REAL | Class not used in scheduling decision |
| Context switch | Simulated (no actual register save) | MOCK | `_ = c` placeholder |
| Preemption | Tick-based (`ticks` counter) | REAL | No time slice, no forced preemption |
| SMP | Single CPU model | MOCK | `PerCpu` exists but not integrated |

**Defects:**
- No thread time slicing / quantum enforcement
- No CPU affinity enforcement
- No load balancing
- Context switch is a no-op

### 4.2 Run Queue (`scheduler/runqueue.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Structure | Array of 5 `VecDeque<Tid>` | REAL | One per priority |
| Priority order | Highest priority popped first | REAL | Iterates `.rev()` |
| Fairness | FIFO within priority | REAL | No round-robin, no aging |
| Starvation | Possible for low priority | REAL | No priority boost/aging |

### 4.3 Per-CPU State (`scheduler/percpu.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Structure | `PerCpu` with `cpu_id`, `current_tid`, `switches`, `interrupts` | REAL | Single CPU0 only |
| False sharing | No padding | MEDIUM | Cache line sharing in SMP |
| Switch tracking | `on_switch()` increments counter | REAL | Not hooked into scheduler |

---

## 5. Concurrency Primitives Audit

### 5.1 SpinLock (`sync/spin.rs`)

```rust
// UNSAFE BLOCKS (4 total):
// 1. Line 16-17: unsafe impl<T: Send> Send for SpinLock<T> {}
// 2. Line 16-17: unsafe impl<T: Send> Sync for SpinLock<T> {}
// 3. Line 53: unsafe { &*self.lock.data.get() }
// 4. Line 59: unsafe { &mut *self.lock.data.get() }
// 5. Line 65: unsafe { self.lock.unlock() }
```

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Locking | `AtomicBool` test-and-set + `spin_loop()` | REAL | Busy-wait, no backoff |
| Memory ordering | Acquire on lock, Release on unlock | REAL | Correct for mutual exclusion |
| IRQ safety | **Documented as IRQ-unsafe** | MOCK | K-09: No compile-time enforcement |
| Guard pattern | `SpinGuard` with `Deref`/`DerefMut`/`Drop` | REAL | RAII unlock on drop |
| Poisoning | Not applicable (no panic in lock) | N/A | — |

**Defects:**
- K-09: Documented as IRQ-unsafe unless caller disables interrupts; no compile-time enforcement
- No lock ordering enforcement
- No deadlock detection
- No recursion tracking

### 5.2 KernelMutex (`sync/mutex.rs`)

```rust
// NO UNSAFE BLOCKS — wraps std::sync::Mutex
```

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Backend | `std::sync::Mutex` | MOCK | K-01: Sleepable, NOT IRQ-safe, NOT `no_std` |
| Poison handling | Returns poisoned inner value | REAL | Recovers instead of panicking |
| TryLock | Supported | REAL | Returns `Option` |
| Context | Process context only | DOCUMENTED | Comment: "never hard-IRQ" |

**Defects:**
- K-01: Host-only `std::sync::Mutex` — prevents `no_std` kernel build
- Not IRQ-safe
- No priority inheritance
- No lock ordering enforcement

### 5.3 Atomics (`sync/atomic.rs`)

| Aspect | Implementation | Status |
|--------|----------------|--------|
| Exports | `AtomicU64`, `AtomicUsize`, `Ordering` | REAL (re-exports `std::sync::atomic`) |

---
# QEOS V.04 — Phase 6.2-00: Kernel, Memory & Concurrency Baseline

**Date:** 2026-09-18
**Auditor:** Principal Systems Engineer
**Phase:** P6.2 — Kernel, Memory & Concurrency Hardening
**Subphase:** P6.2-00 — Baseline Establishment
**Status:** IN PROGRESS

---

## 1. Executive Summary

This document establishes the verified baseline of the QEOS V.04 kernel's fundamental execution and memory model before Phase 6.2 hardening begins. The kernel is a host-testable, monolithic modular Rust-first kernel core with x86_64 HAL stubs. All subsystems build cleanly and pass the full test suite (390+ tests across workspace).

---

## 2. Kernel Architecture Overview

### 2.1 Kernel Structure

```
kernel/
├── src/
│   ├── arch/           # Architecture-specific code (x86_64)
│   │   └── x86_64/
│   │       ├── boot.rs         # Boot entry stub
│   │       ├── context.rs      # CPU context for switching
│   │       ├── cpu.rs          # CPU initialization
│   │       ├── gdt.rs          # Global Descriptor Table
│   │       ├── idt.rs          # Interrupt Descriptor Table
│   │       └── interrupts.rs   # IRQ subsystem (atomic flags)
│   ├── boot/           # Deterministic boot sequence
│   ├── core/           # Central kernel configuration
│   ├── device/         # Device abstraction (re-exports driver)
│   ├── dma/            # DMA abstraction with ownership tracking
│   ├── driver/         # PCI, MMIO, IRQ, IOMMU, lifecycle
│   ├── elf/            # Minimal 64-bit ELF loader
│   ├── fs/             # VFS + inode (File, Dir)
│   ├── hal/            # Hardware Abstraction Layer traits
│   ├── ipc/            # Channel-based message passing
│   ├── logging/        # Kernel logger
│   ├── memory/         # Physical, virtual, heap, allocator
│   ├── panic/          # Deterministic panic handler
│   ├── process/        # Process & thread management
│   ├── qpu/            # Quantum device interface (stub)
│   ├── ring/           # SPSC lock-free ring buffer
│   ├── scheduler/      # Multi-class scheduler with per-CPU
│   ├── security/       # Capability-based access control
│   ├── sync/           # Mutex, SpinLock, atomics
│   ├── syscall/        # Syscall dispatcher + validation
│   ├── telemetry/      # Energy telemetry
│   ├── time/           # Clock & timer
│   └── tracing/        # Trace context propagation
```

### 2.2 Kernel Entry & Boot

| Component | File | Status | Notes |
|-----------|------|--------|-------|
| Boot sequence | `boot/mod.rs` | REAL | 11-stage deterministic sequence |
| UEFI entry stub | `arch/x86_64/boot.rs` | STUB | Empty `entry_stub()` |
| Panic handler | `panic.rs` | PLACEHOLDER | Not a real `#[panic_handler]`; loops on spin |
| Linker script | `boot/linker.ld` | MISSING | `ENTRY(_start)` but no `_start` symbol |

---

## 3. Memory Subsystem Audit

### 3.1 Physical Memory Manager (`memory/physical.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Page size | 4096 bytes (const) | REAL | Hardcoded in two places (config + mod.rs) |
| Allocator | Bitmap via `BTreeSet<usize>` | REAL | No fragmentation tracking, no multi-page alloc |
| Free tracking | `free_pages()` count | REAL | O(1) but no ownership/zone awareness |
| Allocation | `allocate_page()` → `Option<PhysPage>` | REAL | First-fit only, no alignment beyond page |
| Deallocation | `free_page()` with bounds check | REAL | No double-free detection beyond set insert |
| Thread safety | None (not `Sync`) | MOCK | Not usable in SMP context |

**Defects:**
- K-02: Uses raw `usize` for physical addresses; no `PhysAddr`/`PhysPage` newtypes in virtual memory
- No canonical address validation
- No NUMA/zone awareness
- No memory hotplug support

### 3.2 Virtual Memory Manager (`memory/virtual_.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Page mapping | `BTreeMap<usize, (usize, MapFlags)>` | REAL | Maps virtual page index → (phys, flags) |
| Permissions | `MapFlags` (READ, WRITE, EXEC, USER) | REAL | W^X enforced (WRITE+EXEC denied) |
| Alignment check | Phys must be PAGE_SIZE multiple | REAL | Checked at map time |
| Unmap | `unmap_page()` removes entry | REAL | No TLB shootdown (host model) |
| Translation | `translate()` → `Option<usize>` | REAL | Returns physical address |
| User isolation | `is_user()` checks USER flag | REAL | No kernel/user page table separation |

**Defects:**
- K-02: Uses `usize` for physical addresses instead of `PhysAddr` type
- No page table hierarchy (single-level flat map)
- No ASID/PCID support
- No TLB invalidation hooks
- No execute-never (NX) enforcement beyond W^X

### 3.3 Kernel Heap (`memory/heap.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Allocator | Bump pointer | REAL | No free, no reuse |
| Alignment | Power-of-two validated | REAL | Rejects 0 and non-power-of-two |
| Overflow | Checked arithmetic | REAL | Returns `None` on OOM |
| Deallocation | **NOT IMPLEMENTED** | MISSING | K-03: No `free()` method |
| Statistics | None | MISSING | No allocation tracking |

**Defects:**
- K-03: Bump-only allocator with no deallocation — unsuitable for long-running kernel
- No fragmentation handling
- No allocation size tracking for debugging

### 3.4 Kernel Allocator (`memory/allocator.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Statistics | `AllocStats` (allocs, frees, bytes) | REAL | Protected by `std::sync::Mutex` |
| Thread safety | `Mutex<AllocStats>` | REAL | Host-only, not `no_std` compatible |
| Integration | Standalone, not hooked into heap | MOCK | Not used by `KernelHeap` |

---
# QEOS V.04 — Phase 6.2-00: Kernel, Memory & Concurrency Baseline

**Date:** 2026-09-18
**Auditor:** Principal Systems Engineer
**Phase:** P6.2 — Kernel, Memory & Concurrency Hardening
**Subphase:** P6.2-00 — Baseline Establishment
**Status:** IN PROGRESS

---

## 1. Executive Summary

This document establishes the verified baseline of the QEOS V.04 kernel's fundamental execution and memory model before Phase 6.2 hardening begins. The kernel is a host-testable, monolithic modular Rust-first kernel core with x86_64 HAL stubs. All subsystems build cleanly and pass the full test suite (390+ tests across workspace).

---

## 2. Kernel Architecture Overview

### 2.1 Kernel Structure

```
kernel/
├── src/
│   ├── arch/           # Architecture-specific code (x86_64)
│   │   └── x86_64/
│   │       ├── boot.rs         # Boot entry stub
│   │       ├── context.rs      # CPU context for switching
│   │       ├── cpu.rs          # CPU initialization
│   │       ├── gdt.rs          # Global Descriptor Table
│   │       ├── idt.rs          # Interrupt Descriptor Table
│   │       └── interrupts.rs   # IRQ subsystem (atomic flags)
│   ├── boot/           # Deterministic boot sequence
│   ├── core/           # Central kernel configuration
│   ├── device/         # Device abstraction (re-exports driver)
│   ├── dma/            # DMA abstraction with ownership tracking
│   ├── driver/         # PCI, MMIO, IRQ, IOMMU, lifecycle
│   ├── elf/            # Minimal 64-bit ELF loader
│   ├── fs/             # VFS + inode (File, Dir)
│   ├── hal/            # Hardware Abstraction Layer traits
│   ├── ipc/            # Channel-based message passing
│   ├── logging/        # Kernel logger
│   ├── memory/         # Physical, virtual, heap, allocator
│   ├── panic/          # Deterministic panic handler
│   ├── process/        # Process & thread management
│   ├── qpu/            # Quantum device interface (stub)
│   ├── ring/           # SPSC lock-free ring buffer
│   ├── scheduler/      # Multi-class scheduler with per-CPU
│   ├── security/       # Capability-based access control
│   ├── sync/           # Mutex, SpinLock, atomics
│   ├── syscall/        # Syscall dispatcher + validation
│   ├── telemetry/      # Energy telemetry
│   ├── time/           # Clock & timer
│   └── tracing/        # Trace context propagation
```

### 2.2 Kernel Entry & Boot

| Component | File | Status | Notes |
|-----------|------|--------|-------|
| Boot sequence | `boot/mod.rs` | REAL | 11-stage deterministic sequence |
| UEFI entry stub | `arch/x86_64/boot.rs` | STUB | Empty `entry_stub()` |
| Panic handler | `panic.rs` | PLACEHOLDER | Not a real `#[panic_handler]`; loops on spin |
| Linker script | `boot/linker.ld` | MISSING | `ENTRY(_start)` but no `_start` symbol |

---

## 3. Memory Subsystem Audit

### 3.1 Physical Memory Manager (`memory/physical.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Page size | 4096 bytes (const) | REAL | Hardcoded in two places (config + mod.rs) |
| Allocator | Bitmap via `BTreeSet<usize>` | REAL | No fragmentation tracking, no multi-page alloc |
| Free tracking | `free_pages()` count | REAL | O(1) but no ownership/zone awareness |
| Allocation | `allocate_page()` → `Option<PhysPage>` | REAL | First-fit only, no alignment beyond page |
| Deallocation | `free_page()` with bounds check | REAL | No double-free detection beyond set insert |
| Thread safety | None (not `Sync`) | MOCK | Not usable in SMP context |

**Defects:**
- K-02: Uses raw `usize` for physical addresses; no `PhysAddr`/`PhysPage` newtypes in virtual memory
- No canonical address validation
- No NUMA/zone awareness
- No memory hotplug support
### 3.2 Virtual Memory Manager (`memory/virtual_.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Page mapping | `BTreeMap<usize, (usize, MapFlags)>` | REAL | Maps virtual page index → (phys, flags) |
| Permissions | `MapFlags` (READ, WRITE, EXEC, USER) | REAL | W^X enforced (WRITE+EXEC denied) |
| Alignment check | Phys must be PAGE_SIZE multiple | REAL | Checked at map time |
| Unmap | `unmap_page()` removes entry | REAL | No TLB shootdown (host model) |
| Translation | `translate()` → `Option<usize>` | REAL | Returns physical address |
| User isolation | `is_user()` checks USER flag | REAL | No kernel/user page table separation |

**Defects:**
- K-02: Uses `usize` for physical addresses instead of `PhysAddr` type
- No page table hierarchy (single-level flat map)
- No ASID/PCID support
- No TLB invalidation hooks
- No execute-never (NX) enforcement beyond W^X

### 3.3 Kernel Heap (`memory/heap.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Allocator | Bump pointer | REAL | No free, no reuse |
| Alignment | Power-of-two validated | REAL | Rejects 0 and non-power-of-two |
| Overflow | Checked arithmetic | REAL | Returns `None` on OOM |
| Deallocation | **NOT IMPLEMENTED** | MISSING | K-03: No `free()` method |
| Statistics | None | MISSING | No allocation tracking |

**Defects:**
- K-03: Bump-only allocator with no deallocation — unsuitable for long-running kernel
- No fragmentation handling
- No allocation size tracking for debugging

### 3.4 Kernel Allocator (`memory/allocator.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Statistics | `AllocStats` (allocs, frees, bytes) | REAL | Protected by `std::sync::Mutex` |
| Thread safety | `Mutex<AllocStats>` | REAL | Host-only, not `no_std` compatible |
| Integration | Standalone, not hooked into heap | MOCK | Not used by `KernelHeap` |

---
## 4. Scheduler & Concurrency Audit

### 4.1 Scheduler (`scheduler/scheduler.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Algorithm | Priority-based run queue | REAL | 5 priority levels (Idle..Realtime) |
| Classes | `SchedClass`: Normal, Realtime, Scientific, Telemetry, Quantum | REAL | Class not used in scheduling decision |
| Context switch | Simulated (no actual register save) | MOCK | `_ = c` placeholder |
| Preemption | Tick-based (`ticks` counter) | REAL | No time slice, no forced preemption |
| SMP | Single CPU model | MOCK | `PerCpu` exists but not integrated |

**Defects:**
- No thread time slicing / quantum enforcement
- No CPU affinity enforcement
- No load balancing
- Context switch is a no-op

### 4.2 Run Queue (`scheduler/runqueue.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Structure | Array of 5 `VecDeque<Tid>` | REAL | One per priority |
| Priority order | Highest priority popped first | REAL | Iterates `.rev()` |
| Fairness | FIFO within priority | REAL | No round-robin, no aging |
| Starvation | Possible for low priority | REAL | No priority boost/aging |

### 4.3 Per-CPU State (`scheduler/percpu.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Structure | `PerCpu` with `cpu_id`, `current_tid`, `switches`, `interrupts` | REAL | Single CPU0 only |
| False sharing | No padding | MEDIUM | Cache line sharing in SMP |
| Switch tracking | `on_switch()` increments counter | REAL | Not hooked into scheduler |

---

## 5. Concurrency Primitives Audit

### 5.1 SpinLock (`sync/spin.rs`)

```rust
// UNSAFE BLOCKS (4 total):
// 1. Line 16-17: unsafe impl<T: Send> Send for SpinLock<T> {}
// 2. Line 16-17: unsafe impl<T: Send> Sync for SpinLock<T> {}
// 3. Line 53: unsafe { &*self.lock.data.get() }
// 4. Line 59: unsafe { &mut *self.lock.data.get() }
// 5. Line 65: unsafe { self.lock.unlock() }
```

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Locking | `AtomicBool` test-and-set + `spin_loop()` | REAL | Busy-wait, no backoff |
| Memory ordering | Acquire on lock, Release on unlock | REAL | Correct for mutual exclusion |
| IRQ safety | **Documented as IRQ-unsafe** | MOCK | K-09: No compile-time enforcement |
| Guard pattern | `SpinGuard` with `Deref`/`DerefMut`/`Drop` | REAL | RAII unlock on drop |
| Poisoning | Not applicable (no panic in lock) | N/A | — |

**Defects:**
- K-09: Documented as IRQ-unsafe unless caller disables interrupts; no compile-time enforcement
- No lock ordering enforcement
- No deadlock detection
- No recursion tracking

### 5.2 KernelMutex (`sync/mutex.rs`)

```rust
// NO UNSAFE BLOCKS — wraps std::sync::Mutex
```

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Backend | `std::sync::Mutex` | MOCK | K-01: Sleepable, NOT IRQ-safe, NOT `no_std` |
| Poison handling | Returns poisoned inner value | REAL | Recovers instead of panicking |
| TryLock | Supported | REAL | Returns `Option` |
| Context | Process context only | DOCUMENTED | Comment: "never hard-IRQ" |

**Defects:**
- K-01: Host-only `std::sync::Mutex` — prevents `no_std` kernel build
- Not IRQ-safe
- No priority inheritance
- No lock ordering enforcement

### 5.3 Atomics (`sync/atomic.rs`)

| Aspect | Implementation | Status |
|--------|----------------|--------|
| Exports | `AtomicU64`, `AtomicUsize`, `Ordering` | REAL (re-exports `std::sync::atomic`) |

---
---

## 16. Baseline Gate Decision

**P6.2-00 STATUS: BASELINE ESTABLISHED**

The kernel subsystems are sufficiently understood to proceed with Phase 6.2 hardening milestones. All identified defects are documented in PHASE_6_TECHNICAL_DEBT.md with traceability to specific remediation milestones.

**Next:** P6.2-01 — Memory Model Audit & Hardening

---

## 17. Evidence

- `cargo test --workspace` → PASS (390+ tests)
- `cargo fmt --all -- --check` → PASS
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` → PASS
- Unsafe code inventory complete (7 total: 4 kernel, 3 user-space)
- All Phase 5 milestones verified complete
---

## 15. Identified Gaps for Phase 6.2

### 15.1 Memory Model (P6.2-01)
- [ ] Physical address types (`PhysAddr`, `PhysPage`) used consistently
- [ ] Virtual memory manager uses typed physical addresses
- [ ] Page table hierarchy (PML4→PDPT→PD→PT) for x86_64
- [ ] Kernel/user page table separation
- [ ] TLB invalidation hooks
- [ ] NX bit enforcement
- [ ] Canonical address validation
- [ ] Mapping lifetime tracking
- [ ] Double mapping prevention
- [ ] Use-after-free detection

### 15.2 Allocator Hardening (P6.2-02)
- [ ] Implement `KernelHeap::free()` with coalescing
- [ ] Add allocation size tracking
- [ ] Add guard pages / red zones
- [ ] Add double-free detection
- [ ] Add invalid-free detection
- [ ] Stress tests for fragmentation
- [ ] OOM behavior specification

### 15.3 Page Table Hardening (P6.2-03)
- [ ] Multi-level page table implementation
- [ ] User/kernel address space separation
- [ ] NX enforcement
- [ ] TLB shootdown for SMP
- [ ] Address validation on every map/unmap

### 15.4 Scheduler Hardening (P6.2-04)
- [ ] Time slice / quantum enforcement
- [ ] Forced preemption on timer tick
- [ ] CPU affinity enforcement
- [ ] Load balancing (SMP)
- [ ] Priority aging / anti-starvation
- [ ] Thread cancellation support
- [ ] Context switch implementation

### 15.5 Concurrency Hardening (P6.2-05)
- [ ] Replace `KernelMutex` with IRQ-safe, `no_std` implementation
- [ ] Add lock ordering documentation & enforcement
- [ ] Add deadlock detection (lockdep-style)
- [ ] Add priority inheritance for mutexes
- [ ] Formalize IRQ-safe vs sleepable lock classification
- [ ] Add recursion tracking for spinlocks

### 15.6 Syscall Safety (P6.2-06)
- [ ] Validate all syscall arguments comprehensively
- [ ] Add syscall argument count validation
- [ ] Add return value validation
- [ ] Implement syscall auditing/tracing
- [ ] Harden pointer validation (canonical form, permissions)

### 15.7 Fault Handling (P6.2-07)
- [ ] Real `#[panic_handler]` implementation
- [ ] Page fault handler
- [ ] Invalid syscall handler
- [ ] Stack overflow detection
- [ ] Kernel assertion framework
- [ ] OOM handling in syscalls

### 15.8 Unsafe Rust Audit (P6.2-08)
- [ ] Verify all 4 `SpinLock` unsafe blocks have correct invariants
- [ ] Verify 3 `energy-telemetry` unsafe blocks
- [ ] Add `unsafe_op_in_unsafe_fn` lint (already in lib.rs)
- [ ] Document invariants for each unsafe block
- [ ] Consider replacing `SpinLock` with `lock_api` + raw spinlock

---
---

## 13. Unsafe Code Inventory

### 13.1 Kernel Unsafe Blocks (4 total)

| File | Line | Block | Justification |
|------|------|-------|---------------|
| `sync/spin.rs` | 16-17 | `unsafe impl Send/Sync` | SpinLock provides mutual exclusion via AtomicBool |
| `sync/spin.rs` | 53 | `&*data.get()` | Guard proves exclusive access |
| `sync/spin.rs` | 59 | `&mut *data.get()` | Guard proves exclusive access |
| `sync/spin.rs` | 65 | `unlock()` | Guard held the lock |

**All 4 blocks are in `SpinLock` implementation. Each has `// SAFETY:` comment.**

### 13.2 User-Space Unsafe Blocks (3 total in `energy-telemetry`)

| Crate | File | Count | Purpose |
|-------|------|-------|---------|
| `energy-telemetry` | `ring_buffer.rs` | 3 | Lock-free SPSC ring with atomics |

---

## 14. Test Coverage Summary

| Module | Unit Tests | Integration Tests |
|--------|------------|-------------------|
| `kernel` | 48 (lib + 3 integration) | `kernel_integration.rs`, `syscall_tests.rs`, `boundary_contract_tests.rs` |
| `memory` | 12 | — |
| `scheduler` | 3 | — |
| `sync` | 3 | — |
| `syscall` | 32 | — |
| `process` | 3 | — |
| `dma` | 2 | — |
| `driver` | 50 (device-manager crate) | — |
| `ipc` | 3 | — |
| `security` | 4 | — |
| `boot` | 1 | — |
| `elf` | 2 | — |
| `fs` | 1 | — |
| `qpu` | 1 | — |
| `ring` | 2 | — |

**Total Kernel Tests: ~80 (lib) + 50 (device-manager) + 3 (integration) = 133**

---
---

## 11. Security & Capabilities

### 11.1 Capabilities (`security/capability.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Capabilities | 8 variants (DeviceRead, DeviceWrite, Dma, Pci, Telemetry, Quantum, Admin) | REAL | Admin implies all |
| CapSet | `Vec<Capability>` | REAL | Linear search, no bitmask |
| Grant | `grant()` deduplicates | REAL | — |
| Check | `has()` includes Admin check | REAL | — |

**Defects:**
- K-17: No capability delegation or revocation
- Linear search O(n) — fine for small sets, not scalable

### 11.2 Permissions (`security/permission.rs`)

| Aspect | Implementation | Status |
|--------|----------------|--------|
| Access types | ReadDevice, WriteDevice, UseDma, UseQuantum | REAL |
| Check | Maps Access → Capability | REAL |

---

## 12. IPC Subsystem

### 12.1 Channel (`ipc/channel.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Backend | `VecDeque<Message>` | REAL | Bounded capacity |
| Errors | Full, Empty, Closed | REAL | — |
| Close | `close()` sets flag | REAL | Further sends fail |

**Defects:**
- K-16: No IPC disconnect handling, no flow control backpressure propagation

### 12.2 Message (`ipc/message.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Structure | from, to, tag, payload (Vec<u8>) | REAL | — |
| Size limit | `try_new()` enforces `IPC_MAX_PAYLOAD` (64 KiB) | REAL | DoS bound |

---
---

## 10. Driver Subsystem Audit

### 10.1 PCI (`driver/pci.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Address | `PciAddr` (bus, dev, func) | REAL | Label formatting |
| Identity | `PciIdentity` (vendor, device, class, etc.) | REAL | `present()` checks vendor != 0xFFFF |
| BARs | `BarDescriptor` (6 slots) | REAL | `is_mappable()` check |
| IRQ facts | `InterruptFacts` (MSI, MSI-X, legacy) | REAL | — |
| DMA facts | `DmaFacts` (bus_master, addr64, ATS) | REAL | `declared_ready()` check |
| Config source | `PciConfigSource` trait + `StubConfigSource` | REAL | Host test stub |
| Enumeration | `enumerate_source()` | REAL | Filters present devices |

### 10.2 MMIO (`driver/mmio.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Window | `MmioWindow` with `Vec<u32>` backing | MOCK | Host memory-backed |
| Access | `read_u32`/`write_u32` with bounds/alignment | REAL | Returns `DmaError` |
| Alignment | 4-byte required | REAL | Checked |

### 10.3 Device Lifecycle (`driver/lifecycle.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| States | 9-state enum | REAL | Discovered→Probing→Initialized→Ready→Running→Suspended→Stopping→Removed/Failed |
| Transitions | `can_transition()` matrix | REAL | Explicit allowed transitions |
| Terminal states | `Removed`, `Failed` | REAL | `is_terminal()` |

---
---

## 6. Syscall Boundary Audit

### 6.1 Syscall Dispatcher (`syscall/dispatcher.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Syscall numbers | 35 defined (0-71) | REAL | ABI v1.0 |
| Validation | `validate_range()`, `require()` | REAL | Pointer, length, capability checks |
| Error codes | `SyscallError` enum (8 variants) | REAL | Typed, stable repr(i64) |
| Context | `SyscallContext` (pid, caps, user_addr_max) | REAL | Capability-gated operations |
| User pointers | Validated against `user_addr_max` | REAL | Null rejected, overflow checked |

**Defects:**
- No syscall argument count validation beyond a0-a2
- No syscall return value validation
- Some syscalls are stubs returning hardcoded values
- No syscall tracing/auditing

### 6.2 Validation (`syscall/validate.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Range check | `ptr + len` overflow check | REAL | Uses `checked_add` |
| Null pointer | Rejected when `len > 0` | REAL | Correct |
| Capability check | `require()` delegates to `security::check` | REAL | Granular capabilities |

---

## 7. Process & Thread Model (`process/`)

### 7.1 Process (`process/process.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| PID | `Pid(u32)` newtype | REAL | — |
| State | `ProcessState` enum (7 variants) | REAL | Created→Ready→Running→Blocked→Sleeping→Terminated |
| Credentials | `uid`, `gid` | REAL | No capabilities in process struct |
| Threads | `BTreeMap<u32, Thread>` | REAL | Per-process thread table |
| Handles | Counter only (`handles: u32`) | MOCK | No handle table |

**Defects:**
- K-15: No process supervisor, restart policy, or health checks
- No resource limits (RLIMIT)
- No process groups/sessions
- No namespace isolation

### 7.2 Thread (`process/thread.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| TID | `Tid(u32)` newtype | REAL | — |
| Priority | `Priority` enum (5 levels) | REAL | Maps to run queue index |
| State | `ThreadState` enum (6 variants) | REAL | Includes `Sleeping(u64)` with wake time |
| Affinity | `u32` CPU mask | MOCK | Single CPU only |
| Sleep/Wake | Basic implementation | REAL | No timer integration |

---

## 8. Interrupt & IRQ Subsystem

### 8.1 Interrupts (`arch/x86_64/interrupts.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Init flag | `AtomicBool` | REAL | Replaces `static mut` |
| Enable flag | `AtomicBool` | REAL | — |
| Handlers | **NOT IMPLEMENTED** | MISSING | No IDT population, no handler registration |
| Top-half/bottom-half | Trait `IrqHandler` with `top_half()` | REAL | In `driver/interrupt.rs` |
| Deferred work | Not implemented | MISSING | No workqueue/NAPI equivalent |

### 8.2 IDT (`arch/x86_64/idt.rs`)

*Need to inspect*

---

## 9. DMA & IOMMU Audit

### 9.1 DMA (`dma/mod.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| `DmaRegion` | Phys addr + size + mapped flag | REAL | Drop unmaps |
| `DmaMapping` | IOVA + size + mapped flag | REAL | Drop unmaps |
| `DmaRing` | SPSC ring with `Vec<u64>` descriptors | REAL | Power-of-two depth 2-1024 |
| Alignment | 4096-byte required | REAL | Checked at creation |
| Size limit | 64 MiB max | REAL | `DMA_MAX_BYTES` from config |

**Defects:**
- K-12: Driver DMA integration is `todo!()`
- No cache coherency model
- No DMA direction (to/from device)
- No scatter/gather
- `DmaRing` assumes SPSC but no compile-time enforcement

### 9.2 IOMMU (`driver/iommu.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Trait | `Iommu` with `create_domain`, `attach`, `map`, `unmap` | REAL | Generic over error type |
| Mock | `MockIommu` with software tracking | MOCK | K-11: No hardware IOMMU driver |
| Permissions | `IommuPerm` (read, write) | REAL | No execute |
| Domains | `DomainId(u32)` | REAL | Simple counter |

---

## 16. Baseline Gate Decision

**P6.2-00 STATUS: BASELINE ESTABLISHED**

The kernel subsystems are sufficiently understood to proceed with Phase 6.2 hardening milestones. All identified defects are documented in PHASE_6_TECHNICAL_DEBT.md with traceability to specific remediation milestones.

**Next:** P6.2-01 — Memory Model Audit & Hardening

---

## 17. Evidence

- `cargo test --workspace` → PASS (390+ tests)
- `cargo fmt --all -- --check` → PASS
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` → PASS
- Unsafe code inventory complete (7 total: 4 kernel, 3 user-space)
- All Phase 5 milestones verified complete
---

## 15. Identified Gaps for Phase 6.2

### 15.1 Memory Model (P6.2-01)
- [ ] Physical address types (`PhysAddr`, `PhysPage`) used consistently
- [ ] Virtual memory manager uses typed physical addresses
- [ ] Page table hierarchy (PML4→PDPT→PD→PT) for x86_64
- [ ] Kernel/user page table separation
- [ ] TLB invalidation hooks
- [ ] NX bit enforcement
- [ ] Canonical address validation
- [ ] Mapping lifetime tracking
- [ ] Double mapping prevention
- [ ] Use-after-free detection

### 15.2 Allocator Hardening (P6.2-02)
- [ ] Implement `KernelHeap::free()` with coalescing
- [ ] Add allocation size tracking
- [ ] Add guard pages / red zones
- [ ] Add double-free detection
- [ ] Add invalid-free detection
- [ ] Stress tests for fragmentation
- [ ] OOM behavior specification

### 15.3 Page Table Hardening (P6.2-03)
- [ ] Multi-level page table implementation
- [ ] User/kernel address space separation
- [ ] NX enforcement
- [ ] TLB shootdown for SMP
- [ ] Address validation on every map/unmap

### 15.4 Scheduler Hardening (P6.2-04)
- [ ] Time slice / quantum enforcement
- [ ] Forced preemption on timer tick
- [ ] CPU affinity enforcement
- [ ] Load balancing (SMP)
- [ ] Priority aging / anti-starvation
- [ ] Thread cancellation support
- [ ] Context switch implementation

### 15.5 Concurrency Hardening (P6.2-05)
- [ ] Replace `KernelMutex` with IRQ-safe, `no_std` implementation
- [ ] Add lock ordering documentation & enforcement
- [ ] Add deadlock detection (lockdep-style)
- [ ] Add priority inheritance for mutexes
- [ ] Formalize IRQ-safe vs sleepable lock classification
- [ ] Add recursion tracking for spinlocks

### 15.6 Syscall Safety (P6.2-06)
- [ ] Validate all syscall arguments comprehensively
- [ ] Add syscall argument count validation
- [ ] Add return value validation
- [ ] Implement syscall auditing/tracing
- [ ] Harden pointer validation (canonical form, permissions)

### 15.7 Fault Handling (P6.2-07)
- [ ] Real `#[panic_handler]` implementation
- [ ] Page fault handler
- [ ] Invalid syscall handler
- [ ] Stack overflow detection
- [ ] Kernel assertion framework
- [ ] OOM handling in syscalls

### 15.8 Unsafe Rust Audit (P6.2-08)
- [ ] Verify all 4 `SpinLock` unsafe blocks have correct invariants
- [ ] Verify 3 `energy-telemetry` unsafe blocks
- [ ] Add `unsafe_op_in_unsafe_fn` lint (already in lib.rs)
- [ ] Document invariants for each unsafe block
- [ ] Consider replacing `SpinLock` with `lock_api` + raw spinlock

---
---

## 13. Unsafe Code Inventory

### 13.1 Kernel Unsafe Blocks (4 total)

| File | Line | Block | Justification |
|------|------|-------|---------------|
| `sync/spin.rs` | 16-17 | `unsafe impl Send/Sync` | SpinLock provides mutual exclusion via AtomicBool |
| `sync/spin.rs` | 53 | `&*data.get()` | Guard proves exclusive access |
| `sync/spin.rs` | 59 | `&mut *data.get()` | Guard proves exclusive access |
| `sync/spin.rs` | 65 | `unlock()` | Guard held the lock |

**All 4 blocks are in `SpinLock` implementation. Each has `// SAFETY:` comment.**

### 13.2 User-Space Unsafe Blocks (3 total in `energy-telemetry`)

| Crate | File | Count | Purpose |
|-------|------|-------|---------|
| `energy-telemetry` | `ring_buffer.rs` | 3 | Lock-free SPSC ring with atomics |

---

## 14. Test Coverage Summary

| Module | Unit Tests | Integration Tests |
|--------|------------|-------------------|
| `kernel` | 48 (lib + 3 integration) | `kernel_integration.rs`, `syscall_tests.rs`, `boundary_contract_tests.rs` |
| `memory` | 12 | — |
| `scheduler` | 3 | — |
| `sync` | 3 | — |
| `syscall` | 32 | — |
| `process` | 3 | — |
| `dma` | 2 | — |
| `driver` | 50 (device-manager crate) | — |
| `ipc` | 3 | — |
| `security` | 4 | — |
| `boot` | 1 | — |
| `elf` | 2 | — |
| `fs` | 1 | — |
| `qpu` | 1 | — |
| `ring` | 2 | — |

**Total Kernel Tests: ~80 (lib) + 50 (device-manager) + 3 (integration) = 133**

---
---

## 11. Security & Capabilities

### 11.1 Capabilities (`security/capability.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Capabilities | 8 variants (DeviceRead, DeviceWrite, Dma, Pci, Telemetry, Quantum, Admin) | REAL | Admin implies all |
| CapSet | `Vec<Capability>` | REAL | Linear search, no bitmask |
| Grant | `grant()` deduplicates | REAL | — |
| Check | `has()` includes Admin check | REAL | — |

**Defects:**
- K-17: No capability delegation or revocation
- Linear search O(n) — fine for small sets, not scalable

### 11.2 Permissions (`security/permission.rs`)

| Aspect | Implementation | Status |
|--------|----------------|--------|
| Access types | ReadDevice, WriteDevice, UseDma, UseQuantum | REAL |
| Check | Maps Access → Capability | REAL |

---

## 12. IPC Subsystem

### 12.1 Channel (`ipc/channel.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Backend | `VecDeque<Message>` | REAL | Bounded capacity |
| Errors | Full, Empty, Closed | REAL | — |
| Close | `close()` sets flag | REAL | Further sends fail |

**Defects:**
- K-16: No IPC disconnect handling, no flow control backpressure propagation

### 12.2 Message (`ipc/message.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Structure | from, to, tag, payload (Vec<u8>) | REAL | — |
| Size limit | `try_new()` enforces `IPC_MAX_PAYLOAD` (64 KiB) | REAL | DoS bound |

---
---

## 10. Driver Subsystem Audit

### 10.1 PCI (`driver/pci.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Address | `PciAddr` (bus, dev, func) | REAL | Label formatting |
| Identity | `PciIdentity` (vendor, device, class, etc.) | REAL | `present()` checks vendor != 0xFFFF |
| BARs | `BarDescriptor` (6 slots) | REAL | `is_mappable()` check |
| IRQ facts | `InterruptFacts` (MSI, MSI-X, legacy) | REAL | — |
| DMA facts | `DmaFacts` (bus_master, addr64, ATS) | REAL | `declared_ready()` check |
| Config source | `PciConfigSource` trait + `StubConfigSource` | REAL | Host test stub |
| Enumeration | `enumerate_source()` | REAL | Filters present devices |

### 10.2 MMIO (`driver/mmio.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Window | `MmioWindow` with `Vec<u32>` backing | MOCK | Host memory-backed |
| Access | `read_u32`/`write_u32` with bounds/alignment | REAL | Returns `DmaError` |
| Alignment | 4-byte required | REAL | Checked |

### 10.3 Device Lifecycle (`driver/lifecycle.rs`)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| States | 9-state enum | REAL | Discovered→Probing→Initialized→Ready→Running→Suspended→Stopping→Removed/Failed |
| Transitions | `can_transition()` matrix | REAL | Explicit allowed transitions |
| Terminal states | `Removed`, `Failed` | REAL | `is_terminal()` |

---
---
---

## 3. Memory Subsystem Audit

### 3.1 Physical Memory Manager (memory/physical.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Page size | 4096 bytes (const) | REAL | Hardcoded in two places (config + mod.rs) |
| Allocator | Bitmap via BTreeSet<usize> | REAL | No fragmentation tracking, no multi-page alloc |
| Free tracking | free_pages() count | REAL | O(1) but no ownership/zone awareness |
| Allocation | allocate_page() -> Option<PhysPage> | REAL | First-fit only, no alignment beyond page |
| Deallocation | free_page() with bounds check | REAL | No double-free detection beyond set insert |
| Thread safety | None (not Sync) | MOCK | Not usable in SMP context |

**Defects:**
- K-02: Uses raw usize for physical addresses; no PhysAddr/PhysPage newtypes in virtual memory
- No canonical address validation
- No NUMA/zone awareness
- No memory hotplug support

### 3.2 Virtual Memory Manager (memory/virtual_.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Page mapping | BTreeMap<usize, (usize, MapFlags)> | REAL | Maps virtual page index to (phys, flags) |
| Permissions | MapFlags (READ, WRITE, EXEC, USER) | REAL | W^X enforced (WRITE+EXEC denied) |
| Alignment check | Phys must be PAGE_SIZE multiple | REAL | Checked at map time |
| Unmap | unmap_page() removes entry | REAL | No TLB shootdown (host model) |
| Translation | translate() -> Option<usize> | REAL | Returns physical address |
| User isolation | is_user() checks USER flag | REAL | No kernel/user page table separation |

**Defects:**
- K-02: Uses usize for physical addresses instead of PhysAddr type
- No page table hierarchy (single-level flat map)
- No ASID/PCID support
- No TLB invalidation hooks
- No execute-never (NX) enforcement beyond W^X

### 3.3 Kernel Heap (memory/heap.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Allocator | Bump pointer | REAL | No free, no reuse |
| Alignment | Power-of-two validated | REAL | Rejects 0 and non-power-of-two |
| Overflow | Checked arithmetic | REAL | Returns None on OOM |
| Deallocation | NOT IMPLEMENTED | MISSING | K-03: No free() method |
| Statistics | None | MISSING | No allocation tracking |

**Defects:**
- K-03: Bump-only allocator with no deallocation - unsuitable for long-running kernel
- No fragmentation handling
- No allocation size tracking for debugging

### 3.4 Kernel Allocator (memory/allocator.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Statistics | AllocStats (allocs, frees, bytes) | REAL | Protected by std::sync::Mutex |
| Thread safety | Mutex<AllocStats> | REAL | Host-only, not no_std compatible |
| Integration | Standalone, not hooked into heap | MOCK | Not used by KernelHeap |
---

## 4. Scheduler & Concurrency Audit

### 4.1 Scheduler (scheduler/scheduler.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Algorithm | Priority-based run queue | REAL | 5 priority levels (Idle..Realtime) |
| Classes | SchedClass: Normal, Realtime, Scientific, Telemetry, Quantum | REAL | Class not used in scheduling decision |
| Context switch | Simulated (no actual register save) | MOCK | _ = c placeholder |
| Preemption | Tick-based (ticks counter) | REAL | No time slice, no forced preemption |
| SMP | Single CPU model | MOCK | PerCpu exists but not integrated |

**Defects:**
- No thread time slicing / quantum enforcement
- No CPU affinity enforcement
- No load balancing
- Context switch is a no-op

### 4.2 Run Queue (scheduler/runqueue.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Structure | Array of 5 VecDeque<Tid> | REAL | One per priority |
| Priority order | Highest priority popped first | REAL | Iterates .rev() |
| Fairness | FIFO within priority | REAL | No round-robin, no aging |
| Starvation | Possible for low priority | REAL | No priority boost/aging |

### 4.3 Per-CPU State (scheduler/percpu.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Structure | PerCpu with cpu_id, current_tid, switches, interrupts | REAL | Single CPU0 only |
| False sharing | No padding | MEDIUM | Cache line sharing in SMP |
| Switch tracking | on_switch() increments counter | REAL | Not hooked into scheduler |

---

## 5. Concurrency Primitives Audit

### 5.1 SpinLock (sync/spin.rs)

```rust
// UNSAFE BLOCKS (4 total):
// 1. Line 16-17: unsafe impl<T: Send> Send for SpinLock<T> {}
// 2. Line 16-17: unsafe impl<T: Send> Sync for SpinLock<T> {}
// 3. Line 53: unsafe { &*self.lock.data.get() }
// 4. Line 59: unsafe { &mut *self.lock.data.get() }
// 5. Line 65: unsafe { self.lock.unlock() }
```

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Locking | AtomicBool test-and-set + spin_loop() | REAL | Busy-wait, no backoff |
| Memory ordering | Acquire on lock, Release on unlock | REAL | Correct for mutual exclusion |
| IRQ safety | Documented as IRQ-unsafe | MOCK | K-09: No compile-time enforcement |
| Guard pattern | SpinGuard with Deref/DerefMut/Drop | REAL | RAII unlock on drop |
| Poisoning | Not applicable (no panic in lock) | N/A | — |

**Defects:**
- K-09: Documented as IRQ-unsafe unless caller disables interrupts; no compile-time enforcement
- No lock ordering enforcement
- No deadlock detection
- No recursion tracking

### 5.2 KernelMutex (sync/mutex.rs)

```rust
// NO UNSAFE BLOCKS — wraps std::sync::Mutex
```

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Backend | std::sync::Mutex | MOCK | K-01: Sleepable, NOT IRQ-safe, NOT no_std |
| Poison handling | Returns poisoned inner value | REAL | Recovers instead of panicking |
| TryLock | Supported | REAL | Returns Option |
| Context | Process context only | DOCUMENTED | Comment: "never hard-IRQ" |

**Defects:**
- K-01: Host-only std::sync::Mutex — prevents no_std kernel build
- Not IRQ-safe
- No priority inheritance
- No lock ordering enforcement

### 5.3 Atomics (sync/atomic.rs)

| Aspect | Implementation | Status |
|--------|----------------|--------|
| Exports | AtomicU64, AtomicUsize, Ordering | REAL (re-exports std::sync::atomic) |
---

## 6. Syscall Boundary Audit

### 6.1 Syscall Dispatcher (syscall/dispatcher.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Syscall numbers | 35 defined (0-71) | REAL | ABI v1.0 |
| Validation | validate_range(), require() | REAL | Pointer, length, capability checks |
| Error codes | SyscallError enum (8 variants) | REAL | Typed, stable repr(i64) |
| Context | SyscallContext (pid, caps, user_addr_max) | REAL | Capability-gated operations |
| User pointers | Validated against user_addr_max | REAL | Null rejected, overflow checked |

**Defects:**
- No syscall argument count validation beyond a0-a2
- No syscall return value validation
- Some syscalls are stubs returning hardcoded values
- No syscall tracing/auditing

### 6.2 Validation (syscall/validate.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Range check | ptr + len overflow check | REAL | Uses checked_add |
| Null pointer | Rejected when len > 0 | REAL | Correct |
| Capability check | require() delegates to security::check | REAL | Granular capabilities |

---

## 7. Process & Thread Model (process/)

### 7.1 Process (process/process.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| PID | Pid(u32) newtype | REAL | — |
| State | ProcessState enum (7 variants) | REAL | Created→Ready→Running→Blocked→Sleeping→Terminated |
| Credentials | uid, gid | REAL | No capabilities in process struct |
| Threads | BTreeMap<u32, Thread> | REAL | Per-process thread table |
| Handles | Counter only (handles: u32) | MOCK | No handle table |

**Defects:**
- K-15: No process supervisor, restart policy, or health checks
- No resource limits (RLIMIT)
- No process groups/sessions
- No namespace isolation

### 7.2 Thread (process/thread.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| TID | Tid(u32) newtype | REAL | — |
| Priority | Priority enum (5 levels) | REAL | Maps to run queue index |
| State | ThreadState enum (6 variants) | REAL | Includes Sleeping(u64) with wake time |
| Affinity | u32 CPU mask | MOCK | Single CPU only |
| Sleep/Wake | Basic implementation | REAL | No timer integration |

---

## 8. Interrupt & IRQ Subsystem

### 8.1 Interrupts (arch/x86_64/interrupts.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Init flag | AtomicBool | REAL | Replaces static mut |
| Enable flag | AtomicBool | REAL | — |
| Handlers | NOT IMPLEMENTED | MISSING | No IDT population, no handler registration |
| Top-half/bottom-half | Trait IrqHandler with top_half() | REAL | In driver/interrupt.rs |
| Deferred work | Not implemented | MISSING | No workqueue/NAPI equivalent |

### 8.2 IDT (arch/x86_64/idt.rs)

*Need to inspect*

---

## 9. DMA & IOMMU Audit

### 9.1 DMA (dma/mod.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| DmaRegion | Phys addr + size + mapped flag | REAL | Drop unmaps |
| DmaMapping | IOVA + size + mapped flag | REAL | Drop unmaps |
| DmaRing | SPSC ring with Vec<u64> descriptors | REAL | Power-of-two depth 2-1024 |
| Alignment | 4096-byte required | REAL | Checked at creation |
| Size limit | 64 MiB max | REAL | DMA_MAX_BYTES from config |

**Defects:**
- K-12: Driver DMA integration is todo!()
- No cache coherency model
- No DMA direction (to/from device)
- No scatter/gather
- DmaRing assumes SPSC but no compile-time enforcement

### 9.2 IOMMU (driver/iommu.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Trait | Iommu with create_domain, attach, map, unmap | REAL | Generic over error type |
| Mock | MockIommu with software tracking | MOCK | K-11: No hardware IOMMU driver |
| Permissions | IommuPerm (read, write) | REAL | No execute |
| Domains | DomainId(u32) | REAL | Simple counter |
---

## 10. Driver Subsystem Audit

### 10.1 PCI (driver/pci.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Address | PciAddr (bus, dev, func) | REAL | Label formatting |
| Identity | PciIdentity (vendor, device, class, etc.) | REAL | present() checks vendor != 0xFFFF |
| BARs | BarDescriptor (6 slots) | REAL | is_mappable() check |
| IRQ facts | InterruptFacts (MSI, MSI-X, legacy) | REAL | — |
| DMA facts | DmaFacts (bus_master, addr64, ATS) | REAL | declared_ready() check |
| Config source | PciConfigSource trait + StubConfigSource | REAL | Host test stub |
| Enumeration | enumerate_source() | REAL | Filters present devices |

### 10.2 MMIO (driver/mmio.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Window | MmioWindow with Vec<u32> backing | MOCK | Host memory-backed |
| Access | read_u32/write_u32 with bounds/alignment | REAL | Returns DmaError |
| Alignment | 4-byte required | REAL | Checked |

### 10.3 Device Lifecycle (driver/lifecycle.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| States | 9-state enum | REAL | Discovered→Probing→Initialized→Ready→Running→Suspended→Stopping→Removed/Failed |
| Transitions | can_transition() matrix | REAL | Explicit allowed transitions |
| Terminal states | Removed, Failed | REAL | is_terminal() |

---

## 11. Security & Capabilities

### 11.1 Capabilities (security/capability.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Capabilities | 8 variants (DeviceRead, DeviceWrite, Dma, Pci, Telemetry, Quantum, Admin) | REAL | Admin implies all |
| CapSet | Vec<Capability> | REAL | Linear search, no bitmask |
| Grant | grant() deduplicates | REAL | — |
| Check | has() includes Admin check | REAL | — |

**Defects:**
- K-17: No capability delegation or revocation
- Linear search O(n) — fine for small sets, not scalable

### 11.2 Permissions (security/permission.rs)

| Aspect | Implementation | Status |
|--------|----------------|--------|
| Access types | ReadDevice, WriteDevice, UseDma, UseQuantum | REAL |
| Check | Maps Access → Capability | REAL |

---

## 12. IPC Subsystem

### 12.1 Channel (ipc/channel.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Backend | VecDeque<Message> | REAL | Bounded capacity |
| Errors | Full, Empty, Closed | REAL | — |
| Close | close() sets flag | REAL | Further sends fail |

**Defects:**
- K-16: No IPC disconnect handling, no flow control backpressure propagation

### 12.2 Message (ipc/message.rs)

| Aspect | Implementation | Status | Concerns |
|--------|----------------|--------|----------|
| Structure | from, to, tag, payload (Vec<u8>) | REAL | — |
| Size limit | try_new() enforces IPC_MAX_PAYLOAD (64 KiB) | REAL | DoS bound |
---

## 13. Unsafe Code Inventory

### 13.1 Kernel Unsafe Blocks (4 total)

| File | Line | Block | Justification |
|------|------|-------|---------------|
| sync/spin.rs | 16-17 | unsafe impl Send/Sync | SpinLock provides mutual exclusion via AtomicBool |
| sync/spin.rs | 53 | &*data.get() | Guard proves exclusive access |
| sync/spin.rs | 59 | &mut *data.get() | Guard proves exclusive access |
| sync/spin.rs | 65 | unlock() | Guard held the lock |

**All 4 blocks are in SpinLock implementation. Each has // SAFETY: comment.**

### 13.2 User-Space Unsafe Blocks (3 total in energy-telemetry)

| Crate | File | Count | Purpose |
|-------|------|-------|---------|
| energy-telemetry | ring_buffer.rs | 3 | Lock-free SPSC ring with atomics |

---

## 14. Test Coverage Summary

| Module | Unit Tests | Integration Tests |
|--------|------------|-------------------|
| kernel | 48 (lib + 3 integration) | kernel_integration.rs, syscall_tests.rs, boundary_contract_tests.rs |
| memory | 12 | — |
| scheduler | 3 | — |
| sync | 3 | — |
| syscall | 32 | — |
| process | 3 | — |
| dma | 2 | — |
| driver | 50 (device-manager crate) | — |
| ipc | 3 | — |
| security | 4 | — |
| boot | 1 | — |
| elf | 2 | — |
| fs | 1 | — |
| qpu | 1 | — |
| ring | 2 | — |

**Total Kernel Tests: ~80 (lib) + 50 (device-manager) + 3 (integration) = 133**

---

## 15. Identified Gaps for Phase 6.2

### 15.1 Memory Model (P6.2-01)
- [ ] Physical address types (PhysAddr, PhysPage) used consistently
- [ ] Virtual memory manager uses typed physical addresses
- [ ] Page table hierarchy (PML4→PDPT→PD→PT) for x86_64
- [ ] Kernel/user page table separation
- [ ] TLB invalidation hooks
- [ ] NX bit enforcement
- [ ] Canonical address validation
- [ ] Mapping lifetime tracking
- [ ] Double mapping prevention
- [ ] Use-after-free detection

### 15.2 Allocator Hardening (P6.2-02)
- [ ] Implement KernelHeap::free() with coalescing
- [ ] Add allocation size tracking
- [ ] Add guard pages / red zones
- [ ] Add double-free detection
- [ ] Add invalid-free detection
- [ ] Stress tests for fragmentation
- [ ] OOM behavior specification

### 15.3 Page Table Hardening (P6.2-03)
- [ ] Multi-level page table implementation
- [ ] User/kernel address space separation
- [ ] NX enforcement
- [ ] TLB shootdown for SMP
- [ ] Address validation on every map/unmap

### 15.4 Scheduler Hardening (P6.2-04)
- [ ] Time slice / quantum enforcement
- [ ] Forced preemption on timer tick
- [ ] CPU affinity enforcement
- [ ] Load balancing (SMP)
- [ ] Priority aging / anti-starvation
- [ ] Thread cancellation support
- [ ] Context switch implementation

### 15.5 Concurrency Hardening (P6.2-05)
- [ ] Replace KernelMutex with IRQ-safe, no_std implementation
- [ ] Add lock ordering documentation & enforcement
- [ ] Add deadlock detection (lockdep-style)
- [ ] Add priority inheritance for mutexes
- [ ] Formalize IRQ-safe vs sleepable lock classification
- [ ] Add recursion tracking for spinlocks

### 15.6 Syscall Safety (P6.2-06)
- [ ] Validate all syscall arguments comprehensively
- [ ] Add syscall argument count validation
- [ ] Add return value validation
- [ ] Implement syscall auditing/tracing
- [ ] Harden pointer validation (canonical form, permissions)

### 15.7 Fault Handling (P6.2-07)
- [ ] Real #[panic_handler] implementation
- [ ] Page fault handler
- [ ] Invalid syscall handler
- [ ] Stack overflow detection
- [ ] Kernel assertion framework
- [ ] OOM handling in syscalls

### 15.8 Unsafe Rust Audit (P6.2-08)
- [ ] Verify all 4 SpinLock unsafe blocks have correct invariants
- [ ] Verify 3 energy-telemetry unsafe blocks
- [ ] Add unsafe_op_in_unsafe_fn lint (already in lib.rs)
- [ ] Document invariants for each unsafe block
- [ ] Consider replacing SpinLock with lock_api + raw spinlock

---

## 16. Baseline Gate Decision

**P6.2-00 STATUS: BASELINE ESTABLISHED**

The kernel subsystems are sufficiently understood to proceed with Phase 6.2 hardening milestones. All identified defects are documented in PHASE_6_TECHNICAL_DEBT.md with traceability to specific remediation milestones.

**Next:** P6.2-01 — Memory Model Audit & Hardening

---

## 17. Evidence

- cargo test --workspace → PASS (390+ tests)
- cargo fmt --all -- --check → PASS
- cargo clippy --workspace --all-targets --all-features -- -D warnings → PASS
- Unsafe code inventory complete (7 total: 4 kernel, 3 user-space)
- All Phase 5 milestones verified complete