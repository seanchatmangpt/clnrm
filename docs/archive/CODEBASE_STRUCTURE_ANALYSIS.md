# CLNRM Codebase Structure and Architecture Analysis

**Date:** November 16, 2025  
**Version:** 1.4.1  
**Total Source Lines:** 45,507 (clnrm-core)  
**Documentation Files:** 177 in `/docs`  

---

## Executive Summary

**clnrm** (Cleanroom Testing Framework) is a production-grade hermetic integration testing platform written in Rust. It eliminates false-positive testing by validating actual runtime behavior through OpenTelemetry telemetry validation, rather than just checking exit codes.

### Core Proposition
- **Problem:** Traditional testing can pass while features fail (fake-green)
- **Solution:** Schema-first validation using OpenTelemetry + Weaver
- **Result:** Tests prove behavior, not just exit codes

---

## Part 1: Workspace and Crates

### Workspace Structure (`/home/user/clnrm/`)

```
Cargo.toml (workspace root)
├── crates/
│   ├── clnrm/               # CLI binary package
│   ├── clnrm-core/          # Core framework library (45.5K LOC)
│   ├── clnrm-shared/        # Shared utilities
│   ├── clnrm-template/      # Template rendering (Tera)
│   └── clap-noun-verb/      # CLI noun-verb argument parser
└── tests/, examples/, docs/
```

### Crate Details

#### 1. **clnrm** (CLI Binary)
- **Purpose:** User-facing command-line interface
- **Location:** `crates/clnrm/src/`
- **Main Files:**
  - `main.rs` - Entry point (16 LOC, very thin)
  - `lib.rs` - Library exports
  - `bin.rs` - Binary implementation
- **Key Responsibility:** Parse CLI arguments, dispatch to clnrm-core
- **Dependencies:** clnrm-core, clap, tokio, env_logger

#### 2. **clnrm-core** (Framework Library) ⭐ PRIMARY
- **Purpose:** Core testing framework with all business logic
- **Location:** `crates/clnrm-core/src/`
- **Size:** 45,507 lines of Rust code
- **Architecture:** Modular functional modules with trait-based abstractions
- **Key Responsibilities:**
  - Container lifecycle management (TestcontainerBackend)
  - Configuration loading and TOML parsing
  - CLI command implementations
  - OpenTelemetry integration and Weaver validation
  - Service plugin system
  - Test execution orchestration
  - Validation framework for OTEL telemetry

#### 3. **clnrm-shared** (Shared Utilities)
- **Purpose:** Shared types and utilities used across crates
- **Location:** `crates/clnrm-shared/src/`
- **Minimal:** Only error types and shared utilities
- **Dependencies:** serde, serde_json, uuid, thiserror

#### 4. **clnrm-template** (Template Engine)
- **Purpose:** Template rendering with Tera for test configuration
- **Features:** Variable substitution, conditional logic in test TOML files
- **Integration:** Transparent within test loading

#### 5. **clap-noun-verb** (CLI Parser)
- **Purpose:** Extended CLI argument parsing with noun-verb commands
- **Feature:** Allows commands like `clnrm services list` instead of `clnrm services-list`

---

## Part 2: Core Architecture Overview

### High-Level Data Flow

```
User Input (CLI)
    ↓
CLI Parser (clap)
    ↓
Command Dispatcher (cli/mod.rs)
    ↓
Command Handlers (cli/commands/*.rs)
    ↓
Config Loader (config/*)
    ↓
Test Executor (stress_test/executor.rs)
    ├→ TestcontainerBackend (backend/testcontainer.rs)
    ├→ ContainerPool (backend/pool.rs) - v1.4.0+
    ├→ Service Registry (cleanroom.rs)
    └→ Validators (validation/*)
    ↓
OpenTelemetry Export
    ↓
Results & Reports
```

### Key Architectural Patterns

1. **Plugin-Based Service System** - Services are plugins, not hardcoded
2. **Trait-Based Abstraction** - Backend trait allows multiple implementations
3. **Functional Module Structure** - Each module is self-contained
4. **Schema-First Validation** - Weaver integration validates against schemas
5. **Container Pooling** (v1.4.0+) - Pre-warmed containers for performance

---

## Part 3: Main Components and Modules

### Module Map (`crates/clnrm-core/src/`)

