# CLNRM Codebase Architecture Analysis
## Phases 1-7 Comprehensive Review

**Analysis Date:** November 16, 2025  
**Current Branch:** claude/phases-8-10-infrastructure  
**Framework Version:** v1.7.0 (Phases 2-7 Complete)

---

## Executive Summary

The clnrm framework implements a **layered architecture** for hermetic integration testing with phases 2-7 complete:

- **Phase 2**: Σ* environment compilation (ontology system)
- **Phase 3**: Test receipt infrastructure (Γₜ)
- **Phase 4**: μ-Kernel timing validation (τ)
- **Phase 5**: Scenario synthesis (coverage analysis + adversarial testing)
- **Phase 6**: Swarm-scale scheduler (resource governance + policy engine)
- **Phase 7**: Backend-agnostic execution engine (container, WASI, μ-VM)

**Key Architecture Principle:** Weaver (OpenTelemetry) validation is the source of truth; traditional tests provide supporting evidence.

---

## 1. CURRENT SCHEDULER ARCHITECTURE (Phase 6)

### Location
- **Main Module:** `/home/user/clnrm/crates/clnrm-core/src/scheduler/`
  - `mod.rs` - Module documentation and re-exports
  - `swarm.rs` - Core scheduler implementation (608 lines)

### Core Components

#### SwarmScheduler
**Purpose:** Multi-tenant request scheduling with priority-based queuing

**Data Structures:**
```rust
pub struct SwarmScheduler {
    queue: Arc<Mutex<BinaryHeap<TestRequest>>>,        // Priority queue (O(log n))
    active: Arc<DashMap<TenantId, Vec<ExecutionHandle>>>, // Lock-free tracking
    governor: Arc<ResourceGovernor>,                    // Resource enforcement
    policy: Arc<PolicyEngine>,                          // Policy validation
    stats: Arc<SchedulerStats>,                         // Atomic statistics
}
```

**Key Methods:**
- `admit(request)` - Three-tier admission: policy check → effect budget → resource budget
- `dequeue()` - Priority-based FIFO extraction
- `track_execution()` - Per-tenant active tracking
- `mark_complete()` - Execution completion tracking
- `stats()` - Atomic snapshot of queue depth, admitted, rejected, completed counts

**Performance Characteristics:**
- Admission: O(log n) binary heap insertion
- Dequeue: O(log n) binary heap extraction
- Tracking: O(1) DashMap operations
- Statistics: O(1) atomic loads

#### ResourceGovernor
**Purpose:** Enforce tenant-specific and global resource limits

**Features:**
- Per-tenant concurrency limits (Semaphore-based)
- Global concurrency limit (shared Semaphore)
- Rate limiting (executions per hour via AtomicU64)
- Cost tracking (RwLock-protected f64)
- Resource guard (RAII pattern for cleanup)

**Key Methods:**
- `check_effect_budget()` - Validates no forbidden operations (TODO: needs EffectSet integration)
- `check_resource_budget()` - Checks global/tenant/rate/cost limits
- `acquire()` - Grants permits with RAII guard for automatic release

**Thread Safety:**
- Lock-free hot paths: DashMap for active tracking
- Fair limiting: Semaphore with OwnedSemaphorePermit
- Atomic metrics: AtomicU64 for rate tracking

#### PolicyEngine
**Purpose:** Policy-driven admission control

**Implementation:**
- Per-tenant constraint policies (ConstraintSet)
- Capability whitelist/blacklist validation
- Hermeticity and latency band enforcement
- Scenario constraint matching

**Check Order (in `admit()`):**
1. Policy check: tenant policy constraints
2. Effect budget: forbidden operation validation
3. Resource budget: capacity availability
4. Enqueue: add to priority queue

**Integration Points:**
- Takes `ConstraintSet` from Phase 7 (Capability module)
- Validates `CapabilityScenario` from Phase 5
- Checks `LatencyBand` requirements from Phase 1

#### Request & Admission Types

**TestRequest** (incoming request):
```rust
pub struct TestRequest {
    request_id: RequestId,
    agent_id: AgentId,
    tenant: TenantId,
    scenario: CapabilityScenario,      // Phase 5: synthesized scenario
    capability_budget: CapabilityBudget, // CNV policy limits
    effect_budget: EffectBudget,        // Chicago TDD limits
    priority: u8,                       // 0-10 (0=low, 10=high)
    latency_target: LatencyBand,        // Hot/Warm/Cold
    submitted_at: String,               // ISO 8601
}
```

**ExecutionHandle** (active tracking):
```rust
pub struct ExecutionHandle {
    request_id: RequestId,
    scenario_id: ScenarioId,
    started_at: String,
    estimated_completion: Option<String>,
}
```

**AdmissionTicket** (proof of admission):
```rust
pub struct AdmissionTicket {
    request_id: RequestId,
    admitted_at: String,
    queue_position: usize,
    estimated_start: Option<String>,    // TODO: implement estimation
}
```

### Concurrency Model

**Lock Contention Analysis:**
- **Minimal Contention:** BinaryHeap protected by single Mutex (admits/dequeues only)
- **Zero Contention:** DashMap for active tracking (per-entry locks)
- **Zero Contention:** Atomic counters for statistics (no locks)
- **Fair Queuing:** Semaphore-based capacity management

