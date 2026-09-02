# QuantumEnergyOS Service Framework Architecture

**Document Version:** 1.0  
**Last Updated:** 2026-09-01  
**Phase:** Phase 2 (Service Framework)

---

## Overview

The QuantumEnergyOS Service Framework provides a unified, type-safe interface for managing the lifecycle and orchestration of all services in the system. It enables:

- **Centralized service registration and discovery**
- **Synchronized lifecycle management** (initialize → start → stop)
- **Health monitoring and status reporting**
- **Error aggregation and recovery**
- **Type-safe, async-first architecture**

---

## Architecture

### Layered Design

```
┌──────────────────────────────────────────────┐
│   Application / Main Runtime                 │
├──────────────────────────────────────────────┤
│   ServiceManager                             │
│   (Orchestration & Lifecycle)                │
├──────────────────────────────────────────────┤
│   Individual Services (Parallel)             │
│                                              │
│   ┌─────────────┐  ┌─────────────┐          │
│   │Auth Service │  │Energy Svr   │  ...     │
│   └─────────────┘  └─────────────┘          │
├──────────────────────────────────────────────┤
│   Service Bus (IPC)                          │
├──────────────────────────────────────────────┤
│   Hardware / Database / Network Subsystems   │
└──────────────────────────────────────────────┘
```

### Core Components

#### 1. QuantumService Trait

The base interface that all services must implement:

```rust
pub trait QuantumService {
    fn initialize(&mut self) -> Result<(), ServiceError>;
    fn start(&mut self) -> Result<(), ServiceError>;
    fn stop(&mut self) -> Result<(), ServiceError>;
    fn status(&self) -> ServiceStatus;
    fn health(&self) -> HealthStatus;
}
```

**Responsibilities:**
- Resource initialization (databases, connections, etc.)
- Service startup (enter operational mode)
- Graceful shutdown (cleanup and resource release)
- Status reporting (operational state)
- Health assessment (service condition)

#### 2. ServiceManager

Central orchestrator managing all services:

```rust
pub struct ServiceManager {
    services: Arc<RwLock<HashMap<String, ServiceHandle>>>,
    initialized: Arc<RwLock<bool>>,
}
```

**Key Methods:**
- `register()` - Add a service to the manager
- `initialize()` - Initialize all registered services
- `start_all()` - Start all initialized services
- `stop_all()` - Stop all running services
- `status()` / `health()` - Query service state
- `system_health()` - Generate overall system health report

#### 3. Service Status & Health

**ServiceStatus** - Operational state:
- `Initializing` - Being set up
- `Stopped` - Not running
- `Running` - Operational
- `Degraded` - Running but with warnings

**HealthStatus** - Condition assessment:
- `Healthy` - Nominal operation
- `Warning` - Operating with issues
- `Unhealthy` - Non-operational

---

## Service Ecosystem

### Core System Services

#### Security & Identity

**AuthService**
- Manages authentication and credential verification
- Handles JWT token generation and validation
- Dependencies: None
- Criticality: High

**PolicyService**
- Enforces RBAC (Role-Based Access Control)
- Manages permissions and access policies
- Dependencies: AuthService
- Criticality: High

#### Observability & Infrastructure

**TelemetryService**
- Collects system metrics and events
- Manages telemetry aggregation and forwarding
- Dependencies: None
- Criticality: Medium

**EnergyService**
- Monitors energy consumption
- Forecasts power usage
- Coordinates energy optimization
- Dependencies: TelemetryService
- Criticality: Medium

**DeviceService**
- Enumerates and manages hardware devices
- Provides device capability information
- Dependencies: None
- Criticality: Medium

#### Quantum Computing

**QuantumRuntimeService**
- Manages quantum circuit execution
- Coordinates backend selection and execution
- Dependencies: DeviceService, SchedulerService
- Criticality: High (for quantum operations)

**SchedulerService**
- Schedules and orchestrates job execution
- Manages job queues and priorities
- Dependencies: QuantumRuntimeService
- Criticality: High

#### User Interface

