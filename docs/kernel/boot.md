# Boot (§8) — real scope

Firmware → Bootloader → Kernel entry → CPU → Memory → Interrupts → Scheduler
→ Drivers → Userspace. `BootSequence` (11 stages) is host-model; linker script is
placeholder (no `_start`, no UEFI image). `KernelState::advance` enforces ordered
10-phase transition. Boot failure policy (§76): emit diagnostic, never continue
with corrupt state. No UEFI/BIOS support claimed until QEMU boot proven.