**Admission Flow (Async):**
```
1. Policy check (lookup in DashMap)
2. Effect budget check (O(1) validation)
3. Resource budget check:
   - Global semaphore check
   - Tenant semaphore lazy-init + check
   - Rate counter check
   - Cost budget check
4. Lock BinaryHeap for enqueue
5. Return AdmissionTicket
```

### Integration with Other Phases

**Phase 5 → Phase 6:**
- `CapabilityScenario` objects flow into `TestRequest`
- Scenario IDs tracked in `ExecutionHandle`
- Synthesis gaps feed into scheduler load patterns

**Phase 7 → Phase 6:**
- `CapabilityBudget` defines resource limits (from backend capabilities)
- `BackendSelector` (Phase 7) could choose backend based on admitted request

**Upstream Phases:**
- Phase 4 (τ): Latency targets stored in requests
- Phase 3 (Γₜ): Test receipts linked to execution handles
- Phase 2 (Σ*): Environment constraints validated via policies

### Known Limitations & TODOs

1. **Effect Budget Validation** (line 172-177)
   ```rust
   pub async fn check_effect_budget(&self, _request: &TestRequest) -> Result<()> {
       // TODO: Implement effect budget validation
       // This requires adding validate() and is_subset() methods to EffectBudget and EffectSet
       Ok(())
   }
   ```

2. **Estimated Start Time** (line 470)
   - Currently `None`; TODO: implement based on current load

3. **Distributed Scheduling** (noted in Phase 6 future work)
   - Current: Single-node BinaryHeap
   - Future: Multi-node coordination

---

## 2. CURRENT BACKEND ARCHITECTURE (Phase 7)

### Location
- **Main Module:** `/home/user/clnrm/crates/clnrm-core/src/backend/`
  - `mod.rs` - Backend trait definition + AutoBackend wrapper
  - `engine.rs` - ExecutionEngine trait (abstract execution substrate)
  - `pool.rs` - ContainerPool (v1.4.0 performance optimization)
  - `testcontainer.rs` - TestcontainerBackend implementation
  - `mock.rs` - MockBackend for testing
  - `capabilities.rs` - Backend capability discovery
  - `extensions.rs` - Backend extensions and execution modes
  - `volume.rs` - Volume management

### Core Abstractions

#### Backend Trait (Legacy Interface)
**Purpose:** Synchronous command execution in backends

```rust
pub trait Backend: Send + Sync + Debug {
    fn run_cmd(&self, cmd: Cmd) -> Result<RunResult>;
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn supports_hermetic(&self) -> bool;
    fn supports_deterministic(&self) -> bool;
}
```

**Current Implementations:**
- `TestcontainerBackend` - Docker/Podman containers
- `AutoBackend` - Auto-detection wrapper
- `MockBackend` - Fast testing without Docker

#### ExecutionEngine Trait (New Phase 7 Interface)
**Purpose:** Async, backend-agnostic execution engine with multiple backends

```rust
#[async_trait]
pub trait ExecutionEngine: Send + Sync {
    fn backend_type(&self) -> BackendType;
    async fn start(&self, env: &CompiledEnvironment) -> Result<EnvironmentHandle>;
    async fn exec(&self, handle: &EnvironmentHandle, cmd: &[String]) -> Result<Output>;
    async fn stop(&self, handle: &EnvironmentHandle) -> Result<()>;
    async fn health_check(&self, handle: &EnvironmentHandle) -> Result<bool>;
    fn telemetry_exporter(&self) -> Arc<dyn OtelExporter>;
    fn generate_receipt(&self, handle: &EnvironmentHandle) -> Result<TestReceipt>;
    async fn get_resource_usage(&self, handle: &EnvironmentHandle) -> Result<ResourceUsage>;
}
```

**BackendType Enum:**
```rust
pub enum BackendType {
    Container,    // Docker/Podman containers (IMPLEMENTED)
    Wasi,         // WebAssembly WASI (STUBBED)
    MicroVm,      // Firecracker μ-VM (STUBBED)
    MuKernel,     // μ-Kernel node (FUTURE)
    Custom,       // Extensible for custom backends
}
```

#### Backend Implementations

**ContainerEngine** (PRIMARY - IMPLEMENTED)
```rust
pub struct ContainerEngine {
    config: ContainerConfig,
    // TODO: Integrate with existing ContainerPool
}

pub struct ContainerConfig {
    use_pool: bool,
    pool_size: usize,
    network_mode: String,
    auto_remove: bool,
}
```

**Status:** Mostly stubbed (lines 204-289)
- `start()`: Creates UUID handle but no actual container
- `exec()`: Returns empty output
- `stop()`: No-op
- `health_check()`: Returns true
- `generate_receipt()`: Builds minimal receipt
- `get_resource_usage()`: Returns zeros

**Integration Point:** TODO comment on line 205 indicates need to integrate with existing `ContainerPool`

**WasiEngine** (FUTURE - STUBBED)
```rust
pub struct WasiEngine {
    config: WasiConfig,
}

pub struct WasiConfig {
    preopen_dirs: Vec<String>,
    env_vars: HashMap<String, String>,
    max_memory: u64,
}
```

**EnvironmentHandle** (Backend-Opaque Identifier)
```rust
pub struct EnvironmentHandle {
    id: String,
    backend_type: BackendType,
    metadata: HashMap<String, String>,
    created_at: String, // ISO 8601
}
```

**Output Type:**
```rust
pub struct Output {
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    exit_code: i32,
    duration_ms: u64,
}
```

