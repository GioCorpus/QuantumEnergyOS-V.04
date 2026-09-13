/* QEOS x86_64 linker script (UEFI target, placeholder for QEMU work) */
ENTRY(_start)
SECTIONS {
  . = 0x100000;
  .text : { *(.text*) }
  .rodata : { *(.rodata*) }
  .data : { *(.data*) }
  .bss : { *(.bss*) }
}
