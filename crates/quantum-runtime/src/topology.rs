use serde::{Deserialize, Serialize};

/// Topological qubit model for QuantumEnergyOS.
///
/// CLASSIFICATION: MODEL
///
/// This module provides abstractions compatible with Majorana-based topological
/// quantum computing concepts. These are computational models, not claims of
/// physical hardware behavior.
///
/// Key concepts:
/// - Majorana Zero Modes (MZMs): Quasiparticles that may exhibit non-Abelian statistics
/// - Parity: The combined quantum state of paired Majorana modes
/// - Tetrons: Logical qubits encoded using multiple Majorana modes
/// - Braiding: Exchange operations that may implement quantum gates
/// - Topological protection: Error suppression through non-local encoding
// ---------------------------------------------------------------------------
// Majorana Zero Modes
// ---------------------------------------------------------------------------
///   Represents a Majorana Zero Mode (MZM) in the topological model.
///
/// In topological quantum computing theory, Majorana zero modes are
/// quasiparticles that may emerge at the ends of topological superconductors.
/// They are their own antiparticles and may exhibit non-Abelian statistics.
///
/// This is a MODEL abstraction. No physical hardware is accessed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MajoranaZeroMode {
    pub id: String,
    /// Parity state: +1 or -1 in the model
    pub parity: ParityState,
    /// Position in the lattice (if applicable)
    pub position: Option<LatticePosition>,
}

/// Parity state of a Majorana mode or pair.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ParityState {
    Even,
    Odd,
}

impl MajoranaZeroMode {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            parity: ParityState::Even,
            position: None,
        }
    }

    pub fn with_position(mut self, x: i32, y: i32) -> Self {
        self.position = Some(LatticePosition { x, y });
        self
    }

    /// Flip the parity state (model operation).
    pub fn flip_parity(&mut self) {
        self.parity = match self.parity {
            ParityState::Even => ParityState::Odd,
            ParityState::Odd => ParityState::Even,
        };
    }
}

// ---------------------------------------------------------------------------
// Parity
// ---------------------------------------------------------------------------

/// Parity measurement result.
///
/// In topological quantum computing, parity measurements are a primary
/// mechanism for reading out and manipulating quantum information without
/// directly measuring individual qubit states.
///
/// CLASSIFICATION: MODEL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParityMeasurement {
    /// IDs of the Majorana modes involved
    pub mode_ids: Vec<String>,
    /// Measured parity
    pub result: ParityState,
    /// Measurement confidence (model parameter)
    pub confidence: f64,
    /// Timestamp of measurement
    pub timestamp_ns: u64,
}

impl ParityMeasurement {
    pub fn new(mode_ids: Vec<String>, result: ParityState) -> Self {
        Self {
            mode_ids,
            result,
            confidence: 1.0,
            timestamp_ns: 0,
        }
    }

    pub fn with_confidence(mut self, confidence: f64) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }
}

/// Compute the combined parity of multiple Majorana modes.
pub fn compute_combined_parity(modes: &[MajoranaZeroMode]) -> ParityState {
    let odd_count = modes
        .iter()
        .filter(|m| m.parity == ParityState::Odd)
        .count();
    if odd_count % 2 == 0 {
        ParityState::Even
    } else {
        ParityState::Odd
    }
}

// ---------------------------------------------------------------------------
// Tetron
// ---------------------------------------------------------------------------

/// Tetron-like logical qubit structure.
///
/// A tetron is a theoretical topological qubit encoding that uses
/// four Majorana zero modes to encode one logical qubit.
/// The logical information is stored non-locally, providing
/// topological protection against local perturbations.
///
/// CLASSIFICATION: MODEL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TetronLikeLogicalQubit {
    pub id: String,
    pub majorana_modes: Vec<MajoranaZeroMode>,
    /// Whether topological protection is active in the model
    pub protected: bool,
    /// Logical qubit state in the model
    pub logical_state: LogicalQubitState,
}

/// Logical qubit state representation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogicalQubitState {
    Zero,
    One,
    Superposition,
    Unknown,
}

