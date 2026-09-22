# QEOS V.04 — QPU Hardware Readiness

**Status:** SIMULATED (simulator) / UNAVAILABLE (physical hardware)

This document is the authoritative contract for integrating physical quantum
hardware (including topological/Majorana devices) into QEOS. It states exactly
what is required and what is currently available. It does **not** invent
register maps, PCIe protocols, firmware APIs, or vendor commands.

---

## 1. Current Availability

| Target | Status |
|---|---|
| Majorana mathematical model (`quantum-runtime`) | **SIMULATED** (validated math: anticommutation, tetron, parity) |
| Parity-measurement simulator (`qeos-qpu` SimulatorBackend) | **SIMULATED** |
| Generic QPU interface (`qeos-qpu`) | **REAL** (abstraction) |
| Vendor/physical QPU adapter | **UNAVAILABLE** |

A working QPU *abstraction* does **not** imply hardware acceleration or access.

---

## 2. Hardware Readiness Contract

A physical integration is usable only when ALL of the following are satisfied
(see `HardwareReadinessContract`):

| Requirement | Meaning |
|---|---|
| Device discovery | Enumerate the QPU reliably and stably identify it |
| Transport | Documented, safe transport (PCIe/USB/network/SDK) |
| Authentication | Proven identity and authorization for control |
| Command interface | Documented command set; no undocumented commands |
| Measurement interface | Documented readout/measurement protocol |
| Telemetry | Temperature, calibration and status telemetry |
| Calibration | Documented calibration data and procedures |
| Error reporting | Structured error and fault reporting |
| Firmware provenance | Versioned, verified, updatable firmware |
| Security | Memory-safe control; no unsafe MMIO; audit trail |

---

## 3. Integration Point

A real backend must implement `qpu_backend::QpuBackend` (or a
`quantum_hal::qpu::QpuVendorAdapter`). Only the adapter changes; the QPU job
system, isolation, telemetry, experiment engine and dashboard stay unchanged.

---

## 4. Honesty Guarantees

- `QpuCapabilities::vendor_available` is always `false` until validated.
- `HardwareAvailabilityReport::physical_hardware_available == false`.
- Simulation is always tagged `SIMULATED`; simulation is never presented as
  hardware.
- No claim of fault tolerance or quantum advantage without measured evidence.
