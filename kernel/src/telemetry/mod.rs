//! Kernel telemetry primitives only: timestamp + ring event. No voltage/power logic here.
#[derive(Debug, Clone, Copy)] pub struct Sample { pub ts_mono_ns: u64, pub source: u32, pub value: u32 }
impl Sample { pub fn new(ts: u64, source: u32, value: u32) -> Self { Self { ts_mono_ns: ts, source, value } } }
