# Reproducibility (Phase 4.3)

Every experiment records: backend, backend/compiler/runtime versions,
seed, noise config, circuit hash (FNV-1a identity only), shots.
Same seed + same circuit + same noise => identical counts (tested).
Hashes are identity/integrity only, never quantum state.