impl TetronLikeLogicalQubit {
    pub fn new(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            majorana_modes: Vec::new(),
            protected: false,
            logical_state: LogicalQubitState::Unknown,
        }
    }

    /// Add a Majorana mode to this tetron.
    pub fn add_majorana_mode(&mut self, mode: MajoranaZeroMode) {
        self.majorana_modes.push(mode);
    }

    /// Check if the tetron has the correct number of modes (4 for a standard tetron).
    pub fn is_complete(&self) -> bool {
        self.majorana_modes.len() == 4
    }

    /// Get the overall parity of the tetron.
    pub fn overall_parity(&self) -> ParityState {
        compute_combined_parity(&self.majorana_modes)
    }

    /// Enable topological protection in the model.
    pub fn enable_protection(&mut self) {
        self.protected = true;
    }

    /// Disable topological protection in the model.
    pub fn disable_protection(&mut self) {
        self.protected = false;
    }
}

// ---------------------------------------------------------------------------
// Braiding
// ---------------------------------------------------------------------------

/// Represents a braiding operation in the topological model.
///
/// In topological quantum computing, braiding refers to the exchange
/// of Majorana zero modes in space-time. These exchanges may implement
/// quantum gates through non-Abelian statistics.
///
/// CLASSIFICATION: MODEL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BraidingOperation {
    pub from: String,
    pub to: String,
    pub description: String,
    /// Braiding phase accumulated (model parameter)
    pub phase: f64,
}

impl BraidingOperation {
    pub fn new(from: impl Into<String>, to: impl Into<String>) -> Self {
        let from_str: String = from.into();
        let to_str: String = to.into();
        Self {
            from: from_str.clone(),
            to: to_str.clone(),
            description: format!("Braid {} -> {}", from_str, to_str),
            phase: 0.0,
        }
    }

    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    pub fn with_phase(mut self, phase: f64) -> Self {
        self.phase = phase;
        self
    }
}

/// Braiding sequence: an ordered set of braiding operations.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BraidingSequence {
    pub operations: Vec<BraidingOperation>,
}

impl BraidingSequence {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_operation(&mut self, op: BraidingOperation) {
        self.operations.push(op);
    }

    pub fn len(&self) -> usize {
        self.operations.len()
    }

    pub fn is_empty(&self) -> bool {
        self.operations.is_empty()
    }

    /// Compute total phase accumulated through the sequence.
    pub fn total_phase(&self) -> f64 {
        self.operations.iter().map(|op| op.phase).sum()
    }
}

// ---------------------------------------------------------------------------
// Lattice
// ---------------------------------------------------------------------------

/// Position in a 2D lattice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct LatticePosition {
    pub x: i32,
    pub y: i32,
}

/// Lattice configuration for topological qubit arrays.
///
/// CLASSIFICATION: MODEL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatticeConfig {
    pub width: u32,
    pub height: u32,
    /// Spacing between sites (model parameter)
    pub spacing: f64,
    /// Positions of Majorana modes in the lattice
    pub mode_positions: Vec<(String, LatticePosition)>,
}

impl LatticeConfig {
    pub fn new(width: u32, height: u32) -> Self {
        Self {
            width,
            height,
            spacing: 1.0,
            mode_positions: Vec::new(),
        }
    }

    pub fn with_spacing(mut self, spacing: f64) -> Self {
        self.spacing = spacing;
        self
    }

    /// Add a Majorana mode at a lattice position.
    pub fn add_mode(&mut self, id: impl Into<String>, pos: LatticePosition) {
        self.mode_positions.push((id.into(), pos));
    }

    /// Get the number of modes in the lattice.
    pub fn mode_count(&self) -> usize {
        self.mode_positions.len()
    }

    /// Check if a position is within the lattice bounds.
    pub fn in_bounds(&self, pos: LatticePosition) -> bool {
        pos.x >= 0 && pos.x < self.width as i32 && pos.y >= 0 && pos.y < self.height as i32
    }
}

// ---------------------------------------------------------------------------
// Error Model
// ---------------------------------------------------------------------------

/// Configuration for topological error models.
///
/// CLASSIFICATION: MODEL
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorModelConfig {
    pub model_name: String,
    /// Base error rate for non-topological operations
    pub base_error_rate: f64,
    /// Topological protection factor (reduction in error rate)
    pub protection_factor: f64,
    /// Strategy for error correction
    pub correction_strategy: CorrectionStrategy,
}

