use serde::{Deserialize, Serialize};

use crate::coordinate::Quartz5DCoordinate;

/// Configuration for the prediction engine.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionConfig {
    /// Number of historical points to use for prediction.
    pub history_size: usize,
    /// Prediction horizon (how many steps ahead to predict).
    pub horizon: usize,
    /// Smoothing factor for exponential moving average (0.0 - 1.0).
    pub smoothing_factor: f64,
    /// Whether to use linear regression for trend detection.
    pub use_trend: bool,
}

impl Default for PredictionConfig {
    fn default() -> Self {
        Self {
            history_size: 10,
            horizon: 1,
            smoothing_factor: 0.3,
            use_trend: true,
        }
    }
}

/// Result of a prediction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PredictionResult {
    /// Predicted coordinate.
    pub predicted: Quartz5DCoordinate,
    /// Confidence level (0.0 - 1.0).
    pub confidence: f64,
    /// Prediction method used.
    pub method: PredictionMethod,
}

/// Methods used for prediction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PredictionMethod {
    /// Simple linear extrapolation.
    LinearExtrapolation,
    /// Exponential moving average.
    ExponentialMovingAverage,
    /// Polynomial regression.
    PolynomialRegression,
}

/// Predicts future states in the Quartz5D model.
pub struct Quartz5DPredictor {
    config: PredictionConfig,
}

impl Quartz5DPredictor {
    pub fn new(config: PredictionConfig) -> Self {
        Self { config }
    }

    /// Predict the next coordinate based on historical data.
    pub fn predict(&self, history: &[Quartz5DCoordinate]) -> Option<PredictionResult> {
        if history.len() < 2 {
            return None;
        }

        let recent = &history[history.len().saturating_sub(self.config.history_size)..];

        if self.config.use_trend {
            self.predict_linear(recent)
        } else {
            self.predict_ema(recent)
        }
    }

    /// Predict using linear extrapolation.
    fn predict_linear(&self, history: &[Quartz5DCoordinate]) -> Option<PredictionResult> {
        if history.len() < 2 {
            return None;
        }

        let last = &history[history.len() - 1];
        let prev = &history[history.len() - 2];

        let dt = (last.t - prev.t) as f64;
        if dt.abs() < 1e-10 {
            return None;
        }

        let dx = (last.x - prev.x) as f64 / dt;
        let dy = (last.y - prev.y) as f64 / dt;
        let dz = (last.z - prev.z) as f64 / dt;
        let ds = (last.state - prev.state) / dt;

        let next_t = last.t + dt as i64;

        Some(PredictionResult {
            predicted: Quartz5DCoordinate::new(
                (last.x as f64 + dx * dt) as i32,
                (last.y as f64 + dy * dt) as i32,
                (last.z as f64 + dz * dt) as i32,
                next_t,
                (last.state + ds * dt).clamp(0.0, 1.0),
            ),
            confidence: 0.7,
            method: PredictionMethod::LinearExtrapolation,
        })
    }

    /// Predict using exponential moving average.
    fn predict_ema(&self, history: &[Quartz5DCoordinate]) -> Option<PredictionResult> {
        if history.is_empty() {
            return None;
        }

        let alpha = self.config.smoothing_factor;
        let mut ema_x = history[0].x as f64;
        let mut ema_y = history[0].y as f64;
        let mut ema_z = history[0].z as f64;
        let mut ema_state = history[0].state;

        for coord in &history[1..] {
            ema_x = alpha * coord.x as f64 + (1.0 - alpha) * ema_x;
            ema_y = alpha * coord.y as f64 + (1.0 - alpha) * ema_y;
            ema_z = alpha * coord.z as f64 + (1.0 - alpha) * ema_z;
            ema_state = alpha * coord.state + (1.0 - alpha) * ema_state;
        }

        let last = &history[history.len() - 1];
        let dt = if history.len() >= 2 {
            last.t - history[history.len() - 2].t
        } else {
            1
        };

        Some(PredictionResult {
            predicted: Quartz5DCoordinate::new(
                ema_x as i32,
                ema_y as i32,
                ema_z as i32,
                last.t + dt,
                ema_state.clamp(0.0, 1.0),
            ),
            confidence: 0.6,
            method: PredictionMethod::ExponentialMovingAverage,
        })
    }