**BrowserService**
- Manages browser instances and profiles
- Coordinates isolation and session management
- Dependencies: AuthService
- Criticality: Low

**DashboardService**
- Serves web dashboards and visualizations
- Aggregates telemetry and metrics for display
- Dependencies: TelemetryService, EnergyService
- Criticality: Low

---

## Service Lifecycle

### 1. Registration

**When:** Before system initialization

```rust
let manager = ServiceManager::new();
manager.register("auth", Box::new(AuthService::new())).await?;
manager.register("energy", Box::new(EnergyService::new())).await?;
```

**What happens:**
- Service instance added to manager's registry
- Service assigned a unique name
- Status: `Initializing`

### 2. Initialization

**When:** Once all services are registered

```rust
manager.initialize().await?;
```

**What happens per service:**
1. `service.initialize()` called
2. Resources allocated (DB connections, file handles)
3. Configuration loaded
4. Validation checks performed
5. Status transitions: `Initializing` → `Stopped`
6. Health: `Warning` → `Unhealthy` (stopped)

**Failure handling:**
- Errors collected and reported
- Failed services remain non-functional
- Other services continue

### 3. Startup

**When:** After successful initialization

```rust
manager.start_all().await?;
```

**What happens per service:**
1. `service.start()` called
2. Service enters operational mode
3. Listeners activated, processing begins
4. Status: `Stopped` → `Running`
5. Health: `Unhealthy` → `Healthy`

**Ordering:** Services start in registration order

**Failure handling:**
- Failed services don't affect others
- System continues with available services

### 4. Runtime

**During:** Normal operation

```rust
// Query status
let status = manager.status("quantum").await?;

// Check health
let health = manager.health("energy").await?;

// Get system health report
let report = manager.system_health().await?;
```

**Responsibilities:**
- Services operate independently
- ServiceBus routes messages between services
- Telemetry reports metrics and events
- Health checks run periodically

### 5. Shutdown

**When:** System termination or maintenance

```rust
manager.stop_all().await?;
```

**What happens per service:**
1. `service.stop()` called (reverse registration order)
2. Processing halted
3. Resources released
4. Pending operations concluded
5. Status: `Running` → `Stopped`
6. Health: `Healthy` → `Unhealthy`

**Ordering:** Services stop in LIFO order (last registered, first stopped)

**Error handling:**
- Errors logged but don't prevent other shutdowns
- System continues shutdown sequence

---

## Health Monitoring

### System Health Report

```rust
pub struct SystemHealthReport {
    pub total_services: usize,
    pub healthy_services: usize,
    pub warning_services: usize,
    pub unhealthy_services: usize,
}
```

### Generating Reports

```rust
let report = manager.system_health().await?;

println!("Total: {}", report.total_services);
println!("Healthy: {}", report.healthy_services);
println!("Warnings: {}", report.warning_services);
println!("Unhealthy: {}", report.unhealthy_services);

if report.is_healthy() {
    println!("✓ System fully operational");
} else if report.is_degraded() {
    println!("⚠ System degraded (warnings)");
} else {
    println!("✗ System unhealthy");
}
```

### Health Status Interpretation

| Service Status | Service Health | Meaning | Action |
|---|---|---|---|
| `Initializing` | `Warning` | Being set up | Wait for init |
| `Stopped` | `Unhealthy` | Not running | Start when ready |
| `Running` | `Healthy` | Operational | Normal operation |
| `Running` | `Warning` | Operating w/ issues | Monitor and investigate |
| `Degraded` | `Warning` | Partial functionality | Investigate and recover |
| `Degraded` | `Unhealthy` | Non-operational | Restart or recover |

---

## Usage Patterns

### Pattern 1: Simple Startup

```rust
#[tokio::main]
async fn main() -> Result<()> {
    let manager = ServiceManager::new();

    // Register services
    manager.register("auth", Box::new(AuthService::new())).await?;
    manager.register("quantum", Box::new(QuantumRuntimeService::new())).await?;

    // Lifecycle
    manager.initialize().await?;
    manager.start_all().await?;

    // ... run application ...

    manager.stop_all().await?;
    Ok(())
}
```

