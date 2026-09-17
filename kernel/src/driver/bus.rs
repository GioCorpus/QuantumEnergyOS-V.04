#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BusKind {
    Pcie,
    I2c,
    Spi,
    Usb,
}