    /// Compute the velocity vector from recent history.
    pub fn compute_velocity(&self, history: &[Quartz5DCoordinate]) -> Option<VelocityVector> {
        if history.len() < 2 {
            return None;
        }

        let last = &history[history.len() - 1];
        let prev = &history[history.len() - 2];

        let dt = (last.t - prev.t) as f64;
        if dt.abs() < 1e-10 {
            return None;
        }

        Some(VelocityVector {
            vx: (last.x - prev.x) as f64 / dt,
            vy: (last.y - prev.y) as f64 / dt,
            vz: (last.z - prev.z) as f64 / dt,
            vs: (last.state - prev.state) / dt,
        })
    }

    /// Get the current configuration.
    pub fn config(&self) -> &PredictionConfig {
        &self.config
    }
}

impl Default for Quartz5DPredictor {
    fn default() -> Self {
        Self::new(PredictionConfig::default())
    }
}

/// Velocity vector in 5D space.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VelocityVector {
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub vs: f64,
}

impl VelocityVector {
    /// Get the spatial speed (magnitude of spatial velocity).
    pub fn spatial_speed(&self) -> f64 {
        (self.vx * self.vx + self.vy * self.vy + self.vz * self.vz).sqrt()
    }

    /// Get the total speed including state change.
    pub fn total_speed(&self) -> f64 {
        let spatial = self.spatial_speed();
        (spatial * spatial + self.vs * self.vs).sqrt()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_predictor_creation() {
        let predictor = Quartz5DPredictor::new(PredictionConfig::default());
        assert!(predictor.config().use_trend);
    }

    #[test]
    fn test_predict_linear() {
        let predictor = Quartz5DPredictor::new(PredictionConfig {
            use_trend: true,
            ..Default::default()
        });

        let history = vec![
            Quartz5DCoordinate::new(0, 0, 0, 0, 0.0),
            Quartz5DCoordinate::new(1, 0, 0, 1, 0.1),
            Quartz5DCoordinate::new(2, 0, 0, 2, 0.2),
        ];

        let result = predictor.predict(&history);
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.method, PredictionMethod::LinearExtrapolation);
        assert!(result.confidence > 0.0 && result.confidence <= 1.0);
    }

    #[test]
    fn test_predict_ema() {
        let predictor = Quartz5DPredictor::new(PredictionConfig {
            use_trend: false,
            smoothing_factor: 0.5,
            ..Default::default()
        });

        let history = vec![
            Quartz5DCoordinate::new(0, 0, 0, 0, 0.0),
            Quartz5DCoordinate::new(10, 0, 0, 1, 1.0),
        ];

        let result = predictor.predict(&history);
        assert!(result.is_some());

        let result = result.unwrap();
        assert_eq!(result.method, PredictionMethod::ExponentialMovingAverage);
    }

    #[test]
    fn test_predict_insufficient_history() {
        let predictor = Quartz5DPredictor::default();
        let history = vec![Quartz5DCoordinate::new(0, 0, 0, 0, 0.0)];
        assert!(predictor.predict(&history).is_none());
    }

    #[test]
    fn test_compute_velocity() {
        let predictor = Quartz5DPredictor::default();

        let history = vec![
            Quartz5DCoordinate::new(0, 0, 0, 0, 0.0),
            Quartz5DCoordinate::new(5, 0, 0, 1, 0.5),
        ];

        let velocity = predictor.compute_velocity(&history);
        assert!(velocity.is_some());

        let v = velocity.unwrap();
        assert!((v.vx - 5.0).abs() < 1e-10);
        assert!((v.vs - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_velocity_speed() {
        let v = VelocityVector {
            vx: 3.0,
            vy: 4.0,
            vz: 0.0,
            vs: 0.0,
        };
        assert!((v.spatial_speed() - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_prediction_confidence() {
        let predictor = Quartz5DPredictor::default();

        let history = vec![
            Quartz5DCoordinate::new(0, 0, 0, 0, 0.0),
            Quartz5DCoordinate::new(1, 0, 0, 1, 0.1),
        ];

        let result = predictor.predict(&history).unwrap();
        assert!(result.confidence >= 0.0 && result.confidence <= 1.0);
    }
}