### Pattern 2: Monitoring Health

```rust
// Check health periodically
loop {
    let report = manager.system_health().await?;

    if !report.is_healthy() {
        log::warn!("System degraded: {:?}", report);
        // Alert or take corrective action
    }

    tokio::time::sleep(Duration::from_secs(30)).await;
}
```

### Pattern 3: Selective Service Queries

```rust
// Check specific service
let auth_status = manager.status("auth").await?;
let auth_health = manager.health("auth").await?;

// List all services
let services = manager.list_services().await;
for service_name in services {
    let health = manager.health(&service_name).await?;
    println!("{}: {:?}", service_name, health);
}
```

### Pattern 4: Service Replacement (Hot Reload)

```rust
// Unregister old service
manager.stop_all().await?;
manager.unregister("auth").await?;

// Register new version
manager.register("auth", Box::new(AuthService::new())).await?;
manager.initialize().await?;
manager.start_all().await?;
```

---

## Error Handling

### Service-Level Errors

Services can fail to initialize, start, or stop. The ServiceManager:

1. **Catches errors** - Each service error is captured
2. **Continues execution** - Other services unaffected
3. **Aggregates reports** - Collects all errors for diagnostics
4. **Returns summary** - Final result indicates overall success/failure

### Example: Graceful Degradation

```rust
// Try to start all services
match manager.start_all().await {
    Ok(()) => {
        // All services started
        println!("✓ All services started");
    }
    Err(e) => {
        // Some services failed
        println!("⚠ Startup partially failed: {}", e);
        
        // Query individual service states
        let statuses = manager.status_all().await?;
        for (name, status) in statuses {
            println!("  {}: {:?}", name, status);
        }
    }
}
```

---

## Testing

### Unit Tests (Per Service)

Each service implements `QuantumService` with tests:

```rust
#[test]
fn test_auth_service_lifecycle() {
    let mut auth = AuthService::new();
    assert!(auth.initialize().is_ok());
    assert!(auth.start().is_ok());
    assert!(auth.stop().is_ok());
}
```

### Integration Tests (Service Manager)

Test complete lifecycle with multiple services:

```rust
#[tokio::test]
async fn test_full_system_startup() {
    let manager = ServiceManager::new();
    
    // Register all services
    manager.register("auth", Box::new(AuthService::new())).await?;
    manager.register("quantum", Box::new(QuantumRuntimeService::new())).await?;
    // ...
    
    // Test full lifecycle
    manager.initialize().await?;
    manager.start_all().await?;
    manager.stop_all().await?;
}
```

### Test Coverage

**Current Test Suite (23 integration tests):**
- Single service complete lifecycle
- Multiple services orchestration
- Quantum subsystem coordination
- Observability services
- Full system startup (9 services)
- Service replacement
- Error handling
- Health report generation
- Concurrent operations

---

## Performance Characteristics

### Initialization

- **Sequential per service** - Services initialize one at a time
- **Time complexity:** O(n) where n = number of services
- **Typical:** ~100ms per service

### Startup

- **Parallel execution** - All services start concurrently
- **Time complexity:** O(max service startup time)
- **Typical:** ~500ms total for 9 services

### Health Checks

- **O(n) read operations** - Lock-free reads on all services
- **Non-blocking** - Other operations unaffected
- **Typical:** ~1ms for 9 services

### Memory Overhead

- **Per service:** ~1KB per Box<Service>
- **Manager overhead:** ~2KB
- **Typical for 9 services:** <20KB

---

## Best Practices

### 1. Service Naming

Use clear, hierarchical names:
- ✅ `auth`, `quantum`, `energy`, `telemetry`
- ❌ `svc1`, `service_a`, `random_name`

### 2. Initialization Order

Consider dependencies:
```rust
// Correct order
manager.register("auth", ...);      // Foundational
manager.register("policy", ...);    // Depends on auth
manager.register("quantum", ...);   // Depends on policy
```

### 3. Health Monitoring

Check health regularly:
```rust
// Not ideal: Fire and forget
manager.start_all().await?;

// Better: Verify system health
manager.start_all().await?;
let report = manager.system_health().await?;
assert!(report.is_healthy());
```

