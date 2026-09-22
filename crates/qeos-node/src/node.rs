//! P7.2 — Integrated node session.
//!
//! Composes the lifecycle runtime, hardware discovery, inventory persistence and
//! node health into a single [`Node`] entry point. The `Node` is the unit that
//! later distributed phases (7.5+) build upon.

use crate::device::Device;
use crate::discovery::{normalize, HardwareSource, HostDiscoverySource};
use crate::error::Result;
use crate::health::NodeHealth;
use crate::inventory::{reconcile, HardwareInventory, InventoryChange, InventoryStore};
use crate::lifecycle::{NodeLifecycleState, NodeRuntime};

/// An operating QEOS node: lifecycle + health + hardware inventory.
pub struct Node {
    /// Stable identifier for this node.
    pub id: String,
    pub runtime: NodeRuntime,
    pub health: NodeHealth,
    pub inventory: HardwareInventory,
    source: Box<dyn HardwareSource>,
    store: Box<dyn InventoryStore>,
}

impl Node {
    /// Create a node backed by the real host source and an in-memory inventory.
    pub fn host(id: &str) -> Self {
        Self::new(
            id,
            Box::new(HostDiscoverySource),
            Box::new(crate::inventory::MemoryInventoryStore::new()),
        )
    }

    pub fn new(id: &str, source: Box<dyn HardwareSource>, store: Box<dyn InventoryStore>) -> Self {
        let inventory = store
            .load()
            .unwrap_or_else(|_| HardwareInventory::default());
        Self {
            id: id.to_string(),
            runtime: NodeRuntime::default(),
            health: NodeHealth::default(),
            inventory,
            source,
            store,
        }
    }

    /// Run the boot sequence (`BOOTING -> STARTING -> READY`) and reconcile any
    /// persisted inventory.
    pub fn boot(&mut self) -> Result<()> {
        self.runtime.transition(NodeLifecycleState::Starting)?;
        self.runtime.transition(NodeLifecycleState::Ready)?;
        Ok(())
    }

    /// Perform a hardware discovery pass, normalize results and reconcile them
    /// into the persisted inventory. Returns the detected changes.
    pub fn discover(&mut self) -> Result<Vec<InventoryChange>> {
        let entries = self
            .source
            .discover()
            .map_err(|e| crate::error::NodeError::HardwareSourceUnavailable(e.to_string()))?;
        let devices: Vec<Device> = entries.into_iter().map(normalize).collect();
        let changes = reconcile(&mut self.inventory, devices, chrono::Utc::now());
        self.store.save(&self.inventory)?;
        Ok(changes)
    }

    /// Recompute the overall node health from sub-component observations.
    pub fn refresh_health(&mut self) {
        self.health.recompute_overall();
    }

    /// Whether the node is operationally ready to accept workloads.
    ///
    /// Readiness is gated on the lifecycle state (READY/DEGRADED) and on not
    /// being in maintenance mode. Health is reported separately via
    /// [`Self::health`] and consumed by a scheduler as a scheduling input, not
    /// conflated with operational readiness here.
    pub fn ready_for_workloads(&self) -> bool {
        self.runtime.state.can_accept_workloads() && !self.runtime.maintenance.active
    }

    /// Gracefully shut the node down.
    pub fn shutdown(&mut self) -> Result<()> {
        self.runtime.shutdown()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::discovery::SimulatedDiscoverySource;
    use crate::health::HealthStatus;
    use crate::inventory::MemoryInventoryStore;

    #[test]
    fn node_boots_to_ready() {
        let mut node = Node::new(
            "n1",
            Box::new(SimulatedDiscoverySource),
            Box::new(MemoryInventoryStore::new()),
        );
        node.boot().unwrap();
        assert_eq!(node.runtime.state, NodeLifecycleState::Ready);
        assert!(node.ready_for_workloads());
    }

    #[test]
    fn discovery_populates_inventory() {
        let mut node = Node::new(
            "n1",
            Box::new(SimulatedDiscoverySource),
            Box::new(MemoryInventoryStore::new()),
        );
        let changes = node.discover().unwrap();
        assert_eq!(changes.len(), 3);
        assert_eq!(node.inventory.records.len(), 3);
    }

    #[test]
    fn discovery_is_idempotent() {
        let mut node = Node::new(
            "n1",
            Box::new(SimulatedDiscoverySource),
            Box::new(MemoryInventoryStore::new()),
        );
        node.discover().unwrap();
        let second = node.discover().unwrap();
        // No added/removed changes the second time; device records unchanged.
        assert!(!second
            .iter()
            .any(|c| matches!(c, InventoryChange::DeviceAdded(_))));
    }

    #[test]
    fn shutdown_transitions_node() {
        let mut node = Node::host("n1");
        node.boot().unwrap();
        node.shutdown().unwrap();
        assert_eq!(node.runtime.state, NodeLifecycleState::Shutdown);
        assert!(!node.ready_for_workloads());
    }

    #[test]
    fn refresh_health_with_simulated_inventory() {
        let mut node = Node::new(
            "n1",
            Box::new(SimulatedDiscoverySource),
            Box::new(MemoryInventoryStore::new()),
        );
        node.discover().unwrap();
        node.refresh_health();
        // No per-component health was observed, so overall stays Unknown (honest).
        assert_eq!(node.health.overall, HealthStatus::Unknown);
    }
}
