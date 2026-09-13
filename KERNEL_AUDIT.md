# QEOS V.04 — KERNEL AUDIT (STEP 1)

> Fecha: 2026-09-13 · Rama base: `main` @ `231284e` · Alcance: `kernel/` + workspace
> Prioridad: CORRECTNESS > SAFETY > STABILITY > MODULARITY > OBSERVABILITY > PERFORMANCE > EXPERIMENTAL
> Principio: si puede vivir en user-space, queda fuera del kernel.

## 0. Git baseline

```text
Branch: main (up to date with origin/main)
Status: modified .github/workflows/ci.yml, modified crates/quantum-runtime/src/lib.rs
Untracked: boot/ browser/ dashboard/ desktop/ drivers/ energy/ hardware/ identity/
           migrations/ packaging/ quantum/ quartz5d/ scripts/ security/ system/
           telemetry/ tests/ tools/ pnpm-workspace.yaml + 2 nuevos ficheros quantum-runtime
Log (últimos): 231284e testing / 35133a3 testing / 9da880a feat: expand workspace crates...
Toolchain local: cargo/rustc NO instalados (C:\Users\HP\.cargo\bin ausente).
Validación: estática (lectura completa) + CI remoto. No se declara boot real probado.
```

## 1. Current Architecture

### 1.1 Layout real

```text
kernel/
  Cargo.toml (qeos-kernel 1.0.0, lib+bin, sin dependencias externas)
  linker/x86_64.ld (placeholder UEFI, ENTRY(_start), base 0x100000)
  boot/boot.md, docs/{SYSABI.md, debugging.md, MILESTONES.md}
  src/lib.rs (21 módulos, VERSION 1.0.0, forbid(unsafe_op_in_unsafe_fn))
  src/main.rs (host boot simulator, std, println!)
  src/{arch,boot,device,dma,driver,elf,fs,hal,ipc,logging,memory,
       panic,process,qpu,ring,scheduler,security,sync,syscall,telemetry,time}
  tests/kernel_integration.rs (2 tests: boot determinista + full_stack_smoke)
```

Workspace raíz: `kernel + 6 crates` (system-core, quantum-runtime, energy-telemetry,
quartz5d, identity-service, hardware-abstraction) + dashboard (Vite/React) + docs.
El kernel NO depende de tokio/serde; los crates de user-space sí. Separación correcta.

### 1.2 Boot Process (real vs simulado)

`src/boot/mod.rs`: `enum Stage {Entry,Cpu,PhysMem,VirtMem,Irq,Sched,Devices,Vfs,Ipc,Services,Userspace}`
+ `BootSequence::stages() -> Vec<Stage>` determinista (11 etapas, test `order`).
`src/main.rs`: itera stages + `Logger` + `println!("[INIT] ... simulated")`.
`src/arch/x86_64/boot.rs` (no leído a fondo, stub), `linker/x86_64.ld` placeholder.
**Realidad: NO hay entry UEFI/BIOS, ni GDT/IDT real, ni paging CR3, ni salto a userspace.
Es un simulador host. Clasificación: PROTOTYPE (útil como modelo).**

### 1.3 Memory Architecture

* `memory/mod.rs`: `PAGE_SIZE=4096`, re-exports.
* `physical.rs`: `PhysAddr(usize)`, `PhysPage(usize)`, `PhysicalMemoryManager{total,free:BTreeSet,allocated}`.
  `allocate_page()->Option`, `free_page()` con guard `p<total`. Tests alloc_free/oom. **Modelo host.**
* `virtual_.rs`: `VirtAddr/VirtPage`, `MapFlags(READ=1,WRITE=2,EXEC=4,USER=8)`,
  `VirtualMemoryManager{map:BTreeMap<usize,(usize,MapFlags)>}` con map/unmap/translate/is_user.
* `heap.rs`: `KernelHeap{base,size,used}` bump-only `alloc(n,align)->Option<usize>`.
* `allocator.rs`: `KernelAllocator{stats:Mutex<AllocStats>}` solo contabilidad.
* `arch/x86_64/paging.rs`: solo constantes + `p4/p3/p2/p1_index(v)`. Sin tablas reales.

