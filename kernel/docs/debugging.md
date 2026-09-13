# Debugging QEOS Kernel V1

## Host tests (no hardware)

```bash
cargo test -p qeos-kernel
cargo run -p qeos-kernel
```

## QEMU (x86_64 + UEFI/BIOS, once bare-metal target added)

```bash
# install target
rustup target add x86_64-unknown-uefi
# serial console
qemu-system-x86_64 -bios OVMF.fd -serial stdio -display none -kernel target/x86_64-unknown-uefi/debug/qeos-kernel
# GDB
qemu-system-x86_64 -s -S -bios OVMF.fd -serial stdio -kernel target/x86_64-unknown-uefi/debug/qeos-kernel &
gdb -ex 'target remote :1234'
# QEMU monitor: Ctrl-Alt-2, `info registers`, `info mem`
```

## Panic diagnostics

Kernel prints `[PANIC] file:line msg` then halts CPU via spin loop. Future crash-dump support planned.
