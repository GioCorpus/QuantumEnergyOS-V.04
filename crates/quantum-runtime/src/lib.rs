pub mod backend;
pub mod braiding;
pub mod capabilities;
pub mod circuit;
pub mod compiler;
pub mod compute;
pub mod decoder;
pub mod error;
pub mod error_correction;
pub mod experiment;
pub mod export;
pub mod gates;
pub mod job;
pub mod majorana;
pub mod majorana_sim;
pub mod measurement;
pub mod noise;
pub mod qpu_device;
pub mod optimizer;
pub mod resources;
pub mod scheduler;
pub mod simulator;
pub mod tetron;
pub mod topology;

pub use backend::{
    AllocationRequest, AzureQuantumBackend, LocalEmulatorBackend, MajoranaBackend,
    MeasurementRequest, MeasurementResult, QuantumBackendType, QuantumCapabilities,
    QuantumCircuitInfo, QuantumHealth, QuantumProcessor, QuantumResult, QubitRegister,
    SimulationMetadata, SimulatorBackend,
};
pub use braiding::{BraidSequence, BraidStep, apply_braid_unitary};
pub use capabilities::BackendCapabilities;
pub use circuit::QuantumCircuit;
pub use compiler::{
    CompiledCircuit, CompilerConfig, IntermediateRepresentation, IrOperation, QuantumCompiler,
    MAX_SUPPORTED_QUBITS,
};
pub use compute::{execute_on_backend, probe_backend, ComputeBackendKind, ComputeReport};
pub use decoder::{Correction, Decoder, LookupDecoder, RepetitionDecoder};
pub use error::{QuantumError, Result};
pub use error_correction::{CorrectionStrategy, ErrorCorrectionCode, LogicalQubit, RepetitionCode, Syndrome};
pub use experiment::{run_experiment, ExperimentLimits, ExperimentResult, QuantumExperiment};
pub use export::{export_csv, export_json};
pub use gates::{Complex, QuantumGate};
pub use job::{JobPriority, JobQueue, JobStatistics, JobStatus, QuantumJob};
pub use majorana::{two_mode_gamma_matrices, verify_anticommutation};
pub use majorana::{FermionParity, MajoranaMode, ParityOperator};
pub use majorana_sim::{run_majorana_experiment, MajoranaExperimentResult, MajoranaSimConfig};
pub use measurement::{Measurement, MeasurementValue};
pub use noise::RuntimeNoiseModel;
pub use qpu_device::{QpuCapabilities, QpuDevice, QpuDeviceError, QpuJobHandle};
pub use optimizer::{optimize_circuit, OptimizationReport};
pub use resources::{LogicalQubitId, MeasurementId, PhysicalModeId, QubitId, RegisterId};
pub use scheduler::{BackendAvailability, QuantumScheduler, SchedulerConfig, SchedulerStats};
pub use simulator::QuantumSimulator;
pub use tetron::{ConditionalOperation, MajoranaExchange, MajoranaLogicalQubit, Tetron};
pub use topology::{
    describe_topology, BraidingOperation, ErrorModelConfig, LatticeConfig, MajoranaZeroMode,
    ParityMeasurement, TetronLikeLogicalQubit, TopologicalErrorModel,
};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