### 4. Error Handling

Log and report all errors:
```rust
match manager.initialize().await {
    Ok(()) => log::info!("Initialization successful"),
    Err(e) => {
        log::error!("Initialization failed: {}", e);
        return Err(e);
    }
}
```

### 5. Graceful Shutdown

Always call `stop_all()`:
```rust
// Use defer/finally equivalent
let result = manager.start_all().await;
// ... service operations ...
let _ = manager.stop_all().await; // Always runs
```

---

## Integration with IPC Service Bus

Services communicate via the IPC Service Bus (defined in IPC_PROTOCOL.md):

```
ServiceManager controls lifecycle
        ↓
Services use ServiceBus to communicate
        ↓
Messages routed through ServiceRegistry
```

**Example: Quantum job submission**

```
1. Client sends message to ServiceBus: "quantum.job.submit"
2. ServiceRegistry routes to QuantumRuntimeService
3. Service processes and responds
4. Response routed back to client
```

The ServiceManager ensures services are initialized before any IPC communication begins.

---

## Future Enhancements

### Planned (Phase 3+)

1. **Dependency Injection** - Explicit service dependencies
2. **Service Mesh Integration** - External service coordination
3. **Metrics Export** - Prometheus-compatible metrics
4. **Service Versioning** - Multiple service versions
5. **Circuit Breaker** - Automatic failure handling
6. **Service Recovery** - Automatic restart on failure
7. **Health Check Endpoints** - Per-service health APIs

---

## Troubleshooting

### Issue: Service stuck in "Initializing"

**Cause:** Initialization function not completing  
**Solution:** Check service implementation for blocking operations

### Issue: Some services not starting

**Cause:** Dependency on unstarted service  
**Solution:** Verify service registration order

### Issue: System reports "Unhealthy"

**Cause:** One or more services in degraded state  
**Solution:** Check individual service health: `manager.health("service_name")`

### Issue: Services not stopping cleanly

**Cause:** Resources held by service  
**Solution:** Implement proper cleanup in `stop()` method

---

## Code Structure

```
crates/system-core/
├── src/
│   ├── lib.rs                    # Module exports
│   ├── service.rs                # QuantumService trait
│   ├── manager.rs                # ServiceManager implementation
│   ├── services.rs               # Concrete service implementations
│   ├── error.rs                  # Error types
│   └── service_bus.rs            # IPC message routing
└── tests/
    └── service_framework_integration.rs  # Integration tests
```

---

## API Reference

### ServiceManager Methods

| Method | Signature | Purpose |
|--------|-----------|---------|
| `new()` | `() -> Self` | Create new manager |
| `register()` | `async (name, service) -> Result<()>` | Add service |
| `unregister()` | `async (name) -> Result<()>` | Remove service |
| `initialize()` | `async () -> Result<()>` | Init all services |
| `start_all()` | `async () -> Result<()>` | Start all services |
| `stop_all()` | `async () -> Result<()>` | Stop all services |
| `status()` | `async (name) -> Result<Status>` | Get service status |
| `health()` | `async (name) -> Result<Health>` | Get service health |
| `status_all()` | `async () -> Result<Map>` | Get all statuses |
| `health_all()` | `async () -> Result<Map>` | Get all healths |
| `system_health()` | `async () -> Result<Report>` | System health report |
| `list_services()` | `async () -> Vec<String>` | List all services |
| `service_count()` | `async () -> usize` | Count services |
| `has_service()` | `async (name) -> bool` | Check if registered |
| `all_healthy()` | `async () -> bool` | Overall health flag |

---

## References

- [IPC_PROTOCOL.md](./IPC_PROTOCOL.md) - Message routing between services
- [ARCHITECTURE.md](./ARCHITECTURE.md) - System architecture overview
- [THREAT_MODEL.md](./THREAT_MODEL.md) - Security considerations

---

**Document Author:** QuantumEnergyOS Development Team  
**Approval Status:** Phase 2 Implementation  
**Next Review:** 2026-12-01
