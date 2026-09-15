//! Phase 4.1 Device Manager Integration Tests.
//!
//! Tests end-to-end device enumeration, driver lifecycle, MMIO access,
//! capability enforcement, and diagnostic snapshots.

use device_manager::{
    BarKind, DeviceCapability, DeviceManager, DeviceManagerConfig, HotplugPolicy, IommuPolicy,
    MmioMapper, PciBus, SimulatedMmioMapper, SimulatedPciBackend,
};

#[test]
fn test_end_to_end_enumeration_and_binding() {
    let mut backend = SimulatedPciBackend::new();
    let mut bus = PciBus::new(&mut backend);
    let mut manager = DeviceManager::with_default_drivers(DeviceManagerConfig::default());

    let count = manager.enumerate_bus(&mut bus).expect("enumeration should succeed");
    assert_eq!(count, 5);

    let snapshot = manager.snapshot();
    assert_eq!(snapshot.len(), 5);

    // Verify NVMe device binding
    let nvme = snapshot.iter().find(|d| d.vendor_id == 0x1B36).unwrap();
    assert_eq!(nvme.driver_name.as_deref(), Some("qeos-nvme-driver"));
    assert_eq!(nvme.driver_state, "bound");
    assert!(nvme.dma_ready);

    // Verify GPU device binding
    let gpu = snapshot.iter().find(|d| d.vendor_id == 0x10DE).unwrap();
    assert_eq!(gpu.driver_name.as_deref(), Some("qeos-gpu-driver"));
    assert_eq!(gpu.driver_state, "bound");

    // Verify Telemetry device binding
    let telemetry = snapshot.iter().find(|d| d.vendor_id == 0x51E0).unwrap();
    assert_eq!(telemetry.driver_name.as_deref(), Some("qeos-telemetry-driver"));
    assert_eq!(telemetry.driver_state, "bound");
}

#[test]
fn test_device_lifecycle_and_mmio_mapping() {
    let mut backend = SimulatedPciBackend::new();
    let mut bus = PciBus::new(&mut backend);
    let mut manager = DeviceManager::with_default_drivers(DeviceManagerConfig {
        iommu_policy: IommuPolicy::AllowUnmanagedDma,
        ..Default::default()
    });

    manager.enumerate_bus(&mut bus).unwrap();
    let gpu_rec = manager
        .devices()
        .iter()
        .find(|d| d.device.vendor_id() == 0x10DE)
        .unwrap();
    let gpu_id = gpu_rec.id;
    let gpu_pci = gpu_rec.device.clone();

    // 1. Lifecycle: Initialize -> Start
    manager.initialize_device(gpu_id).expect("initialize should succeed");
    assert_eq!(manager.device(gpu_id).unwrap().driver_state.to_string(), "initialized");

    manager.start_device(gpu_id).expect("start should succeed");
    assert_eq!(manager.device(gpu_id).unwrap().driver_state.to_string(), "running");

    // 2. Capability check & grant
    manager.grant_capability(gpu_id, DeviceCapability::MmioAccess).expect("grant mmio");
    manager.check_capability(gpu_id, DeviceCapability::MmioAccess).expect("check mmio");

    // 3. MMIO mapping via SimulatedMmioMapper
    let mut mapper = SimulatedMmioMapper::new();
    let bar0 = gpu_pci.bar(0).unwrap();
    assert_eq!(bar0.kind, BarKind::Memory64);

    let mut region = mapper.map(gpu_pci.address(), bar0).expect("map BAR0");
    assert_eq!(region.size(), 64 * 1024);

    // Read/write MMIO
    region.write_u32(0x100, 0xCAFE_BABE).expect("write mmio");
    let val = region.read_u32(0x100).expect("read mmio");
    assert_eq!(val, 0xCAFE_BABE);

    // 4. Lifecycle: Stop -> Reset
    manager.stop_device(gpu_id).expect("stop should succeed");
    assert_eq!(manager.device(gpu_id).unwrap().driver_state.to_string(), "stopped");

    manager.grant_capability(gpu_id, DeviceCapability::Reset).expect("grant reset");
    manager.reset_device(gpu_id).expect("reset should succeed");
    assert_eq!(manager.device(gpu_id).unwrap().driver_state.to_string(), "initialized");
}

#[test]
fn test_hotplug_removal_flow() {
    let mut backend = SimulatedPciBackend::new();
    let mut bus = PciBus::new(&mut backend);
    let mut manager = DeviceManager::with_default_drivers(DeviceManagerConfig {
        hotplug_policy: HotplugPolicy::AllowReported,
        ..Default::default()
    });

    manager.enumerate_bus(&mut bus).unwrap();
    let gpu_id = manager
        .devices()
        .iter()
        .find(|d| d.device.vendor_id() == 0x10DE)
        .unwrap()
        .id;

    // Start GPU
    manager.initialize_device(gpu_id).unwrap();
    manager.start_device(gpu_id).unwrap();

    // Hot-remove: should cleanly stop and remove
    manager.remove_device(gpu_id).expect("removal should succeed");
    assert!(manager.device(gpu_id).is_none());
    assert_eq!(manager.counters().hotplug_events, 1);
}
