# Majorana Simulator (Phase 4.7) — MODEL, not hardware

Algebra `{γi,γj}=2δij` verified numerically (`majorana.rs`); parity
`P_ij=iγiγj`; tetron + logical qubit (`tetron.rs`, `topology.rs`);
measurement-based control with feed-forward; braiding as symbolic ledger
(`braiding.rs`, simulation model label); seeded noise (`noise.rs`);
logical-error experiments via `majorana_sim.rs`. Property tests cover
anticommutation, parity round-trip, braid validation. No physical-device
claims; vendor QPU path stays disabled without an adapter.
