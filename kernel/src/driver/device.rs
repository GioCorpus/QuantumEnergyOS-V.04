use super::bus::BusKind;
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct DeviceId(pub String);
#[derive(Debug)]
pub struct Device {
    pub id: DeviceId,
    pub bus: BusKind,
    pub vendor: u16,
    pub product: u16,
}
impl Device {
    pub fn new(id: &str, bus: BusKind, vendor: u16, product: u16) -> Self {
        Self {
            id: DeviceId(id.into()),
            bus,
            vendor,
            product,
        }
    }
}
pub trait Driver {
    fn name(&self) -> &'static str;
    fn probe(&self, dev: &Device) -> bool;
}
#[derive(Default)]
pub struct DriverRegistry {
    pub devices: Vec<Device>,
}
impl DriverRegistry {
    pub fn new() -> Self {
        Self {
            devices: Vec::new(),
        }
    }
    pub fn add(&mut self, d: Device) {
        self.devices.push(d);
    }
    pub fn enumerate(&self) -> usize {
        self.devices.len()
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reg() {
        let mut r = DriverRegistry::new();
        r.add(Device::new("pci0", BusKind::Pcie, 0x8086, 0x1234));
        assert_eq!(r.enumerate(), 1);
    }
}
