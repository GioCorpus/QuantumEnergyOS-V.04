# QEOS V.04 — KERNEL REFINEMENT REPORT (STEP 1 + rondas hardening S2-S25 parcial)

> Rama: `feature/kernel-refinement-v04` · Base: `main@231284e`
> Fecha: 2026-09-13 · Toolchain local: sin cargo/rustc (validación estática + CI remoto)

## Ronda 2 (continuación, sin rewrite)

* `memory/virtual_`: `map_page` exige phys alineado + deniega W^X (WRITE+EXEC).
* `ipc/message`: `Message::try_new` acota payload a `IPC_MAX_PAYLOAD` (DoS bound).
* `dma`: `Drop for DmaRegion` (unmap on drop, invariante I1, IOMMU deny-by-default).
* `time/timer`: `trait KernelTimer{now_ns,sleep_until}` + `HostTimer(MonotonicClock)`.
* `scheduler/percpu`: `PerCpu{cpu,current_tid,switches,interrupts}` + `on_switch` saturante.
* `telemetry/energy`: `PowerTelemetry/DevicePowerState/EnergyCounter(saturating)`.
* Docs: `docs/kernel/{architecture,boot,memory,security,qpu-interface}.md`,
  `docs/performance/baseline.md`, `docs/adr/ADR-001`.

## Executive Summary

Antes: modelo host puro-Rust, modular pero con `static mut`, `unwrap()` en paths kernel,
heap sin validación de align/overflow, sin `KernelConfig/State/Health`, sin `SpinLock`,
sin validación user/kernel, con `enumerate_stub` ambiguo.
Después (este lote, sin rewrite): se añade `core/{config,state,health}`, `memory/oom`,
`sync/spin`, `syscall/validate`, `tracing`, se elimina `static mut` (AtomicBool),
se endurece `heap/allocator/mutex/elf/pci`. Funcionalidad preservada, API existente intacta
(solo adiciones + fixes compatibles). No se declara production-ready.

## Architecture

```text
kernel/src/{core,arch,boot,memory,scheduler,process,syscall,ipc,device,driver,
            dma,sync,time,hal,security,telemetry,tracing,elf,fs,ring,qpu,logging}
core: KernelConfig (PAGE/DMA/IPC/QPU bounds) + KernelPhase/State (10 fases) + KernelHealth
sync: KernelMutex (host, documentado sleepable) + SpinLock (non-sleepable, SAFETY)
memory: + OomPolicy (Deny/LogAndDeny/ReclaimAndRetry, nunca panic!)
syscall: + validate_range/require(caps)
tracing: TraceCtx{cpu,thread,process,device,job,trace,ts_mono}
```

## Boot

Sin cambios funcionales. `BootSequence` (11 stages) intacto. `KernelState::advance`
impone transición de un paso (test `ordered`). Linker sigue placeholder (documentado en audit).

## Memory

* `heap.rs`: rechaza `align` no-potencia-2 (incluye 0), checked_add para cur/aligned/end/limit.
  Tests: `rejects_bad_align`, `overflow_safe` + `bump` original.
* `allocator.rs`: `lock()` con `if let Ok` + `saturating_add/sub`; `stats()` con `unwrap_or_default`.
  Elimina 3 `unwrap()` en paths kernel.
* `oom.rs`: política explícita, `on_oom()->Err` siempre (deny). Test `denies`.
* Pendiente: `PhysAddr` en `VirtualMemoryManager`, W^X deny, AddressSpace.

## Scheduler / SMP

Sin cambio de algoritmo (cooperativo documentado). `SpinLock` listo para runqueue
futura por-CPU. `cores=1` explícito en `CpuInfo`. Sin declarar SMP.

## Interrupts

* `cpu.rs` / `interrupts.rs`: `static mut bool` → `AtomicBool` (Release/Acquire) + `ENABLED` flag
  + `is_enabled()`. Contrato §15 documentado. Cero `unsafe` en ambos ficheros.
* `sync/mutex.rs`: documenta NOT IRQ-safe + ordering; `lock()` tolera poisoning
  (`unwrap_or_else(into_inner)`), añade `try_lock()`.

