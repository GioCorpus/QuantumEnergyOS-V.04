pub mod backend;
pub mod circuit;
pub mod compiler;
pub mod error;
pub mod error_correction;
pub mod gates;
pub mod job;
pub mod measurement;
pub mod scheduler;
pub mod simulator;
pub mod topology;

pub use error::{QuantumError, Result};
pub use gates::{Complex, QuantumGate};
pub use simulator::QuantumSimulator;
pub use circuit::QuantumCircuit;
pub use backend::{
    QuantumProcessor, SimulatorBackend, MajoranaBackend, AzureQuantumBackend,
    LocalEmulatorBackend, QuantumBackendType, SimulationMetadata,
    QuantumCapabilities, AllocationRequest, QubitRegister, QuantumCircuitInfo,
    QuantumResult, MeasurementRequest, MeasurementResult, QuantumHealth,
};
pub use job::{QuantumJob, JobStatus, JobPriority, JobQueue, JobStatistics};
pub use compiler::{
    CompiledCircuit, CompilerConfig, IntermediateRepresentation, IrOperation, QuantumCompiler,
    MAX_SUPPORTED_QUBITS,
};
pub use scheduler::{
    BackendAvailability, QuantumScheduler, SchedulerConfig, SchedulerStats,
};
pub use measurement::{Measurement, MeasurementValue};
pub use error_correction::{CorrectionStrategy, LogicalQubit, Syndrome};
pub use topology::{
    MajoranaZeroMode, TetronLikeLogicalQubit, TopologicalErrorModel,
    BraidingOperation, ParityMeasurement, LatticeConfig, ErrorModelConfig,
    describe_topology,
};

pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}