#### Core Testing Framework
- **`cleanroom.rs`** (41KB)
  - `ServicePlugin` trait - Interface for services
  - `ServiceRegistry` - Plugin management
  - `ServiceHandle` - Service instance tracking
  - `CleanroomEnvironment` - Main test environment

- **`scenario.rs`** (11KB)
  - Test scenario definitions
  - Step execution logic
  - Artifact collection

- **`stress_test/`**
  - `executor.rs` - High-performance test executor with semaphore-based concurrency
  - `pool.rs` - Resource pooling for stress testing
  - `metrics.rs` - Performance metrics tracking
  - `config.rs` - Stress test configuration

#### Container Management
- **`backend/mod.rs`**
  - `Backend` trait - Abstract backend interface
  - `Cmd` struct - Command configuration
  - `RunResult` struct - Execution results

- **`backend/testcontainer.rs`**
  - `TestcontainerBackend` - Primary Docker implementation
  - Container lifecycle (create, start, stop, cleanup)
  - Command execution in containers

- **`backend/pool.rs`** ⭐ v1.4.0 FEATURE
  - `ContainerPool` - Pre-warmed container management
  - `PoolConfig` - Configuration (max size, idle timeout, health check interval)
  - `PoolStats` - Performance metrics
  - **Key Innovation:** Lock-free hot paths using DashMap
  - **Performance Target:** 0.1-0.5ms acquisition (pool hit) vs 2-5s (pool miss)

- **`backend/engine.rs`**
  - `ContainerEngine` - Container execution abstraction
  - `ExecutionEngine` - Unified execution interface
  - Support for multiple runtime types (Docker, WASI)

#### Configuration System
- **`config/loader.rs`**
  - `load_cleanroom_config()` - Auto-discover and load .toml files
  - `load_cleanroom_config_from_file()` - Load specific file
  - Supports glob patterns, environment variable substitution

- **`config/types.rs`**
  - `TestConfig` - Main test structure
  - `StepConfig` - Individual test step
  - `ServiceConfig` - Service definition
  - `parse_shell_command()` - Shell command parsing utility

- **`config/otel.rs`**
  - `OtelConfig` - OpenTelemetry configuration
  - Export types (stdout, OTLP HTTP/gRPC, Jaeger, Zipkin)

- **`config/weaver.rs`**
  - Weaver integration configuration
  - Registry path resolution

#### CLI System
- **`cli/types.rs`**
  - `Cli` - Main CLI struct (clap Parser)
  - `Commands` enum - All subcommands
  - `OutputFormat` - Display format options

- **`cli/mod.rs`**
  - `run_cli()` - Main CLI entry point
  - Command dispatcher and routing

- **`cli/commands/`** (40+ command implementations)
  - `run/` - Test execution pipeline
  - `init.rs` - Project initialization
  - `validate.rs` - Configuration validation
  - `plugins.rs` - Plugin listing
  - `self_test.rs` - Framework self-testing
  - `live_check.rs` - Weaver live-check integration
  - `template.rs` - Template generation
  - `stress.rs` - Stress testing
  - `health.rs` - Health check commands
  - `analyze.rs` - Code analysis
  - `fmt.rs` - Configuration formatting
  - `dev.rs` - Development mode (watch, reload)
  - And many more...

#### Validation Framework
- **`validation/mod.rs`** - Exports all validators

- **`validation/span_validator.rs`** (43KB)
  - `SpanValidator` - Core span assertion validation
  - `SpanAssertion` - Assertion definition
  - Validates: name, kind, attributes, events, duration, status

- **`validation/graph_validator.rs`**
  - `GraphValidator` - Parent-child relationship validation
  - Ensures trace structure correctness
  - Acyclic validation (proves correct trace DAG)

- **`validation/order_validator.rs`**
  - `OrderValidator` - Temporal ordering validation
  - Ensures operations happen in correct sequence

- **`validation/count_validator.rs`**
  - Span/event/error count validation
  - Cardinality expectations

- **`validation/hermeticity_validator.rs`** (17KB)
  - `HermeticityValidator` - Isolation violation detection
  - Catches external service calls
  - Forbids specific attributes

- **`validation/window_validator.rs`**
  - Temporal window validation
  - Ensures operations occur within time boundaries

- **`validation/otel/`**
  - `OtelValidator` - OTEL telemetry validation
  - `ValidationSpanProcessor` - Span collection and validation

#### Telemetry & OpenTelemetry
- **`telemetry/init.rs`**
  - `init_otel()` - Initialize OpenTelemetry stack
  - `OtelGuard` - Lifetime management