**ResourceUsage Type:**
```rust
pub struct ResourceUsage {
    cpu_percent: f64,          // 0.0-100.0 per core
    memory_bytes: u64,
    network_io: (u64, u64),    // (sent, received)
    disk_io: (u64, u64),       // (read, written)
    uptime_seconds: u64,
}
```

### Backend Selection & Discovery

**BackendCapabilityRegistry**
- Located in `backend/capabilities.rs`
- Declares what capabilities each backend supports
- Used for backend selection in Phase 6 scheduler

**BackendSelector** (from engine.rs exports)
- Selects appropriate backend based on:
  - Scenario capabilities
  - Resource constraints
  - Tenant policy requirements

### Container Pooling (v1.4.0)

**Location:** `/home/user/clnrm/crates/clnrm-core/src/backend/pool.rs`

**Architecture:**
- `idle_queue`: VecDeque of pre-warmed containers
- `active_containers`: DashMap for lock-free tracking
- `size_limiter`: Semaphore for fair capacity management
- Background health check worker

**Performance Gains:**
- Container acquisition: 2-5s (pool miss) → 0.1-0.5ms (pool hit)
- Pool hit rate: 92-95%
- Throughput: 50-100 tests/s → 500-1000 tests/s

**Integration with Phase 7:**
- ExecutionEngine should use pool when instantiating ContainerEngine
- Pool manages container lifecycle automatically
- Metrics tracked atomically (zero contention)

### Integration with Other Phases

**Phase 2 (Σ*) → Phase 7:**
- `CompiledEnvironment` passed to `start()` method
- Environment compiler defines what services needed
- Backend provisions those services

**Phase 3 (Γₜ) → Phase 7:**
- `generate_receipt()` creates test receipt
- Receipt captures backend type + environment handle
- Receipt proves execution backend

**Phase 4 (τ) → Phase 7:**
- `ResourceUsage` contains timing data
- Timing validation can check execution duration
- Cross-validate with OTEL spans

**Phase 5 (Synthesis) → Phase 7:**
- Scenario determines which backend to use
- `CapabilityScenario` specifies backend requirements
- Hermetic scenarios map to container backends

**Phase 6 (Scheduler) → Phase 7:**
- Scheduler's `ExecutionHandle` maps to backend's `EnvironmentHandle`
- Resource limits flow through policy engine
- Scheduler decides admission; backend executes

### Backend Extension System

**Location:** `backend/extensions.rs`

**BackendExt Trait:**
- Adds capabilities to backends
- Execution modes (serial, parallel, hot)
- Resource limits enforcement
- Custom instrumentation

**ExecutionMode Enum:**
```rust
pub enum ExecutionMode {
    Serial,      // One test at a time
    Parallel,    // Multiple concurrent tests
    Hot,         // Pre-warmed containers
}
```

### Known Limitations & TODOs

1. **ContainerEngine Stubs** (multiple TODOs)
   - `start()`: Line 205 - TODO: Integrate with ContainerPool
   - `exec()`: Line 215 - TODO: Implement actual command execution
   - `stop()`: Line 225 - TODO: Implement cleanup
   - `health_check()`: Line 230 - TODO: Implement health check
   - `get_resource_usage()`: Line 279 - TODO: Implement actual resource tracking

2. **WASI/MicroVM/MuKernel** - Completely stubbed
   - Basic structure defined
   - No implementation
   - Future phases

3. **OtelExporter** - NoOp implementation
   - Returns NoOpExporter
   - Should integrate with telemetry module (Phase 0)

---

## 3. CURRENT TELEMETRY/OBSERVABILITY

### Location
- **Main Module:** `/home/user/clnrm/crates/clnrm-core/src/telemetry/`
- **Entry Point:** `src/telemetry.rs` (38.7 KB)
- **Sub-modules:** 27 files across telemetry/ directory

### Core Components

#### OtelConfig (User Configuration)
```rust
pub struct OtelConfig {
    pub service_name: &'static str,
    pub deployment_env: &'static str,  // "dev" | "prod"
    pub sample_ratio: f64,             // 0.0-1.0 (1.0 = always_on)
    pub export: Export,
    pub enable_fmt_layer: bool,        // Pretty console logs
    pub headers: Option<HashMap<String, String>>, // OTLP auth
}

pub enum Export {
    OtlpHttp { endpoint: &'static str },  // e.g., http://localhost:4318
    OtlpGrpc { endpoint: &'static str },  // e.g., http://localhost:4317
    Stdout,                                // Human-readable
    StdoutNdjson,                          // Machine-readable (one JSON/line)
}
```

#### OtelGuard (Lifecycle Management)
```rust
pub struct OtelGuard {
    tracer_provider: SdkTracerProvider,
    meter_provider: Option<SdkMeterProvider>,
    logger_provider: Option<SdkLoggerProvider>,
    export_monitor: Option<ExportMonitor>,
    adaptive_flush: Option<adaptive_flush::AdaptiveFlush>,
}
```

**Lifecycle:**
- Initializes on creation
- Flushes on drop (guard pattern)
- Handles graceful shutdown
- Monitors export health

#### ExportMonitor (Health Tracking)
```rust
pub struct ExportMonitor {
    successful_exports: Arc<AtomicU64>,
    failed_exports: Arc<AtomicU64>,
    last_export_at: Arc<Mutex<Option<Instant>>>,
}

pub struct ExportStats {
    successful_exports: u64,
    failed_exports: u64,
    last_export_at: Option<Instant>,
}
```