/// Error correction strategies for topological qubits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CorrectionStrategy {
    /// No error correction applied
    None,
    /// Surface code error correction
    SurfaceCode,
    /// Color code error correction
    ColorCode,
    /// Topological code specific to Majorana architectures
    TopologicalCode,
}

impl ErrorModelConfig {
    pub fn new(model_name: impl Into<String>) -> Self {
        Self {
            model_name: model_name.into(),
            base_error_rate: 0.001,
            protection_factor: 100.0,
            correction_strategy: CorrectionStrategy::TopologicalCode,
        }
    }

    pub fn with_error_rate(mut self, rate: f64) -> Self {
        self.base_error_rate = rate.clamp(0.0, 1.0);
        self
    }

    pub fn with_protection_factor(mut self, factor: f64) -> Self {
        self.protection_factor = factor.max(1.0);
        self
    }

    pub fn with_strategy(mut self, strategy: CorrectionStrategy) -> Self {
        self.correction_strategy = strategy;
        self
    }

    /// Compute the effective error rate given topological protection.
    pub fn effective_error_rate(&self) -> f64 {
        self.base_error_rate / self.protection_factor
    }
}

/// Topological error model for tracking error states.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct TopologicalErrorModel {
    pub config: Option<ErrorModelConfig>,
    pub detected_errors: Vec<DetectedError>,
}

/// A detected error in the topological model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectedError {
    pub error_type: ErrorType,
    pub affected_modes: Vec<String>,
    pub syndrome: String,
    pub timestamp_ns: u64,
}

/// Types of errors in the topological model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorType {
    /// Quasiparticle poisoning
    QuasiparticlePoisoning,
    /// Measurement error
    MeasurementError,
    /// Braiding error
    BraidingError,
    /// Decoherence
    Decoherence,
    /// Control error
    ControlError,
}

impl TopologicalErrorModel {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_config(mut self, config: ErrorModelConfig) -> Self {
        self.config = Some(config);
        self
    }

    /// Record a detected error.
    pub fn record_error(&mut self, error: DetectedError) {
        self.detected_errors.push(error);
    }

    /// Get the total number of detected errors.
    pub fn error_count(&self) -> usize {
        self.detected_errors.len()
    }

    /// Clear all recorded errors.
    pub fn clear_errors(&mut self) {
        self.detected_errors.clear();
    }
}

// ---------------------------------------------------------------------------
// Description
// ---------------------------------------------------------------------------

