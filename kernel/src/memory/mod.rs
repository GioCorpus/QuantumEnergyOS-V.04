pub mod physical; pub mod virtual_; pub mod heap; pub mod allocator;
pub use physical::{PhysAddr, PhysPage, PhysicalMemoryManager};
pub use virtual_::{VirtAddr, VirtPage, VirtualMemoryManager, MapFlags};
pub use allocator::{KernelAllocator, AllocStats};
pub const PAGE_SIZE: usize = 4096;
