# Error Correction (Phase 4.8)

`ErrorCorrectionCode` (encode/syndrome/decode/correct/logical_error_rate),
`Decoder` (repetition + lookup-table; MWPM/BP are extension points),
`Syndrome`, `RepetitionCode`, `LogicalQubit` in `quantum-runtime::
{error_correction, decoder}`. All MODEL-labeled, seeded, unit-tested.
Benchmarks: repetition-code sweep in `quantum-runtime benches` (FUTURE:
surface as CLI flag). Dataset export: `export::{export_json, export_csv}`.
