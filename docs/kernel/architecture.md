# QEOS Kernel — Architecture (V.04 refinement)

> Layers: Arch → Core → MM/Sched/IPC → Device → Drivers → Syscall → User.
> No circular deps. QPU/GPU/Energy are devices + user runtimes, never in-kernel simulators.

```text
QEOS KERNEL
  core/{config,state,health} — KernelConfig, KernelPhase(10), KernelHealth
  arch/x86_64/{cpu,interrupts} — AtomicBool flags, no static mut
  memory/{physical,virtual_,heap,allocator,oom} — checked arith, W^X, OOM deny
  sync/{mutex,spin,atomic} — Mutex sleepable host-only; SpinLock IRQ/non-sleepable
  syscall/{dispatcher,numbers,validate} — validate_range + caps
  time/{clock,timer} — MonotonicClock + KernelTimer trait
  dma — DmaRegion + Drop unmap (IOMMU deny-by-default)
  driver/pci — stub_for_host_tests only
  tracing — TraceCtx{cpu,thread,process,device,job,trace,ts_mono}
  qpu — QuantumDevice trait, UnsupportedDevice only (runtime in userspace)
```

Invariants (§65): I1 DMA belongs to active ctx + Drop; I2 no destroy while referenced;
I3 never deref user ptr without validate_range; I4 runqueue under defined primitive.
Lock order (§34): Memory < Device < Process. Never sleep in hard-IRQ.