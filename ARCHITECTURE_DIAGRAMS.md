# CLNRM v2.0.0 - Architecture Diagrams (C4 Model)

This directory contains comprehensive C4 model diagrams representing the "perfect architecture" for CLNRM v2.0.0. The diagrams are written in PlantUML format and can be rendered using:

- **PlantUML Online Editor**: https://www.plantuml.com/plantuml/uml/
- **VS Code Extension**: PlantUML
- **Command Line**: `plantuml -Tpng C4_ARCHITECTURE.puml`

## Diagram Overview

### 1. **C4_ARCHITECTURE.puml** - System Context (C1)
**Purpose**: Shows how CLNRM fits into the broader ecosystem

**Key Elements**:
- **External Users**: Test Developers, CI/CD Engineers, Product Owners
- **CLNRM System**: Core testing framework
- **External Systems**:
  - Container Registry (OCI images)
  - OpenTelemetry Backend (observability)
  - CI/CD System (orchestration)
  - SurrealDB (optional service)

**Key Relationships**:
- Developers write TOML configs and execute tests
- CLNRM pulls gVisor OCI images (default, no Docker)
- Sends telemetry to OTEL for observability
- Reports results to CI/CD systems

---

### 2. **C2_CONTAINERS.puml** - Containers (C2)
**Purpose**: Shows the major software containers and their interactions

**Key Containers**:

1. **CLI Application** - Entry point for test execution
2. **Core Library** - Framework logic and orchestration
3. **gVisor Backend** (DEFAULT)
   - Container runtime interface using gVisor
   - Zero Docker dependency
   - Activated by default: `feature=backend-gvisor`

4. **Testcontainers Backend** (OPTIONAL)
   - Legacy Docker-based interface
   - For backward compatibility
   - Activated by: `--features backend-testcontainers`

5. **Service Layer** - Manages SurrealDB, generic containers, LLM services
6. **Configuration Loader** - TOML/YAML parsing with Weaver validation
7. **OpenTelemetry Integration** - Distributed tracing and metrics
8. **Local Storage** - Test artifacts, configurations, reports

**External Systems**:
- Container Registry (image source)
- SurrealDB (graph database - optional)
- OTEL Collector (telemetry aggregation)
- OTEL Backend (Prometheus/Jaeger)

---

### 3. **C3_COMPONENTS.puml** - Components (C3)
**Purpose**: Deep dive into the Core Library's internal architecture

**Key Component Groups**:

#### **Configuration & Validation**
- Config Parser (TOML/YAML)
- Config Validator (Weaver schemas)
- Spec Loader (service definitions)

#### **Backend Abstraction Layer**
- Backend Trait (unified interface)
- Backend Factory (feature-gated selection)
- gVisor Implementation (native, no Docker)
- Testcontainers Implementation (optional, legacy)

**Why This Layer Exists**:
- Allows switching between backends based on feature flags
- Enables future Docker API backend
- Maintains backward compatibility
- Reduces Docker coupling

#### **Service Management**
- Service Registry (plugin system)
- Service Factory (creates instances)
- Individual Plugins:
  - SurrealDB Plugin
  - Generic Container Plugin
  - LLM Plugins (Ollama, TGI, VLLM)

#### **Test Execution**
- Cleanroom Environment (isolated contexts)
- Lifecycle Manager (state machine)
- Health Checker (readiness validation)

#### **Observability**
- Tracer (distributed spans)
- Metrics Collector (OpenTelemetry)
- Logger (structured logging)
- Semantic Validator (OTEL conventions)
- Backward Compatibility Layer (v1.9 ID migration)

#### **Error Handling**
- CleanroomError (unified error type)
- Error Converter (feature-gated conversions)

#### **Storage**
- Artifact Manager (test output storage)
- Port Allocator (network port management)

---

### 4. **C4_CODE_CLASSES.puml** - Code/Classes (C4)
**Purpose**: Shows key Rust types and their relationships

**Key Traits & Structs**:

```rust
// Backend Abstraction
pub trait Backend {
    fn create_container(config: ContainerConfig) -> Result<ContainerId>;
    fn execute(id: ContainerId, cmd: &str) -> Result<ExecResult>;
    fn stop(id: ContainerId) -> Result<()>;
    fn cleanup() -> Result<()>;
}

impl Backend for GvisorBackend { ... }        // Default
impl Backend for TestcontainersBackend { ... } // Optional

// Service Plugin System
pub trait ServicePlugin {
    fn name(&self) -> &str;
    fn start(&self) -> Result<ServiceHandle>;
    fn stop(&self, handle: ServiceHandle) -> Result<()>;
    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus;
}

impl ServicePlugin for SurrealDbPlugin { ... }
impl ServicePlugin for GenericContainerPlugin { ... }

// Configuration
pub struct ScenarioConfig { ... }
pub struct ServiceConfig { ... }
pub struct ContainerConfig { ... }

// Execution Orchestration
pub struct CleanroomEnvironment {
    backend: Box<dyn Backend>,
    services: ServiceRegistry,
    state: ExecutionState,
}
```

