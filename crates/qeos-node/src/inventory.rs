//! P7.2-05 — Hardware Inventory Persistence.
//!
//! Persists hardware metadata with `first_seen`/`last_seen` tracking and
//! detects hardware *changes* (added, removed, or mutated devices) between
//! discovery passes. Persistence is pluggable via [`InventoryStore`]; the
//! bundled [`MemoryInventoryStore`] is the default, and
//! [`JsonFileInventoryStore`] provides durable file persistence.
//!
//! The inventory is **append-conscious**: an existing device's record is
//! updated in place (keeping `first_seen`), while a new device is added and a
//! vanished device is marked removed. Research data is never silently
//! overwritten or dropped.

use std::collections::HashMap;
use std::path::Path;

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::device::Device;
use crate::error::{NodeError, Result};

/// A persisted inventory record for one device.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryRecord {
    pub device: Device,
    pub first_seen: DateTime<Utc>,
    pub last_seen: DateTime<Utc>,
}

/// The full persisted inventory snapshot.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HardwareInventory {
    pub records: Vec<InventoryRecord>,
}

/// A change detected between two inventory snapshots.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum InventoryChange {
    DeviceAdded(String),
    DeviceRemoved(String),
    DeviceUpdated(String),
}

/// Abstraction over persistence so the inventory can be stored in-memory, in a
/// file, or in a database without changing this module's logic.
pub trait InventoryStore: Send + Sync {
    fn load(&self) -> Result<HardwareInventory>;
    fn save(&self, inventory: &HardwareInventory) -> Result<()>;
}

/// Default in-memory store. Not durable across a restart.
#[derive(Debug, Default)]
pub struct MemoryInventoryStore {
    inventory: std::sync::Mutex<HardwareInventory>,
}

impl MemoryInventoryStore {
    pub fn new() -> Self {
        Self::default()
    }
}

impl InventoryStore for MemoryInventoryStore {
    fn load(&self) -> Result<HardwareInventory> {
        let guard = self
            .inventory
            .lock()
            .map_err(|_| NodeError::Persistence("memory store poisoned".into()))?;
        Ok(guard.clone())
    }

    fn save(&self, inventory: &HardwareInventory) -> Result<()> {
        let mut guard = self
            .inventory
            .lock()
            .map_err(|_| NodeError::Persistence("memory store poisoned".into()))?;
        *guard = inventory.clone();
        Ok(())
    }
}

/// Durable JSON-file store. Writes are atomic (write temp file then rename) so
/// a crash does not leave a partially-written inventory.
#[derive(Debug, Clone)]
pub struct JsonFileInventoryStore {
    path: std::path::PathBuf,
}

impl JsonFileInventoryStore {
    pub fn new(path: impl AsRef<Path>) -> Self {
        Self {
            path: path.as_ref().to_path_buf(),
        }
    }
}

impl InventoryStore for JsonFileInventoryStore {
    fn load(&self) -> Result<HardwareInventory> {
        if !self.path.exists() {
            return Ok(HardwareInventory::default());
        }
        let raw = std::fs::read(&self.path)?;
        let inv = serde_json::from_slice(&raw)?;
        Ok(inv)
    }

