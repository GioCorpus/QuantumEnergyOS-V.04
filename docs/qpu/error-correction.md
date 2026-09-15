# Error Correction (Phase 4.3, MODEL)

Pipeline: physical state -> syndrome -> decoder -> correction -> logical state.

- `Syndrome { bits, error_detected }`, pairwise repetition checks.
- `trait Decoder { decode(&Syndrome) -> Correction }`.
- `RepetitionDecoder`, `LookupDecoder` implemented + tested.
- Future: MWPM / neural decoders behind the same trait (not implemented).
- `run_majorana_experiment` exports logical/physical-style rates.
