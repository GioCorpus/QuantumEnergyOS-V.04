//! Stable identifier types used by the device stack.

use std::fmt;

use serde::{Deserialize, Serialize};

/// Identifier of a discovered device, assigned at enumeration time.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DeviceId(u32);

impl DeviceId {
    /// Builds an id from its raw representation (snapshots, tests).
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Raw numeric value, for telemetry and audit records.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl fmt::Display for DeviceId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "device-{:04}", self.0)
    }
}

/// Identifier of a driver instance registered with the device manager.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DriverId(u32);

impl DriverId {
    /// Builds an id from its raw representation (snapshots, tests).
    pub const fn from_raw(raw: u32) -> Self {
        Self(raw)
    }

    /// Raw numeric value, for telemetry and audit records.
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl fmt::Display for DriverId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "driver-{:03}", self.0)
    }
}