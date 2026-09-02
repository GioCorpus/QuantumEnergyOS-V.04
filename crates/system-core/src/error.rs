use thiserror::Error;

/// Result type for system-core operations
pub type Result<T> = std::result::Result<T, SystemCoreError>;

/// Comprehensive error type for QuantumEnergyOS core services
#[derive(Debug, Error)]
pub enum SystemCoreError {
    // Service framework errors
    #[error("service initialization failed: {0}")]
    ServiceInitializationFailed(String),

    #[error("service startup failed: {0}")]
    ServiceStartupFailed(String),

    #[error("service shutdown failed: {0}")]
    ServiceShutdownFailed(String),

    #[error("service not found: {0}")]
    ServiceNotFound(String),

    #[error("service is in degraded state: {0}")]
    ServiceDegraded(String),

    // IPC errors
    #[error("IPC transport error: {0}")]
    IpcTransportError(String),

    #[error("IPC serialization failed: {0}")]
    IpcSerializationFailed(#[from] serde_json::Error),

    #[error("IPC serialization error: {0}")]
    IpcSerializationErrorMsg(String),

    #[error("IPC message validation failed: {0}")]
    IpcMessageValidationFailed(String),

    #[error("IPC service not registered: {0}")]
    IpcServiceNotRegistered(String),

    #[error("IPC timeout: {0}")]
    IpcTimeout(String),

    #[error("IPC connection closed")]
    IpcConnectionClosed,

    // Quantum errors
    #[error("quantum operation failed: {0}")]
    QuantumOperationFailed(String),

    #[error("qubit allocation failed: {0}")]
    QubitAllocationFailed(String),

    #[error("circuit execution failed: {0}")]
    CircuitExecutionFailed(String),

    #[error("measurement failed: {0}")]
    MeasurementFailed(String),

    #[error("invalid quantum state: {0}")]
    InvalidQuantumState(String),

    #[error("backend not available: {0}")]
    BackendNotAvailable(String),

    // Database errors
    #[error("database connection failed: {0}")]
    DatabaseConnectionFailed(String),

    #[error("database query failed: {0}")]
    DatabaseQueryFailed(String),

    #[error("database transaction failed: {0}")]
    DatabaseTransactionFailed(String),

    #[error("database record not found")]
    DatabaseRecordNotFound,

    #[error("database constraint violation: {0}")]
    DatabaseConstraintViolation(String),

    // Authentication & Security errors
    #[error("authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("authorization failed: {0}")]
    AuthorizationFailed(String),

    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("token validation failed: {0}")]
    TokenValidationFailed(String),

    #[error("permission denied: {0}")]
    PermissionDenied(String),

    // Configuration errors
    #[error("configuration error: {0}")]
    ConfigurationError(String),

    #[error("missing configuration: {0}")]
    MissingConfiguration(String),

    // Generic errors
    #[error("internal error: {0}")]
    InternalError(String),

    #[error("invalid argument: {0}")]
    InvalidArgument(String),

    #[error("operation not supported: {0}")]
    NotSupported(String),
}

/// Service-specific error type
#[derive(Debug, Error)]
pub enum ServiceError {
    #[error("service initialization failed")]
    InitializationFailed,

    #[error("service startup failed")]
    StartFailed,

    #[error("service shutdown failed")]
    StopFailed,

    #[error("service health check failed")]
    HealthCheckFailed,
}

/// IPC-specific error type
#[derive(Debug, Error)]
pub enum IpcError {
    #[error("transport error: {0}")]
    TransportError(String),

    #[error("serialization error: {0}")]
    SerializationError(String),

    #[error("deserialization error: {0}")]
    DeserializationError(String),

    #[error("message validation failed: {0}")]
    ValidationError(String),

    #[error("service not registered: {0}")]
    ServiceNotRegistered(String),

    #[error("timeout")]
    Timeout,

    #[error("connection closed")]
    ConnectionClosed,

    #[error("invalid message version: {0}")]
    InvalidVersion(u32),

    #[error("io error: {0}")]
    IoError(#[from] std::io::Error),
}

/// Quantum-specific error type
#[derive(Debug, Error)]
pub enum QuantumError {
    #[error("operation failed: {0}")]
    OperationFailed(String),

    #[error("invalid number of qubits: expected {expected}, got {got}")]
    InvalidQubitCount { expected: usize, got: usize },

    #[error("circuit compilation failed: {0}")]
    CompilationFailed(String),

    #[error("backend error: {0}")]
    BackendError(String),

    #[error("simulator error: {0}")]
    SimulatorError(String),

    #[error("measurement failed: {0}")]
    MeasurementFailed(String),

    #[error("invalid state: {0}")]
    InvalidState(String),
}

/// Database-specific error type
#[derive(Debug, Error)]
pub enum DatabaseError {
    #[error("connection failed: {0}")]
    ConnectionFailed(String),

    #[error("query failed: {0}")]
    QueryFailed(String),

    #[error("transaction failed: {0}")]
    TransactionFailed(String),

    #[error("record not found")]
    NotFound,

    #[error("constraint violation: {0}")]
    ConstraintViolation(String),

