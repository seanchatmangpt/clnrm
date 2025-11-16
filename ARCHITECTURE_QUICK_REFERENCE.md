# Quick Reference: clnrm Architecture

## Key Files by Purpose

### CLI Entry Points
- **`crates/clnrm/src/main.rs`** (16 LOC) - Binary entry point, delegates to clnrm-core
- **`crates/clnrm-core/src/cli/mod.rs`** - CLI routing and command dispatcher
- **`crates/clnrm-core/src/cli/types.rs`** - Command definitions with clap

### Core Framework
- **`crates/clnrm-core/src/lib.rs`** - Library exports and public API
- **`crates/clnrm-core/src/cleanroom.rs`** (41KB) - Main CleanroomEnvironment + ServicePlugin trait
- **`crates/clnrm-core/src/error.rs`** (12KB) - Error types and Result

### Test Configuration & Loading
- **`crates/clnrm-core/src/config/loader.rs`** - Auto-discover and load .toml files
- **`crates/clnrm-core/src/config/types.rs`** - TestConfig, StepConfig, ServiceConfig structs
- **`crates/clnrm-core/src/config/otel.rs`** - OpenTelemetry configuration

### Test Execution
- **`crates/clnrm-core/src/cli/commands/run/mod.rs`** - Main run command implementation
- **`crates/clnrm-core/src/stress_test/executor.rs`** - High-performance executor with concurrency control
- **`crates/clnrm-core/src/scenario.rs`** (11KB) - Scenario/step execution logic

### Container Management
- **`crates/clnrm-core/src/backend/testcontainer.rs`** - Docker/Podman implementation
- **`crates/clnrm-core/src/backend/pool.rs`** ⭐ - Container pooling (v1.4.0+)
- **`crates/clnrm-core/src/backend/engine.rs`** - Backend trait and execution engine

### Service Plugins
- **`crates/clnrm-core/src/services/generic.rs`** - GenericContainerPlugin (any Docker image)
- **`crates/clnrm-core/src/services/surrealdb.rs`** - SurrealDB plugin
- **`crates/clnrm-core/src/services/service_manager.rs`** (20KB) - Service orchestration

### Validation (OTEL)
- **`crates/clnrm-core/src/validation/span_validator.rs`** (43KB) - Core span validation
- **`crates/clnrm-core/src/validation/graph_validator.rs`** - Parent-child structure validation
- **`crates/clnrm-core/src/validation/orchestrator.rs`** - Validation coordinator
- **`crates/clnrm-core/src/validation/hermeticity_validator.rs`** - Isolation/hermetic checks

### Observability
- **`crates/clnrm-core/src/telemetry/init.rs`** - OTEL initialization
- **`crates/clnrm-core/src/telemetry/weaver_controller.rs`** (588 LOC) - Weaver integration
- **`crates/clnrm-core/src/telemetry/exporters.rs`** - Exporter setup (OTLP, Jaeger, Zipkin)

### Output & Reporting
- **`crates/clnrm-core/src/formatting/human.rs`** - Human-readable output
- **`crates/clnrm-core/src/formatting/junit.rs`** - JUnit XML output
- **`crates/clnrm-core/src/reporting/mod.rs`** - Report generation

---

## Code Flow for `clnrm run tests/ --parallel --jobs 4`

```
1. crates/clnrm/src/main.rs
   ↓ tokio::main calls run_cli()
   
2. crates/clnrm-core/src/cli/mod.rs::run_cli()
   ↓ Parses CLI with clap (cli/types.rs)
   ↓ Routes to Commands::Run handler
   
3. crates/clnrm-core/src/cli/commands/run/mod.rs::run_tests_with_shard_and_report()
   ↓ Discovers test files (*.clnrm.toml)
   
4. crates/clnrm-core/src/config/loader.rs
   ↓ Loads and parses TOML files
   ↓ Renders templates (Tera)
   
5. crates/clnrm-core/src/stress_test/executor.rs::execute_tests()
   ↓ Creates TestcontainerBackend
   ↓ Initializes ContainerPool (optional, v1.4.0+)
   ↓ Spawns semaphore-limited concurrent tasks
   
6. For each test:
   ├─ crates/clnrm-core/src/scenario.rs::execute_scenario()
   │  ↓ Execute steps in order
   │  ↓ Collect telemetry (spans)
   │
   ├─ crates/clnrm-core/src/telemetry/span_storage.rs
   │  ↓ Store spans during execution
   │
   └─ crates/clnrm-core/src/validation/orchestrator.rs
      ↓ Coordinate all validators
      ├─ span_validator.rs - Name, kind, attributes
      ├─ graph_validator.rs - Parent-child structure
      ├─ order_validator.rs - Temporal ordering
      ├─ count_validator.rs - Cardinality
      └─ hermeticity_validator.rs - No external calls
      
7. crates/clnrm-core/src/formatting/human.rs
   ↓ Format results for console
   
8. crates/clnrm-core/src/reporting/mod.rs
   ↓ Generate reports (if requested)
   
9. Return results to CLI
```

---

## Module Dependency Tree (Simplified)

