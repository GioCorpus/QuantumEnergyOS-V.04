# Phase 4.4–4.9 Audit — QEOS V.04 (2026-09-16)

Branch: `feature/qeos-v04-phase-4-4-4-9` (from `feature/phase-4.3-majorana-runtime`).
Baseline: Phase 4.3 green — `quantum-runtime` 116 lib tests pass; workspace
`cargo check` green for new crates per PHASE_4_3_REPORT.

## 1. What already exists (do NOT rewrite)

| Spec item | Existing implementation | Verdict |
|---|---|---|
| PCI config decode / BAR / caps / MSI-X | `crates/device-manager/src/pci/*` (decode, BAR sizing, cap walk, `SimulatedPciBackend`, `SimulatedMmioMapper`) | REAL decode + SIMULATED transport. Keep. |
| DeviceManager lifecycle + capabilities | `manager.rs`, `driver.rs`, `capability.rs` (Unbound→Bound→Initialized→Running→Stopped→Failed; deny-by-default DMA) | Keep; extend states only additively. |
| DMA abstraction (basic) | `kernel/src/dma/mod.rs` (`DmaRegion`, `DmaBuffer`) | Keep; extend with mapping/ring/IOMMU. |
| GPU abstraction | `quantum-hal::accelerator` + `quantum-runtime::compute` (CPU ref, honest fallback, probe) | Keep; add `qeos-gpu-compute` crate additively. |
| QPU device API / IR / scheduler / jobs | `quantum-hal` (device/ir/job/simulator/qpu), `quantum-runtime` (compiler/circuit/job/scheduler/experiment) | Keep; add spec-named job states additively. |
| Majorana algebra/parity/tetron/noise | `quantum-runtime::{majorana,majorana_sim,tetron,topology,noise}` + docs/qpu | Keep as MODEL; harden braiding/noise APIs. |
| Error correction / decoders / experiments | `error_correction.rs`, `decoder.rs` (repetition+lookup), `experiment.rs` (seeded, reproducible) | Keep; add trait + export + benchmarks. |
| Telemetry pipeline / energy / SPSC ring | `energy-telemetry` (SPSC ring, TelemetryService, EnergyService), `kernel::telemetry` primitives | Keep; add classification + fault injection. |
| Security boundaries | `device-manager` capabilities, `kernel::security::capability`, HAL gating | Keep; audit + document. |

## 2. Gaps vs spec DONE checklist (delta only)

1. **4.4**: kernel `PciAddr` stub lacks vendor/class/BAR/caps; no kernel `MmioRegion`
   (documented unsafe), `DmaMapping`/`DmaRing`, `Iommu` trait, 9-state device
   lifecycle alias, QEMU checklist doc.
2. **4.5**: no standalone GPU compute crate (`GpuBuffer/Kernel/Queue/Fence`,
   CPU vs mock equivalence tests, benchmarks).
3. **4.6**: runtime `JobStatus` lacks Compiling/Measuring/PostProcessing/Timeout;
   no unified `QpuDevice` trait re-export with capabilities/submit/measure/reset.
4. **4.7**: braiding/noise need explicit seeded model APIs + logical-error
   experiment entry point (math stays identical, classification MODEL).
5. **4.8**: `ErrorCorrectionCode` trait + dataset export (JSON/CSV) missing.
6. **4.9**: telemetry `Source`/`Provenance` (Measured/Estimated/Simulated/
   Unavailable), top/bottom-half IRQ doc, fault-injection matrix, docs +
   final report.

## 3. Dependency map (no cycles)

```text
kernel (dma/driver/hal/qpu/telemetry/security)
  ↑ uses traits only
device-manager (pci decode + manager + capabilities) ──→ hardware-abstraction (facts)
quantum-runtime (circuit/compiler/sim/noise/Majorana/EC/experiment)
  ↑ used by
quantum-hal (device/ir/job/simulator/accelerator/qpu-adapter)
  ↑ used by
qeos-qpu CLI
NEW qeos-gpu-compute (CPU ref + mock + probe; no CUDA dep) ──→ quantum-runtime (verify)
energy-telemetry (ring + services + provenance + faults)
docs (HARDWARE_ARCHITECTURE, PCIe_DMA, IOMMU, GPU_RUNTIME, QPU_RUNTIME,
      QUANTUM_IR, MAJORANA_SIMULATOR, ERROR_CORRECTION, TELEMETRY,
      SECURITY_MODEL, EXPERIMENT_ENGINE, PHASE_4_4_4_9_REPORT)
```

No cycles: kernel ↔ crates communicate via traits/snapshots only; quantum-hal
depends on quantum-runtime (one direction); gpu-compute depends on runtime only
for verification vectors.

## 4. Risks / TODO / unsafe inventory

- `energy-telemetry::ring_buffer`: `UnsafeCell` + atomics, `zeroed()` init —
  sound for `Copy` types (documented SAFETY inline); keep, add SPSC contract tests.
- `kernel::dma`: host model only, no IOMMU enforcement yet — deny-by-default kept;
  `MockIommu` added in 4.4, real IOMMU stays FUTURE.
- No `transmute`/`from_raw_parts`/raw MMIO pointers in workspace (verified by
  inspection of new crates; kernel arch stubs contain no raw deref).
- `unwrap/expect` present in CLI/bin paths and tests only; library paths return
  `Result` (device-manager enforces via clippy lints).

## 5. Plan (additive, phase-gated)

4.4 → kernel pci/dma/iommu/mmio/device-state extension + tests + QEMU doc.
4.5 → NEW `crates/qeos-gpu-compute` + equivalence tests + benches.
4.6 → runtime job-state extension + `qpu_device` module + tests.
4.7 → `braiding` + seeded noise + logical-error experiment + property tests.
4.8 → `ErrorCorrectionCode` trait + CSV/JSON export + benches.
4.9 → telemetry provenance + IRQ/fault matrix + security audit doc + report.
Global validation: `cargo test --workspace`, clippy, docs.
