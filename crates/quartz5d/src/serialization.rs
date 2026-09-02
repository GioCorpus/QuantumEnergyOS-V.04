use serde::{Deserialize, Serialize};

use crate::coordinate::Quartz5DCoordinate;
use crate::error::{Quartz5DError, Result};
use crate::model::Quartz5DModel;

/// Serialize a slice of coordinates to JSON.
pub fn serialize_coordinates(coords: &[Quartz5DCoordinate]) -> Result<String> {
    serde_json::to_string(coords).map_err(|e| Quartz5DError::SerializationError(e.to_string()))
}

/// Deserialize coordinates from JSON.
pub fn deserialize_coordinates(json: &str) -> Result<Vec<Quartz5DCoordinate>> {
    serde_json::from_str(json).map_err(|e| Quartz5DError::SerializationError(e.to_string()))
}

/// Serialize a model to JSON.
pub fn serialize_model(model: &Quartz5DModel) -> Result<String> {
    serde_json::to_string(model).map_err(|e| Quartz5DError::SerializationError(e.to_string()))
}

/// Deserialize a model from JSON.
pub fn deserialize_model(json: &str) -> Result<Quartz5DModel> {
    serde_json::from_str(json).map_err(|e| Quartz5DError::SerializationError(e.to_string()))
}

/// Serialize coordinates to a compact binary format (using bincode-style JSON).
pub fn serialize_compact(coords: &[Quartz5DCoordinate]) -> Result<Vec<u8>> {
    serde_json::to_vec(coords).map_err(|e| Quartz5DError::SerializationError(e.to_string()))
}

/// Deserialize coordinates from compact format.
pub fn deserialize_compact(bytes: &[u8]) -> Result<Vec<Quartz5DCoordinate>> {
    serde_json::from_slice(bytes).map_err(|e| Quartz5DError::SerializationError(e.to_string()))
}

/// A serializable snapshot of the entire 5D state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quartz5DSnapshot {
    pub version: u32,
    pub coordinates: Vec<Quartz5DCoordinate>,
    pub timestamp_ns: u64,
}

impl Quartz5DSnapshot {
    pub const FORMAT_VERSION: u32 = 1;

    pub fn new(coordinates: Vec<Quartz5DCoordinate>) -> Self {
        Self {
            version: Self::FORMAT_VERSION,
            coordinates,
            timestamp_ns: 0,
        }
    }

    pub fn with_timestamp(mut self, timestamp_ns: u64) -> Self {
        self.timestamp_ns = timestamp_ns;
        self
    }

    pub fn to_json(&self) -> Result<String> {
        serde_json::to_string(self).map_err(|e| Quartz5DError::SerializationError(e.to_string()))
    }

    pub fn from_json(json: &str) -> Result<Self> {
        serde_json::from_str(json).map_err(|e| Quartz5DError::SerializationError(e.to_string()))
    }

    pub fn validate(&self) -> Result<()> {
        if self.version != Self::FORMAT_VERSION {
            return Err(Quartz5DError::SerializationError(format!(
                "unsupported snapshot version: {}",
                self.version
            )));
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_serialize_coordinates() {
        let coords = vec![
            Quartz5DCoordinate::new(1, 2, 3, 100, 0.5),
            Quartz5DCoordinate::new(4, 5, 6, 200, 1.0),
        ];

        let json = serialize_coordinates(&coords).unwrap();
        let deserialized = deserialize_coordinates(&json).unwrap();

        assert_eq!(coords, deserialized);
    }

    #[test]
    fn test_serialize_model() {
        let mut model = Quartz5DModel::new(crate::model::ModelConfig::default());
        model.add_coordinate(Quartz5DCoordinate::new(1, 2, 3, 100, 0.5));

        let json = serialize_model(&model).unwrap();
        let deserialized = deserialize_model(&json).unwrap();

        assert_eq!(model.len(), deserialized.len());
    }

    #[test]
    fn test_compact_format() {
        let coords = vec![
            Quartz5DCoordinate::new(1, 0, 0, 0, 0.1),
            Quartz5DCoordinate::new(2, 0, 0, 0, 0.2),
        ];

        let bytes = serialize_compact(&coords).unwrap();
        let deserialized = deserialize_compact(&bytes).unwrap();

        assert_eq!(coords, deserialized);
    }

    #[test]
    fn test_snapshot() {
        let coords = vec![Quartz5DCoordinate::new(1, 2, 3, 100, 0.5)];
        let snapshot = Quartz5DSnapshot::new(coords).with_timestamp(1234567890);

        assert_eq!(snapshot.version, Quartz5DSnapshot::FORMAT_VERSION);
        assert_eq!(snapshot.timestamp_ns, 1234567890);

        let json = snapshot.to_json().unwrap();
        let deserialized = Quartz5DSnapshot::from_json(&json).unwrap();

        assert_eq!(snapshot.version, deserialized.version);
        assert_eq!(snapshot.coordinates, deserialized.coordinates);
    }

    #[test]
    fn test_snapshot_validation() {
        let snapshot = Quartz5DSnapshot::new(vec![]);
        assert!(snapshot.validate().is_ok());

        let invalid = Quartz5DSnapshot {
            version: 999,
            coordinates: vec![],
            timestamp_ns: 0,
        };
        assert!(invalid.validate().is_err());
    }

    #[test]
    fn test_empty_coordinates() {
        let coords: Vec<Quartz5DCoordinate> = vec![];
        let json = serialize_coordinates(&coords).unwrap();
        let deserialized = deserialize_coordinates(&json).unwrap();
        assert!(deserialized.is_empty());
    }
}
