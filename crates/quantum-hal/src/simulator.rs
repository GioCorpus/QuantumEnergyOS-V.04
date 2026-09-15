//! Reference simulator device: the SIMULATED backend of the Quantum HAL.
//!
//! CLASSIFICATION: SIMULATED (REAL as software)
//!
//! This device executes the device-level IR on the state-vector engine that
//! lives in `crates/quantum-runtime`. It is a classical simulation:
//!
//! - it requires no hardware and runs in CI;
//! - measurement outcomes are sampled from the state-vector distribution;
//! - results are reproducible when `SimulationConfig::seed` is set;
//! - the configured noise model is a CLASSICAL bit-flip model, not device noise.
//!
//! The module also exposes `execute_reference`, the single shared reference
//! execution path used by the accelerator and Majorana devices. Sharing one
//! implementation avoids divergent "simulation physics" across backends.

use std::collections::HashMap;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use quantum_runtime::{QuantumGate, QuantumSimulator};

use crate::device::{BackendClass, DeviceHealth, DeviceInfo, DeviceState, QuantumDevice};
use crate::error::{QuantumError, Result};
use crate::ir::{QuantumIR, QuantumOperation};
use crate::job::{JobHandle, JobStatus, QuantumJob, QuantumResult, SimulationConfig};

/// Bookkeeping for one submitted job.
#[derive(Debug, Clone)]
struct JobRecord {
    status: JobStatus,
    result: Option<QuantumResult>,
    error: Option<String>,
}

/// Reference state-vector simulator device.
///
/// Satisfies the `QuantumDevice` contract without any hardware dependency.
#[derive(Debug)]
pub struct SimulatorDevice {
    info: DeviceInfo,
    state: DeviceState,
    health: DeviceHealth,
    max_qubits: usize,
    calibrated: bool,
    next_handle: u64,
    jobs: HashMap<u64, JobRecord>,
    jobs_completed: u64,
    jobs_failed: u64,
    calibrations: u64,
}

impl SimulatorDevice {
    /// Maximum register size of the reference engine.
    pub const DEFAULT_MAX_QUBITS: usize = 20;

    /// Create an uninitialised reference simulator device.
    pub fn new() -> Self {
        Self {
            info: DeviceInfo {
                id: "sim0".to_string(),
                name: "QuantumEnergyOS Reference Simulator".to_string(),
                vendor: "QuantumEnergyOS".to_string(),
                model: "state-vector-reference".to_string(),
                class: BackendClass::Simulator,
                qubits: Self::DEFAULT_MAX_QUBITS,
                logical_qubits: Self::DEFAULT_MAX_QUBITS,
                supported_gates: vec![
                    "H".to_string(),
                    "X".to_string(),
                    "Y".to_string(),
                    "Z".to_string(),
                    "CNOT".to_string(),
                    "Measure".to_string(),
                ],
                firmware_version: Some(env!("CARGO_PKG_VERSION").to_string()),
                simulation_only: true,
            },
            state: DeviceState::Uninitialized,
            health: DeviceHealth::Unknown,
            max_qubits: Self::DEFAULT_MAX_QUBITS,
            calibrated: false,
            next_handle: 1,
            jobs: HashMap::new(),
            jobs_completed: 0,
            jobs_failed: 0,
            calibrations: 0,
        }
    }

    /// Constrain the register size (bounded by the engine limit).
    pub fn with_max_qubits(mut self, max_qubits: usize) -> Self {
        let bounded = max_qubits.clamp(1, Self::DEFAULT_MAX_QUBITS);
        self.max_qubits = bounded;
        self.info.qubits = bounded;
        self.info.logical_qubits = bounded;
        self
    }

    /// True after a successful `calibrate()` call.
    pub fn is_calibrated(&self) -> bool {
        self.calibrated
    }

    /// Number of completed jobs.
    pub fn jobs_completed(&self) -> u64 {
        self.jobs_completed
    }

    /// Number of failed jobs.
    pub fn jobs_failed(&self) -> u64 {
        self.jobs_failed
    }

    /// Number of calibration events recorded.
    pub fn calibrations(&self) -> u64 {
        self.calibrations
    }

    /// Current lifecycle state.
    pub fn device_state(&self) -> DeviceState {
        self.state
    }

