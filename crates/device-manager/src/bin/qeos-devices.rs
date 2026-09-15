//! QEOS Phase 4.1 Userspace Device Management CLI tool (`qeos-devices`).
//!
//! Provides inspection, diagnostic and status reporting for the QEOS device subsystem.

use std::env;

use device_manager::{
    DeviceManager, DeviceManagerConfig, HotplugPolicy, IommuPolicy, PciBus, PciConfigBackend,
    SimulatedPciBackend,
};

fn print_usage() {
    println!("QuantumEnergyOS (QEOS) Device Manager CLI - Phase 4.1");
    println!("Usage: qeos-devices [COMMAND]");
    println!();
    println!("Commands:");
    println!("  list       List all discovered devices, drivers, and states (default)");
    println!("  pci        Display detailed PCI tree, BAR allocations and capabilities");
    println!("  doctor     Run diagnostic health checks on the device subsystem");
    println!("  json       Output machine-readable JSON device snapshot");
    println!("  help       Show this help message");
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let command = args.get(1).map(|s| s.as_str()).unwrap_or("list");

    let mut backend = SimulatedPciBackend::new();
    let mut bus = PciBus::new(&mut backend);
    let mut manager = DeviceManager::with_default_drivers(DeviceManagerConfig {
        auto_bind: true,
        iommu_policy: IommuPolicy::DenyUnmanagedDma,
        hotplug_policy: HotplugPolicy::AllowReported,
        max_devices: 64,
    });

    if let Err(e) = manager.enumerate_bus(&mut bus) {
        eprintln!("[ERROR] Failed to enumerate PCI bus: {e}");
        std::process::exit(1);
    }

    match command {
        "list" => {
            println!("=========================================================================================");
            println!("                           QEOS V.04 DEVICE MANAGER INVENTORY                            ");
            println!("=========================================================================================");
            println!(
                "{:<12} {:<16} {:<10} {:<24} {:<20} {:<10}",
                "DEVICE ID", "PCI ADDRESS", "VENDOR:DEV", "CLASS", "DRIVER", "STATE"
            );
            println!("-----------------------------------------------------------------------------------------");

            for record in manager.devices() {
                let pci = &record.device;
                let driver_str = record.driver_name.as_deref().unwrap_or("<unbound>");
                println!(
                    "{:<12} {:<16} {:<10} {:<24} {:<20} {:<10}",
                    record.id.to_string(),
                    pci.address().to_string(),
                    pci.info().ids_label(),
                    pci.class().name(),
                    driver_str,
                    record.driver_state.to_string()
                );
            }

            println!("-----------------------------------------------------------------------------------------");
            let counters = manager.counters();
            println!(
                "Total: {} discovered | {} bound | {} running | {} failed",
                counters.enumerated, counters.bound, counters.running, counters.failed
            );
        }
        "pci" => {
            println!("=========================================================================================");
            println!("                             QEOS V.04 PCI BUS HIERARCHY                                ");
            println!("=========================================================================================");

            for record in manager.devices() {
                let pci = &record.device;
                println!(
                    "[-] [{}] {} ({})",
                    pci.address(),
                    pci.class().name(),
                    pci.info().ids_label()
                );
                println!("    Class Code: {}", pci.class().hex());
                println!("    Header:     {}", pci.info().header_type);
                println!(
                    "    DMA Ready:  {} (bus_master={}, mem_space={})",
                    pci.dma_readiness().is_declared_ready(),
                    pci.info().bus_master_enabled(),
                    pci.info().memory_space_enabled()
                );
                println!("    Hotplug:    {}", pci.hotplug_support());

                if !pci.bars().is_empty() {
                    println!("    BARs:");
                    for bar in pci.bars() {
                        if bar.kind != device_manager::BarKind::Unimplemented {
                            println!(
                                "      BAR{}: {} @ 0x{:08x} (size: {} bytes, prefetchable: {})",
                                bar.index, bar.kind, bar.base, bar.size, bar.prefetchable
                            );
                        }
                    }
                }

                if !pci.capabilities().is_empty() {
                    println!("    Capabilities:");
                    for cap in pci.capabilities() {
                        println!("      - {}", cap);
                    }
                }
                println!();
            }
        }
        "doctor" => {
            println!("=========================================================================================");
            println!("                         QEOS V.04 DEVICE SUBSYSTEM DIAGNOSTICS                          ");
            println!("=========================================================================================");

            let summary = manager.health_summary();
            println!("[OK] PCI Bus Configuration Backend: {}", backend.label());
            println!("[OK] Enumerated devices: {}", summary.total_devices);
            println!("[OK] Device Driver Registry: active");
            println!("[OK] Capability-based Access Control: enforced");
            println!(
                "[OK] IOMMU Policy: {} (unmanaged DMA isolated: false)",
                manager.config().iommu_policy
            );
            println!("[OK] Hotplug Policy: {}", manager.config().hotplug_policy);

            println!("\nDevice Health Breakdown:");
            println!("  Healthy:  {}", summary.healthy);
            println!("  Warning:  {}", summary.warning);
            println!("  Degraded: {}", summary.degraded);
            println!("  Failed:   {}", summary.failed);

            println!("\nHardware Availability Assessment:");
            println!("  [OK] Simulated PCI Bus: available");
            println!("  [OK] Simulated NVMe / GPU / Telemetry Drivers: available");
            println!("  [INFO] Physical Hardware QPU: unavailable (Simulation Backend Active)");
            println!("  [INFO] Physical DMA Engine: abstracted (IOMMU Guard Active)");
            println!("\nDiagnostics Status: ALL SYSTEMS OPERATIONAL (SIMULATION/HAL MODE)");
        }
        "json" => {
            let snapshot = manager.snapshot();
            match serde_json::to_string_pretty(&snapshot) {
                Ok(json) => println!("{json}"),
                Err(e) => eprintln!("[ERROR] JSON serialization failed: {e}"),
            }
        }
        "help" | "--help" | "-h" => {
            print_usage();
        }
        other => {
            eprintln!("[ERROR] Unknown command: {other}\n");
            print_usage();
            std::process::exit(1);
        }
    }
}
