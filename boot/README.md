# Boot (V.04 Phase 1)

Arch-compatible boot: Linux kernel, init/systemd, filesystem, network, shell,
Rust toolchain, pacman-compatible package management, logging.

QuantumEnergyOS does not fork the kernel without documented reason. Kernel work
is config + drivers + telemetry + power management + HAL modules (see `kernel/`).
Goal: bootable QuantumEnergyOS on x86_64 and ARM64.