    fn save(&self, inventory: &HardwareInventory) -> Result<()> {
        let raw = serde_json::to_vec_pretty(inventory)?;
        let tmp = self.path.with_extension("tmp");
        std::fs::write(&tmp, raw)?;
        std::fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

/// Merges a freshly discovered device list into the persisted inventory,
/// preserving `first_seen` for stable devices and detecting changes.
pub fn reconcile(
    inventory: &mut HardwareInventory,
    discovered: Vec<Device>,
    now: DateTime<Utc>,
) -> Vec<InventoryChange> {
    let mut changes = Vec::new();

    // Index existing records by device id.
    let mut by_id: HashMap<String, usize> = HashMap::new();
    for (i, rec) in inventory.records.iter().enumerate() {
        by_id.insert(rec.device.identity.device_id.clone(), i);
    }

    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for dev in discovered {
        let id = dev.identity.device_id.clone();
        seen.insert(id.clone());
        match by_id.get(&id) {
            Some(&i) => {
                // Update record in place; preserve first_seen.
                let rec = &mut inventory.records[i];
                let changed = rec.device != dev; // includes updated telemetry/caps
                rec.device = dev;
                rec.last_seen = now;
                if changed {
                    changes.push(InventoryChange::DeviceUpdated(id));
                }
            }
            None => {
                inventory.records.push(InventoryRecord {
                    device: dev,
                    first_seen: now,
                    last_seen: now,
                });
                changes.push(InventoryChange::DeviceAdded(id));
            }
        }
    }

    // Devices that were present before but not discovered now are removed.
    let present_ids: Vec<String> = by_id
        .keys()
        .filter(|id| !seen.contains(*id))
        .cloned()
        .collect();
    for id in present_ids {
        if let Some(pos) = inventory
            .records
            .iter()
            .position(|r| r.device.identity.device_id == id)
        {
            changes.push(InventoryChange::DeviceRemoved(id));
            inventory.records.remove(pos);
        }
    }

    changes
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_device(id: &str, class: crate::device::DeviceClass) -> Device {
        Device {
            identity: crate::device::DeviceIdentity {
                device_id: id.to_string(),
                class,
                vendor: "V".into(),
                model: "M".into(),
                ..Default::default()
            },
            ..Default::default()
        }
    }

    #[test]
    fn add_detect_new_devices() {
        let mut inv = HardwareInventory::default();
        let now = Utc::now();
        let discovered = vec![sample_device("cpu0", crate::device::DeviceClass::Cpu)];
        let changes = reconcile(&mut inv, discovered, now);
        assert_eq!(
            changes,
            vec![InventoryChange::DeviceAdded("cpu0".to_string())]
        );
        assert_eq!(inv.records.len(), 1);
    }

    #[test]
    fn stable_device_keeps_first_seen() {
        let mut inv = HardwareInventory::default();
        let t0 = Utc::now();
        reconcile(
            &mut inv,
            vec![sample_device("cpu0", crate::device::DeviceClass::Cpu)],
            t0,
        );
        let first = inv.records[0].first_seen;
        let t1 = t0 + chrono::Duration::seconds(60);
        let changes = reconcile(
            &mut inv,
            vec![sample_device("cpu0", crate::device::DeviceClass::Cpu)],
            t1,
        );
        assert_eq!(changes.len(), 0); // identical → no update change
        assert_eq!(inv.records[0].first_seen, first);
        assert_eq!(inv.records[0].last_seen, t1);
    }

    #[test]
    fn device_change_detected() {
        let mut inv = HardwareInventory::default();
        reconcile(
            &mut inv,
            vec![sample_device("gpu0", crate::device::DeviceClass::Gpu)],
            Utc::now(),
        );
        // Mutate the device (e.g. changed firmware) then re-discover.
        let mut dev = sample_device("gpu0", crate::device::DeviceClass::Gpu);
        dev.identity.firmware_version = Some("2.0.0".to_string());
        let changes = reconcile(&mut inv, vec![dev], Utc::now());
        assert_eq!(
            changes,
            vec![InventoryChange::DeviceUpdated("gpu0".to_string())]
        );
    }

    #[test]
    fn removal_detected() {
        let mut inv = HardwareInventory::default();
        reconcile(
            &mut inv,
            vec![
                sample_device("cpu0", crate::device::DeviceClass::Cpu),
                sample_device("gpu0", crate::device::DeviceClass::Gpu),
            ],
            Utc::now(),
        );
        // GPU disappears on next discovery.
        let changes = reconcile(
            &mut inv,
            vec![sample_device("cpu0", crate::device::DeviceClass::Cpu)],
            Utc::now(),
        );
        assert_eq!(
            changes,
            vec![InventoryChange::DeviceRemoved("gpu0".to_string())]
        );
        assert_eq!(inv.records.len(), 1);
    }

    #[test]
    fn memory_store_roundtrip() {
        let store = MemoryInventoryStore::new();
        let mut inv = HardwareInventory::default();
        reconcile(
            &mut inv,
            vec![sample_device("cpu0", crate::device::DeviceClass::Cpu)],
            Utc::now(),
        );
        store.save(&inv).unwrap();
        let loaded = store.load().unwrap();
        assert_eq!(loaded.records.len(), 1);
    }

    #[test]
    fn json_store_empty_when_missing() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("qeos-inv-test-{}.json", uuid::Uuid::new_v4()));
        let store = JsonFileInventoryStore::new(&path);
        let inv = store.load().unwrap();
        assert!(inv.records.is_empty());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn json_store_roundtrip() {
        let dir = std::env::temp_dir();
        let path = dir.join(format!("qeos-inv-rw-{}.json", uuid::Uuid::new_v4()));
        let store = JsonFileInventoryStore::new(&path);
        let mut inv = HardwareInventory::default();
        reconcile(
            &mut inv,
            vec![sample_device("gpu0", crate::device::DeviceClass::Gpu)],
            Utc::now(),
        );
        store.save(&inv).unwrap();

        let store2 = JsonFileInventoryStore::new(&path);
        let loaded = store2.load().unwrap();
        assert_eq!(loaded.records.len(), 1);
        assert_eq!(loaded.records[0].device.identity.device_id, "gpu0");
        let _ = std::fs::remove_file(&path);
    }
}
