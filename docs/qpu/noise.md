# Noise (Phase 4.3, SIMULATION)

Effective classical model for simulator studies:

```toml
[noise]
enabled = true
seed = 42
[noise.measurement]
error_probability = 0.001
[noise.dephasing]
rate = 0.0001
```

Fields: `measurement_error_probability`, `dephasing_rate` (documented,
reserved), `operation_error_probability`, `quasiparticle_poisoning_probability`.
All in [0,1]; validated. Quasiparticle poisoning is an effective bit-flip
sampler, not microscopic physics.
