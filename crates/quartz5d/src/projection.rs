use serde::{Deserialize, Serialize};

use crate::coordinate::Quartz5DCoordinate;

/// Configuration for 5D to 3D projection.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ProjectionConfig {
    /// How to map the S (state) dimension.
    pub state_mapping: StateMapping,
    /// Temporal window for aggregation (0 = all time).
    pub temporal_window: i64,
    /// Spatial scale factor.
    pub spatial_scale: f64,
    /// State scale factor.
    pub state_scale: f64,
}

impl Default for ProjectionConfig {
    fn default() -> Self {
        Self {
            state_mapping: StateMapping::Intensity,
            temporal_window: 0,
            spatial_scale: 1.0,
            state_scale: 1.0,
        }
    }
}

/// How the state dimension is mapped to a visual property.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StateMapping {
    /// Map state to color intensity.
    Intensity,
    /// Map state to node size.
    Size,
    /// Map state to opacity.
    Opacity,
    /// Map state to height (Z-axis displacement).
    Height,
}

/// A 3D-projected coordinate with visual properties.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Projected3D {
    /// Projected X coordinate.
    pub x: f64,
    /// Projected Y coordinate.
    pub y: f64,
    /// Projected Z coordinate.
    pub z: f64,
    /// Intensity (0.0 - 1.0) for visualization.
    pub intensity: f64,
    /// Size for visualization.
    pub size: f64,
    /// Original state value.
    pub original_state: f64,
    /// Original time.
    pub original_time: i64,
}

/// Projects 5D coordinates into 3D space for visualization.
///
/// The projection maps:
/// - X, Y, Z → geometry (direct mapping with scale)
/// - T → animation (timeline position)
/// - S → intensity / scale / state (visual property)
pub struct Quartz5DProjector {
    config: ProjectionConfig,
}

impl Quartz5DProjector {
    pub fn new(config: ProjectionConfig) -> Self {
        Self { config }
    }

    /// Project a single 5D coordinate into 3D.
    pub fn project(&self, coord: &Quartz5DCoordinate) -> Projected3D {
        let base_x = coord.x as f64 * self.config.spatial_scale;
        let base_y = coord.y as f64 * self.config.spatial_scale;
        let base_z = coord.z as f64 * self.config.spatial_scale;

        let normalized_state = (coord.state * self.config.state_scale).clamp(0.0, 1.0);

        match self.config.state_mapping {
            StateMapping::Height => Projected3D {
                x: base_x,
                y: base_y,
                z: base_z + normalized_state * 10.0,
                intensity: 1.0,
                size: 1.0,
                original_state: coord.state,
                original_time: coord.t,
            },
            StateMapping::Intensity => Projected3D {
                x: base_x,
                y: base_y,
                z: base_z,
                intensity: normalized_state,
                size: 1.0,
                original_state: coord.state,
                original_time: coord.t,
            },
            StateMapping::Size => Projected3D {
                x: base_x,
                y: base_y,
                z: base_z,
                intensity: 1.0,
                size: normalized_state * 5.0 + 0.5,
                original_state: coord.state,
                original_time: coord.t,
            },
            StateMapping::Opacity => Projected3D {
                x: base_x,
                y: base_y,
                z: base_z,
                intensity: normalized_state,
                size: 1.0,
                original_state: coord.state,
                original_time: coord.t,
            },
        }
    }

    /// Project multiple coordinates.
    pub fn project_batch(&self, coords: &[Quartz5DCoordinate]) -> Vec<Projected3D> {
        coords.iter().map(|c| self.project(c)).collect()
    }

    /// Project coordinates within a temporal window.
    pub fn project_temporal_window(
        &self,
        coords: &[Quartz5DCoordinate],
        center_time: i64,
    ) -> Vec<Projected3D> {
        let window = self.config.temporal_window;
        coords
            .iter()
            .filter(|c| {
                if window <= 0 {
                    true
                } else {
                    (c.t - center_time).abs() <= window
                }
            })
            .map(|c| self.project(c))
            .collect()
    }

    /// Get the current projection configuration.
    pub fn config(&self) -> &ProjectionConfig {
        &self.config
    }

