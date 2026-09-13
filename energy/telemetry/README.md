# Energy Telemetry (V.04 §14, §17)

Canonical implementation: `crates/energy-telemetry` (SPSC ring buffer, Acquire/Release, bounded queues, backpressure, timestamps, dropped-frame counters).

Pipeline: Hardware -> Interrupt/DMA -> Top Half -> Ring Buffer -> Bottom Half -> Telemetry Service -> QuantumEnergyOS. No zero-copy claims unless implemented.