- **`telemetry/exporters.rs`**
  - Exporter configuration and setup
  - Supports: stdout, OTLP HTTP, OTLP gRPC, Jaeger, Zipkin

- **`telemetry/weaver_controller.rs`** (588 lines)
  - `WeaverController` - Manages Weaver subprocess
  - Live-check coordination with running tests
  - Registry validation

- **`telemetry/weaver_emit.rs`**
  - Emit telemetry conforming to Weaver schemas
  - Type-safe telemetry emission

- **`telemetry/span_storage.rs`**
  - In-memory span collection during tests
  - Retrieval for validation

- **`telemetry/json_exporter.rs`**
  - Export spans to JSON format
  - Useful for debugging and analysis

#### Service Plugins
- **`services/mod.rs`** - Module exports
- **`services/generic.rs`** - `GenericContainerPlugin`
  - Run any Docker image
  - Most flexible plugin type

- **`services/surrealdb.rs`** - `SurrealDbPlugin`
  - Database service plugin
  - Pre-configured for testing

- **`services/ollama.rs`** - `OllamaPlugin`
  - LLM inference service (Ollama)
  - AI model serving

- **`services/vllm.rs`** - `VllmPlugin`
  - vLLM inference server
  - High-performance LLM serving

- **`services/tgi.rs`** - `TgiPlugin`
  - Text Generation Inference
  - Hugging Face TGI integration

- **`services/otel_collector.rs`**
  - OpenTelemetry Collector service
  - Spans aggregation and export

- **`services/chaos_engine.rs`**
  - Chaos engineering plugin
  - Network chaos, latency injection, failure injection

- **`services/service_manager.rs`** (20KB)
  - Service lifecycle orchestration
  - Multi-service coordination
  - Readiness checks

- **`services/factory.rs`**
  - Service plugin factory
  - Dynamic service creation

#### Advanced Features
- **`synthesis/`** (Phase 5)
  - Scenario synthesis engine
  - Test case generation

- **`scheduler/`** (Phase 6)
  - Swarm scheduler
  - Resource governance

- **`environment/`** (Phase 7)
  - Σ*-aware environment compiler
  - State management

- **`phases/`** (Phases 8-10)
  - Determinism validation
  - Conformance checking
  - Resource contracts

- **`coverage/`**
  - Behavior coverage tracking
  - Coverage reports
  - Dimension-based analysis

- **`determinism/`**
  - Deterministic execution validation
  - Port allocation (deterministic)
  - Random seed management
  - Time validation

- **`cache/`**
  - File-based caching
  - Memory-based caching
  - Content hashing

#### Result Formatting
- **`formatting/`**
  - `human.rs` - Human-readable output
  - `json.rs` - JSON output
  - `junit.rs` - JUnit XML output
  - `tap.rs` - TAP format
  - `toml_fmt.rs` - TOML formatting

#### Support Modules
- **`error.rs`** (12KB)
  - `CleanroomError` - Comprehensive error type
  - `ErrorKind` enum - 30+ error variants
  - Error context and chaining

- **`policy.rs`** (19KB)
  - `Policy` - Security policy framework
  - `SecurityLevel` and `SecurityPolicy`
  - Policy enforcement

- **`utils.rs`** - Utility functions
- **`macros.rs`** (14KB) - Test helper macros
- **`reporting/`** - Report generation

---

## Part 4: Key Traits and Abstractions

### Core Traits (Dependency Injection Points)

#### 1. `ServicePlugin` (cleanroom.rs)
```rust
#[async_trait]
pub trait ServicePlugin: Send + Sync + Debug {
    fn name(&self) -> &str;
    async fn start(&self) -> Result<ServiceHandle>;
    async fn stop(&self, handle: ServiceHandle) -> Result<()>;
    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus;
}
```
**Purpose:** Plugin interface for services (databases, APIs, etc.)  
**Implementations:** GenericContainerPlugin, SurrealDbPlugin, OllamaPlugin, etc.

#### 2. `Backend` (backend/engine.rs)
```rust
pub trait Backend: Send + Sync {
    async fn run_command(&self, cmd: &Cmd) -> Result<RunResult>;
    async fn create_container(&self, config: &ContainerConfig) -> Result<String>;
    async fn stop_container(&self, container_id: &str) -> Result<()>;
    fn get_capabilities(&self) -> &BackendCapabilityRegistry;
}
```
**Purpose:** Abstract container execution backend  
**Implementations:** TestcontainerBackend, MockBackend, WasiEngine

