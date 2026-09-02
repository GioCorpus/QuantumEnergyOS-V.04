use serde::{Deserialize, Serialize};

use crate::coordinate::Quartz5DCoordinate;

/// Configuration for 5D storage.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Maximum number of coordinates to store.
    pub max_capacity: usize,
    /// Whether to enable automatic compression.
    pub compression: bool,
    /// Whether to index by time for fast temporal queries.
    pub temporal_indexing: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            max_capacity: 100000,
            compression: false,
            temporal_indexing: true,
        }
    }
}

/// Statistics about the storage.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_stored: usize,
    pub capacity: usize,
    pub memory_estimate_bytes: usize,
}

/// Storage for 5D coordinates with optional indexing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Quartz5DStorage {
    config: StorageConfig,
    coordinates: Vec<Quartz5DCoordinate>,
    /// Temporal index: maps time values to coordinate indices.
    #[serde(skip)]
    time_index: Vec<(i64, usize)>,
}

impl Quartz5DStorage {
    pub fn new(config: StorageConfig) -> Self {
        Self {
            config,
            coordinates: Vec::new(),
            time_index: Vec::new(),
        }
    }

    /// Store a coordinate.
    pub fn store(&mut self, coord: Quartz5DCoordinate) -> bool {
        if self.coordinates.len() >= self.config.max_capacity {
            return false;
        }
        let idx = self.coordinates.len();
        self.coordinates.push(coord);

        if self.config.temporal_indexing {
            self.time_index.push((coord.t, idx));
            self.time_index.sort_by_key(|&(t, _)| t);
        }

        true
    }

    /// Store multiple coordinates.
    pub fn store_batch(&mut self, coords: &[Quartz5DCoordinate]) -> usize {
        let mut stored = 0;
        for coord in coords {
            if self.store(*coord) {
                stored += 1;
            } else {
                break;
            }
        }
        stored
    }

    /// Query coordinates by time range.
    pub fn query_time_range(&self, start: i64, end: i64) -> Vec<&Quartz5DCoordinate> {
        if self.config.temporal_indexing {
            self.time_index
                .iter()
                .filter(|&&(t, _)| t >= start && t <= end)
                .map(|&(_, idx)| &self.coordinates[idx])
                .collect()
        } else {
            self.coordinates
                .iter()
                .filter(|c| c.t >= start && c.t <= end)
                .collect()
        }
    }

    /// Query coordinates by spatial region.
    pub fn query_spatial(
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

    /// Get all coordinates.
    pub fn coordinates(&self) -> &[Quartz5DCoordinate] {
        &self.coordinates
    }

    /// Get the number of stored coordinates.
    pub fn len(&self) -> usize {
        self.coordinates.len()
    }

    /// Check if storage is empty.
    pub fn is_empty(&self) -> bool {
        self.coordinates.is_empty()
    }

    /// Get storage statistics.
    pub fn stats(&self) -> StorageStats {
        let coord_size = std::mem::size_of::<Quartz5DCoordinate>();
        StorageStats {
            total_stored: self.coordinates.len(),
            capacity: self.config.max_capacity,
            memory_estimate_bytes: self.coordinates.len() * coord_size,
        }
    }

    /// Clear all stored coordinates.
    pub fn clear(&mut self) {
        self.coordinates.clear();
        self.time_index.clear();
    }

    /// Get the configuration.
    pub fn config(&self) -> &StorageConfig {
        &self.config
    }
}

impl Default for Quartz5DStorage {
    fn default() -> Self {
        Self::new(StorageConfig::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_storage_creation() {
        let storage = Quartz5DStorage::new(StorageConfig::default());
        assert!(storage.is_empty());
        assert_eq!(storage.len(), 0);
    }

    #[test]
    fn test_store_coordinate() {
        let mut storage = Quartz5DStorage::new(StorageConfig::default());
        let coord = Quartz5DCoordinate::new(1, 2, 3, 100, 0.5);
        assert!(storage.store(coord));
        assert_eq!(storage.len(), 1);
    }

    #[test]
    fn test_store_batch() {
        let mut storage = Quartz5DStorage::new(StorageConfig::default());
        let coords = vec![
            Quartz5DCoordinate::new(1, 0, 0, 100, 0.1),
            Quartz5DCoordinate::new(2, 0, 0, 200, 0.2),
            Quartz5DCoordinate::new(3, 0, 0, 300, 0.3),
        ];
        assert_eq!(storage.store_batch(&coords), 3);
    }

    #[test]
    fn test_capacity_limit() {
        let mut storage = Quartz5DStorage::new(StorageConfig {
            max_capacity: 2,
            ..Default::default()
        });

        assert!(storage.store(Quartz5DCoordinate::new(0, 0, 0, 0, 0.0)));
        assert!(storage.store(Quartz5DCoordinate::new(1, 0, 0, 0, 0.0)));
        assert!(!storage.store(Quartz5DCoordinate::new(2, 0, 0, 0, 0.0)));
    }

    #[test]
    fn test_query_time_range() {
        let mut storage = Quartz5DStorage::new(StorageConfig::default());
        storage.store(Quartz5DCoordinate::new(0, 0, 0, 100, 0.0));
        storage.store(Quartz5DCoordinate::new(0, 0, 0, 200, 0.0));
        storage.store(Quartz5DCoordinate::new(0, 0, 0, 300, 0.0));

        let result = storage.query_time_range(150, 250);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_query_spatial() {
        let mut storage = Quartz5DStorage::new(StorageConfig::default());
        storage.store(Quartz5DCoordinate::new(5, 5, 5, 0, 0.0));
        storage.store(Quartz5DCoordinate::new(15, 15, 15, 0, 0.0));

        let result = storage.query_spatial(0, 10, 0, 10, 0, 10);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn test_stats() {
        let mut storage = Quartz5DStorage::new(StorageConfig::default());
        storage.store(Quartz5DCoordinate::new(0, 0, 0, 0, 0.0));

        let stats = storage.stats();
        assert_eq!(stats.total_stored, 1);
        assert!(stats.memory_estimate_bytes > 0);
    }

    #[test]
    fn test_clear() {
        let mut storage = Quartz5DStorage::new(StorageConfig::default());
        storage.store(Quartz5DCoordinate::new(0, 0, 0, 0, 0.0));
        storage.clear();
        assert!(storage.is_empty());
    }
}
