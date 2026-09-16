pub mod ring_buffer;
pub mod telemetry;
pub mod energy;
pub mod provenance;
pub mod faults;

pub use ring_buffer::{
    LockFreeSpscRingBuffer,
    TelemetrySample,
    EnergySample,
    SampleType,
    RingBufferStats,
};
pub use telemetry::{TelemetryService, TelemetryConfig};
pub use energy::{EnergyService, EnergyConfig};
pub use provenance::{ClassifiedSample, Provenance};
pub use faults::{FaultKind, FaultReport, inject};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