#### 3. `Validator` (validation/*)
```rust
pub trait Validator: Send + Sync {
    fn validate(&self, spans: &[SpanData]) -> ValidationResult;
}
```
**Implementations:** SpanValidator, GraphValidator, OrderValidator, etc.

#### 4. `Formatter` (formatting/mod.rs)
```rust
pub trait Formatter {
    fn format_results(&self, results: &TestSuite) -> String;
}
```
**Purpose:** Output formatting (human, JSON, JUnit, TAP)

#### 5. `Cache` (cache/*)
```rust
pub trait CacheManager {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>, ttl: Duration) -> Result<()>;
    async fn clear(&self, key: &str) -> Result<()>;
}
```
**Implementations:** FileCache, MemoryCache

---

## Part 5: CLI Command Architecture

### Commands Hierarchy

```
clnrm
├── run              → Test execution (parallel, watch, cache)
├── init             → Project initialization
├── validate         → Configuration validation
├── plugins          → List available plugins
├── services         → Service management
├── template         → Generate test templates
├── health           → System health check
├── self-test        → Framework self-testing
├── live-check       → Weaver validation
├── stress           → Stress testing
├── dev              → Development mode (watch, reload)
├── analyze          → Code analysis
├── fmt              → Format TOML
├── lint             → Lint configuration
├── report           → Generate reports
├── diff             → Compare test results
├── dry-run          → Dry-run execution
├── spans            → Inspect spans/traces
├── record           → Record test execution
├── repro            → Reproduce test failures
├── redgreen         → Red-green refactoring helper
└── graph            → Visualize trace graphs
```

### Command Implementation Pattern

Each command typically:
1. Parse specific flags/arguments
2. Load configuration (if needed)
3. Execute main logic
4. Format and return results
5. Handle errors with context

**Example:** `run` command
- Located in `cli/commands/run/mod.rs`
- Submodules: executor.rs, cache.rs, scenario.rs, services.rs, single.rs, watch.rs
- Integrates: TestcontainerBackend, ContainerPool, Validators, OtelExporter

---

## Part 6: Data Flow for a Test Execution

### Execution Pipeline (run command)

```
1. CLI Parse
   ↓ clap parses: clnrm run tests/ --parallel --jobs 4
   
2. Command Dispatch
   ↓ cli/mod.rs routes to run_tests_with_shard_and_report()
   
3. Test Discovery
   ↓ Walk file system for *.clnrm.toml files
   ↓ Load config files via config/loader.rs
   
4. Configuration Loading
   ↓ Parse TOML with serde
   ↓ Render templates (Tera)
   ↓ Resolve environment variables
   ↓ Validate against schema
   
5. Backend Initialization
   ↓ Create TestcontainerBackend
   ↓ Initialize ContainerPool (if CLNRM_ENABLE_POOLING=1)
   
6. OTEL Setup
   ↓ Initialize OpenTelemetry exporter
   ↓ Start Weaver subprocess (if live-check enabled)
   ↓ Register span processors
   
7. Service Registry
   ↓ Register services (GenericContainerPlugin, etc.)
   ↓ Start services (database, API server, etc.)
   
8. Concurrent Test Execution
   ↓ stress_test/executor.rs with Semaphore-based limits
   ├─ For each test:
   │  ├─ Create container (from pool or new)
   │  ├─ Execute steps sequentially
   │  ├─ Collect telemetry spans
   │  └─ Validate spans against assertions
   │
   
9. Span Validation
   ↓ validation/orchestrator.rs coordinates validators
   ├─ SpanValidator: Check name, kind, attributes
   ├─ GraphValidator: Check parent-child structure
   ├─ OrderValidator: Check temporal ordering
   ├─ CountValidator: Check cardinality
   ├─ HermeticityValidator: Check no external calls
   └─ StatusValidator: Check error/ok status
   
10. Result Aggregation
    ↓ Collect validation results
    ↓ Format results (human/JSON/JUnit)
    
11. Report Generation
    ↓ Generate HTML/JSON/JUnit reports
    ↓ Print summary to stdout
    
12. Cleanup
    ↓ Stop services
    ↓ Close OTEL exporter
    ↓ Return results
```

---

## Part 7: Current Documentation Structure

