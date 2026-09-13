# Quartz5D Integration (V.04 §15–§16)

Canonical implementation: `crates/quartz5d`
(`model.rs`, `coordinate.rs`, `storage.rs`, `projection.rs`, `prediction.rs`,
`serialization.rs`).

- Coordinate: `(X,Y,Z,T,S)` where S is computational state-space, not a physical dimension.
- Visualization: 5D → 3D projection (X/Y/Z → geometry, T → animation, S → intensity/scale).
- UI must provide timeline, state slider, projection controls; see `dashboard/src/components/Quartz5DView.tsx`.