**Purpose:** Detect silent telemetry data loss to Weaver

### Telemetry Module Organization

**Core Modules:**
1. `config.rs` - Configuration types
2. `init.rs` - Initialization logic
3. `exporters.rs` - Span/metric/log exporters

**Integration with Phases:**
4. `weaver_controller.rs` - Weaver schema validation (source of truth)
5. `weaver_emit.rs` - Type-safe telemetry builders (from Weaver schemas)
6. `weaver_stats.rs` - Statistics aggregation
7. `weaver_coordination.rs` - State machine for Weaver validation

**OTEL Pipeline:**
8. `test_execution.rs` - Test lifecycle telemetry
9. `span_storage.rs` - In-memory span collection
10. `validation_processor.rs` - Runtime validation

**Performance & Tuning:**
11. `adaptive_flush.rs` - Adaptive batch flushing (v1.3.0)
12. `metrics_export.rs` - Metrics publication
13. `semantic_conventions.rs` - Standard OTEL conventions

**JSON Export:**
14. `json_exporter.rs` - stdout NDJSON exporter
15. `cli_helpers.rs` - CLI-friendly telemetry builders

**Weaver Live-Check:**
16. `live_check/` - 6 sub-modules for Weaver validation

### Weaver Integration (Critical for Phases 8-10)

**WeaverController** (36 KB, most complex module)
- Validates runtime telemetry against Weaver schemas
- Live-check orchestration
- Schema compliance reporting

**WeaverCoordination** (Type-Safe State Machine)
```rust
pub enum WeaverState {
    Unstarted,
    Running,
    Stopped,
}

// Type-safe transitions prevent invalid states
pub struct TypeSafeWeaverController<S: WeaverState> {
    // State encoded in type system
}
```

**Live-Check System** (`live_check/` sub-modules):
1. `config.rs` - Live-check configuration
2. `validation.rs` - Schema validation logic
3. `port_allocator.rs` - Dynamic port assignment
4. `stop_coordinator.rs` - Graceful shutdown
5. `orchestrator.rs` - Multi-phase orchestration
6. `diagnostics.rs` - Failure diagnosis

### Semantic Conventions

**semantic_conventions.rs**
- Defines OTEL attribute standards
- Resource attributes (service, environment, version)
- Span attributes (operation, result, error)
- Metric names and units

### Feature Flags

From Cargo.toml:
```toml
[features]
otel = ["otel-traces", "otel-metrics", "otel-logs"]
otel-traces = []        # Enable OTEL trace export
otel-metrics = []       # Enable OTEL metrics
otel-logs = []          # Enable OTEL logs
otel-testing = ["opentelemetry_sdk/testing"]  # Testing mode
```

**Note:** OTEL is always compiled in (happy path), features control export channels

### Integration with Execution Phases

**Phase 2 (Σ*):**
- Environment compiler emits telemetry about ontology compilation
- Schema-based instrumentation

**Phase 3 (Γₜ):**
- Receipts include Weaver proof
- Link test executions to telemetry spans
- Receipt signature includes span data hash

**Phase 4 (τ):**
- Span durations validated against latency bands
- Hot/warm/cold path timing assertions
- Cross-validate with μ-kernel receipts

**Phase 5 (Synthesis):**
- Scenario synthesis emits observability data
- Gap analysis telemetry
- Synthesis decision logging

**Phase 6 (Scheduler):**
- Admission telemetry (policy checks, budgets)
- Queue depth metrics
- Resource contention tracking

**Phase 7 (Backend):**
- Backend selection telemetry
- Container lifecycle events
- Execution metrics per backend

### Known Limitations

1. **Weaver Live-Check** - Docker-dependent
   - Requires running Weaver in container
   - Port allocation for live-check server
   - Status: Infrastructure complete (v1.2.0), pending execution

2. **Cross-Layer Validation** - Partially implemented
   - OTEL → Weaver validation: DONE
   - Weaver ↔ μ-kernel: FUTURE (depends on μ-kernel spec)
   - Receipt signature chain: DONE

---

## 4. TEST INFRASTRUCTURE

### Location
- **Integration Tests:** `/home/user/clnrm/crates/clnrm-core/tests/`
- **Unit Tests:** Inline with `#[cfg(test)]` modules
- **Test Fixtures:** TOML files in `tests/`, `examples/`
- **Benchmarks:** `/benches/` directory

### Test Organization

**Integration Test Files** (selected):
```
tests/
├── run_live_check_tests.rs          # Weaver validation tests
├── weaver_innovations.rs            # Weaver schema tests
├── semantic_conventions_tests.rs   # OTEL conventions
├── v1_2_1_regression.rs            # Regression testing
├── lock_free_queue_test.rs         # Concurrency tests
├── concurrency_stress_tests.rs     # Load testing
├── performance_failfast_tdd.rs     # Chicago TDD tests
├── toml_tdd_mocks.rs               # TOML parsing tests
├── telemetry/                       # Telemetry sub-tests
│   ├── weaver_integration.rs
│   ├── validation_tests.rs
│   ├── otlp_export.rs
│   └── export_edge_cases.rs
```

### Feature Gates

**docker-integration Feature**
```toml
[features]
docker-integration = []              # Enable Docker-dependent tests
full-integration = ["docker-integration"]  # Full suite
```