### 1.4 Interrupt Architecture

`arch/x86_64/{idt.rs,gdt.rs,interrupts.rs,cpu.rs}`: stubs.
`cpu.rs`: `static mut INIT:bool` + `init()` con `unsafe{INIT=true}` + `info()->{GenuineIntel(emulated),1}`.
`interrupts.rs`: `static mut INIT` + `init/enable/disable` vacíos. Sin IDT/APIC/MSI/MSI-X,
sin dispatcher, sin deferred work, sin IRQ numbers. **BROKEN como driver real, STABLE como stub.**

### 1.5 Scheduler Architecture

`process/thread.rs`: `Tid(u32)`, `Priority{Idle0,Low1,Normal2,High3,Realtime4}`,
`ThreadState{Ready,Running,Blocked,Sleeping(u64),Terminated}`, `Thread{tid,prio,state,affinity}` + sleep/wake.
`process/process.rs`: `Pid(u32)`, `ProcessState{Created,Ready,Running,Blocked,Sleeping,Terminated}`,
`Credentials{uid,gid}`, `Process{pid,state,creds,threads:BTreeMap,handles:u32}`.
`process/context.rs`: `ThreadContext{entry:u64,stack:u64}` sin registros.
`scheduler/runqueue.rs`: `RunQueue{queues:[VecDeque;5]}` push por prioridad, pop reverso (mayor primero).
`scheduler/scheduler.rs`: `SchedClass{Normal,Realtime,Scientific,Telemetry,Quantum}`,
`Scheduler{rq,current:Option<Tid>,ticks}` + spawn/schedule (ticks++). Sin preemption,
sin affinity, sin load-balancing, sin SMP. **Modelo cooperativo correcto para host.**

### 1.6 Process / Thread Model

Separados correctamente (Process vs Thread vs Scheduler vs RunQueue). Falta:
AddressSpace, KernelStack/UserStack, SavedRegisters, Credentials completas (caps),
Handles table, exit codes, reaping. `handles:u32` es contador, no tabla.

### 1.7 IPC

`ipc/message.rs`: `Message{from,to,tag,payload:Vec<u8>}` sin límite de tamaño.
`ipc/channel.rs`: `Channel{q:VecDeque,cap,closed}` + send/recv/close + errores Full/Empty/Closed.
`ipc/endpoint.rs`: `Handle(u32)`, `Endpoint{id,owner}` sin validación de owner.
Sin shared-memory, sin event notification, sin ownership por proceso, sin bloqueo/wakeup.
**STABLE como cola acotada host; PROTOTYPE como IPC kernel.**

### 1.8 Syscalls

`syscall/numbers.rs`: 11 números (Read0,Write1,Open2,Close3,Spawn10,Exit11,Sleep20,Send30,Recv31,Map40,Alloc41).
`syscall/dispatcher.rs`: `dispatch_syscall(n,a0,a1,a2)->Result<u64,SyscallError{Unknown,Denied,InvalidArg}>`
ignora args y retorna constantes (0,3,100,0x1000,0x2000). Sin validación de punteros,
sin tabla de handles, sin capabilities. Comentario "never exposes internals" correcto
en intención pero no enforced. **PROTOTYPE.**

### 1.9 Device Model / Drivers

`driver/bus.rs`: `BusKind` (asumido). `driver/device.rs`: `DeviceId(String)`,
`Device{id,bus,vendor,product}`, `trait Driver{name,probe}`, `DriverRegistry{devices:Vec}`.
`device/mod.rs`: re-export. Sin estados (Discovered/Ready/Error...), sin lifecycle
(probe/init/start/stop/reset/remove), sin ownership. **PROTOTYPE.**

### 1.10 PCIe / DMA / IOMMU

`driver/pci.rs`: `PciAddr{bus,dev,func}` + `enumerate_stub()->vec![0:1:0]`. Sin BAR, sin caps, sin MSI-X, sin config space.
`dma/mod.rs` + `driver/dma.rs`: `DmaRegion{phys,size,mapped}` con checks
(misaligned si phys%4096!=0, tooLarge si size==0||>64MiB). `map/unmap` booleanos.
Sin IOMMU, sin lifetime ligado a device, sin sync, sin unmap obligatorio (Drop no implementado).
**EXPERIMENTAL pero bien acotado.**

