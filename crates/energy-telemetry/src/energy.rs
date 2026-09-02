use crate::ring_buffer::{EnergySample, LockFreeSpscRingBuffer};
use serde::{Deserialize, Serialize};

/// Configuration for the energy monitoring service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyConfig {
    /// Buffer capacity for energy samples (must be power of 2).
    pub buffer_capacity: usize,
    /// Sampling interval in microseconds.
    pub sampling_interval_us: u64,
    /// Voltage threshold for anomaly detection (volts).
    pub voltage_threshold: f64,
    /// Current threshold for anomaly detection (amps).
    pub current_threshold: f64,
    /// Power threshold for anomaly detection (watts).
    pub power_threshold: f64,
}

impl Default for EnergyConfig {
    fn default() -> Self {
        Self {
            buffer_capacity: 512,
            sampling_interval_us: 100,
            voltage_threshold: 13.0,
            current_threshold: 30.0,
            power_threshold: 350.0,
        }
    }
}

/// Energy monitoring service.
///
/// Collects voltage, current, and power measurements from sensors
/// and provides anomaly detection based on configurable thresholds.
pub struct EnergyService {
    config: EnergyConfig,
    buffer: LockFreeSpscRingBuffer<EnergySample, 512>,
    total_energy_joules: f64,
    sample_count: u64,
}

impl EnergyService {
    pub fn new(config: EnergyConfig) -> Self {
        Self {
            config,
            buffer: LockFreeSpscRingBuffer::new(),
            total_energy_joules: 0.0,
            sample_count: 0,
        }
    }

    /// Record an energy sample.
    pub fn record_sample(&mut self, voltage: f64, current: f64, timestamp_ns: u64) -> bool {
        let sample = EnergySample::new(voltage, current).with_timestamp(timestamp_ns);

        // Accumulate energy (power * time interval)
        let interval_seconds = self.config.sampling_interval_us as f64 / 1_000_000.0;
        self.total_energy_joules += sample.power * interval_seconds;
        self.sample_count += 1;

        self.buffer.push(sample).is_ok()
    }

    /// Read the next energy sample.
    pub fn read_sample(&mut self) -> Option<EnergySample> {
        self.buffer.pop()
    }

    /// Check if a sample indicates an anomaly.
    pub fn is_anomaly(&self, sample: &EnergySample) -> bool {
        sample.voltage > self.config.voltage_threshold
            || sample.current > self.config.current_threshold
            || sample.power > self.config.power_threshold
    }

    /// Get the average power over all recorded samples.
    pub fn average_power(&self) -> f64 {
        if self.sample_count == 0 {
            return 0.0;
        }
        self.total_energy_joules / (self.sample_count as f64 * self.config.sampling_interval_us as f64 / 1_000_000.0)
    }

    /// Get total energy consumed in joules.
    pub fn total_energy_joules(&self) -> f64 {
        self.total_energy_joules
    }

    /// Get total number of samples recorded.
    pub fn sample_count(&self) -> u64 {
        self.sample_count
    }

    /// Get the number of dropped samples.
    pub fn dropped_samples(&self) -> usize {
        self.buffer.dropped_count()
    }

    /// Reset the service state.
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.total_energy_joules = 0.0;
        self.sample_count = 0;
    }
}

/// Energy forecast result.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnergyForecast {
    /// Predicted average power for the next interval (watts).
    pub predicted_power: f64,
    /// Predicted total energy for the next interval (joules).
    pub predicted_energy: f64,
    /// Confidence level (0.0 - 1.0).
    pub confidence: f64,
}

/// Simple moving average forecast.
pub fn forecast_energy(samples: &[EnergySample], interval_seconds: f64) -> Option<EnergyForecast> {
    if samples.is_empty() {
        return None;
    }

    let avg_power: f64 = samples.iter().map(|s| s.power).sum::<f64>() / samples.len() as f64;
    let predicted_energy = avg_power * interval_seconds;

    Some(EnergyForecast {
        predicted_power: avg_power,
        predicted_energy,
        confidence: 0.8,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_energy_service_creation() {
        let config = EnergyConfig::default();
        let service = EnergyService::new(config);
        assert_eq!(service.sample_count(), 0);
        assert!((service.total_energy_joules()).abs() < 1e-10);
    }

    #[test]
    fn test_record_sample() {
        let mut service = EnergyService::new(EnergyConfig::default());

        assert!(service.record_sample(12.0, 2.0, 1000));
        assert_eq!(service.sample_count(), 1);

        let sample = service.read_sample();
        assert!(sample.is_some());
        assert!((sample.unwrap().power - 24.0).abs() < 1e-10);
    }

    #[test]
    fn test_anomaly_detection() {
        let config = EnergyConfig::default();
        let service = EnergyService::new(config);

        let normal_sample = EnergySample::new(12.0, 2.0);
        let anomaly_sample = EnergySample::new(15.0, 40.0);

        assert!(!service.is_anomaly(&normal_sample));
        assert!(service.is_anomaly(&anomaly_sample));
    }

    #[test]
    fn test_energy_accumulation() {
        let mut service = EnergyService::new(EnergyConfig::default());

        service.record_sample(12.0, 2.0, 1000);
        service.record_sample(12.0, 2.0, 2000);

        assert!(service.total_energy_joules() > 0.0);
    }

    #[test]
    fn test_forecast() {
        let samples = vec![
            EnergySample::new(12.0, 2.0),
            EnergySample::new(12.0, 2.5),
            EnergySample::new(12.0, 3.0),
        ];

        let forecast = forecast_energy(&samples, 60.0);
        assert!(forecast.is_some());

        let forecast = forecast.unwrap();
        assert!(forecast.predicted_power > 0.0);
        assert!(forecast.predicted_energy > 0.0);
    }

    #[test]
    fn test_forecast_empty() {
        let samples: Vec<EnergySample> = vec![];
        assert!(forecast_energy(&samples, 60.0).is_none());
    }

    #[test]
    fn test_reset() {
        let mut service = EnergyService::new(EnergyConfig::default());

        service.record_sample(12.0, 2.0, 1000);
        service.reset();

        assert_eq!(service.sample_count(), 0);
        assert!((service.total_energy_joules()).abs() < 1e-10);
    }
}
