//! IRQ abstraction: top-half ACK -> ring event -> bottom-half.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Irq(pub u8);
pub trait IrqHandler {
    fn top_half(&mut self, irq: Irq);
}