### 1.11 SMP / Synchronization / Time

Sin SMP (cores=1 hardcodeado). Sin PerCpu.
`sync/mutex.rs`: `KernelMutex<T>{inner:StdMutex}` + `lock()->MutexGuard` con `unwrap()`.
`sync/atomic.rs`: re-export `AtomicU64/AtomicUsize/Ordering`. Sin SpinLock/RwLock/WaitQueue/Semaphore,
sin reglas IRQ-safe/sleepable, sin lock-ordering.
`ring/mod.rs`: `SpscRing<T:Copy,N>{buf:[Option<T>;N],head/tail:Atomic,policy}` con
DropNew/OverwriteOld/Backpressure. Usa `&mut self` (no concurrente real) — claim "lock-free" no demostrado.
`time/clock.rs`: `MonotonicClock{Instant}` + `Timer{deadline_ns}` + expired(). Correcto host; sin backend HAL, sin ticks.
`hal/mod.rs`: traits `CpuHal/TimerHal/PcieHal` + `NullHal`. Buena dirección, sin backends reales.

### 1.12 Security / Observability / Testing / Performance / Unsafe

* Security: `capability.rs` (`Capability{DeviceRead/Write,Dma,Pci,Telemetry,Quantum,Admin}`, `CapSet(Vec)` + grant/has con Admin bypass),
  `permission.rs` (`check(caps,Access)`). No enforced fuera de tests. Sin aislamiento de memoria/DMA, sin validación user/kernel.
* Observability: `logging/logger.rs` (`Level{Trace..Error}`, `Logger{level}` + println!), `telemetry/mod.rs` (`Sample{ts_mono_ns,source,value}`).
  Sin trace_id/cpu_id/thread_id, sin métricas (ctx switches, page faults...), sin niveles por build.
* Testing: unit tests por módulo + `tests/kernel_integration.rs` (full_stack_smoke). CI: fmt/clippy/test/build (x86_64+aarch64) + frontend + python + migrations + audit. Sólido en intención.
* Performance: sin benchmarks, sin baseline. `docs/` sin `performance/`.
* Unsafe: 5 ocurrencias, solo `static mut INIT` en `arch/x86_64/{cpu,interrupts}.rs` + `forbid(unsafe_op_in_unsafe_fn)` en lib.rs. Sin SAFETY en interrupts.rs. Sin MMIO/DMA unsafe real (porque todo es modelo host).

### 1.13 Clasificación por componente

```text
PRODUCTION: — (nada reclama producción; correcto no reclamarlo)
STABLE: boot/Stage sequence, memory/physical+virtual modelo, scheduler/runqueue,
        process/thread tipos, ipc/channel, security/capability tipos, dma checks,
        elf validación básica, ring SPSC modelo, logging/telemetry primitivas, hal traits
EXPERIMENTAL: qpu/QuantumDevice trait + UnsupportedDevice (bien aislado), MapFlags EXEC/USER
PROTOTYPE: main.rs simulator, syscall dispatcher (args ignorados), pci enumerate_stub,
           device registry, vfs/inode (no auditado a fondo pero usado en test), heap bump-only
BROKEN (como kernel real): GDT/IDT/APIC/MSI, paging real, context switch asm, SMP/PerCpu,
                           IOMMU, timer IRQ, driver lifecycle, user/kernel boundary
UNUSED: SchedClass::Scientific/Telemetry/Quantum (solo enum), arch/x86_64/{gdt,idt,boot,context} (stubs sin uso en tests)
DUPLICATED: memory::PAGE_SIZE vs arch::paging::PAGE_SIZE; dma/mod.rs vs driver/dma.rs (revisar solape)
```

## 2. Critical Problems (ordenados por riesgo)

