pub mod coordinate;
pub mod error;
pub mod model;
pub mod prediction;
pub mod projection;
pub mod serialization;
pub mod storage;

pub use coordinate::{Quartz5DCoordinate, Quartz5DRegion};
pub use error::{Quartz5DError, Result};
pub use model::{ModelConfig, Quartz5DModel, Quartz5DModelBuilder};
pub use prediction::{PredictionConfig, PredictionResult, Quartz5DPredictor};
pub use projection::{Projected3D, ProjectionConfig, Quartz5DProjector};
pub use serialization::{
    deserialize_coordinates, deserialize_model, serialize_coordinates, serialize_model,
};
pub use storage::{Quartz5DStorage, StorageConfig, StorageStats};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