    #[error("migration failed: {0}")]
    MigrationFailed(String),

    #[error("serialization error: {0}")]
    SerializationError(String),
}

impl From<ServiceError> for SystemCoreError {
    fn from(err: ServiceError) -> Self {
        match err {
            ServiceError::InitializationFailed => {
                SystemCoreError::ServiceInitializationFailed("unknown".to_string())
            }
            ServiceError::StartFailed => SystemCoreError::ServiceStartupFailed("unknown".to_string()),
            ServiceError::StopFailed => SystemCoreError::ServiceShutdownFailed("unknown".to_string()),
            ServiceError::HealthCheckFailed => {
                SystemCoreError::ServiceDegraded("health check failed".to_string())
            }
        }
    }
}

impl From<IpcError> for SystemCoreError {
    fn from(err: IpcError) -> Self {
        match err {
            IpcError::TransportError(msg) => SystemCoreError::IpcTransportError(msg),
            IpcError::SerializationError(msg) => SystemCoreError::IpcSerializationErrorMsg(msg),
            IpcError::ValidationError(msg) => SystemCoreError::IpcMessageValidationFailed(msg),
            IpcError::ServiceNotRegistered(name) => SystemCoreError::IpcServiceNotRegistered(name),
            IpcError::Timeout => SystemCoreError::IpcTimeout("operation timed out".to_string()),
            IpcError::ConnectionClosed => SystemCoreError::IpcConnectionClosed,
            IpcError::InvalidVersion(v) => SystemCoreError::IpcMessageValidationFailed(format!(
                "invalid message version: {}",
                v
            )),
            IpcError::IoError(io_err) => {
                SystemCoreError::IpcTransportError(format!("io error: {}", io_err))
            }
            IpcError::DeserializationError(msg) => {
                SystemCoreError::IpcMessageValidationFailed(format!("deserialization failed: {}", msg))
            }
        }
    }
}

impl From<QuantumError> for SystemCoreError {
    fn from(err: QuantumError) -> Self {
        match err {
            QuantumError::OperationFailed(msg) => SystemCoreError::QuantumOperationFailed(msg),
            QuantumError::InvalidQubitCount { expected, got } => {
                SystemCoreError::QubitAllocationFailed(format!(
                    "invalid qubit count: expected {}, got {}",
                    expected, got
                ))
            }
            QuantumError::CompilationFailed(msg) => SystemCoreError::CircuitExecutionFailed(msg),
            QuantumError::BackendError(msg) => SystemCoreError::BackendNotAvailable(msg),
            QuantumError::SimulatorError(msg) => {
                SystemCoreError::QuantumOperationFailed(format!("simulator error: {}", msg))
            }
            QuantumError::MeasurementFailed(msg) => SystemCoreError::MeasurementFailed(msg),
            QuantumError::InvalidState(msg) => SystemCoreError::InvalidQuantumState(msg),
        }
    }
}

impl From<DatabaseError> for SystemCoreError {
    fn from(err: DatabaseError) -> Self {
        match err {
            DatabaseError::ConnectionFailed(msg) => SystemCoreError::DatabaseConnectionFailed(msg),
            DatabaseError::QueryFailed(msg) => SystemCoreError::DatabaseQueryFailed(msg),
            DatabaseError::TransactionFailed(msg) => SystemCoreError::DatabaseTransactionFailed(msg),
            DatabaseError::NotFound => SystemCoreError::DatabaseRecordNotFound,
            DatabaseError::ConstraintViolation(msg) => {
                SystemCoreError::DatabaseConstraintViolation(msg)
            }
            DatabaseError::MigrationFailed(msg) => {
                SystemCoreError::DatabaseQueryFailed(format!("migration failed: {}", msg))
            }
            DatabaseError::SerializationError(msg) => {
                SystemCoreError::DatabaseQueryFailed(format!("serialization failed: {}", msg))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_system_core_error_display() {
        let err = SystemCoreError::ServiceInitializationFailed("test".to_string());
        assert_eq!(err.to_string(), "service initialization failed: test");
    }

    #[test]
    fn test_service_error_conversion() {
        let service_err = ServiceError::InitializationFailed;
        let system_err: SystemCoreError = service_err.into();
        assert!(matches!(system_err, SystemCoreError::ServiceInitializationFailed(_)));
    }

    #[test]
    fn test_ipc_error_conversion() {
        let ipc_err = IpcError::ServiceNotRegistered("quantum".to_string());
        let system_err: SystemCoreError = ipc_err.into();
        assert!(matches!(
            system_err,
            SystemCoreError::IpcServiceNotRegistered(_)
        ));
    }

    #[test]
    fn test_quantum_error_conversion() {
        let quantum_err = QuantumError::BackendError("simulator not found".to_string());
        let system_err: SystemCoreError = quantum_err.into();
        assert!(matches!(system_err, SystemCoreError::BackendNotAvailable(_)));
    }

    #[test]
    fn test_database_error_conversion() {
        let db_err = DatabaseError::NotFound;
        let system_err: SystemCoreError = db_err.into();
        assert!(matches!(
            system_err,
            SystemCoreError::DatabaseRecordNotFound
        ));
    }
}