**Key Design Patterns**:
- **Strategy Pattern**: Backend trait allows switching implementations
- **Factory Pattern**: BackendFactory, ServiceFactory create instances
- **Plugin System**: ServicePlugin enables extensibility
- **Error Wrapper**: CleanroomError with context stacking

---

### 5. **C4_FEATURE_FLAGS.puml** - Feature Flags & Build Configuration
**Purpose**: Documents the feature flag system that enables zero-Docker builds

**Feature Flags**:

```toml
[features]
default = ["backend-gvisor"]

# Backend selection (mutually exclusive by convention)
backend-gvisor = []                          # DEFAULT: gVisor-native
backend-testcontainers = ["dep:testcontainers", "dep:testcontainers-modules"]  # OPTIONAL
backend-docker = []                          # FUTURE
backend-auto = []                            # FUTURE

# Observability
otel = ["otel-traces", "otel-metrics", "otel-logs"]  # OPTIONAL
```

**Build Configurations**:

| Build Type | Command | Features | Docker? | Use Case |
|-----------|---------|----------|---------|----------|
| **Production** | `cargo build --release` | backend-gvisor | ❌ NO | Deployment, minimal attack surface |
| **Full Featured** | `cargo build --all-features` | all | ✓ YES | Development, testing compatibility |
| **Minimal** | `cargo build --no-default-features` | none | ❌ NO | Dependency analysis, core verification |
| **CI with Docker** | `--features backend-testcontainers,otel` | selected | ✓ YES | Docker-based CI/CD, legacy testing |