/// Return a description of the topological model classification.
pub fn describe_topology() -> &'static str {
    "Topological qubit model: Majorana-like logical degrees of freedom \
     represented as a computational model abstraction. This is NOT a claim \
     of physical hardware behavior. All operations are classical simulations \
     of theoretical concepts."
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_majorana_mode_creation() {
        let mode = MajoranaZeroMode::new("mzm-1");
        assert_eq!(mode.id, "mzm-1");
        assert_eq!(mode.parity, ParityState::Even);
        assert!(mode.position.is_none());
    }

    #[test]
    fn test_majorana_mode_with_position() {
        let mode = MajoranaZeroMode::new("mzm-1").with_position(3, 4);
        assert_eq!(mode.position, Some(LatticePosition { x: 3, y: 4 }));
    }

    #[test]
    fn test_parity_flip() {
        let mut mode = MajoranaZeroMode::new("mzm-1");
        assert_eq!(mode.parity, ParityState::Even);
        mode.flip_parity();
        assert_eq!(mode.parity, ParityState::Odd);
        mode.flip_parity();
        assert_eq!(mode.parity, ParityState::Even);
    }

    #[test]
    fn test_combined_parity() {
        let mut mode1 = MajoranaZeroMode::new("mzm-1");
        let mut mode2 = MajoranaZeroMode::new("mzm-2");
        let mut mode3 = MajoranaZeroMode::new("mzm-3");

        assert_eq!(
            compute_combined_parity(&[mode1.clone(), mode2.clone()]),
            ParityState::Even
        );

        mode1.flip_parity();
        mode2.flip_parity();
        assert_eq!(
            compute_combined_parity(&[mode1.clone(), mode2.clone()]),
            ParityState::Even
        );

        mode3.flip_parity();
        assert_eq!(
            compute_combined_parity(&[mode1, mode2, mode3]),
            ParityState::Odd
        );
    }

    #[test]
    fn test_parity_measurement() {
        let measurement = ParityMeasurement::new(
            vec!["mzm-1".to_string(), "mzm-2".to_string()],
            ParityState::Even,
        );
        assert_eq!(measurement.result, ParityState::Even);
        assert_eq!(measurement.mode_ids.len(), 2);
    }

    #[test]
    fn test_tetron_creation() {
        let mut tetron = TetronLikeLogicalQubit::new("tetron-1");
        assert_eq!(tetron.id, "tetron-1");
        assert!(!tetron.is_complete());

        for i in 0..4 {
            tetron.add_majorana_mode(MajoranaZeroMode::new(format!("mzm-{}", i)));
        }
        assert!(tetron.is_complete());
    }

    #[test]
    fn test_tetron_protection() {
        let mut tetron = TetronLikeLogicalQubit::new("tetron-1");
        assert!(!tetron.protected);
        tetron.enable_protection();
        assert!(tetron.protected);
        tetron.disable_protection();
        assert!(!tetron.protected);
    }

    #[test]
    fn test_braiding_operation() {
        let op = BraidingOperation::new("mzm-1", "mzm-2")
            .with_description("Exchange modes 1 and 2")
            .with_phase(std::f64::consts::PI / 4.0);
        assert_eq!(op.from, "mzm-1");
        assert_eq!(op.to, "mzm-2");
        assert!((op.phase - std::f64::consts::PI / 4.0).abs() < 1e-10);
    }

    #[test]
    fn test_braiding_sequence() {
        let mut seq = BraidingSequence::new();
        assert!(seq.is_empty());

        seq.add_operation(BraidingOperation::new("a", "b").with_phase(0.1));
        seq.add_operation(BraidingOperation::new("c", "d").with_phase(0.2));

        assert_eq!(seq.len(), 2);
        assert!((seq.total_phase() - 0.3).abs() < 1e-10);
    }

    #[test]
    fn test_lattice_config() {
        let mut lattice = LatticeConfig::new(10, 10).with_spacing(2.5);
        assert_eq!(lattice.width, 10);
        assert_eq!(lattice.height, 10);
        assert!((lattice.spacing - 2.5).abs() < 1e-10);

        lattice.add_mode("mzm-1", LatticePosition { x: 3, y: 4 });
        assert_eq!(lattice.mode_count(), 1);
        assert!(lattice.in_bounds(LatticePosition { x: 5, y: 5 }));
        assert!(!lattice.in_bounds(LatticePosition { x: 15, y: 5 }));
    }

    #[test]
    fn test_error_model_config() {
        let config = ErrorModelConfig::new("test-model")
            .with_error_rate(0.01)
            .with_protection_factor(100.0)
            .with_strategy(CorrectionStrategy::TopologicalCode);

        assert_eq!(config.model_name, "test-model");
        assert!((config.base_error_rate - 0.01).abs() < 1e-10);
        assert!((config.effective_error_rate() - 0.0001).abs() < 1e-10);
    }

    #[test]
    fn test_error_model_recording() {
        let mut model = TopologicalErrorModel::new();
        assert_eq!(model.error_count(), 0);

        model.record_error(DetectedError {
            error_type: ErrorType::QuasiparticlePoisoning,
            affected_modes: vec!["mzm-1".to_string()],
            syndrome: "syndrome-1".to_string(),
            timestamp_ns: 0,
        });

        assert_eq!(model.error_count(), 1);
        model.clear_errors();
        assert_eq!(model.error_count(), 0);
    }

    #[test]
    fn test_logical_qubit_state() {
        let state = LogicalQubitState::Zero;
        assert_eq!(state, LogicalQubitState::Zero);
        assert_ne!(state, LogicalQubitState::One);
    }

    #[test]
    fn test_correction_strategy() {
        let strategies = [
            CorrectionStrategy::None,
            CorrectionStrategy::SurfaceCode,
            CorrectionStrategy::ColorCode,
            CorrectionStrategy::TopologicalCode,
        ];
        assert_eq!(strategies.len(), 4);
    }
}
