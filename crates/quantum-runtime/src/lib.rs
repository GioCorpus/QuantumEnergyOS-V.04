pub mod backend;
pub mod measurement;
pub mod topology;

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
