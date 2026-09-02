pub mod ring_buffer;
pub mod telemetry;
pub mod energy;

pub use ring_buffer::{
    LockFreeSpscRingBuffer,
    TelemetrySample,
    EnergySample,
    SampleType,
    RingBufferStats,
};
pub use telemetry::{TelemetryService, TelemetryConfig};
pub use energy::{EnergyService, EnergyConfig};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
