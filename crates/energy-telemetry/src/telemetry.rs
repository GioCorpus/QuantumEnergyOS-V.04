use crate::ring_buffer::{LockFreeSpscRingBuffer, TelemetrySample, SampleType};
use serde::{Deserialize, Serialize};

/// Configuration for the telemetry service.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryConfig {
    /// Buffer capacity for each sample type (must be power of 2).
    pub buffer_capacity: usize,
    /// Sampling interval in microseconds.
    pub sampling_interval_us: u64,
    /// Enable overflow detection and reporting.
    pub overflow_detection: bool,
}

impl Default for TelemetryConfig {
    fn default() -> Self {
        Self {
            buffer_capacity: 1024,
            sampling_interval_us: 1000,
            overflow_detection: true,
        }
    }
}

/// Telemetry service for collecting and buffering system measurements.
///
/// This service manages ring buffers for different sample types,
/// providing lock-free SPSC communication between data producers
/// (e.g., sensor interrupt handlers) and consumers (e.g., dashboard).
pub struct TelemetryService {
    config: TelemetryConfig,
    cpu_temp_buffer: LockFreeSpscRingBuffer<TelemetrySample, 1024>,
    gpu_temp_buffer: LockFreeSpscRingBuffer<TelemetrySample, 1024>,
    cpu_power_buffer: LockFreeSpscRingBuffer<TelemetrySample, 1024>,
    gpu_power_buffer: LockFreeSpscRingBuffer<TelemetrySample, 1024>,
    fan_speed_buffer: LockFreeSpscRingBuffer<TelemetrySample, 256>,
}

impl TelemetryService {
    pub fn new(config: TelemetryConfig) -> Self {
        Self {
            config,
            cpu_temp_buffer: LockFreeSpscRingBuffer::new(),
            gpu_temp_buffer: LockFreeSpscRingBuffer::new(),
            cpu_power_buffer: LockFreeSpscRingBuffer::new(),
            gpu_power_buffer: LockFreeSpscRingBuffer::new(),
            fan_speed_buffer: LockFreeSpscRingBuffer::new(),
        }
    }

    /// Record a CPU temperature sample.
    pub fn record_cpu_temp(&mut self, value: f64, timestamp_ns: u64) -> bool {
        let sample = TelemetrySample::new(SampleType::CpuTemperature, value)
            .with_timestamp(timestamp_ns);
        self.cpu_temp_buffer.push(sample).is_ok()
    }

    /// Record a GPU temperature sample.
    pub fn record_gpu_temp(&mut self, value: f64, timestamp_ns: u64) -> bool {
        let sample = TelemetrySample::new(SampleType::GpuTemperature, value)
            .with_timestamp(timestamp_ns);
        self.gpu_temp_buffer.push(sample).is_ok()
    }

    /// Record a CPU power sample.
    pub fn record_cpu_power(&mut self, value: f64, timestamp_ns: u64) -> bool {
        let sample = TelemetrySample::new(SampleType::CpuPower, value)
            .with_timestamp(timestamp_ns);
        self.cpu_power_buffer.push(sample).is_ok()
    }

    /// Record a GPU power sample.
    pub fn record_gpu_power(&mut self, value: f64, timestamp_ns: u64) -> bool {
        let sample = TelemetrySample::new(SampleType::GpuPower, value)
            .with_timestamp(timestamp_ns);
        self.gpu_power_buffer.push(sample).is_ok()
    }

    /// Record a fan speed sample.
    pub fn record_fan_speed(&mut self, value: f64, timestamp_ns: u64) -> bool {
        let sample = TelemetrySample::new(SampleType::FanSpeed, value)
            .with_timestamp(timestamp_ns);
        self.fan_speed_buffer.push(sample).is_ok()
    }

    /// Read the next CPU temperature sample.
    pub fn read_cpu_temp(&mut self) -> Option<TelemetrySample> {
        self.cpu_temp_buffer.pop()
    }

    /// Read the next GPU temperature sample.
    pub fn read_gpu_temp(&mut self) -> Option<TelemetrySample> {
        self.gpu_temp_buffer.pop()
    }

    /// Read the next CPU power sample.
    pub fn read_cpu_power(&mut self) -> Option<TelemetrySample> {
        self.cpu_power_buffer.pop()
    }

    /// Get statistics for a specific buffer.
    pub fn buffer_stats(&self, sample_type: SampleType) -> BufferStats {
        match sample_type {
            SampleType::CpuTemperature => BufferStats {
                sample_type,
                dropped: self.cpu_temp_buffer.dropped_count(),
                total_written: self.cpu_temp_buffer.total_written(),
                total_read: self.cpu_temp_buffer.total_read(),
            },
            SampleType::GpuTemperature => BufferStats {
                sample_type,
                dropped: self.gpu_temp_buffer.dropped_count(),
                total_written: self.gpu_temp_buffer.total_written(),
                total_read: self.gpu_temp_buffer.total_read(),
            },
            SampleType::CpuPower => BufferStats {
                sample_type,
                dropped: self.cpu_power_buffer.dropped_count(),
                total_written: self.cpu_power_buffer.total_written(),
                total_read: self.cpu_power_buffer.total_read(),
            },
            SampleType::GpuPower => BufferStats {
                sample_type,
                dropped: self.gpu_power_buffer.dropped_count(),
                total_written: self.gpu_power_buffer.total_written(),
                total_read: self.gpu_power_buffer.total_read(),
            },
            SampleType::FanSpeed => BufferStats {
                sample_type,
                dropped: self.fan_speed_buffer.dropped_count(),
                total_written: self.fan_speed_buffer.total_written(),
                total_read: self.fan_speed_buffer.total_read(),
            },
            _ => BufferStats {
                sample_type,
                dropped: 0,
                total_written: 0,
                total_read: 0,
            },
        }
    }

    /// Reset all buffers.
    pub fn reset(&mut self) {
        self.cpu_temp_buffer.clear();
        self.gpu_temp_buffer.clear();
        self.cpu_power_buffer.clear();
        self.gpu_power_buffer.clear();
        self.fan_speed_buffer.clear();
    }
}

/// Statistics for a telemetry buffer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BufferStats {
    pub sample_type: SampleType,
    pub dropped: usize,
    pub total_written: usize,
    pub total_read: usize,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_service_creation() {
        let config = TelemetryConfig::default();
        let service = TelemetryService::new(config);
        assert_eq!(service.config.buffer_capacity, 1024);
    }

    #[test]
    fn test_record_and_read() {
        let mut service = TelemetryService::new(TelemetryConfig::default());

        assert!(service.record_cpu_temp(65.0, 1000));
        let sample = service.read_cpu_temp();
        assert!(sample.is_some());
        assert!((sample.unwrap().value - 65.0).abs() < 1e-10);
    }

    #[test]
    fn test_buffer_stats() {
        let mut service = TelemetryService::new(TelemetryConfig::default());

        service.record_cpu_power(45.0, 1000);
        service.record_cpu_power(46.0, 2000);

        let stats = service.buffer_stats(SampleType::CpuPower);
        assert_eq!(stats.total_written, 2);
        assert_eq!(stats.sample_type, SampleType::CpuPower);
    }

    #[test]
    fn test_reset() {
        let mut service = TelemetryService::new(TelemetryConfig::default());

        service.record_gpu_temp(70.0, 1000);
        service.reset();

        assert!(service.read_gpu_temp().is_none());
    }
}
