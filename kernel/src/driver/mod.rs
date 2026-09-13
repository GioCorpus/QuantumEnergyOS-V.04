pub mod device; pub mod bus; pub mod pci; pub mod dma; pub mod interrupt;
pub use device::{Device, DeviceId, Driver, DriverRegistry};
pub use bus::BusKind;