1. `static mut INIT` en cpu.rs/interrupts.rs — data race / UB en presencia de threads; `is_init()` lee sin sincronización. Reemplazar por `AtomicBool`.
2. `KernelMutex` = `std::Mutex` + `.lock().unwrap()` — puede `panic!` por poisoning; duerme (no IRQ-safe); depende de `std` (no `no_std`); sin documentación de contexto.
3. `KernelHeap::alloc(n,align)` — si `align==0` → división por cero (`!(align-1)`); sin overflow check (`a+n` puede wrapear); sin `free`; `used` no distingue alineamiento interno.
4. `VirtualMemoryManager` usa `usize` para phys, no `PhysAddr/PhysPage`; no valida canonical/W^X/RWX; `USER` flag existe pero nunca enforced; `translate` expone phys sin checks.
5. `dispatch_syscall` ignora `a0/a1/a2`, retorna constantes mágicas (`100`, `0x1000`) — cualquier refactor que confíe en esto es inseguro; falta validación de punteros/longitudes/handles/caps.
6. `SpscRing` declara "lock-free" pero API es `&mut self` (exclusiva) y `buf` no es `Sync`; ordering Acquire/Release sin justificación de sincronización con `buf` (no hay fence que proteja `buf` si se compartiera). Riesgo de copiar patrón a IRQ/SMP real.
7. `elf::load` usa `try_into().unwrap()` en slice de 8 bytes (línea entry) — si `data.len()>=64` es seguro hoy, pero `unwrap()` en kernel es deuda; además solo valida header mínimo, no segmentos/program headers.
8. `Channel/Message` sin límite de `payload.len()` — un job QPU o IPC puede agotar memoria (DoS); sin accounting por proceso.
9. `DmaRegion` sin `Drop` que fuerce unmap, sin owner `DeviceId`, sin IOMMU — DMA escape posible en diseño futuro si se conecta a HW real.
10. Boot/linker: `ENTRY(_start)` sin símbolo `_start` en Rust; script placeholder; `main.rs` usa `std`+`println!` — no compila como `no_std`. No hay `panic_handler` real (`panic.rs::panic_info` no es `#[panic_handler]`).

## 3. Technical Debt

* `std` en todo el kernel (`BTreeSet/Map`, `Vec`, `VecDeque`, `Mutex`, `Instant`, `println!/eprintln!`) — impide `no_std`; documentado como "host-testable" pero sin roadmap `no_std`.
* Constantes mágicas dispersas: `4096`, `64MiB`, `11 stages`, `100/0x1000/0x2000` en syscalls, `0x100000` en linker.
* Sin `KernelConfig` central, sin feature flags (`smp/iommu/gpu/qpu/debug/tracing/experimental` exigidos por spec §70).
* Sin `KernelState/Health`, sin fases explícitas `EarlyInit→Running→Shutdown` (§7).
* `TODO/FIXME/HACK`: solo 1 hallazgo (`linker/x86_64.ld` placeholder) — indica falta de marcado, no falta de deuda. Código experimental (`qpu`, `SchedClass::Quantum`) no marcado `#[cfg(feature="experimental")]`.
* Duplicación: `PAGE_SIZE` en dos sitios; `dma` en dos módulos; `Device` re-exportado vía `device/` y `driver/`.
* `unwrap/expect` en paths kernel: `sync/mutex.rs` (lock), `elf/mod.rs` (try_into), tests (aceptable) — pero el patrón se puede copiar a código IRQ.
* Documentación: `kernel/docs/{SYSABI.md,debugging.md,MILESTONES.md}` + `boot/boot.md` existen pero no cubren `architecture/boot/memory/scheduler/interrupts/smp/ipc/security/drivers/dma/qpu-interface/telemetry` exigidos en §71; sin ADRs (`docs/adr/`).

## 4. Unsafe Areas

| Sitio | Código | Clase | Veredicto |
|---|---|---|---|
| `lib.rs:10` | `#![forbid(unsafe_op_in_unsafe_fn)]` | lint | REQUIRED, mantener |
| `arch/x86_64/cpu.rs:3-7` | `static mut INIT` + `unsafe{INIT=true}` + `unsafe{INIT}` | global mutable | REPLACEABLE → `AtomicBool` + `// SAFETY:` |
| `arch/x86_64/interrupts.rs:1-3` | `static mut INIT` sin SAFETY | global mutable | SUSPICIOUS → `AtomicBool`, añadir SAFETY o eliminar unsafe |
| Resto | — | — | No hay MMIO/DMA/ASM unsafe porque no hay HW real. Correcto no inventarlo. |