### Documentation Organization (/docs)

**Total Files:** 177 markdown files  
**Size:** ~2.6MB  
**Organization:** Topic-based + historical archive

### Main Documentation Categories

#### 1. **Getting Started**
- `quick-start.md`
- `USAGE_EXAMPLES.md`
- `CLI_GUIDE.md`

#### 2. **Validation & Testing**
- `VALIDATION_GUIDE.md`
- `TESTING.md`
- `PRODUCTION_VALIDATION_GUIDE.md`
- `DEFINITION_OF_DONE_V1.md`

#### 3. **OpenTelemetry & Weaver**
- `OPENTELEMETRY_INTEGRATION_GUIDE.md` (31KB)
- `WEAVER_USER_GUIDE.md`
- `WEAVER_BEST_PRACTICES.md` (25KB)
- `SCHEMA_WRITING_GUIDE.md`
- `/weaver/` - Complete Weaver documentation

#### 4. **Architecture**
- `/architecture/` - System architecture docs
- `V1_4_0_ARCHITECTURE_DIAGRAMS.md` (45KB)
- `V1_4_0_CONCURRENCY_ARCHITECTURE.md` (32KB)
- `CONTAINER_POOL_ARCHITECTURE.md`

#### 5. **Performance & Benchmarking**
- `PERFORMANCE_TUNING.md`
- `PERFORMANCE_BENCHMARKING.md`
- `CONTAINER_POOLING.md` (14KB)
- `/stress-test-architecture.md` (31KB)

#### 6. **Backend & Infrastructure**
- `/backend/` - Backend docs
- `DOCKER_VALIDATION.md`
- `DEPLOYMENT.md`
- `/runbooks/` - Operational guides

#### 7. **Advanced Topics**
- `BEHAVIOR_COVERAGE_DESIGN.md` (13KB)
- `FAKE_GREEN_DETECTION_USER_GUIDE.md` (21KB)
- `FAILURE_MODES_AND_RECOVERY.md`
- `ENV_VARIABLE_RESOLUTION.md`

#### 8. **Reference**
- `CARGO_MAKE_GUIDE.md` (23KB)
- `TOML_REFERENCE.md` (implied)
- `VALIDATOR_QUICK_REFERENCE.md`
- `/validation/` - Validation tools

#### 9. **Release & Historical**
- `RELEASE_NOTES_v1.2.1.md`
- `/archive/` - 20+ subdirectories of historical docs
- Multiple VERSION_SUMMARY.md files

### Documentation Issues/Gaps

1. **Over-Documentation:** 177 files is excessive; many are historical/archived
2. **Disorganization:** Mix of user guides, architecture, implementation details, agent reports
3. **Diataxis Alignment Issues:**
   - Tutorial/Learn: Scattered across multiple files
   - How-to/Do: Embedded in multiple guides
   - Reference: Not clearly separated
   - Explanation: Scattered in agent reports
4. **Navigation:** index.md exists but navigation is unclear
5. **Version Skew:** Multiple v1.2.1, v1.3.0, v1.4.0 docs with different info
6. **Agent Reports:** 20+ agent completion reports not curated for users

---

## Part 8: Data Structures and Types

### Major Config Types

#### `TestConfig` (config/types.rs)
```rust
pub struct TestConfig {
    pub metadata: TestMetadata,
    pub services: HashMap<String, ServiceConfig>,
    pub scenarios: Vec<ScenarioConfig>,
    pub expected_results: ExpectationSet,
    pub determinism_config: Option<DeterminismConfig>,
}
```

#### `ServiceConfig`
```rust
pub struct ServiceConfig {
    pub plugin: String,          // "generic_container", "surrealdb", etc.
    pub image: String,           // Docker image
    pub port_mappings: HashMap<u16, u16>,
    pub environment: HashMap<String, String>,
    pub volumes: Vec<VolumeMount>,
    pub health_check: Option<HealthCheck>,
    pub depends_on: Vec<String>,
}
```

#### `ScenarioConfig`
```rust
pub struct ScenarioConfig {
    pub name: String,
    pub description: Option<String>,
    pub service: Option<String>,  // Which service to run on
    pub steps: Vec<StepConfig>,
    pub assertions: AssertionSet,
    pub artifacts: ArtifactCollection,
}
```

