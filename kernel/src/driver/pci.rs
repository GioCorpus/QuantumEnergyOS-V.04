#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PciAddr {
    pub bus: u8,
    pub dev: u8,
    pub func: u8,
}
impl PciAddr {
    pub const fn new(bus: u8, dev: u8, func: u8) -> Self {
        Self { bus, dev, func }
    }
    pub fn label(&self) -> AddrLabel {
        AddrLabel {
            bus: self.bus,
            dev: self.dev,
            func: self.func,
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AddrLabel {
    bus: u8,
    dev: u8,
    func: u8,
}
impl core::fmt::Display for AddrLabel {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "0000:{:02x}:{:02x}.{}", self.bus, self.dev, self.func)
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PciIdentity {
    pub addr: PciAddr,
    pub vendor_id: u16,
    pub device_id: u16,
    pub class_code: u8,
    pub subclass: u8,
    pub prog_if: u8,
    pub revision: u8,
    pub header_type: u8,
}
impl PciIdentity {
    pub const fn present(vendor_id: u16) -> bool {
        vendor_id != 0xFFFF
    }
    pub fn is_bridge(&self) -> bool {
        self.class_code == 0x06 && self.subclass == 0x04
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BarDescriptor {
    pub index: u8,
    pub is_memory: bool,
    pub is_64bit: bool,
    pub prefetchable: bool,
    pub base: u64,
    pub size: u64,
}
impl BarDescriptor {
    pub const fn is_mappable(&self) -> bool {
        self.is_memory && self.size > 0
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct InterruptFacts {
    pub has_msi: bool,
    pub has_msix: bool,
    pub legacy_pin: u8,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DmaFacts {
    pub bus_master: bool,
    pub mem_space: bool,
    pub addr64: bool,
    pub ats: bool,
}
impl DmaFacts {
    pub const fn declared_ready(&self) -> bool {
        self.bus_master && self.mem_space
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PciFunction {
    pub identity: PciIdentity,
    pub bars: [Option<BarDescriptor>; 6],
    pub irq: InterruptFacts,
    pub dma: DmaFacts,
}
impl PciFunction {
    pub fn bar(&self, index: usize) -> Option<BarDescriptor> {
        if index < 6 {
            self.bars[index]
        } else {
            None
        }
    }
    pub fn mappable_bars(&self) -> usize {
        self.bars
            .iter()
            .filter(|b| b.map(|d| d.is_mappable()).unwrap_or(false))
            .count()
    }
}
pub trait PciConfigSource {
    fn read_u16(&self, addr: PciAddr, offset: u16) -> u16;
    fn read_u32(&self, addr: PciAddr, offset: u16) -> u32;
}
pub struct StubConfigSource {
    functions: Vec<PciFunction>,
}
impl StubConfigSource {
    pub fn new(functions: Vec<PciFunction>) -> Self {
        Self { functions }
    }
    pub fn functions(&self) -> &[PciFunction] {
        &self.functions
    }
    pub fn qemu_virtio_net() -> Self {
        Self::new(vec![PciFunction {
            identity: PciIdentity {
                addr: PciAddr::new(0, 1, 0),
                vendor_id: 0x1AF4,
                device_id: 0x1000,
                class_code: 0x02,
                subclass: 0x00,
                prog_if: 0x00,
                revision: 0x01,
                header_type: 0x00,
            },
            bars: [
                Some(BarDescriptor {
                    index: 0,
                    is_memory: true,
                    is_64bit: false,
                    prefetchable: false,
                    base: 0,
                    size: 0x1000,
                }),
                None,
                None,
                None,
                None,
                None,
            ],
            irq: InterruptFacts {
                has_msi: true,
                has_msix: false,
                legacy_pin: 1,
            },
            dma: DmaFacts {
                bus_master: true,
                mem_space: true,
                addr64: false,
                ats: false,
            },
        }])
    }
}
impl PciConfigSource for StubConfigSource {
    fn read_u16(&self, addr: PciAddr, offset: u16) -> u16 {
        let Some(f) = self.functions.iter().find(|f| f.identity.addr == addr) else {
            return 0xFFFF;
        };
        match offset {
            0x00 => f.identity.vendor_id,
            0x02 => f.identity.device_id,
            _ => 0,
        }
    }
    fn read_u32(&self, _a: PciAddr, _o: u16) -> u32 {
        0
    }
}
pub fn enumerate_stub() -> Vec<PciAddr> {
    vec![PciAddr::new(0, 1, 0)]
}
pub fn stub_for_host_tests() -> Vec<PciAddr> {
    enumerate_stub()
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PciFunctionDesc {
    pub addr: PciAddr,
    pub vendor_id: u16,
    pub device_id: u16,
}
pub fn enumerate_source<S: PciConfigSource>(s: &S, addrs: &[PciAddr]) -> Vec<PciFunctionDesc> {
    addrs
        .iter()
        .filter(|a| PciIdentity::present(s.read_u16(**a, 0x00)))
        .map(|a| PciFunctionDesc {
            addr: *a,
            vendor_id: s.read_u16(*a, 0x00),
            device_id: s.read_u16(*a, 0x02),
        })
        .collect()
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stub_fixture() {
        assert_eq!(stub_for_host_tests(), vec![PciAddr::new(0, 1, 0)]);
    }
    #[test]
    fn absent_is_ffff() {
        let s = StubConfigSource::new(vec![]);
        assert!(enumerate_source(&s, &[PciAddr::new(0, 9, 0)]).is_empty());
    }
    #[test]
    fn qemu_fixture_honest() {
        let s = StubConfigSource::qemu_virtio_net();
        let f = &s.functions()[0];
        assert_eq!(f.identity.vendor_id, 0x1AF4);
        assert!(!f.dma.addr64);
        assert!(!f.dma.ats);
        assert!(f.bar(0).map(|b| b.is_mappable()).unwrap_or(false));
        assert_eq!(f.mappable_bars(), 1);
        assert!(f.irq.has_msi);
        assert!(!f.irq.has_msix);
    }
    #[test]
    fn label_fmt() {
        assert_eq!(PciAddr::new(0, 0x1f, 0).label().to_string(), "0000:00:1f.0");
    }
}