Regla §53: concentrar `unsafe` en arch/MMU/IRQ/ctx/MMIO/DMA/boot — hoy se cumple por ausencia, no por aislamiento. Al añadir HW real, exigir `// SAFETY:` con invariantes/ownership/alignment/lifetime/concurrency.

## 5. Memory Risks

* Bump allocator sin free → fragmentación/OOOM silencioso; `alloc` retorna `None` (bien) pero sin política OOM (§14) y con riesgo de `align==0`/overflow.
* `PhysicalMemoryManager::free_page` silencia doble-free de página nunca alocada si `p<total` y no estaba en set? No: `insert` retorna false si ya estaba libre → no decrementa (bien). Pero no detecta free de página nunca alocada fuera de rango más que ignorar.
* Sin validación de direcciones canónicas x86_64, sin permisos RWX/W^X, sin `PhysFrame/Page` con lifetimes.
* `std::BTreeSet/BTreeMap` alocan en heap host — en kernel real necesitarían allocator intrínseco (circularidad).
* Preguntas §87: ¿puede un proceso acceder a memoria de otro? Hoy no hay procesos reales con AddressSpace aislado → NO hay aislamiento que auditar; `VirtualMemoryManager` es un mapa global sin ASID. Debe documentarse como limitación, no como seguridad.

## 6. Scheduler Risks

* Sin preemption/ticks IRQ: `ticks` solo incrementa en `schedule()` cooperativo. Sin fairness (strict priority puede starvar `Idle/Low`), sin aging, sin affinity enforcement (`affinity:u32` nunca leído), sin `SchedClass` usado (solo `Priority`).
* `current.take()` descarta contexto sin guardarlo — correcto como stub, peligroso si se copia a ctx real.
* Sin `WaitQueue`, sin bloqueo real (Thread::sleep es estado, sin timer que despierte), sin SMP runqueues por CPU.
* Riesgo bajo hoy (single-thread host), alto si se añade SMP sin PerCpu + lock ordering.

## 7. Interrupt Risks

* `enable/disable` vacíos → sección crítica inexistente; `KernelMutex` + `SpscRing` no IRQ-safe.
* Sin IDT/GDT/APIC/MSI-X reales; sin `dispatcher`, sin `deferred work`, sin `Timer` tick.
* Riesgo: cualquier driver futuro que haga I/O o alloc en handler violaría §15. Documentar contrato ahora.

## 8. Driver Risks

* `enumerate_stub` retorna HW ficticio (`0:1:0`) — viola "No fake hardware" si se interpreta como soporte. Debe renombrarse a `stub_for_host_tests` y documentar que NO hay PCI real.
* Sin BAR/Capability/IRQ/DMA binding; `Driver::probe(&Device)->bool` sin `initialize/start/stop/reset/remove`; `DriverRegistry` sin ownership ni `remove`.
* `dma` vs `driver/dma` solape sin frontera clara (violación §6 si scheduler→driver→scheduler futuro).

## 9. Security Risks (§87)

| Pregunta | Respuesta hoy | Acción |
|---|---|---|
| ¿Proceso accede a memoria de otro? | No hay aislamiento; todo es un proceso host. | Documentar; diseñar AddressSpace + KPTI-like más tarde |
| ¿Mapear phys arbitraria desde user? | `map_page(VirtPage,usize,flags)` acepta `usize` sin validación de owner/caps. | Añadir `validate_map(caps,phys,flags)` + denegar EXEC+WRITE |
| ¿DMA arbitrario? | `DmaRegion::new` valida align/tamaño pero sin owner/IOMMU. | Ligar a `DeviceId` + `CapSet`, exigir `unmap` en `Drop` |
| ¿Operación privilegiada desde user? | `dispatch_syscall` no chequea caps/privilegio. | Añadir `check(caps,Access)` por syscall |
| ¿Driver corrupto compromete todo? | Sí, todo en un address space sin aislamiento. | Roadmap: capabilities + dominios |
| ¿Job QPU agota memoria? | `QuantumJob{bytes:Vec<u8>}` sin límite. | Límite + accounting + `CapSet::Quantum` |
| ¿Device accede sin IOMMU? | No hay IOMMU. | Declarar `IOMMU=absent` + política deny-by-default |