**Usage in Tests:**
```rust
#![cfg(feature = "docker-integration")]

#[test]
fn test_container_execution() -> Result<()> {
    let backend = TestcontainerBackend::new("alpine:latest")?;
    // ...
}
```

**Test Execution Strategy:**
- **Default (no feature):** Unit tests only, runs everywhere
- **With feature:** Integration tests + Docker-dependent tests
- **CI/CD:** `.github/workflows/` manages feature enablement

### Test Patterns

#### AAA Pattern (Arrange-Act-Assert)
```rust
#[tokio::test]
async fn test_container_creation_with_valid_image_succeeds() -> Result<()> {
    // Arrange
    let environment = TestEnvironments::unit_test().await?;

    // Act
    let container = environment.create_container("alpine:latest").await?;

    // Assert
    assert!(container.is_running());
    Ok(())
}
```

#### Dogfooding Tests
```
examples/
├── framework-self-testing/
│   ├── complete-dogfooding-suite.rs
│   ├── container-lifecycle-test.rs
│   ├── hermetic_isolation_test.rs
│   ├── plugin_system_test.rs
│   ├── observability_test.rs
│   └── simple-framework-test.rs
├── observability/
│   ├── observability-demo.rs
│   ├── observability-self-test.rs
│   └── otel_graph_validation.rs
└── plugins/
    ├── custom-plugin-demo.rs
    └── plugin-self-test.rs
```

### TOML-Based Test Configuration

**Format:** `.clnrm.toml` files define tests declaratively

**Example Structure:**
```toml
[test.metadata]
name = "my_test"
description = "Test description"

[services.my_service]
type = "generic_container"
image = "alpine:latest"

[[steps]]
name = "step_1"
command = ["echo", "hello"]
expected_output_regex = "hello"
service = "my_service"

[assertions]
container_should_have_executed_commands = 1
execution_should_be_hermetic = true
```

### Timeout Configuration

**Test Timeout:** 1 second (from Cargo.toml)
```toml
[package.metadata.cargo-make]
test_timeout = 1
```

**Strategy:** Forces fast tests, integrations use `examples/` instead

### Dependencies

**Testing Dependencies:**
```toml
proptest = "1.4"              # Property-based testing (160K+ cases)
criterion = { ... }           # Benchmarking
serial_test = "3.2"          # Test isolation
mockall = { ... }            # London TDD mocking
chicago-tdd-tools = "1.3.0"  # Chicago TDD framework
```

### Validation Hierarchy

**CRITICAL: Only Weaver validation is source of truth**

```
LEVEL 1: Weaver Schema Validation (MANDATORY)
├── weaver registry check -r registry/
└── weaver registry live-check --registry registry/

LEVEL 2: Compilation & Clippy (Baseline)
├── cargo build --release --features otel
└── cargo clippy -- -D warnings

LEVEL 3: Traditional Tests (Supporting Evidence)
├── cargo test --lib
├── cargo test --test '*'
├── clnrm self-test
└── clnrm self-test --suite otel --otel-exporter stdout
```

**Critical Rule:** Help text (--help) proves NOTHING
- `clnrm dev --help` only proves help is registered
- Actual execution required: `clnrm dev tests/ --watch`
- Only Weaver validation proves runtime behavior

---

## 5. ERROR HANDLING

### Location
`/home/user/clnrm/crates/clnrm-core/src/error.rs` (260+ lines)

### CleanroomError Type

```rust
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CleanroomError {
    pub kind: ErrorKind,
    pub message: String,
    pub context: Option<String>,
    pub source: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub type Result<T> = std::result::Result<T, CleanroomError>;
```

### ErrorKind Enum (20+ variants)

**Error Categories:**
```rust
pub enum ErrorKind {
    // Infrastructure errors
    ContainerError,
    NetworkError,
    ResourceLimitExceeded,
    Timeout,
    IoError,
    
    // Framework errors
    ConfigurationError,
    PolicyViolation,
    DeterministicError,
    
    // Testing errors
    ValidationError,
    CoverageError,
    SnapshotError,
    
    // Observability errors
    TracingError,
    RedactionError,
    
    // Integration errors
    ReportError,
    ServiceError,
    TemplateError,
    
    // Status errors
    NotImplementedError,
    InvalidState,
    InternalError,
}
```

### Error Construction Patterns

**Detailed Errors with Context:**
```rust
let err = CleanroomError::new(ErrorKind::ConfigurationError, "Missing service definition")
    .with_context("In test configuration")
    .with_source("config.toml");
```

**Type-Specific Helpers:**
```rust
CleanroomError::container_error("Docker daemon unavailable")
CleanroomError::timeout_error("Test exceeded 5s limit")
CleanroomError::policy_violation_error("Capability not allowed")
CleanroomError::deterministic_error("Non-deterministic behavior detected")
```

### Core Team Standards (MANDATORY)

**Rule 1: Never use .unwrap() or .expect()**
```rust
// ❌ WRONG - panics in production
let result = operation().unwrap();

// ✅ CORRECT - proper error handling
let result = operation().map_err(|e| {
    CleanroomError::internal_error(format!("Operation failed: {}", e))
})?;
```

**Rule 2: Never fake Ok() returns**
```rust
// ❌ WRONG - false positive
pub fn execute_test(&self) -> Result<()> {
    println!("Test executed");
    Ok(())  // Did nothing!
}

// ✅ CORRECT - honest about incompleteness
pub fn execute_test(&self) -> Result<()> {
    unimplemented!("execute_test: needs container execution")
}
```

