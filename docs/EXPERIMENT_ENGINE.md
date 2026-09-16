# Experiment Engine (Phase 4.8)

`quantum-runtime::experiment::{QuantumExperiment, ExperimentResult,
ExperimentLimits, run_experiment}` + `majorana_sim::run_majorana_experiment`
+ `export::{export_json, export_csv}`. Every run records seed, software/
backend versions, config, noise model, timestamp, params, results.
Reproduce via experiment ID + seed + config + code/backend versions.
`qeos-qpu experiment --shots N --seed S` emits full JSON artifacts.