Además: `unwrap()` en `lock()` puede `panic!` en kernel; `static mut` es UB; `println!/eprintln!` en `panic_info` + `loop{spin}` sin halt HLT ni stack trace.

## 10. Proposed Architecture (sin mover archivos por estética)

Mantener `kernel/src/*` actual (no rewrite). Añadir solo:

```text
kernel/src/core/{config.rs, state.rs, health.rs}  // KernelConfig, fases Boot→Running→Shutdown, KernelHealth
kernel/src/sync/spin.rs                           // SpinLock IRQ-aware (host: AtomicBool+spin; doc IRQ-safe)
kernel/src/memory/oom.rs                          // OomPolicy explícita (deny/log/reclaim) sin panic!
kernel/src/syscall/validate.rs                    // validación puntero/longitud/handle/caps (host: índices+rangos)
kernel/src/tracing/mod.rs                         // trace_id + cpu/thread/process/device/job + timestamp mono
docs/kernel/{architecture,boot,memory,scheduler,interrupts,smp,ipc,security,drivers,dma,qpu-interface,telemetry}.md
docs/adr/ADR-00{1..8}.md
```

Fronteras (§6, §90):

```text
Generic Kernel → arch::ArchHal → x86_64 / (futuro aarch64)
Kernel IPC → QEOS Service Bus (user-space) → Services (QPU runtime, AI, energy, GPU, dashboard)
QPU: kernel solo QuantumDevice trait + DMA/IRQ/IPC/caps; runtime/sim en crates/quantum-runtime
```

## 11. Migration Plan (mapeo a §83, pasos pequeños compilables)

```text
S1 Audit (este fichero) → S2 Boot: documentar simulador + renombrar enumerate_stub + linker como placeholder explícito
S3 HAL: AtomicBool INIT + traits TimerHal/PcieHal usados por time/driver (inyectar NullHal en tests)
S4 PhysMem: PhysAddr/Frame con new() validado + OOM policy + test OOM/double-free
S5 VirtMem: PhysAddr en vez de usize + deny RWX + test W^X
S6 Heap: validar align!=0/potencia2 + overflow checked + test
S7 IRQ: contrato handler mínimo + enable/disable con bandera + test no-bloqueo
S8 Timer: KernelTimer trait + MonotonicClock backend + test timeouts
S9 Sync: SpinLock + docs IRQ-safe/sleepable + lock-ordering + test
S10 Sched: documentar cooperativo + test starvation consciente (no cambiar algoritmo aún)
S11 SMP: PerCpu stub + cores=1 explícito (no declarar SMP)
S12 Proc/Thread: AddressSpace stub + handles table stub (tipos, sin aislamiento real)
S13 Syscall: validate.rs + caps por número + test invalid/denied
S14 IPC: límite payload + owner checks + test
S15 Device: estados + lifecycle trait (probe/init/start/stop/reset/remove) sin romper Registry
S16 PCIe: PciAddr + BAR stub tipado (sin registros inventados)
S17 DMA/IOMMU: owner DeviceId + Drop unmap + IOMMU::Absent + test
S18 Driver lifecycle: migrar Driver trait (paso compatible)
S19 Security: check() en syscall/IPC/DMA/QPU + test privilege-escalation
S20 Telemetry/tracing: Sample + trace ctx + contadores (ctx switches, irqs, faults, allocs)
S21 QPU boundary: límites job + caps + doc runtime-en-userspace
S22 Power/Energy: PowerTelemetry stub (sin física ficticia)
S23 Testing: property/stress/fault-injection host + QEMU smoke cuando haya toolchain
S24 Performance: benches reproducibles en docs/performance/
S25 Docs: 11 docs kernel + 8 ADRs
Validación cada paso: cargo fmt/check/test/clippy (CI) + QEMU boot cuando aplique. Sin toolchain local, validar por revisión + CI.
```