**Rule 3: All functions return Result**
```rust
// ❌ WRONG - no error information
pub fn run_test() {
    // ...
}

// ✅ CORRECT - propagates errors
pub fn run_test(&self) -> Result<TestOutput> {
    // ...
}
```

### Error Propagation

**Question Mark Operator:**
```rust
pub async fn run_test(&self) -> Result<()> {
    let container = self.backend.create_container("alpine:latest")?; // Propagates error
    let output = self.backend.run_cmd(cmd)?;                         // Propagates error
    Ok(())
}
```

### Display & Debug Implementation

```rust
impl fmt::Display for CleanroomError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}: {}", self.kind, self.message)?;
        if let Some(context) = &self.context {
            write!(f, " (Context: {})", context)?;
        }
        if let Some(source) = &self.source {
            write!(f, " (Source: {})", source)?;
        }
        Ok(())
    }
}
```

**Output Example:**
```
ConfigurationError: Missing service definition (Context: In test configuration) (Source: config.toml)
```

### Integration Error Conversions

**From std::io::Error:**
```rust
impl From<std::io::Error> for CleanroomError {
    fn from(err: std::io::Error) -> Self {
        CleanroomError::io_error(err.to_string())
    }
}
```

**From Template Errors:**
```rust
impl From<clnrm_template::TemplateError> for CleanroomError {
    fn from(err: clnrm_template::TemplateError) -> Self {
        match err {
            clnrm_template::TemplateError::RenderError(msg) => 
                CleanroomError::template_error(msg),
            // ...
        }
    }
}
```

---

## 6. ARCHITECTURE: PHASES 2-5 FOUNDATION

### Phase 2: Environment Compiler (Σ*)

**Location:** `src/environment/`

**Core Types:**
- `SigmaBase` - Immutable environment ontology snapshot
- `SigmaDelta` - Overlay/delta operations
- `EnvironmentCompiler` - Transforms Σ* + ΔΣ + Q → executable environments
- `OntologyStore` - Content-addressable storage
- `CompiledEnvironment` - Executable environment output

**Key Features:**
- Content hashing of ontologies (SHA-256)
- Immutable snapshots for reproducibility
- Delta encoding for efficiency
- Container graph modeling
- Dependency tracking

**Integration Point with Phase 7:**
- `CompiledEnvironment` passed to `ExecutionEngine.start()`
- Proof metadata generated during compilation
- Receipt automatically created

### Phase 3: Test Receipt Infrastructure (Γₜ)

**Location:** `src/receipts/`

**Core Types:**
- `TestReceipt` - Cryptographically verifiable execution proof
- `ReceiptStore` - Content-addressable receipt storage
- `HermeticityWitness` - Proof of isolation
- `TimingFootprint` - Timing validation data
- `WeaverProof` - Schema validation proof

**Key Features:**
- Hash chain for tamper-evidence
- Ed25519 signatures (with crypto feature)
- Content-addressable (SHA-256)
- Links to environment (Σ*)
- Links to telemetry spans

**Receipt Contents:**
- Scenario ID and capabilities
- Image digests (reproducibility)
- Environment ontology hash
- Weaver validation proof
- Timing footprint (hot/warm/cold)
- Hermeticity witness
- Previous receipt hash (chain)

**Integration with Phases:**
- Phase 2: Stores environment hash
- Phase 4: Stores timing data
- Phase 6: Stores execution handle
- Phase 7: Generated by backend

### Phase 4: Timing Validation (τ)

**Location:** `src/timing/`

**Core Types:**
- `TimingValidator` - Validates span timing
- `OtelSpan` - OTEL span data
- `LatencyBand` - Hot/Warm/Cold constraints
- `TimingFootprint` - Timing proof structure
- `MuKernelReceipt` - Low-level timing (placeholder)

**Latency Bands:**
- **Hot (τ ≤ 8):** Sub-millisecond (microsecond precision)
- **Warm:** Millisecond range (no human delay)
- **Cold:** Seconds range (user expects delay)

**Validation Flow:**
1. Collect OTEL spans with duration
2. Check each span against declared band
3. Optional: cross-validate with μ-kernel receipts
4. Generate `TimingFootprint` with violations
5. Store in receipt

**Integration Point:**
- OTEL spans from Phase 0 observability
- Constraints from Phase 1 capability declarations
- Footprint stored in Phase 3 receipt

### Phase 5: Scenario Synthesis (Dark Matter)

**Location:** `src/synthesis/`

**Core Types:**
- `CoverageAnalyzer` - Finds untested capability combinations
- `ScenarioSynthesizer` - Generates scenarios to fill gaps
- `CapabilityGap` - Untested capability pair
- `OntologyGap` - Untested service config
- `HermeticityGap` - Untested isolation boundary

**Gap Types:**
1. **Capability Gaps:** Combinations never tested together
2. **Ontology Gaps:** Service configurations never executed
3. **Hermeticity Gaps:** Isolation boundaries never validated

**Synthesis Variants:**
- **Coverage Scenarios:** Fill identified gaps
- **Adversarial Scenarios:** Chaos testing variants
  - Network delay injection
  - Resource exhaustion
  - Partial failure scenarios

**Integration:**
- Analyzes Phase 3 receipts (test history)
- Validates against Phase 1 capability registry
- Produces `CapabilityScenario` objects
- Feeds into Phase 6 scheduler

---