**Conditional Compilation Gates** (#[cfg(...)]):
```rust
// Services gated by feature
#[cfg(feature = "backend-testcontainers")]
pub mod generic;
pub mod surrealdb;
pub mod otel_collector;

// Error conversions gated by feature
#[cfg(feature = "backend-testcontainers")]
impl From<testcontainers::TestcontainersError> for CleanroomError { ... }

// Service creation gated by feature
#[cfg(feature = "backend-testcontainers")]
"surrealdb" => Self::create_surrealdb_plugin(name, config),

#[cfg(feature = "backend-testcontainers")]
"generic_container" => Self::create_generic_plugin(name, config),
```

---

### 6. **C4_EXECUTION_FLOW.puml** - Test Execution Workflow
**Purpose**: Shows the complete test execution pipeline from config to results

**Phases**:

1. **Configuration Phase**
   - Developer writes TOML config
   - Parser parses scenario
   - Validator checks against Weaver schemas
   - SpecLoader prepares service definitions

2. **Initialization Phase**
   - BackendFactory selects backend (gVisor by default)
   - ServiceFactory creates plugin instances
   - ServiceRegistry registers all plugins
   - CleanroomEnvironment initialized

3. **Startup Phase**
   - LifecycleManager initializes services
   - gVisor pulls OCI images
   - Creates containers with runsc
   - Port allocator maps network ports
   - HealthChecker verifies readiness
   - OpenTelemetry emits SPAN_STARTED

4. **Test Execution Phase**
   - Execute test commands
   - gVisor executes in containers
   - Capture output & metrics
   - OpenTelemetry records data
   - Backward compatibility emits dual IDs for v1.9 migration

5. **Cleanup Phase**
   - LifecycleManager stops services
   - gVisor kills containers
   - Port allocator releases ports
   - OpenTelemetry emits SPAN_ENDED
   - Artifact manager stores reports

6. **Results & Reporting**
   - Test results
   - Metrics (response time, memory, etc.)
   - Logs (structured JSON)
   - Traces (distributed, parent-child relationships)

7. **Observability Export**
   - OTEL Collector gathers telemetry
   - Export to Prometheus (metrics)
   - Export to Jaeger (traces)
   - Export to Datadog/Honeycomb (logs)

**Critical Data Flows**:
- Configuration → Backend Selection (depends on feature flags)
- Service Registration → Execution (all services use unified Backend trait)
- OpenTelemetry → Backward Compat (dual-ID migration for v1.9→v2.0)
- Container Lifecycle → Port Allocation (automatic, zero-config networking)

---

### 7. **C4_DATA_MODEL.puml** - Data Model & Domain Entities
**Purpose**: Documents the domain model and data structures

**Core Domain Entities**:

```rust
// Configuration
ScenarioConfig {
    scenario_id: UUID,
    name: String,
    services: HashMap<ServiceName, ServiceConfig>,
    tests: Vec<TestCase>,
    timeout: Duration,
    isolation_level: IsolationLevel,
}

ServiceConfig {
    service_id: UUID,
    r#type: String, // surrealdb|generic|ollama|...
    image: Option<String>,
    env: HashMap<String, String>,
    ports: Vec<PortMapping>,
    volumes: Vec<VolumeMount>,
}

// Execution
ServiceHandle {
    handle_id: UUID,
    service_name: String,
    container_id: ContainerId,
    port_mappings: HashMap<u16, u16>,
    status: ServiceStatus,
}

ExecutionContext {
    context_id: UUID,
    scenario: ScenarioConfig,
    services: HashMap<ServiceName, ServiceHandle>,
    variables: HashMap<String, String>,
    network: NetworkContext,
    state: ExecutionState,
}

TestResult {
    result_id: UUID,
    test_id: UUID,
    status: TestStatus, // Passed|Failed|Skipped|Timeout
    duration: Duration,
    output: String,
    assertions_passed: usize,
    assertions_failed: usize,
}
```

**Isolation & Security**:
```rust
enum IsolationLevel {
    Process,
    Container,
    VM,
}

IsolationBoundary {
    level: IsolationLevel,
    container_id: ContainerId,
    resources: ResourceLimits,
    capabilities: Capabilities,
    seccomp_profile: Option<String>,
}
```

**Observability Entities**:
```rust
SpanContext {
    span_id: String,
    trace_id: String,
    parent_span_id: Option<String>,
    legacy_span_id: Option<String>, // v1.9 compatibility
}

Metric {
    name: String,
    value: f64,
    unit: String,
    timestamp: Timestamp,
    attributes: Attributes,
}

LogEntry {
    level: LogLevel, // DEBUG|INFO|WARN|ERROR
    message: String,
    attributes: Attributes,
    span_context: Option<SpanContext>,
}
```

---

## Architecture Principles

### 1. **Zero Docker by Default**
- gVisor is the default runtime
- `cargo build` produces Docker-free binary
- testcontainers optional via feature flag
- Reduces attack surface and deployment complexity

### 2. **Backend Abstraction**
- Unified `Backend` trait abstracts container runtime
- Current implementations: gVisor, Testcontainers
- Future implementations: Docker API, containerd
- Enables seamless switching between backends

### 3. **Feature Flags for Conditional Compilation**
- testcontainers code only compiles when opted-in
- Reduces binary size (default: ~50 crates vs full: ~200+ crates)
- Enables CI/CD optimization (disable testcontainers for gVisor-only pipelines)

### 4. **Plugin System for Extensibility**
- `ServicePlugin` trait defines service interface
- Dynamic service registration
- Enables adding new services without modifying core
- Services: SurrealDB, Generic, Ollama, TGI, VLLM, OTEL Collector

### 5. **Observability First**
- OpenTelemetry integration for tracing, metrics, logs
- Semantic conventions validation
- Backward compatibility layer for v1.9→v2.0 migration
- Dual-ID system maintains existing dashboards

### 6. **Error Context Stacking**
- CleanroomError wraps errors with context
- Stack trace through the error chain
- Source location tracking (file:line:function)
- Helps debugging in distributed execution

### 7. **Automatic Resource Management**
- Port allocator eliminates manual port configuration
- Lifecycle manager handles container startup/shutdown
- Health checker validates service readiness
- Cleanup is automatic and idempotent

---

## Building from Diagrams

These C4 diagrams represent the "perfect architecture" for CLNRM v2.0.0. The implementation should follow these principles:

1. **Use the Backend trait** as the primary abstraction for container operations
2. **Implement both GvisorBackend and TestcontainersBackend** for full feature parity
3. **Gate all testcontainers code** with `#[cfg(feature = "backend-testcontainers")]`
4. **Keep service plugins independent** of backend implementation
5. **Validate all configurations** against Weaver schemas before execution
6. **Emit telemetry throughout execution** with proper span hierarchy
7. **Maintain backward compatibility** through dual-ID system for observability

---

## Viewing Diagrams

To view these diagrams:

1. **Online**: Use PlantUML Online Editor
   ```
   https://www.plantuml.com/plantuml/uml/[encoded-puml]
   ```

2. **Local**: Install PlantUML and run
   ```bash
   plantuml -Tpng *.puml
   ```

3. **VS Code**: Install PlantUML extension and preview inline

4. **Generate all diagrams**:
   ```bash
   for file in C4_*.puml; do
     plantuml -Tpng "$file"
   done
   ```

---

## References

- **C4 Model**: https://c4model.com/
- **PlantUML**: https://plantuml.com/
- **CLNRM Repository**: https://github.com/seanchatmangpt/clnrm
- **gVisor Documentation**: https://gvisor.dev/
- **OpenTelemetry**: https://opentelemetry.io/
- **Weaver**: https://github.com/open-telemetry/weaver