## 12. Test Plan

* Existentes (deben seguir pasando): unit tests por módulo (~20) + `kernel_integration::{boot_sequence_is_deterministic, full_stack_smoke}`.
* Nuevos host (sin HW): allocator OOM/align/overflow, phys double-free, virt RWX-deny, heap checked, spinlock smoke, syscall invalid/denied, IPC payload-limit/closed, DMA misaligned/tooLarge/unmap-on-drop, ELF fuzz (bad magic/class/machine/segments), ring overflow policies, caps Admin-bypass + deny, timer expiry, boot order.
* Fuzzing prioritario (§64): `elf::load`, `dispatch_syscall`, `Message`/`Channel`, `DmaRegion::new`, `VirtualMemoryManager::map_page`.
* Fault injection (§62): alloc failure (heap None), DMA failure, invalid syscall, page double-map, process crash (estado Terminated), driver reset (Registry remove), timeout (Timer).
* QEMU (§60): pendiente de toolchain + imagen UEFI real; hoy `cargo run` es simulador, no boot. No declarar soporte BIOS/UEFI probado.
* Matriz (§61): x86_64 principal (CI), aarch64 compilación (CI) sin declarar soporte funcional.

## 13. Performance Baseline

Sin benchmarks en repo. Baseline cualitativo host (no medido, no declarar mejora):

```text
boot: O(11) println! — dominado por I/O host, irrelevante para kernel real
context switch: N/A (sin ctx real)
syscall: O(1) match — ~ns host
IPC: O(1) VecDeque push/pop + Vec<u8> clone
scheduler: O(5) scan prioridades
mem alloc: phys O(log n) BTreeSet, virt O(log n) BTreeMap, heap O(1) bump
DMA: O(1) checks
```

Acción: crear `kernel/benches/` + `docs/performance/` con `criterion`-like o harness manual
cuando haya toolchain; guardar antes/después. No optimizar prematuramente (§55).

---

## Apéndice A — Invariantes formales propuestas (§65)

```text
I1. Un mapping DMA pertenece a un device context activo y se desmapea en Drop.
I2. Un objeto kernel no se destruye mientras esté referenciado (handles/RC).
I3. Un puntero user nunca se dereferencia sin validación (validate.rs).
I4. Una runqueue está protegida por su primitiva declarada (hoy &mut exclusivo host).
I5. Ninguna página es RWX salvo justificación + test que lo demuestre.
I6. Ningún handler IRQ bloquea, aloca arbitrariamente ni hace I/O complejo.
```

## Apéndice B — Deuda explícita NO eliminar automáticamente (§80)

`enumerate_stub`, `SchedClass::Quantum/Scientific`, `QuantumJob::bytes` sin límite,
`handles:u32`, `affinity` sin uso, `ENTRY(_start)` sin símbolo, `println!` en kernel.
Todos quedan marcados en código con `// DEBT(AUDIT-2026-09-13): ...` en la fase de refinamiento,
no borrados en este paso.

## Apéndice C — Respuesta a Definition of Done (§85)

Hoy: NINGÚN ítem Done como kernel productivo. Como modelo host: boot simulado, HAL traits,
memoria modelo, heap contable, IRQ stubs, timer monótono, sync documentada como std-only,
scheduler cooperativo, proceso/hilo separados, syscall boundary sin validar, IPC acotado,
device/PCIe stubs, DMA con checks, driver lifecycle ausente, seguridad tipada no enforced,
telemetría primitiva, sin GDB/QEMU, fault handling mínimo, QPU boundary correcta en diseño,
energy ausente (correcto), tests host verdes (pendiente toolchain local), clippy/fmt por CI,
unsafe auditado (este fichero), docs parciales, sin fake HW salvo `enumerate_stub` (a renombrar),
sin física cuántica ficticia. **No declarar production-ready (§93).**