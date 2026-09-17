pub mod energy;
pub mod faults;
pub mod provenance;
pub mod ring_buffer;
pub mod telemetry;

pub use energy::{EnergyConfig, EnergyService};
pub use faults::{inject, FaultKind, FaultReport};
pub use provenance::{ClassifiedSample, Provenance};
pub use ring_buffer::{
    EnergySample, LockFreeSpscRingBuffer, RingBufferStats, SampleType, TelemetrySample,
};
pub use telemetry::{TelemetryConfig, TelemetryService};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
