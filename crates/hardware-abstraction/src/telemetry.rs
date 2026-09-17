use serde::{Deserialize, Serialize};

use crate::device::{DeviceHealth, DeviceInfo, HardwareDevice};
use crate::error::HardwareError;

/// Telemetry hardware device information.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TelemetryInfo {
    pub sensor_count: u32,
    pub sampling_rate_hz: u32,
    pub supported_sensors: Vec<SensorType>,
}

/// Types of sensors supported by telemetry hardware.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SensorType {
    Temperature,
    Voltage,
    Current,
    Power,
    Fan,
    Frequency,
    Humidity,
    Pressure,
}

/// Telemetry hardware device abstraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TelemetryHardware {
    info: DeviceInfo,
    telemetry_info: TelemetryInfo,
    state: crate::device::DeviceState,
}

impl TelemetryHardware {
    pub fn new(info: DeviceInfo, telemetry_info: TelemetryInfo) -> Self {
        Self {
            info,
            telemetry_info,
            state: crate::device::DeviceState::Uninitialized,
        }
    }

    /// Get telemetry-specific information.
    pub fn telemetry_info(&self) -> &TelemetryInfo {
        &self.telemetry_info
    }

    /// Get the number of sensors.
    pub fn sensor_count(&self) -> u32 {
        self.telemetry_info.sensor_count
    }

    /// Get the sampling rate.
    pub fn sampling_rate(&self) -> u32 {
        self.telemetry_info.sampling_rate_hz
    }

    /// Check if a sensor type is supported.
    pub fn supports_sensor(&self, sensor_type: SensorType) -> bool {
        self.telemetry_info.supported_sensors.contains(&sensor_type)
    }

    /// Get the list of supported sensors.
    pub fn supported_sensors(&self) -> &[SensorType] {
        &self.telemetry_info.supported_sensors
    }
}

impl HardwareDevice for TelemetryHardware {
    fn identify(&self) -> DeviceInfo {
        self.info.clone()
    }

    fn initialize(&mut self) -> Result<(), HardwareError> {
        if self.state == crate::device::DeviceState::Ready {
            return Ok(());
        }
        self.state = crate::device::DeviceState::Ready;
        Ok(())
    }

    fn health(&self) -> DeviceHealth {
        match self.state {
            crate::device::DeviceState::Ready => DeviceHealth::Healthy,
            crate::device::DeviceState::Error => DeviceHealth::Error,
            _ => DeviceHealth::Unknown,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_telemetry_device_creation() {
        let info = DeviceInfo {
            id: "telemetry0".to_string(),
            name: "System Telemetry".to_string(),
            vendor: "TestVendor".to_string(),
            model: "TELE100".to_string(),
            serial_number: None,
            firmware_version: None,
        };

        let telemetry_info = TelemetryInfo {
            sensor_count: 8,
            sampling_rate_hz: 10,
            supported_sensors: vec![
                SensorType::Temperature,
                SensorType::Voltage,
                SensorType::Current,
                SensorType::Power,
                SensorType::Fan,
            ],
        };

        let device = TelemetryHardware::new(info, telemetry_info);
        assert_eq!(device.sensor_count(), 8);
        assert_eq!(device.sampling_rate(), 10);
    }

    #[test]
    fn test_sensor_support() {
        let info = DeviceInfo::default();
        let telemetry_info = TelemetryInfo {
            supported_sensors: vec![SensorType::Temperature, SensorType::Power],
            ..Default::default()
        };

        let device = TelemetryHardware::new(info, telemetry_info);
        assert!(device.supports_sensor(SensorType::Temperature));
        assert!(device.supports_sensor(SensorType::Power));
        assert!(!device.supports_sensor(SensorType::Fan));
    }

    #[test]
    fn test_telemetry_initialization() {
        let info = DeviceInfo::default();
        let telemetry_info = TelemetryInfo::default();
        let mut device = TelemetryHardware::new(info, telemetry_info);

        assert!(device.initialize().is_ok());
        assert_eq!(device.health(), DeviceHealth::Healthy);
    }
}
