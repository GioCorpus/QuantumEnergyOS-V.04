use serde::{Deserialize, Serialize};

/// A coordinate in the Quartz 5D computational model.
///
/// The Quartz 5D model defines a computational state space with five dimensions:
/// - X, Y, Z: Spatial coordinates (conventional 3D space)
/// - T: Temporal coordinate (time)
/// - S: System state (a computational state-space dimension)
///
/// IMPORTANT: S is a computational state-space dimension. It is NOT a claim
/// that QuantumEnergyOS has discovered or created a physically demonstrated
/// fifth spatial dimension. S represents the internal state of the system
/// being modeled (e.g., energy level, quantum state index, operational mode).
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quartz5DCoordinate {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub t: i64,
    pub state: f64,
}

impl Quartz5DCoordinate {
    /// Create a new 5D coordinate.
    pub fn new(x: i32, y: i32, z: i32, t: i64, state: f64) -> Self {
        Self { x, y, z, t, state }
    }

    /// Create a coordinate with zero state.
    pub fn at_position(x: i32, y: i32, z: i32, t: i64) -> Self {
        Self {
            x,
            y,
            z,
            t,
            state: 0.0,
        }
    }

    /// Get the spatial distance from another coordinate (ignoring time and state).
    pub fn spatial_distance(&self, other: &Self) -> f64 {
        let dx = (self.x - other.x) as f64;
        let dy = (self.y - other.y) as f64;
        let dz = (self.z - other.z) as f64;
        (dx * dx + dy * dy + dz * dz).sqrt()
    }

    /// Get the temporal distance from another coordinate.
    pub fn temporal_distance(&self, other: &Self) -> i64 {
        (self.t - other.t).abs()
    }

    /// Get the state difference from another coordinate.
    pub fn state_difference(&self, other: &Self) -> f64 {
        (self.state - other.state).abs()
    }

    /// Check if two coordinates are at the same spatial position.
    pub fn same_position(&self, other: &Self) -> bool {
        self.x == other.x && self.y == other.y && self.z == other.z
    }

    /// Check if two coordinates are at the same time.
    pub fn same_time(&self, other: &Self) -> bool {
        self.t == other.t
    }

    /// Create a new coordinate with modified state.
    pub fn with_state(&self, state: f64) -> Self {
        Self { state, ..*self }
    }

    /// Create a new coordinate at a different time.
    pub fn at_time(&self, t: i64) -> Self {
        Self { t, ..*self }
    }
}

impl Default for Quartz5DCoordinate {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            z: 0,
            t: 0,
            state: 0.0,
        }
    }
}

/// A region in 5D space.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Quartz5DRegion {
    pub min: Quartz5DCoordinate,
    pub max: Quartz5DCoordinate,
}

impl Quartz5DRegion {
    pub fn new(min: Quartz5DCoordinate, max: Quartz5DCoordinate) -> Self {
        Self { min, max }
    }

    /// Check if a coordinate is within this region.
    pub fn contains(&self, coord: &Quartz5DCoordinate) -> bool {
        coord.x >= self.min.x
            && coord.x <= self.max.x
            && coord.y >= self.min.y
            && coord.y <= self.max.y
            && coord.z >= self.min.z
            && coord.z <= self.max.z
            && coord.t >= self.min.t
            && coord.t <= self.max.t
            && coord.state >= self.min.state
            && coord.state <= self.max.state
    }

    /// Get the spatial volume of the region.
    pub fn spatial_volume(&self) -> f64 {
        let dx = (self.max.x - self.min.x) as f64;
        let dy = (self.max.y - self.min.y) as f64;
        let dz = (self.max.z - self.min.z) as f64;
        dx * dy * dz
    }

    /// Get the temporal span of the region.
    pub fn temporal_span(&self) -> i64 {
        (self.max.t - self.min.t).abs()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coordinate_creation() {
        let coord = Quartz5DCoordinate::new(1, 2, 3, 1000, 0.5);
        assert_eq!(coord.x, 1);
        assert_eq!(coord.y, 2);
        assert_eq!(coord.z, 3);
        assert_eq!(coord.t, 1000);
        assert!((coord.state - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_coordinate_default() {
        let coord = Quartz5DCoordinate::default();
        assert_eq!(coord.x, 0);
        assert_eq!(coord.y, 0);
        assert_eq!(coord.z, 0);
        assert_eq!(coord.t, 0);
        assert!((coord.state).abs() < 1e-10);
    }

    #[test]
    fn test_spatial_distance() {
        let a = Quartz5DCoordinate::new(0, 0, 0, 0, 0.0);
        let b = Quartz5DCoordinate::new(3, 4, 0, 0, 0.0);
        assert!((a.spatial_distance(&b) - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_temporal_distance() {
        let a = Quartz5DCoordinate::new(0, 0, 0, 100, 0.0);
        let b = Quartz5DCoordinate::new(0, 0, 0, 250, 0.0);
        assert_eq!(a.temporal_distance(&b), 150);
    }

    #[test]
    fn test_state_difference() {
        let a = Quartz5DCoordinate::new(0, 0, 0, 0, 1.0);
        let b = Quartz5DCoordinate::new(0, 0, 0, 0, 3.5);
        assert!((a.state_difference(&b) - 2.5).abs() < 1e-10);
    }

    #[test]
    fn test_same_position() {
        let a = Quartz5DCoordinate::new(1, 2, 3, 100, 0.5);
        let b = Quartz5DCoordinate::new(1, 2, 3, 200, 1.0);
        assert!(a.same_position(&b));
        assert!(!a.same_time(&b));
    }

    #[test]
    fn test_with_state() {
        let a = Quartz5DCoordinate::new(1, 2, 3, 100, 0.5);
        let b = a.with_state(2.0);
        assert!((b.state - 2.0).abs() < 1e-10);
        assert_eq!(b.x, a.x);
        assert_eq!(b.t, a.t);
    }

    #[test]
    fn test_region_contains() {
        let region = Quartz5DRegion::new(
            Quartz5DCoordinate::new(0, 0, 0, 0, 0.0),
            Quartz5DCoordinate::new(10, 10, 10, 1000, 1.0),
        );

        assert!(region.contains(&Quartz5DCoordinate::new(5, 5, 5, 500, 0.5)));
        assert!(!region.contains(&Quartz5DCoordinate::new(15, 5, 5, 500, 0.5)));
        assert!(!region.contains(&Quartz5DCoordinate::new(5, 5, 5, 500, 2.0)));
    }

    #[test]
    fn test_region_volume() {
        let region = Quartz5DRegion::new(
            Quartz5DCoordinate::new(0, 0, 0, 0, 0.0),
            Quartz5DCoordinate::new(10, 10, 10, 1000, 1.0),
        );
        assert!((region.spatial_volume() - 1000.0).abs() < 1e-10);
        assert_eq!(region.temporal_span(), 1000);
    }
}
