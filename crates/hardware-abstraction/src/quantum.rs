use serde::{Deserialize, Serialize};

use crate::device::{DeviceHealth, DeviceInfo, HardwareDevice};
use crate::error::HardwareError;

/// Quantum device information.
///
/// This structure represents metadata about a quantum processing unit (QPU)
/// or quantum simulator accessible through the hardware abstraction layer.
///
/// IMPORTANT: No physical quantum hardware access is assumed. This is a
/// capability-gated adapter that requires documented hardware interfaces.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuantumDeviceInfo {
    pub qubits: usize,
    pub logical_qubits: usize,
    pub supported_gates: Vec<String>,
    pub backend_type: QuantumBackendType,
    pub is_simulator: bool,
}

/// Types of quantum backends.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum QuantumBackendType {
    #[default]
    Simulator,
    Emulator,
    Remote,
    Physical,
}

/// Quantum device health monitoring.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuantumDeviceHealth {
    pub temperature_mk: Option<f64>,
    pub calibration_drift: f64,
    pub error_rate: f64,
    pub uptime_seconds: u64,
    pub jobs_completed: u64,
    pub jobs_failed: u64,
}

/// Quantum device abstraction.
///
/// CLASSIFICATION: HARDWARE ABSTRACTION (capability-gated)
///
/// This adapter provides a uniform interface for quantum devices.
/// Physical QPU integration is only available when documented hardware
/// interfaces exist. Otherwise, only simulators and emulators are functional.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuantumHardware {
    info: DeviceInfo,
    quantum_info: QuantumDeviceInfo,
    health: QuantumDeviceHealth,
    state: crate::device::DeviceState,
    enabled: bool,
}

impl QuantumHardware {
    pub fn new(info: DeviceInfo, quantum_info: QuantumDeviceInfo) -> Self {
        let enabled =
            quantum_info.is_simulator || quantum_info.backend_type != QuantumBackendType::Physical;
        Self {
            info,
            quantum_info,
            health: QuantumDeviceHealth::default(),
            state: crate::device::DeviceState::Uninitialized,
            enabled,
        }
    }

    /// Get quantum-specific information.
    pub fn quantum_info(&self) -> &QuantumDeviceInfo {
        &self.quantum_info
    }

    /// Get quantum health data.
    pub fn quantum_health(&self) -> &QuantumDeviceHealth {
        &self.health
    }

    /// Update quantum health data.
    pub fn update_health(&mut self, health: QuantumDeviceHealth) {
        self.health = health;
    }

    /// Get the number of qubits.
    pub fn qubits(&self) -> usize {
        self.quantum_info.qubits
    }

    /// Get the number of logical qubits.
    pub fn logical_qubits(&self) -> usize {
        self.quantum_info.logical_qubits
    }

    /// Check if the device is enabled.
    pub fn is_enabled(&self) -> bool {
        self.enabled
    }

    /// Check if this is a simulator.
    pub fn is_simulator(&self) -> bool {
        self.quantum_info.is_simulator
    }

    /// Get the backend type.
    pub fn backend_type(&self) -> QuantumBackendType {
        self.quantum_info.backend_type
    }

    /// Get the error rate.
    pub fn error_rate(&self) -> f64 {
        self.health.error_rate
    }

    /// Enable the device (only valid for documented hardware interfaces).
    pub fn enable(&mut self) {
        if self.quantum_info.backend_type != QuantumBackendType::Physical {
            self.enabled = true;
        }
    }

    /// Enable physical backend (requires documented interface).
    pub fn enable_physical(&mut self) {
        self.enabled = true;
    }

    /// Disable the device.
    pub fn disable(&mut self) {
        self.enabled = false;
    }

    /// Check if a gate is supported.
    pub fn supports_gate(&self, gate: &str) -> bool {
        self.quantum_info
            .supported_gates
            .contains(&gate.to_string())
    }
}

impl HardwareDevice for QuantumHardware {
    fn identify(&self) -> DeviceInfo {
        self.info.clone()
    }

