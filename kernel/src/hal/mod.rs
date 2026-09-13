//! QEOS HAL — kernel depends on traits, not concrete sensors/QPUs.
pub trait CpuHal { fn init(); }
pub trait TimerHal { fn now_ns() -> u64; }
pub trait PcieHal { fn scan() -> usize; }
pub struct NullHal;
impl CpuHal for NullHal { fn init() {} }
impl TimerHal for NullHal { fn now_ns() -> u64 { 0 } }
impl PcieHal for NullHal { fn scan() -> usize { 0 } }
