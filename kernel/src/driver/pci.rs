#[derive(Debug, Clone, Copy)] pub struct PciAddr { pub bus: u8, pub dev: u8, pub func: u8 }
impl PciAddr { pub fn new(bus: u8, dev: u8, func: u8) -> Self { Self { bus, dev, func } } }
// DEBT(AUDIT-2026-09-13): host-test stub, NOT real PCI enumeration. No BAR/caps/MSI-X.
// Kept for API shape only; real backend must come from HAL/PcieHal + QEMU validation.
pub fn enumerate_stub() -> Vec<PciAddr> { vec![PciAddr::new(0, 1, 0)] }
/// Explicit host-test alias — makes "fake hardware" obvious at call sites.
pub fn stub_for_host_tests() -> Vec<PciAddr> { enumerate_stub() }
