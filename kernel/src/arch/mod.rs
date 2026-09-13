//! Architecture abstraction. All arch-specific code lives under `arch/`.
pub mod x86_64;
pub trait ArchHal {
    fn name() -> &'static str;
    fn init_cpu();
    fn halt() -> !;
}