    /// Attach a classification note to a stored result.
    ///
    /// Used by wrapper devices (accelerator, Majorana) that delegate execution
    /// to this engine and must disclose that no vendor hardware was used.
    /// Returns `true` when a stored result was annotated.
    pub(crate) fn annotate(&mut self, handle: JobHandle, note: &str) -> bool {
        match self.jobs.get_mut(&handle.0) {
            Some(record) => match &mut record.result {
                Some(result) => {
                    result.push_note(note.to_string());
                    true
                }
                None => false,
            },
            None => false,
        }
    }
}

impl Default for SimulatorDevice {
    fn default() -> Self {
        Self::new()
    }
}

/// Apply every non-measurement operation to the state vector.
///
/// Operations outside the minimal device op set are rejected explicitly: the
/// reference device never approximates a gate it does not implement.
fn apply_operations(simulator: &mut QuantumSimulator, ir: &QuantumIR) -> Result<()> {
    for operation in &ir.operations {
        match operation {
            QuantumOperation::H(q) => {
                simulator.apply_single_qubit_gate(&QuantumGate::Hadamard, *q as usize)?
            }
            QuantumOperation::X(q) => {
                simulator.apply_single_qubit_gate(&QuantumGate::PauliX, *q as usize)?
            }
            QuantumOperation::Y(q) => {
                simulator.apply_single_qubit_gate(&QuantumGate::PauliY, *q as usize)?
            }
            QuantumOperation::Z(q) => {
                simulator.apply_single_qubit_gate(&QuantumGate::PauliZ, *q as usize)?
            }
            QuantumOperation::CNOT(control, target) => {
                simulator.apply_cnot(*control as usize, *target as usize)?
            }
            QuantumOperation::Measure(_) => {
                // Measurement operations are resolved during sampling so that
                // shot statistics come from the prepared state.
            }
            QuantumOperation::Reset(_) => {
                return Err(QuantumError::UnsupportedGate(
                    "Reset is not implemented by the reference simulator device; use QuantumDevice::reset() to reinitialise the register".to_string(),
                ));
            }
            QuantumOperation::Custom(label) => {
                return Err(QuantumError::UnsupportedGate(format!(
                    "operation '{}' is not implemented by the reference simulator device",
                    label
                )));
            }
        }
    }
    Ok(())
}

/// Sample a computational-basis index from a probability vector.
fn sample_index(probabilities: &[f64], rng: &mut impl Rng) -> usize {
    let total: f64 = probabilities.iter().sum();
    if !(total > 0.0) {
        return 0;
    }

    let target = rng.gen::<f64>() * total;
    let mut accumulated = 0.0;
    for (index, probability) in probabilities.iter().enumerate() {
        accumulated += *probability;
        if target <= accumulated {
            return index;
        }
    }
    probabilities.len().saturating_sub(1)
}

/// Render a basis index as a bitstring (leftmost character = highest qubit).
fn bitstring(index: usize, num_qubits: usize) -> String {
    let width = num_qubits.max(1);
    format!("{:0width$b}", index, width = width)
}

/// Reference execution path shared by all simulated HAL backends.
///
/// The whole register is prepared, then every shot samples one basis index from
/// the state-vector distribution, which preserves multi-qubit correlations
/// (for example Bell-state outcomes are always `00` or `11`).
pub(crate) fn execute_reference(
    job_id: &str,
    backend: &str,
    ir: &QuantumIR,
    config: &SimulationConfig,
) -> Result<QuantumResult> {
    ir.validate()?;
    if ir.num_qubits > SimulatorDevice::DEFAULT_MAX_QUBITS {
        return Err(QuantumError::InvalidQubitCount {
            expected: SimulatorDevice::DEFAULT_MAX_QUBITS,
            got: ir.num_qubits,
        });
    }

    let mut simulator = QuantumSimulator::new(ir.num_qubits)?;
    apply_operations(&mut simulator, ir)?;

    let mut result = QuantumResult::new(job_id, backend, config.shots);
    result.status = JobStatus::Completed;
    result.probabilities = simulator.probabilities();
    result
        .notes
        .push("SIMULATED: classical state-vector execution; not physical hardware".to_string());
    result.push_note("model=state-vector-reference");

    if config.noise.is_ideal() {
        result.push_note("noise model: ideal (no decoherence simulated)");
    } else {
        result.push_note(format!(
            "noise model: classical bit-flip probability {:.6} (SIMULATED, not device noise)",
            config.noise.total_bit_flip_probability()
        ));
    }

    let measured = ir.measured_qubits();
    if measured.is_empty() {
        result.push_note("no measurement operations: state-vector probabilities only");
        return Ok(result);
    }

    if ir.has_mid_circuit_measurement() {
        result.push_note(
            "mid-circuit measurement detected: all measurements are sampled after state preparation; classical feed-forward is not modelled",
        );
    }

    let mut rng: StdRng = match config.seed {
        Some(seed) => StdRng::seed_from_u64(seed),
        None => StdRng::from_entropy(),
    };
    let bit_flip = config.noise.total_bit_flip_probability();
    let shots = config.shots as usize;
    result.bitstrings.reserve(shots);

    for _ in 0..shots {
        let index = sample_index(&result.probabilities, &mut rng);
        let mut bits: Vec<char> = bitstring(index, ir.num_qubits).chars().collect();

        if bit_flip > 0.0 {
            for qubit in &measured {
                let qubit_index = *qubit as usize;
                if qubit_index >= ir.num_qubits {
                    continue;
                }
                let position = ir.num_qubits - 1 - qubit_index;
                if let Some(bit) = bits.get_mut(position) {
                    if rng.gen_bool(bit_flip) {
                        *bit = if *bit == '0' { '1' } else { '0' };
                    }
                }
            }
        }

        let outcome: String = bits.into_iter().collect();
        *result.counts.entry(outcome.clone()).or_insert(0) += 1;
        result.bitstrings.push(outcome);
    }

    Ok(result)
}

