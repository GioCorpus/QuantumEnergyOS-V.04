#[derive(Debug, Clone, Copy)] pub struct PciAddr { pub bus: u8, pub dev: u8, pub func: u8 }
impl PciAddr { pub fn new(bus: u8, dev: u8, func: u8) -> Self { Self { bus, dev, func } } }
pub fn enumerate_stub() -> Vec<PciAddr> { vec![PciAddr::new(0, 1, 0)] }
