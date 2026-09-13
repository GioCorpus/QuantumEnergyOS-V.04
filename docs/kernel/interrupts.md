# Kernel Interrupts (Host Stub)

`kernel/src/arch/x86_64/interrupts.rs` + `cpu.rs`: `AtomicBool` flags
(`INIT`, `ENABLED`), zero `unsafe`. No IDT/GDT/APIC/MSI-X.

Contract (§15): handlers must stay minimal — no blocking, no arbitrary alloc,
no complex I/O. `enable/disable` are flags only; no real critical sections.
`KernelMutex` is NOT IRQ-safe; `SpinLock` is for non-sleepable paths.

Real IRQ work (dispatcher, deferred work, timer tick) is future and requires
`// SAFETY:` + IRQ-safe primitives + QEMU validation.