# Desktop Architecture (V.04 §20)

Two flavors, both Wayland-based, decoupled from the quantum runtime:

- Flavor A (Minimal): Wayland + tinywl + QuantumEnergyOS services.
  Target: lightweight, embedded, research, low-resource.
- Flavor B (Full): Wayland + KDE Plasma + QuantumEnergyOS integration.
  Target: workstation, developer, researcher, daily use.

Desktop components must not import `quantum-runtime` directly; they talk to
services via the versioned IPC bus (`docs/IPC_PROTOCOL.md`).