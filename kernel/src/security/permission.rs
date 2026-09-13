use super::capability::{CapSet, Capability};
#[derive(Debug)] pub enum Access { ReadDevice, WriteDevice, UseDma, UseQuantum }
pub fn check(caps: &CapSet, a: Access) -> bool {
    match a { Access::ReadDevice => caps.has(Capability::DeviceRead), Access::WriteDevice => caps.has(Capability::DeviceWrite), Access::UseDma => caps.has(Capability::Dma), Access::UseQuantum => caps.has(Capability::Quantum), }
}