#### `StepConfig`
```rust
pub struct StepConfig {
    pub name: String,
    pub command: Vec<String>,
    pub working_directory: Option<PathBuf>,
    pub environment: HashMap<String, String>,
    pub timeout_ms: Option<u64>,
    pub expected_exit_code: Option<i32>,
    pub expected_output_regex: Vec<String>,
}
```

### Result Types

#### `ExecutionResult` (cleanroom.rs)
```rust
pub struct ExecutionResult {
    pub success: bool,
    pub duration_ms: u64,
    pub test_name: String,
    pub output: String,
    pub error: Option<String>,
    pub validation_results: Vec<ValidationResult>,
    pub spans_collected: usize,
}
```

#### `ValidationResult`
```rust
pub struct ValidationResult {
    pub validator_name: String,
    pub passed: bool,
    pub errors: Vec<ValidationError>,
    pub duration_ms: u64,
}
```

---

## Part 9: Key Implementation Highlights

### v1.4.0+ Container Pooling

**Location:** `backend/pool.rs`  
**Innovation:** Lock-free container management

```rust
pub struct ContainerPool {
    idle_queue: Arc<Mutex<VecDeque<PooledContainer>>>,
    active_containers: Arc<DashMap<String, PooledContainer>>,
    size_limiter: Arc<Semaphore>,
    stats_acquired: Arc<AtomicU64>,
    stats_created: Arc<AtomicU64>,
}
```

**Performance Gains:**
- Startup: 2-5s → 0.1-0.5ms (pool hit)
- Throughput: 50-100 tests/s → 500-1000 tests/s
- Max concurrency: 100 → 500-1000

### Weaver Integration

**Location:** `telemetry/weaver_controller.rs`  
**Purpose:** Schema validation during test execution

**Workflow:**
1. Weaver subprocess spawned before tests
2. Tests emit telemetry to OTLP endpoint
3. Weaver validates telemetry against schema
4. Validation report returned after tests
5. Framework marks test as FAIL if schema violated

### Determinism Engine

**Location:** `determinism/`  
**Features:**
- Port allocation determinism (`ports.rs`)
- Random seed management (`rng.rs`)
- Time mocking (`time.rs`)
- Volume snapshot validation (`volumes.rs`)

---

## Part 10: Documentation Reorganization Opportunity

### Diataxis Framework Application

**Current State:**
- 177 files, many overlapping
- Historical agent reports mixed with user docs
- No clear tutorial/how-to/reference separation

**Opportunity for Reorganization:**

**1. Tutorials** (Learn)
- Getting started with clnrm
- First test TOML
- Running with container pooling
- Setting up Weaver validation

**2. How-to Guides** (Do)
- Running tests in parallel
- Setting up OTEL export
- Writing custom plugins
- Troubleshooting common issues
- CI/CD integration

**3. Reference** (Know)
- CLI commands reference
- TOML schema documentation
- API/trait documentation
- Configuration options
- Error messages catalog

**4. Explanation** (Understand)
- Architecture overview
- Why Weaver validation matters
- How container pooling works
- Plugin system design
- Determinism engine internals

**Archive Issues:**
- Move 20+ agent reports to `/archive/historical/`
- Keep version-specific docs in `/archive/releases/`
- Remove duplicate version docs

---

## Part 11: Main User-Facing Features

### Core Capabilities

1. **TOML-Based Test Definition**
   - Declarative test syntax
   - Service composition
   - Environment variable substitution
   - Template variable support (Tera)

2. **OpenTelemetry Validation**
   - Span structure validation
   - Parent-child relationship validation
   - Temporal ordering validation
   - Attribute validation
   - Weaver schema conformance

3. **Multi-Backend Support**
   - Docker (primary)
   - Podman (compatible)
   - WASI (experimental)
   - Mock backend (testing)

4. **Service Plugin System**
   - Generic containers
   - SurrealDB (database)
   - Ollama, vLLM, TGI (LLM serving)
   - Chaos engineering
   - OTEL Collector

5. **Advanced Testing**
   - Parallel execution with configurable workers
   - Container pooling for 80% faster startup
   - Watch mode (hot reload)
   - Stress testing
   - Deterministic execution validation
   - Behavior coverage tracking

6. **Observability**
   - OTLP export (HTTP/gRPC)
   - Jaeger integration
   - Zipkin integration
   - Structured logging with tracing
   - JSON span export

7. **Reporting**
   - Human-readable output
   - JSON reports
   - JUnit XML (CI integration)
   - TAP format
   - HTML reports