## 7. KEY ARCHITECTURAL DECISIONS

### Decision 1: Weaver as Source of Truth

**Why:** Tests can pass with broken features; schema validation cannot
- Help text exists but command may be unimplemented
- Tests can mock incorrectly
- Tests may test wrong things
- **Weaver schema validation is the only honest feedback**

**Impact on Phases 8-10:**
- New features must emit telemetry
- Telemetry must conform to Weaver schemas
- Live-check validation must pass
- This is non-negotiable

### Decision 2: Async/Sync Trait Boundary

**Critical Rule:** Never make trait methods async
```rust
// ❌ WRONG - breaks dyn trait compatibility
pub trait ServicePlugin {
    async fn start(&self) -> Result<ServiceHandle>;
}

// ✅ CORRECT - dyn compatible, use block_in_place internally
pub trait ServicePlugin {
    fn start(&self) -> Result<ServiceHandle>;
}
```

**Impact on Phases 8-10:**
- Phase 8 framework traits must stay sync
- Use `tokio::task::block_in_place` for internal async
- ExecutionEngine is exception (new async trait)

### Decision 3: Lock-Free Concurrency

**Pattern:** Use DashMap + Semaphore + AtomicU64
- DashMap for active tracking (zero contention)
- Semaphore for fair limiting
- AtomicU64 for metrics
- Single Mutex only for unavoidable critical sections

**Impact on Phases 8-10:**
- Expect high concurrency (trillions of agents)
- Lock-free data structures are non-negotiable
- One contended lock invalidates entire design

### Decision 4: Content-Addressable Storage

**Pattern:** Everything indexed by SHA-256 hash
- Ontologies (Σ*) - Phase 2
- Receipts (Γₜ) - Phase 3
- Environment configs - Phase 2
- Scenario definitions - Phase 5

**Benefits:**
- Deduplication (identical executions share hash)
- Immutability (hash proves contents)
- Reproducibility (same input → same output)

**Impact on Phases 8-10:**
- New structures should use content hashing
- Enables efficient caching/deduplication

### Decision 5: Multi-Tenant Resource Isolation

**Architecture:**
- Global semaphore: total capacity
- Per-tenant semaphores: tenant-specific limits
- Atomic counters: rate limiting per tenant
- RwLock: cost tracking per tenant

**Impact on Phases 8-10:**
- Tenant ID required for all requests
- Resource limits enforced at admission
- No tenant can starve others

### Decision 6: Hermetic Testing Foundation

**Core Principle:** Tests run in complete isolation
- Each test gets fresh environment
- Network: isolated or controlled
- Filesystem: isolated or mocked
- Process: isolated (container or WASI)
- No external dependencies

**Impact on Phases 8-10:**
- Hermeticity is non-negotiable design requirement
- Every new feature must support hermetic mode
- Weaver validation will prove hermeticity

---

## 8. INTEGRATION POINTS FOR PHASES 8-10

### Expected Inputs to Phase 8

**From Phase 6 (Scheduler):**
- `TestRequest` objects
- Resource budgets (compute, memory, network)
- Policy constraints
- Admission tickets

**From Phase 7 (Backend):**
- Backend selection decisions
- Container/WASI/VM handles
- Resource usage snapshots

**From Phase 0 (Observability):**
- OTEL telemetry spans
- Weaver schema validation results

### Expected Outputs from Phase 8

**To Phase 9 (likely):**
- Test execution results
- Coverage reports
- Performance metrics
- Failure categorization

**To Phase 10 (likely):**
- Optimization recommendations
- Cost analysis
- Capacity planning data
- Failure recovery strategies

### Architecture Readiness Checklist

- [x] Core error handling (`CleanroomError`)
- [x] Async/sync trait boundaries (traits sync, internal async)
- [x] Lock-free concurrency (DashMap, Semaphore, AtomicU64)
- [x] Content-addressable storage (SHA-256 hashing)
- [x] Feature gating (docker-integration, otel, crypto)
- [x] Weaver schema validation infrastructure
- [x] Telemetry emission framework
- [x] Multi-tenant resource governance
- [x] Hermetic testing foundation
- [ ] Phase 8 design (TBD)

---

## 9. DIRECTORY STRUCTURE SUMMARY

```
crates/clnrm-core/src/
├── lib.rs                          # Module declarations
├── telemetry.rs                    # OTEL bootstrap (38.7 KB)
├── error.rs                        # Error types
├── backend/                        # Phase 7
│   ├── mod.rs
│   ├── engine.rs                   # ExecutionEngine trait
│   ├── pool.rs                     # Container pooling (v1.4.0)
│   ├── testcontainer.rs
│   ├── mock.rs
│   ├── capabilities.rs             # Backend capabilities
│   ├── extensions.rs
│   └── volume.rs
├── scheduler/                      # Phase 6
│   ├── mod.rs
│   └── swarm.rs                    # SwarmScheduler (608 lines)
├── environment/                    # Phase 2
│   ├── mod.rs
│   ├── compiler.rs
│   ├── sigma.rs                    # Σ* (base ontology)
│   ├── delta.rs                    # ΔΣ (deltas)
│   └── store.rs                    # Content-addressable storage
├── receipts/                       # Phase 3
│   ├── mod.rs
│   ├── receipt.rs
│   └── store.rs
├── timing/                         # Phase 4
│   ├── mod.rs
│   └── validator.rs
├── synthesis/                      # Phase 5
│   ├── mod.rs
│   ├── synthesizer.rs
│   └── coverage.rs
├── capabilities/                   # Phase 1 (capability framework)
│   ├── mod.rs
│   ├── scenario.rs
│   ├── effects.rs
│   └── constraints.rs
├── telemetry/                      # Phase 0 (telemetry)
│   ├── weaver_controller.rs        # Weaver validation (36 KB)
│   ├── weaver_coordination.rs      # Type-safe state machine
│   ├── weaver_emit.rs              # Type-safe builders
│   ├── test_execution.rs
│   ├── init.rs
│   ├── config.rs
│   ├── exporters.rs
│   ├── live_check/                 # Weaver live-check
│   │   ├── validation.rs
│   │   ├── config.rs
│   │   ├── orchestrator.rs
│   │   ├── port_allocator.rs
│   │   ├── stop_coordinator.rs
│   │   └── diagnostics.rs
│   └── ...                         # 21 more files
├── cleanroom.rs                    # Main framework API
├── services/                       # Service plugins
├── testing/                        # Test utilities
├── config/                         # TOML configuration
├── formatting/                     # Output formatting
└── ...                            # Other infrastructure
```