impl QuantumDevice for SimulatorDevice {
    fn device_info(&self) -> DeviceInfo {
        self.info.clone()
    }

    fn initialize(&mut self) -> Result<()> {
        self.state = DeviceState::Ready;
        self.health = DeviceHealth::Healthy;
        tracing::info!(
            device = %self.info.id,
            "quantum-hal: reference simulator device initialised (SIMULATED)"
        );
        Ok(())
    }

    fn calibrate(&mut self) -> Result<()> {
        if self.state != DeviceState::Ready {
            return Err(QuantumError::DeviceNotInitialized(
                "call initialize() before calibrate()".to_string(),
            ));
        }
        self.calibrations += 1;
        self.calibrated = true;
        tracing::info!(
            device = %self.info.id,
            calibrations = self.calibrations,
            "quantum-hal: reference simulator calibration recorded (no physical calibration performed)"
        );
        Ok(())
    }

    fn reset(&mut self) -> Result<()> {
        if self.state == DeviceState::Uninitialized {
            return Err(QuantumError::DeviceNotInitialized(
                "reference simulator device was never initialised".to_string(),
            ));
        }
        self.jobs.clear();
        self.calibrated = false;
        self.state = DeviceState::Ready;
        self.health = DeviceHealth::Healthy;
        Ok(())
    }

    fn submit(&mut self, job: QuantumJob) -> Result<JobHandle> {
        if self.state != DeviceState::Ready {
            return Err(QuantumError::DeviceNotInitialized(format!(
                "device '{}' is in state {:?}; call initialize() first",
                self.info.id, self.state
            )));
        }

        if job.circuit.num_qubits > self.max_qubits {
            return Err(QuantumError::InvalidQubitCount {
                expected: self.max_qubits,
                got: job.circuit.num_qubits,
            });
        }

        let handle = JobHandle(self.next_handle);
        self.next_handle += 1;
        self.jobs.insert(
            handle.0,
            JobRecord {
                status: JobStatus::Running,
                result: None,
                error: None,
            },
        );

        match execute_reference(&job.job_id, &self.info.id, &job.circuit, &job.simulation) {
            Ok(result) => {
                self.jobs_completed += 1;
                self.jobs.insert(
                    handle.0,
                    JobRecord {
                        status: JobStatus::Completed,
                        result: Some(result),
                        error: None,
                    },
                );
            }
            Err(err) => {
                self.jobs_failed += 1;
                tracing::warn!(
                    device = %self.info.id,
                    job_id = %job.job_id,
                    error = %err,
                    "quantum-hal: reference simulator job failed"
                );
                self.jobs.insert(
                    handle.0,
                    JobRecord {
                        status: JobStatus::Failed,
                        result: None,
                        error: Some(err.to_string()),
                    },
                );
            }
        }

        Ok(handle)
    }

    fn poll(&mut self, job: JobHandle) -> Result<JobStatus> {
        self.jobs
            .get(&job.0)
            .map(|record| record.status)
            .ok_or_else(|| QuantumError::JobNotFound(format!("handle {}", job.0)))
    }

