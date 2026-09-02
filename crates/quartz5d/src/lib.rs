pub mod coordinate;
pub mod error;
pub mod model;
pub mod prediction;
pub mod projection;
pub mod serialization;
pub mod storage;

pub use coordinate::{Quartz5DCoordinate, Quartz5DRegion};
pub use error::{Quartz5DError, Result};
pub use model::{Quartz5DModel, Quartz5DModelBuilder, ModelConfig};
pub use prediction::{Quartz5DPredictor, PredictionResult, PredictionConfig};
pub use projection::{Quartz5DProjector, ProjectionConfig, Projected3D};
pub use serialization::{serialize_coordinates, deserialize_coordinates, serialize_model, deserialize_model};
pub use storage::{Quartz5DStorage, StorageConfig, StorageStats};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