8. **Security & Policy**
   - SecurityPolicy framework
   - Hermeticity validation
   - Forbidden attribute enforcement
   - Resource limits

---

## Part 12: Technology Stack

### Language & Runtime
- **Rust 1.70+** - Type safety, performance
- **Tokio** - Async runtime
- **async-trait** - Trait async/await support

### Container & Orchestration
- **testcontainers-rs** 0.25 - Container lifecycle
- **Docker API** - Container operations
- **testcontainers-modules** - Pre-built containers

### Configuration & Templating
- **TOML** - Configuration format
- **Tera** - Template engine (variable substitution)
- **serde** - Serialization/deserialization

### Observability
- **OpenTelemetry 0.31.0** - Telemetry standard
- **opentelemetry-otlp** - OTLP export
- **opentelemetry-jaeger** - Jaeger export
- **opentelemetry-zipkin** - Zipkin export
- **tracing** - Structured logging
- **tracing-subscriber** - Logging configuration

### CLI & Output
- **clap 4.5** - Argument parsing
- **clap-noun-verb** - Noun-verb syntax
- **serde_json** - JSON formatting
- **junit-report** - JUnit XML
- **quick-xml** - XML parsing

### Data Structures
- **DashMap 6.1** - Lock-free concurrent map (container pool)
- **crossbeam** - Lock-free queues
- **uuid** - Unique identifiers
- **chrono** - Date/time handling

### Testing & Quality
- **criterion** - Benchmarking
- **proptest** - Property-based testing
- **mockall** - Mocking framework
- **insta** - Snapshot testing
- **chicago-tdd-tools** - Chicago-TDD support

---

## Part 13: Metrics and Statistics

### Code Metrics
- **Total LOC:** 45,507 (clnrm-core)
- **Modules:** 50+ discrete modules
- **Traits:** 15+ core abstractions
- **Commands:** 25+ CLI commands
- **Service Plugins:** 8 built-in plugins
- **Validators:** 8+ span validators

### Documentation Metrics
- **Total Docs:** 177 files
- **Largest Doc:** V1_4_0_ARCHITECTURE_DIAGRAMS.md (45KB)
- **Main Index:** index.md with 130+ links
- **Version Docs:** 5 major versions documented

### Test Coverage
- **Integration Tests:** TOML-based (not Rust unit tests)
- **Stress Tests:** Concurrent test execution benchmarks
- **Self-Tests:** Framework validates itself with clnrm self-test
- **Examples:** 20+ working examples

---

## Summary: Reorganization Recommendations for Diataxis

### Critical Documentation Issues
1. **177 files is unsustainable** - consolidate to ~30-40 core docs
2. **No clear user journey** - tutorials are scattered
3. **Duplicate version docs** - consolidate to one "latest" + one "archive"
4. **Agent reports in user space** - move to `/archive/historical/`
5. **Architecture docs scattered** - consolidate to `/architecture/` with clear structure

### Proposed Diataxis Structure (Target: 40 files)

```
docs/
├── tutorials/                    # Learn
│   ├── 01-getting-started.md
│   ├── 02-first-test.md
│   ├── 03-container-pooling.md
│   └── 04-otel-validation.md
│
├── how-to/                       # Do (problem-solving)
│   ├── parallel-execution.md
│   ├── custom-plugin.md
│   ├── otel-setup.md
│   ├── weaver-validation.md
│   ├── troubleshooting.md
│   └── ci-integration.md
│
├── reference/                    # Know (lookup)
│   ├── cli-commands.md
│   ├── toml-schema.md
│   ├── api.md
│   ├── error-codes.md
│   └── configuration.md
│
├── explanation/                  # Understand (why)
│   ├── architecture.md
│   ├── container-pooling.md
│   ├── weaver-validation.md
│   ├── determinism-engine.md
│   ├── plugin-system.md
│   └── otel-integration.md
│
├── archive/                      # Historical
│   ├── historical-reports/      # Agent reports
│   ├── releases/                 # Version-specific docs
│   └── implementation/           # Implementation history
│
└── index.md                      # Main navigation
```

### Benefits of Reorganization
- **Clear Learning Path:** Tutorials → How-to → Reference → Understanding
- **Reduced Cognitive Load:** 40 files vs 177
- **Better Discoverability:** Diataxis structure is well-known
- **Easier Maintenance:** Clear ownership of each section
- **Scalable:** Easy to add new docs in right place