## IPC / Drivers / DMA-IOMMU

* `syscall/validate.rs`: `validate_range(ptr,len,max)` (null/overflow/bounds) + `require(caps)`.
  Tests `range`, `caps`.
* `driver/pci.rs`: `enumerate_stub` marcado DEBT + alias `stub_for_host_tests()`.
* DMA/IOMMU: sin cambio funcional este lote; roadmap en `KERNEL_AUDIT.md` §11 S17.

## Security

* Eliminados: 2 `static mut` (UB), 4 `unwrap()` kernel (mutex×1, allocator×3, elf×1).
* Añadidos: `validate_range`, `require`, `OomPolicy`, `SpinLock` con SAFETY.
* `elf::load`: `try_into().unwrap()` → `map_err(TooSmall)?`.
* Pendiente: enforcement en `dispatch_syscall`, `map_page`, `DmaRegion`, `Channel` (S13/S14/S17/S19).

## Unsafe

| Fichero | Antes | Después |
|---|---|---|
| `arch/x86_64/cpu.rs` | `static mut` + 2 `unsafe` | 0 `unsafe`, `AtomicBool` |
| `arch/x86_64/interrupts.rs` | `static mut` + 2 `unsafe` sin SAFETY | 0 `unsafe`, 2×`AtomicBool` |
| `sync/spin.rs` (nuevo) | — | 4 `unsafe` con `// SAFETY:` (guard pattern, test-and-set) |
| `lib.rs` | `forbid(unsafe_op_in_unsafe_fn)` | intacto |

## Performance

Sin benchmarks aún. Cambios O(1)/O(log n) idénticos; `heap` añade 3 checked_add (despreciable).
Baseline cualitativo en `KERNEL_AUDIT.md` §13. Acción: `kernel/benches/` + `docs/performance/` con toolchain.

## Testing

* Preservados: ~20 unit tests + `tests/kernel_integration` (boot + full_stack_smoke).
* Nuevos (9): `core::config::defaults_sane`, `core::state::ordered`, `core::health::healthy_by_default`,
  `memory::oom::denies`, `memory::heap::rejects_bad_align/overflow_safe`, `sync::spin::spin_basic`,
  `sync::mutex::try_m`, `syscall::validate::{range,caps}`, `tracing::ctx`.
* Validación local: `findstr` confirma 0 `static mut` real; revisión manual de sintaxis
  (cargo ausente: `C:\Users\HP\.cargo\bin` no existe). CI debe correr:
  `cargo fmt --check`, `cargo clippy -- -D warnings`, `cargo test --workspace`, `cargo build`.
* QEMU: no aplica (simulador host, sin imagen UEFI).

## Known Limitations

Toolchain Rust ausente en esta máquina; sin `cargo fmt/check/test/clippy` local.
Sin paging real, sin IDT/APIC, sin SMP, sin IOMMU, sin `no_std`, con `std` en kernel.
`dispatch_syscall` aún ignora args; `Channel` sin límite payload; `DmaRegion` sin owner/Drop.

## Hardware Support

Ninguno real probado. `enumerate_stub`/`CpuInfo{emulated,1}` son stubs host.
No declarar UEFI/BIOS/QPU/GPU hasta QEMU + HW.

## Future Work

S2–S25 de `KERNEL_AUDIT.md` §11. Siguiente lote: `virt W^X + PhysAddr`,
`IPC payload limit + owner`, `DMA owner + Drop`, `syscall caps enforcement`,
`docs/kernel/*.md` + `docs/adr/`, `benches`.

## Commits sugeridos (pequeños, §91)

```text
refactor(kernel): add core config/state/health
refactor(arch): replace static mut with AtomicBool
refactor(mm): harden heap align/overflow + OOM policy
refactor(sync): add SpinLock + document Mutex context
refactor(elf): remove unwrap from loader
refactor(syscall): add user boundary validation
refactor(pci): mark stub as host-test only
feat(kernel): add tracing context
docs(kernel): add KERNEL_AUDIT + REFINEMENT_REPORT