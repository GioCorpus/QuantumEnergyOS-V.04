//! P7.3-02 — Safe device handles.
//!
//! Raw kernel/OS pointers are never exposed. Device access flows through
//! [`DeviceHandle`]s that encode ownership, lifetime (a device **generation**),
//! a per-handle id, and the capabilities the holder is permitted to use.
//!
//! A handle becomes stale when the device's generation advances (device
//! reset/removal/re-registration). Any use of a stale handle is rejected at the
//! boundary, preventing use-after-remove and privilege/capability confusion.

use serde::{Deserialize, Serialize};

use crate::error::{NodeError, Result};

/// A device-touching capability that a handle may grant.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum HandleCapability {
    Inspect,
    Operate,
    Mmio,
    Dma,
    Reset,
}

impl HandleCapability {
    pub const fn mask(self) -> u32 {
        1u32 << (self as u32)
    }
}

/// A validated, generation-bound handle to a device.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DeviceHandle {
    pub handle_id: u64,
    pub device_id: String,
    generation: u64,
    cap_mask: u32,
}

impl DeviceHandle {
    pub fn generation(&self) -> u64 {
        self.generation
    }

    /// Whether the handle carries the given capability.
    pub fn has_capability(&self, cap: HandleCapability) -> bool {
        self.cap_mask & cap.mask() != 0
    }
}

/// Mints and validates device handles, and invalidates them on device
/// lifecycle events (generation bump).
#[derive(Debug, Default)]
pub struct HandleManager {
    next_id: u64,
    /// device_id -> current generation.
    generations: std::collections::HashMap<String, u64>,
}

impl HandleManager {
    pub fn new() -> Self {
        Self::default()
    }

    /// Mint a new handle granting `caps` to `device_id`. The device is given a
    /// fresh generation if not yet known.
    pub fn register(&mut self, device_id: &str, caps: &[HandleCapability]) -> DeviceHandle {
        let next_gen = self.generations.entry(device_id.to_string()).or_insert(1);
        let generation = *next_gen;
        let handle_id = self.next_id;
        self.next_id += 1;

        let mut cap_mask = 0u32;
        for c in caps {
            cap_mask |= c.mask();
        }

        DeviceHandle {
            handle_id,
            device_id: device_id.to_string(),
            generation,
            cap_mask,
        }
    }

    /// Validate that a handle is current (not stale) and carries `required`.
    pub fn check(&self, handle: &DeviceHandle, required: HandleCapability) -> Result<()> {
        if !handle.has_capability(required) {
            return Err(NodeError::InvalidInventory(format!(
                "handle {:?} lacks capability {required:?}",
                handle.handle_id
            )));
        }
        match self.generations.get(&handle.device_id) {
            Some(current) if *current == handle.generation => Ok(()),
            _ => Err(NodeError::InvalidInventory("stale device handle".into())),
        }
    }

    /// Invalidate all outstanding handles for a device (reset/removal).
    pub fn invalidate_device(&mut self, device_id: &str) {
        let next = self.generations.get(device_id).copied().unwrap_or(1) + 1;
        self.generations.insert(device_id.to_string(), next);
    }

    /// Whether a handle is stale relative to the manager's generation.
    pub fn is_stale(&self, handle: &DeviceHandle) -> bool {
        self.generations.get(&handle.device_id) != Some(&handle.generation)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mint_and_check() {
        let mut mgr = HandleManager::new();
        let h = mgr.register("gpu0", &[HandleCapability::Reset]);
        assert!(mgr.check(&h, HandleCapability::Reset).is_ok());
        assert!(h.has_capability(HandleCapability::Reset));
    }

    #[test]
    fn missing_capability_rejected() {
        let mut mgr = HandleManager::new();
        let h = mgr.register("gpu0", &[HandleCapability::Inspect]);
        assert!(mgr.check(&h, HandleCapability::Reset).is_err());
    }

    #[test]
    fn stale_after_invalidation() {
        let mut mgr = HandleManager::new();
        let h = mgr.register("gpu0", &[HandleCapability::Reset]);
        mgr.invalidate_device("gpu0");
        assert!(mgr.is_stale(&h));
        assert!(mgr.check(&h, HandleCapability::Reset).is_err());
    }

    #[test]
    fn new_handle_after_invalidation_is_valid() {
        let mut mgr = HandleManager::new();
        let old = mgr.register("gpu0", &[HandleCapability::Reset]);
        mgr.invalidate_device("gpu0");
        let fresh = mgr.register("gpu0", &[HandleCapability::Reset]);
        assert!(mgr.is_stale(&old));
        assert!(mgr.check(&fresh, HandleCapability::Reset).is_ok());
    }

    #[test]
    fn unique_handle_ids() {
        let mut mgr = HandleManager::new();
        let a = mgr.register("cpu0", &[HandleCapability::Operate]);
        let b = mgr.register("cpu0", &[HandleCapability::Operate]);
        assert_ne!(a.handle_id, b.handle_id);
    }
}