    fn initialize(&mut self) -> Result<(), HardwareError> {
        if !self.enabled {
            return Err(HardwareError::InitializationFailed(
                "quantum device is not enabled".to_string(),
            ));
        }
        self.state = crate::device::DeviceState::Ready;
        Ok(())
    }

    fn health(&self) -> DeviceHealth {
        if !self.enabled {
            return DeviceHealth::Unknown;
        }
        if self.health.error_rate > 0.1 {
            DeviceHealth::Warning
        } else if self.health.calibration_drift > 0.5 {
            DeviceHealth::Degraded
        } else {
            DeviceHealth::Healthy
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_quantum_device_creation() {
        let info = DeviceInfo {
            id: "qpu0".to_string(),
            name: "Quantum Simulator".to_string(),
            vendor: "QuantumEnergyOS".to_string(),
            model: "Sim100".to_string(),
            serial_number: None,
            firmware_version: None,
        };

        let quantum_info = QuantumDeviceInfo {
            qubits: 20,
            logical_qubits: 16,
            supported_gates: vec!["H".to_string(), "CNOT".to_string(), "Measure".to_string()],
            backend_type: QuantumBackendType::Simulator,
            is_simulator: true,
        };

        let device = QuantumHardware::new(info, quantum_info);
        assert_eq!(device.qubits(), 20);
        assert!(device.is_simulator());
        assert!(device.is_enabled());
    }

    #[test]
    fn test_quantum_device_physical_disabled() {
        let info = DeviceInfo::default();
        let quantum_info = QuantumDeviceInfo {
            backend_type: QuantumBackendType::Physical,
            ..Default::default()
        };

        let device = QuantumHardware::new(info, quantum_info);
        assert!(!device.is_enabled());
        assert!(!device.is_simulator());
    }

    #[test]
    fn test_quantum_enable_physical() {
        let info = DeviceInfo::default();
        let quantum_info = QuantumDeviceInfo {
            backend_type: QuantumBackendType::Physical,
            ..Default::default()
        };

        let mut device = QuantumHardware::new(info, quantum_info);
        assert!(!device.is_enabled());

        device.enable_physical();
        assert!(device.is_enabled());
    }

    #[test]
    fn test_quantum_gate_support() {
        let info = DeviceInfo::default();
        let quantum_info = QuantumDeviceInfo {
            supported_gates: vec!["H".to_string(), "CNOT".to_string()],
            ..Default::default()
        };

        let device = QuantumHardware::new(info, quantum_info);
        assert!(device.supports_gate("H"));
        assert!(!device.supports_gate("Toffoli"));
    }

    #[test]
    fn test_quantum_initialization() {
        let info = DeviceInfo::default();
        let quantum_info = QuantumDeviceInfo {
            backend_type: QuantumBackendType::Simulator,
            is_simulator: true,
            ..Default::default()
        };

        let mut device = QuantumHardware::new(info, quantum_info);
        assert!(device.initialize().is_ok());
    }

    #[test]
    fn test_quantum_initialization_disabled() {
        let info = DeviceInfo::default();
        let quantum_info = QuantumDeviceInfo {
            backend_type: QuantumBackendType::Physical,
            ..Default::default()
        };

        let mut device = QuantumHardware::new(info, quantum_info);
        assert!(device.initialize().is_err());
    }

    #[test]
    fn test_quantum_health_status() {
        let info = DeviceInfo::default();
        let quantum_info = QuantumDeviceInfo {
            backend_type: QuantumBackendType::Simulator,
            is_simulator: true,
            ..Default::default()
        };

        let mut device = QuantumHardware::new(info, quantum_info);

        device.update_health(QuantumDeviceHealth {
            error_rate: 0.05,
            ..Default::default()
        });

        assert_eq!(device.health(), DeviceHealth::Healthy);
        assert!((device.error_rate() - 0.05).abs() < 1e-10);
    }
}
