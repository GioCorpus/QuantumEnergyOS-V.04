#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Capability {
    DeviceRead,
    DeviceWrite,
    Dma,
    Pci,
    Telemetry,
    Quantum,
    Admin,
}
#[derive(Debug, Default, Clone)]
pub struct CapSet(pub Vec<Capability>);
impl CapSet {
    pub fn new() -> Self {
        Self(Vec::new())
    }
    pub fn grant(&mut self, c: Capability) {
        if !self.0.contains(&c) {
            self.0.push(c);
        }
    }
    pub fn has(&self, c: Capability) -> bool {
        self.0.contains(&c) || self.0.contains(&Capability::Admin)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn caps() {
        let mut s = CapSet::new();
        s.grant(Capability::Dma);
        assert!(s.has(Capability::Dma));
        assert!(!s.has(Capability::Quantum));
    }
}