---

## 10. SUMMARY TABLE: Phases 2-7 Architecture

| Phase | Name | Status | Key Files | Key Types | Trait Design |
|-------|------|--------|-----------|-----------|--------------|
| 2 | Environment Compiler (Σ*) | COMPLETE | `environment/compiler.rs` | `SigmaBase`, `CompiledEnvironment` | `EnvironmentCompiler` |
| 3 | Receipt Infrastructure (Γₜ) | COMPLETE | `receipts/receipt.rs` | `TestReceipt`, `ReceiptStore` | Content-addressable |
| 4 | Timing Validation (τ) | COMPLETE | `timing/validator.rs` | `TimingValidator`, `TimingFootprint` | OTEL span validation |
| 5 | Scenario Synthesis | COMPLETE | `synthesis/synthesizer.rs` | `ScenarioSynthesizer`, `CoverageAnalyzer` | Gap analysis + generation |
| 6 | Swarm Scheduler | COMPLETE | `scheduler/swarm.rs` | `SwarmScheduler`, `PolicyEngine` | Multi-tenant governance |
| 7 | Execution Engine | PARTIAL | `backend/engine.rs` | `ExecutionEngine`, `ContainerEngine` | Backend-agnostic (stubs) |

---

## 11. CRITICAL NOTES FOR PHASES 8-10

### Mandatory Design Principles

1. **Weaver Validation is Non-Negotiable**
   - Every new feature must emit telemetry
   - All telemetry must conform to Weaver schemas
   - Live-check validation must pass before shipping
   - Help text proves nothing; actual execution is proof

2. **Error Handling is Mandatory**
   - No `.unwrap()` or `.expect()` in production code
   - No fake `Ok(())` returns from incomplete functions
   - All functions must return `Result<T, CleanroomError>`
   - Use `.map_err(|e| CleanroomError::...)` for conversions

3. **Async/Sync Trait Boundary is Sacred**
   - Never make shared traits async
   - Use `tokio::task::block_in_place` internally
   - Exception: New ExecutionEngine is async (isolated)

4. **Lock-Free Concurrency is Required**
   - DashMap for active tracking
   - Semaphore for fair limiting
   - AtomicU64 for metrics
   - Single Mutex only for unavoidable critical sections

5. **Hermetic Testing is Foundation**
   - Tests must run in complete isolation
   - No external network by default
   - No non-hermetic filesystem access
   - Deterministic execution everywhere possible

### Anti-Patterns (Strictly Forbidden)

```rust
// ❌ WRONG - fake success
pub fn synthesize_test_cases(&self) -> Result<Vec<TestCase>> {
    println!("Generating test cases...");
    Ok(vec![])  // FALSE POSITIVE!
}

// ✌️ CORRECT - honest about status
pub fn synthesize_test_cases(&self) -> Result<Vec<TestCase>> {
    unimplemented!("Test case synthesis requires constraint solver integration")
}
```

```rust
// ❌ WRONG - async in trait
pub trait ExecutionOrchestrator {
    async fn orchestrate(&self) -> Result<()>;  // BREAKS DYN!
}

// ✅ CORRECT - sync trait, async internally
pub trait ExecutionOrchestrator {
    fn orchestrate(&self) -> Result<()>;
    // Use block_in_place inside
}
```

```rust
// ❌ WRONG - one lock to rule them all
pub struct Phase8Framework {
    everything: Arc<Mutex<FrameworkState>>,  // CONTENTION!
}

// ✅ CORRECT - lock-free where possible
pub struct Phase8Framework {
    active_tests: Arc<DashMap<RequestId, TestHandle>>,  // Zero contention
    queue: Arc<Mutex<VecDeque<TestRequest>>>,          // Minimal contention
    stats: Arc<AtomicU64>,                             // Zero contention
}
```

---

## Conclusion

The clnrm framework provides a **solid, well-thought-out foundation** for Phases 8-10:

1. **Error handling** is comprehensive and enforced
2. **Concurrency** is carefully optimized for scale
3. **Telemetry** is integrated at every layer
4. **Weaver validation** provides honest feedback
5. **Hermetic testing** is the core principle
6. **Resource governance** prevents tenant interference

Phases 8-10 should build on these foundations, respecting the architectural decisions and constraints documented here.