    fn read_result(&mut self, job: JobHandle) -> Result<QuantumResult> {
        let record = self
            .jobs
            .get(&job.0)
            .ok_or_else(|| QuantumError::JobNotFound(format!("handle {}", job.0)))?;

        match &record.result {
            Some(result) => Ok(result.clone()),
            None => match &record.error {
                Some(message) => Err(QuantumError::SimulatorError(message.clone())),
                None => Err(QuantumError::BackendNotAvailable(format!(
                    "job handle {} has no result yet (status {})",
                    job.0, record.status
                ))),
            },
        }
    }

    fn health(&self) -> DeviceHealth {
        match self.state {
            DeviceState::Ready => self.health,
            DeviceState::Uninitialized => DeviceHealth::Unknown,
            _ => DeviceHealth::Unavailable,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::job::NoiseModel;

    fn bell_ir() -> QuantumIR {
        let mut ir = QuantumIR::new("bell", 2);
        ir.push(QuantumOperation::H(0));
        ir.push(QuantumOperation::CNOT(0, 1));
        ir.push(QuantumOperation::Measure(0));
        ir.push(QuantumOperation::Measure(1));
        ir
    }

    fn ready_device() -> SimulatorDevice {
        let mut device = SimulatorDevice::new();
        device.initialize().unwrap();
        device
    }

    #[test]
    fn test_device_info_describes_simulation_honestly() {
        let device = SimulatorDevice::new();
        let info = device.device_info();
        assert_eq!(info.class, BackendClass::Simulator);
        assert!(info.simulation_only);
        assert_eq!(info.qubits, SimulatorDevice::DEFAULT_MAX_QUBITS);
        assert!(info.supports_gate("H"));
        assert_eq!(device.health(), DeviceHealth::Unknown);
        assert_eq!(device.device_state(), DeviceState::Uninitialized);
    }

    #[test]
    fn test_submit_requires_initialization() {
        let mut device = SimulatorDevice::new();
        let job = QuantumJob::new("sim0", bell_ir()).with_job_id("job-bell");
        assert!(matches!(
            device.submit(job),
            Err(QuantumError::DeviceNotInitialized(_))
        ));
    }

    #[test]
    fn test_calibrate_requires_initialization() {
        let mut device = SimulatorDevice::new();
        assert!(device.calibrate().is_err());

        let mut device = ready_device();
        device.calibrate().unwrap();
        assert!(device.is_calibrated());
        assert_eq!(device.calibrations(), 1);
    }

    #[test]
    fn test_seeded_bell_run_is_reproducible() {
        let mut first = ready_device();
        let mut second = ready_device();

        let build_job = || {
            QuantumJob::new("sim0", bell_ir())
                .with_job_id("job-bell")
                .with_shots(256)
                .with_seed(2026)
                .with_metadata("experiment", "bell-correlation")
        };

        let handle_a = first.submit(build_job()).unwrap();
        let handle_b = second.submit(build_job()).unwrap();

        assert_eq!(first.poll(handle_a).unwrap(), JobStatus::Completed);
        let result_a = first.read_result(handle_a).unwrap();
        let result_b = second.read_result(handle_b).unwrap();

        assert_eq!(result_a.status, JobStatus::Completed);
        assert_eq!(result_a.job_id, "job-bell");
        assert_eq!(result_a.bitstrings, result_b.bitstrings);
        assert_eq!(result_a.counts, result_b.counts);
        assert_eq!(result_a.total_counts(), 256);
        assert_eq!(result_a.shots, 256);
        assert!(result_a.simulation_only);
        assert!(result_a
            .notes
            .iter()
            .any(|note| note.contains("SIMULATED: classical state-vector execution")));

        for outcome in result_a.counts.keys() {
            assert!(
                outcome == "00" || outcome == "11",
                "unexpected Bell outcome {}",
                outcome
            );
        }
        assert_eq!(result_a.probabilities.len(), 4);
        assert!((result_a.probabilities[0] - 0.5).abs() < 1e-9);
        assert!((result_a.probabilities[3] - 0.5).abs() < 1e-9);
    }

    #[test]
    fn test_noise_model_is_reported_as_simulated() {
        let mut device = ready_device();
        let job = QuantumJob::new("sim0", bell_ir())
            .with_shots(64)
            .with_seed(5)
            .with_noise(NoiseModel::depolarizing(0.05));

        let handle = device.submit(job).unwrap();
        let result = device.read_result(handle).unwrap();
        assert!(result
            .notes
            .iter()
            .any(|note| note.contains("classical bit-flip probability")));
    }

    #[test]
    fn test_circuit_without_measurement_returns_probabilities_only() {
        let mut device = ready_device();
        let mut ir = QuantumIR::new("superposition", 1);
        ir.push(QuantumOperation::H(0));

        let handle = device
            .submit(QuantumJob::new("sim0", ir).with_shots(32).with_seed(1))
            .unwrap();
        let result = device.read_result(handle).unwrap();

        assert!(result.bitstrings.is_empty());
        assert!(result.counts.is_empty());
        assert_eq!(result.probabilities.len(), 2);
        assert!(result
            .notes
            .iter()
            .any(|note| note.contains("no measurement operations")));
    }

    #[test]
    fn test_unsupported_operation_fails_the_job() {
        let mut device = ready_device();

        let mut custom = QuantumIR::new("custom", 1);
        custom.push(QuantumOperation::Custom("ry(0; angle=0.5)".to_string()));
        let handle = device.submit(QuantumJob::new("sim0", custom)).unwrap();
        assert_eq!(device.poll(handle).unwrap(), JobStatus::Failed);
        assert!(matches!(
            device.read_result(handle),
            Err(QuantumError::SimulatorError(_))
        ));

        let mut reset = QuantumIR::new("reset", 1);
        reset.push(QuantumOperation::Reset(0));
        let handle = device.submit(QuantumJob::new("sim0", reset)).unwrap();
        assert_eq!(device.poll(handle).unwrap(), JobStatus::Failed);
        assert_eq!(device.jobs_failed(), 2);
        assert_eq!(device.jobs_completed(), 0);
    }

    #[test]
    fn test_unknown_handle_is_reported() {
        let mut device = ready_device();
        assert!(matches!(
            device.poll(JobHandle(999)),
            Err(QuantumError::JobNotFound(_))
        ));
        assert!(matches!(
            device.read_result(JobHandle(999)),
            Err(QuantumError::JobNotFound(_))
        ));
    }

    #[test]
    fn test_register_bound_is_enforced() {
        let mut device = ready_device().with_max_qubits(2);
        let mut ir = QuantumIR::new("three", 3);
        ir.push(QuantumOperation::H(0));

        assert!(matches!(
            device.submit(QuantumJob::new("sim0", ir)),
            Err(QuantumError::InvalidQubitCount { .. })
        ));
    }

    #[test]
    fn test_reset_clears_job_history_and_calibration() {
        let mut device = ready_device();
        device.calibrate().unwrap();
        let handle = device
            .submit(QuantumJob::new("sim0", bell_ir()).with_shots(8).with_seed(3))
            .unwrap();
        assert_eq!(device.poll(handle).unwrap(), JobStatus::Completed);

        device.reset().unwrap();
        assert!(!device.is_calibrated());
        assert!(matches!(device.poll(handle), Err(QuantumError::JobNotFound(_))));
        assert_eq!(device.health(), DeviceHealth::Healthy);
    }

    #[test]
    fn test_reset_before_initialization_is_rejected() {
        let mut device = SimulatorDevice::new();
        assert!(matches!(
            device.reset(),
            Err(QuantumError::DeviceNotInitialized(_))
        ));
    }

    #[test]
    fn test_execute_reference_rejects_oversized_register() {
        let mut ir = QuantumIR::new("wide", 32);
        ir.push(QuantumOperation::H(0));
        assert!(execute_reference("job", "sim0", &ir, &SimulationConfig::default()).is_err());
    }

    #[test]
    fn test_bitstring_uses_fixed_width() {
        assert_eq!(bitstring(0, 3), "000");
        assert_eq!(bitstring(5, 3), "101");
        assert_eq!(bitstring(1, 0), "1");
    }

    #[test]
    fn test_mid_circuit_measurement_is_disclosed() {
        let mut device = ready_device();
        let mut ir = QuantumIR::new("mid-circuit", 2);
        ir.push(QuantumOperation::H(0));
        ir.push(QuantumOperation::Measure(0));
        ir.push(QuantumOperation::X(1));
        ir.push(QuantumOperation::Measure(1));

        let handle = device
            .submit(QuantumJob::new("sim0", ir).with_shots(16).with_seed(11))
            .unwrap();
        let result = device.read_result(handle).unwrap();
        assert!(result
            .notes
            .iter()
            .any(|note| note.contains("mid-circuit measurement detected")));
    }
}