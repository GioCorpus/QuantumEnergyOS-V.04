use serde::{Deserialize, Serialize};

/// Configuration for the Quartz5D model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// Maximum number of coordinates to store.
    pub max_coordinates: usize,
    /// Default state value for new coordinates.
    pub default_state: f64,
    /// Whether to enable temporal interpolation.
    pub temporal_interpolation: bool,
    /// Whether to enable state smoothing.
    pub state_smoothing: bool,
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            max_coordinates: 10000,
            default_state: 0.0,
            temporal_interpolation: true,
            state_smoothing: false,
        }
    }
}

/// The Quartz5D computational model.
///
/// This model represents a 5-dimensional computational state space:
/// - X, Y, Z: Spatial coordinates
/// - T: Temporal coordinate
/// - S: System state (computational state-space dimension)
///
/// The model stores coordinates and provides operations for querying
/// and manipulating the state space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quartz5DModel {
    config: ModelConfig,
    coordinates: Vec<Quartz5DCoordinate>,
}

impl Quartz5DModel {
    /// Create a new model with the given configuration.
    pub fn new(config: ModelConfig) -> Self {
        Self {
            config,
            coordinates: Vec::new(),
        }
    }

    /// Add a coordinate to the model.
    pub fn add_coordinate(&mut self, coord: Quartz5DCoordinate) -> bool {
        if self.coordinates.len() >= self.config.max_coordinates {
            return false;
        }
        self.coordinates.push(coord);
        true
    }

    /// Get all coordinates.
    pub fn coordinates(&self) -> &[Quartz5DCoordinate] {
        &self.coordinates
    }

    /// Get the number of coordinates.
    pub fn len(&self) -> usize {
        self.coordinates.len()
    }

    /// Check if the model is empty.
    pub fn is_empty(&self) -> bool {
        self.coordinates.is_empty()
    }

    /// Get coordinates at a specific time.
    pub fn coordinates_at_time(&self, t: i64) -> Vec<&Quartz5DCoordinate> {
        self.coordinates.iter().filter(|c| c.t == t).collect()
    }

    /// Get coordinates within a spatial region.
    pub fn coordinates_in_region(
        &self,
        min_x: i32,
        max_x: i32,
        min_y: i32,
        max_y: i32,
        min_z: i32,
        max_z: i32,
    ) -> Vec<&Quartz5DCoordinate> {
        self.coordinates
            .iter()
            .filter(|c| {
                c.x >= min_x
                    && c.x <= max_x
                    && c.y >= min_y
                    && c.y <= max_y
                    && c.z >= min_z
                    && c.z <= max_z
            })
            .collect()
    }

    /// Get the state range across all coordinates.
    pub fn state_range(&self) -> Option<(f64, f64)> {
        if self.coordinates.is_empty() {
            return None;
        }
        let min = self.coordinates.iter().map(|c| c.state).fold(f64::INFINITY, f64::min);
        let max = self.coordinates.iter().map(|c| c.state).fold(f64::NEG_INFINITY, f64::max);
        Some((min, max))
    }

    /// Clear all coordinates.
    pub fn clear(&mut self) {
        self.coordinates.clear();
    }

    /// Get the model configuration.
    pub fn config(&self) -> &ModelConfig {
        &self.config
    }
}

/// Builder for constructing Quartz5DModel instances.
pub struct Quartz5DModelBuilder {
    config: ModelConfig,
}

impl Quartz5DModelBuilder {
    pub fn new() -> Self {
        Self {
            config: ModelConfig::default(),
        }
    }

    pub fn with_max_coordinates(mut self, max: usize) -> Self {
        self.config.max_coordinates = max;
        self
    }

    pub fn with_default_state(mut self, state: f64) -> Self {
        self.config.default_state = state;
        self
    }

    pub fn with_temporal_interpolation(mut self, enabled: bool) -> Self {
        self.config.temporal_interpolation = enabled;
        self
    }

    pub fn with_state_smoothing(mut self, enabled: bool) -> Self {
        self.config.state_smoothing = enabled;
        self
    }

    pub fn build(self) -> Quartz5DModel {
        Quartz5DModel::new(self.config)
    }
}

impl Default for Quartz5DModelBuilder {
    fn default() -> Self {
        Self::new()
    }
}

use super::coordinate::Quartz5DCoordinate;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_model_creation() {
        let model = Quartz5DModel::new(ModelConfig::default());
        assert!(model.is_empty());
        assert_eq!(model.len(), 0);
    }

    #[test]
    fn test_add_coordinate() {
        let mut model = Quartz5DModel::new(ModelConfig::default());
        let coord = Quartz5DCoordinate::new(1, 2, 3, 100, 0.5);
        assert!(model.add_coordinate(coord));
        assert_eq!(model.len(), 1);
    }

    #[test]
    fn test_max_coordinates() {
        let mut model = Quartz5DModel::new(ModelConfig {
            max_coordinates: 2,
            ..Default::default()
        });

        assert!(model.add_coordinate(Quartz5DCoordinate::new(0, 0, 0, 0, 0.0)));
        assert!(model.add_coordinate(Quartz5DCoordinate::new(1, 0, 0, 0, 0.0)));
        assert!(!model.add_coordinate(Quartz5DCoordinate::new(2, 0, 0, 0, 0.0)));
    }

    #[test]
    fn test_coordinates_at_time() {
        let mut model = Quartz5DModel::new(ModelConfig::default());
        model.add_coordinate(Quartz5DCoordinate::new(0, 0, 0, 100, 0.0));
        model.add_coordinate(Quartz5DCoordinate::new(1, 0, 0, 100, 0.0));
        model.add_coordinate(Quartz5DCoordinate::new(2, 0, 0, 200, 0.0));

        let at_100 = model.coordinates_at_time(100);
        assert_eq!(at_100.len(), 2);
    }

    #[test]
    fn test_coordinates_in_region() {
        let mut model = Quartz5DModel::new(ModelConfig::default());
        model.add_coordinate(Quartz5DCoordinate::new(5, 5, 5, 0, 0.0));
        model.add_coordinate(Quartz5DCoordinate::new(15, 15, 15, 0, 0.0));

        let in_region = model.coordinates_in_region(0, 10, 0, 10, 0, 10);
        assert_eq!(in_region.len(), 1);
    }

    #[test]
    fn test_state_range() {
        let mut model = Quartz5DModel::new(ModelConfig::default());
        assert!(model.state_range().is_none());

        model.add_coordinate(Quartz5DCoordinate::new(0, 0, 0, 0, 1.0));
        model.add_coordinate(Quartz5DCoordinate::new(0, 0, 0, 0, 5.0));
        model.add_coordinate(Quartz5DCoordinate::new(0, 0, 0, 0, 3.0));

        let (min, max) = model.state_range().unwrap();
        assert!((min - 1.0).abs() < 1e-10);
        assert!((max - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_builder() {
        let model = Quartz5DModelBuilder::new()
            .with_max_coordinates(100)
            .with_default_state(1.0)
            .with_temporal_interpolation(false)
            .build();

        assert_eq!(model.config().max_coordinates, 100);
        assert!((model.config().default_state - 1.0).abs() < 1e-10);
        assert!(!model.config().temporal_interpolation);
    }

    #[test]
    fn test_clear() {
        let mut model = Quartz5DModel::new(ModelConfig::default());
        model.add_coordinate(Quartz5DCoordinate::new(0, 0, 0, 0, 0.0));
        model.clear();
        assert!(model.is_empty());
    }
}