    /// Update the projection configuration.
    pub fn set_config(&mut self, config: ProjectionConfig) {
        self.config = config;
    }
}

impl Default for Quartz5DProjector {
    fn default() -> Self {
        Self::new(ProjectionConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_projector_creation() {
        let projector = Quartz5DProjector::new(ProjectionConfig::default());
        assert!(matches!(
            projector.config().state_mapping,
            StateMapping::Intensity
        ));
    }

    #[test]
    fn test_project_intensity() {
        let projector = Quartz5DProjector::new(ProjectionConfig {
            state_mapping: StateMapping::Intensity,
            spatial_scale: 1.0,
            state_scale: 1.0,
            ..Default::default()
        });

        let coord = Quartz5DCoordinate::new(1, 2, 3, 100, 0.5);
        let projected = projector.project(&coord);

        assert!((projected.x - 1.0).abs() < 1e-10);
        assert!((projected.y - 2.0).abs() < 1e-10);
        assert!((projected.z - 3.0).abs() < 1e-10);
        assert!((projected.intensity - 0.5).abs() < 1e-10);
        assert_eq!(projected.original_time, 100);
    }

    #[test]
    fn test_project_size() {
        let projector = Quartz5DProjector::new(ProjectionConfig {
            state_mapping: StateMapping::Size,
            spatial_scale: 1.0,
            state_scale: 1.0,
            ..Default::default()
        });

        let coord = Quartz5DCoordinate::new(0, 0, 0, 0, 0.5);
        let projected = projector.project(&coord);

        assert!(projected.size > 0.5 && projected.size < 5.5);
        assert!((projected.intensity - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_project_height() {
        let projector = Quartz5DProjector::new(ProjectionConfig {
            state_mapping: StateMapping::Height,
            spatial_scale: 1.0,
            state_scale: 1.0,
            ..Default::default()
        });

        let coord = Quartz5DCoordinate::new(0, 0, 0, 0, 1.0);
        let projected = projector.project(&coord);

        assert!((projected.z - 10.0).abs() < 1e-10);
    }

    #[test]
    fn test_project_batch() {
        let projector = Quartz5DProjector::default();
        let coords = vec![
            Quartz5DCoordinate::new(1, 0, 0, 0, 0.1),
            Quartz5DCoordinate::new(2, 0, 0, 0, 0.2),
            Quartz5DCoordinate::new(3, 0, 0, 0, 0.3),
        ];

        let projected = projector.project_batch(&coords);
        assert_eq!(projected.len(), 3);
    }

    #[test]
    fn test_temporal_window() {
        let projector = Quartz5DProjector::new(ProjectionConfig {
            temporal_window: 50,
            ..Default::default()
        });

        let coords = vec![
            Quartz5DCoordinate::new(0, 0, 0, 100, 0.0),
            Quartz5DCoordinate::new(0, 0, 0, 120, 0.0),
            Quartz5DCoordinate::new(0, 0, 0, 200, 0.0),
        ];

        let projected = projector.project_temporal_window(&coords, 110);
        assert_eq!(projected.len(), 2);
    }

    #[test]
    fn test_spatial_scale() {
        let projector = Quartz5DProjector::new(ProjectionConfig {
            spatial_scale: 2.0,
            ..Default::default()
        });

        let coord = Quartz5DCoordinate::new(5, 10, 15, 0, 0.0);
        let projected = projector.project(&coord);

        assert!((projected.x - 10.0).abs() < 1e-10);
        assert!((projected.y - 20.0).abs() < 1e-10);
        assert!((projected.z - 30.0).abs() < 1e-10);
    }

    #[test]
    fn test_state_clamping() {
        let projector = Quartz5DProjector::default();

        let coord_high = Quartz5DCoordinate::new(0, 0, 0, 0, 10.0);
        let projected_high = projector.project(&coord_high);
        assert!(projected_high.intensity <= 1.0);

        let coord_neg = Quartz5DCoordinate::new(0, 0, 0, 0, -5.0);
        let projected_neg = projector.project(&coord_neg);
        assert!(projected_neg.intensity >= 0.0);
    }
}