```
cleanroom.rs (core)
├── ServicePlugin trait
├── ServiceRegistry
└── CleanroomEnvironment

cli/ (command layer)
├── cli/types.rs - Command definitions
├── cli/commands/
│   ├── run/ - Test execution
│   ├── init/ - Project init
│   ├── validate/ - Config validation
│   └── [20+ others]
└── cli/mod.rs - Router

config/ (configuration)
├── loader.rs - File loading
├── types.rs - Data structures
└── otel.rs - OTEL config

backend/ (container execution)
├── testcontainer.rs - Docker/Podman
├── pool.rs - Container pooling
├── engine.rs - Backend trait
└── mock.rs - Testing backend

services/ (plugins)
├── generic.rs - Generic container
├── surrealdb.rs - Database
├── ollama.rs - LLM serving
└── [5 more plugins]

stress_test/ (execution)
├── executor.rs - Main executor
├── pool.rs - Resource pooling
├── metrics.rs - Performance tracking
└── config.rs - Stress test config

validation/ (telemetry validation)
├── span_validator.rs - Spans
├── graph_validator.rs - Structure
├── order_validator.rs - Temporal
├── count_validator.rs - Cardinality
└── orchestrator.rs - Coordinator

telemetry/ (observability)
├── init.rs - OTEL setup
├── weaver_controller.rs - Weaver integration
├── exporters.rs - Export backends
└── span_storage.rs - Span collection

formatting/ (output)
├── human.rs - Console output
├── json.rs - JSON format
├── junit.rs - JUnit XML
└── tap.rs - TAP format
```

---

## Where to Find...

### I want to understand how tests run
- Start: `crates/clnrm-core/src/cli/commands/run/mod.rs`
- Then: `crates/clnrm-core/src/stress_test/executor.rs`
- Then: `crates/clnrm-core/src/scenario.rs`

### I want to add a new service plugin
- Look at: `crates/clnrm-core/src/services/generic.rs` (template)
- Implement: `ServicePlugin` trait from `crates/clnrm-core/src/cleanroom.rs`
- Register: In `cleanroom.rs::ServiceRegistry::with_default_plugins()`

### I want to add a new CLI command
- Add command: `crates/clnrm-core/src/cli/types.rs::Commands` enum
- Implement: `crates/clnrm-core/src/cli/commands/mycommand.rs`
- Route: `crates/clnrm-core/src/cli/mod.rs::run_cli()` match statement

### I want to understand validation
- Start: `crates/clnrm-core/src/validation/orchestrator.rs` (coordinator)
- Core logic: `crates/clnrm-core/src/validation/span_validator.rs` (43KB)
- Special cases:
  - Graph structure: `graph_validator.rs`
  - Temporal ordering: `order_validator.rs`
  - Isolation: `hermeticity_validator.rs`

### I want to understand container pooling
- See: `crates/clnrm-core/src/backend/pool.rs`
- Configure: `CLNRM_ENABLE_POOLING=1` environment variable
- Performance tuning: `crates/clnrm-core/src/backend/pool.rs::PoolConfig`

### I want to understand Weaver integration
- See: `crates/clnrm-core/src/telemetry/weaver_controller.rs` (588 LOC)
- Configuration: `crates/clnrm-core/src/config/weaver.rs`
- Usage: `--live-check` flag in run command

### I want to understand configuration loading
- See: `crates/clnrm-core/src/config/loader.rs`
- Types: `crates/clnrm-core/src/config/types.rs`
- Template rendering: `clnrm-template` crate

---

## Key Abstractions (Dependency Injection Points)

### `ServicePlugin` trait
```rust
pub trait ServicePlugin: Send + Sync + Debug {
    fn name(&self) -> &str;
    async fn start(&self) -> Result<ServiceHandle>;
    async fn stop(&self, handle: ServiceHandle) -> Result<()>;
    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus;
}
```
**Where:** `cleanroom.rs`  
**Implementations:** GenericContainerPlugin, SurrealDbPlugin, OllamaPlugin, etc.

### `Backend` trait
```rust
pub trait Backend: Send + Sync {
    async fn run_command(&self, cmd: &Cmd) -> Result<RunResult>;
    async fn create_container(&self, config: &ContainerConfig) -> Result<String>;
    async fn stop_container(&self, container_id: &str) -> Result<()>;
}
```
**Where:** `backend/engine.rs`  
**Implementations:** TestcontainerBackend, MockBackend

### `Formatter` trait
```rust
pub trait Formatter {
    fn format_results(&self, results: &TestSuite) -> String;
}
```
**Where:** `formatting/mod.rs`  
**Implementations:** HumanFormatter, JsonFormatter, JunitFormatter, TapFormatter

---

## Performance Tuning Knobs

### Container Pooling (v1.4.0+)
```bash
# Enable pooling (80% faster startup)
CLNRM_ENABLE_POOLING=1 clnrm run tests/ --parallel --jobs 16

# Configure pool size
CLNRM_POOL_MAX_SIZE=50 clnrm run tests/
```
**Code location:** `backend/pool.rs::PoolConfig`

### Concurrent Execution
```bash
# Run with 8 workers
clnrm run --parallel --jobs 8

# Run with all CPU cores
clnrm run --parallel --jobs $(nproc)
```
**Code location:** `stress_test/executor.rs`

### Weaver Validation Mode
```bash
# Strict validation (all schemas must match)
clnrm run --live-check --validation-mode strict

# Lenient mode (warnings only)
clnrm run --live-check --validation-mode lenient

# 80/20 mode (critical paths only)
clnrm run --live-check --validation-mode 80_20
```
**Code location:** `telemetry/weaver_controller.rs`

---

## Testing the Framework Itself

The framework uses "eat your own dog food" - it tests itself using its own capabilities:

```bash
# Run framework self-tests
clnrm self-test

# Run with OTEL validation
clnrm self-test --suite otel --otel-exporter stdout

# Run with Weaver live-check
clnrm self-test --live-check --registry registry/
```

**Test location:** `tests/`, `examples/`

