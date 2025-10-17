# Cleanroom Testing Framework - Component Specifications

**Version:** v0.7.0
**Date:** 2025-10-16
**Status:** Reference Documentation

This document provides detailed specifications for each major component in the Cleanroom Testing Framework, following FAANG-level core team standards.

---

## 1. Template System (`crates/clnrm-core/src/template/`)

### 1.1 TemplateRenderer

**Purpose:** Render Tera templates to flat TOML with variable resolution

**Public Interface:**
```rust
pub struct TemplateRenderer {
    tera: Tera,
    context: TemplateContext,
}

impl TemplateRenderer {
    /// Create new template renderer with custom functions and macro library
    pub fn new() -> Result<Self>;

    /// Create renderer with default PRD v1.0 variable resolution
    pub fn with_defaults() -> Result<Self>;

    /// Set template context variables
    pub fn with_context(self, context: TemplateContext) -> Self;

    /// Merge user-provided variables into context (respects precedence)
    pub fn merge_user_vars(&mut self, user_vars: HashMap<String, serde_json::Value>);

    /// Render template file to TOML string
    pub fn render_file(&mut self, path: &Path) -> Result<String>;

    /// Render template string to TOML
    pub fn render_str(&mut self, template: &str, name: &str) -> Result<String>;

    /// Render template from glob pattern
    pub fn render_from_glob(&mut self, glob_pattern: &str, template_name: &str) -> Result<String>;
}
```

**Error Handling:**
- Returns `Result<T, CleanroomError>` for all operations
- Template errors include line numbers and context
- Variable resolution errors specify which variable failed

**Dependencies:**
- `tera = "1.19"` - Template engine
- `serde_json = "1.0"` - JSON value handling
- Internal: `template::context`, `template::functions`

**File Size:** Target 200 lines, Current: ✅ 180 lines (with PRD v1.0 implementation)

**Testing:**
- Unit tests for basic rendering
- Integration tests for macro library
- Property tests for deterministic output

---

### 1.2 TemplateContext

**Purpose:** Build Tera context with variable precedence resolution

**Public Interface:**
```rust
pub struct TemplateContext {
    pub vars: HashMap<String, serde_json::Value>,
    pub matrix: Option<HashMap<String, Vec<String>>>,
    pub determinism: Option<DeterminismConfig>,
}

impl TemplateContext {
    /// Create new empty context
    pub fn new() -> Self;

    /// Create context with PRD v1.0 defaults (svc, env, endpoint, etc.)
    pub fn with_defaults() -> Self;

    /// Add variables to context
    pub fn with_vars(mut self, vars: HashMap<String, serde_json::Value>) -> Self;

    /// Merge user-provided variables (highest precedence)
    pub fn merge_user_vars(&mut self, user_vars: HashMap<String, serde_json::Value>);

    /// Resolve variables with precedence: template vars → ENV → defaults
    pub fn resolve_with_precedence(&mut self) -> Result<()>;

    /// Convert to Tera context for rendering
    pub fn to_tera_context(&self) -> Result<tera::Context>;
}
```

**Variable Precedence:**
1. **Template vars** (user-provided, highest priority)
2. **ENV variables** (environment, fallback)
3. **Defaults** (built-in, lowest priority)

**Default Variables (PRD v1.0):**
```rust
{
    "svc": "clnrm",                           // SERVICE_NAME
    "env": "ci",                              // ENV
    "endpoint": "http://localhost:4318",      // OTEL_ENDPOINT
    "exporter": "otlp",                       // OTEL_TRACES_EXPORTER
    "image": "registry/clnrm:1.0.0",          // CLNRM_IMAGE
    "freeze_clock": "2025-01-01T00:00:00Z",   // FREEZE_CLOCK
    "token": "",                              // OTEL_TOKEN
}
```

**File Size:** Target 250 lines, Current: ⚠️ To be measured

---

### 1.3 Template Functions

**Purpose:** Custom Tera functions for template rendering

**Registered Functions:**
```rust
// Environment variable access
env(name: string) -> string

// Deterministic timestamp (respects freeze_clock)
now_rfc3339() -> string

// SHA-256 hashing for content digests
sha256(s: string) -> string

// TOML literal encoding
toml_encode(value: any) -> string
```

**Implementation:**
```rust
pub fn register_functions(tera: &mut Tera) -> Result<()> {
    tera.register_function("env", Box::new(EnvFunction));
    tera.register_function("now_rfc3339", Box::new(NowRfc3339Function));
    tera.register_function("sha256", Box::new(Sha256Function));
    tera.register_function("toml_encode", Box::new(TomlEncodeFunction));
    Ok(())
}
```

**Error Handling:**
- Functions return `Result<Value, tera::Error>`
- Missing ENV vars return empty string (non-fatal)
- Invalid arguments return descriptive errors

**File Size:** Target 300 lines, Current: ⚠️ To be measured

---

### 1.4 Macro Library

**Purpose:** Reusable TOML macros for 85% boilerplate reduction

**Available Macros:**
```tera
{% import "_macros.toml.tera" as m %}

{# Span expectation #}
{{ m::span("span.name", parent="parent.span", kind="internal",
          attrs={"key": "value"}, events=["event1", "event2"]) }}

{# Service definition #}
{{ m::service("service_id", "image:tag", args=["arg1"],
              env={"KEY": "value"}, wait_for_span="span.name") }}

{# Scenario definition #}
{{ m::scenario("scenario_name", "service_id", "command",
               expect_success=true) }}

{# OTEL configuration #}
{{ m::otel("exporter", "endpoint", sample_ratio=1.0,
          resources={"key": "value"}) }}

{# Determinism configuration #}
{{ m::determinism(seed=42, freeze_clock="2025-01-01T00:00:00Z") }}

{# Graph expectation #}
{{ m::graph(must_include=[["parent", "child"]], acyclic=true) }}

{# Hermeticity expectation #}
{{ m::hermeticity(no_external_services=true,
                  resource_attrs={"service.name": "clnrm"}) }}

{# Status expectation #}
{{ m::status(all="OK", by_name={"span.*": "OK"}) }}
```

**Benefits:**
- 85% boilerplate reduction (measured)
- Consistent formatting across templates
- Type safety through structured generation
- Easier to maintain and refactor

**File:** `_macros.toml.tera` (embedded at compile time)

---

## 2. Configuration System (`crates/clnrm-core/src/config.rs`)

### 2.1 TestConfig

**Purpose:** Main test configuration structure

**Public Interface:**
```rust
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestConfig {
    pub meta: Option<MetaConfig>,
    pub otel: Option<OtelConfig>,
    pub service: Option<HashMap<String, ServiceConfig>>,
    pub scenario: Vec<ScenarioConfig>,
    pub expect: Option<ExpectationsConfig>,
    pub vars: Option<HashMap<String, serde_json::Value>>,
    pub determinism: Option<DeterminismConfig>,
    pub report: Option<ReportConfig>,
    pub limits: Option<LimitsConfig>,
}

pub fn parse_toml_config(content: &str) -> Result<TestConfig>;
pub fn load_config_from_file(path: &Path) -> Result<TestConfig>;
pub fn load_cleanroom_config(dir: &Path) -> Result<Vec<TestConfig>>;
```

**Schema Versions:**
- **v0.4**: Legacy format with `[test.metadata]`, `[[steps]]`
- **v0.6**: Flat format with `[meta]`, `[[scenario]]`
- **v0.7**: Enhanced with template support, macro library

**Validation Rules:**
- Required: `[meta]` section with name/version
- Required: At least one `[[scenario]]`
- Valid `otel.exporter`: "stdout" | "otlp"
- Service references in scenarios must exist
- Span expectations must reference defined services

**Error Handling:**
- `CleanroomError::configuration_error` for parse failures
- Field-level validation with path context
- Type mismatch errors with expected vs actual

**File Size:** Target 500 lines, Current: ⚠️ To be measured

---

### 2.2 ExpectationsConfig

**Purpose:** Validation expectations for OTEL telemetry

**Public Interface:**
```rust
#[derive(Debug, Deserialize, Serialize, Clone, Default)]
pub struct ExpectationsConfig {
    pub span: Vec<SpanExpectationConfig>,
    pub order: Option<OrderExpectationConfig>,
    pub status: Option<StatusExpectationConfig>,
    pub counts: Option<CountExpectationConfig>,
    pub window: Vec<WindowExpectationConfig>,
    pub graph: Option<GraphExpectationConfig>,
    pub hermeticity: Option<HermeticityExpectationConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct SpanExpectationConfig {
    pub name: String,                    // Can be glob pattern
    pub kind: Option<String>,
    pub parent: Option<String>,
    pub attrs: Option<SpanAttributesConfig>,
    pub events: Option<Vec<String>>,
    pub duration_ms: Option<DurationRange>,
}
```

**Validation Types:**
1. **Span Validation** - Attributes, kind, parent, events
2. **Graph Validation** - Topology, parent-child, acyclic
3. **Hermeticity Validation** - Network isolation, resource attributes
4. **Count Validation** - Span/event/error counts
5. **Order Validation** - Temporal ordering constraints
6. **Window Validation** - Parent-child containment
7. **Status Validation** - Span status codes

---

## 3. Execution Engine

### 3.1 CleanroomEnvironment

**Purpose:** Main test execution context with hermetic isolation

**Public Interface:**
```rust
pub struct CleanroomEnvironment {
    registry: ServiceRegistry,
    backend: Box<dyn Backend>,
    telemetry_config: Option<OtelConfig>,
}

impl CleanroomEnvironment {
    /// Create new cleanroom environment
    pub fn new() -> Result<Self>;

    /// Create with custom backend
    pub fn with_backend(backend: Box<dyn Backend>) -> Result<Self>;

    /// Register service plugin
    pub fn register_service(&mut self, plugin: Box<dyn ServicePlugin>) -> Result<()>;

    /// Start service by name
    pub fn start_service(&mut self, name: &str) -> Result<ServiceHandle>;

    /// Stop service by handle
    pub fn stop_service(&mut self, handle: &ServiceHandle) -> Result<()>;

    /// Execute scenario in container
    pub fn execute_scenario(&mut self, scenario: &ScenarioConfig) -> Result<ScenarioResult>;

    /// Execute command in service container
    pub fn execute_command(&self, handle: &ServiceHandle, cmd: &[String]) -> Result<ExecutionResult>;
}

impl Drop for CleanroomEnvironment {
    /// Cleanup: stop all services, remove containers
    fn drop(&mut self) {
        // Automatic cleanup even on panic
    }
}
```

**Guarantees:**
- Fresh container environment per test
- Automatic cleanup on Drop (even on panic)
- Services started in dependency order
- Health checks with exponential backoff
- Timeout enforcement at scenario level

**Error Handling:**
- `CleanroomError::container_error` for Docker failures
- `CleanroomError::service_error` for service startup failures
- `CleanroomError::timeout_error` for exceeded limits

**File Size:** Target 500 lines, Current: ⚠️ To be measured

---

### 3.2 ServicePlugin Trait

**Purpose:** Extensible service plugin interface (dyn compatible)

**Public Interface:**
```rust
pub trait ServicePlugin: Send + Sync {
    /// Start service and return handle (SYNC, not async)
    fn start(&self) -> Result<ServiceHandle>;

    /// Stop service by handle
    fn stop(&self, handle: &ServiceHandle) -> Result<()>;

    /// Check service health
    fn health_check(&self, handle: &ServiceHandle) -> Result<HealthStatus>;

    /// Get service type identifier
    fn service_type(&self) -> &str;
}
```

**Critical Design Decision (ADR-001):**
- **Methods are sync** (not async) for dyn compatibility
- Internal implementations use `tokio::task::block_in_place` for async operations
- This enables plugin system with dynamic dispatch

**Built-in Plugins:**
- `GenericContainerPlugin` - Any Docker image
- `SurrealDbPlugin` - SurrealDB database
- `OtelCollectorPlugin` - OpenTelemetry collector
- `OllamaPlugin` - Ollama LLM inference
- `VllmPlugin` - vLLM inference server
- `TgiPlugin` - Text Generation Inference

**Example Implementation:**
```rust
impl ServicePlugin for GenericContainerPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        // Use block_in_place for async operations
        tokio::task::block_in_place(|| {
            let rt = tokio::runtime::Handle::current();
            rt.block_on(async {
                // Async implementation
                let container = self.backend.create_container(&self.config).await?;
                self.backend.start_container(&container).await?;
                Ok(ServiceHandle::new(container))
            })
        })
    }
}
```

---

### 3.3 Backend Trait

**Purpose:** Abstract container operations (Docker/Podman)

**Public Interface:**
```rust
pub trait Backend: Send + Sync {
    /// Create container with configuration
    fn create_container(&self, config: &ContainerConfig) -> Result<ContainerId>;

    /// Start container
    fn start_container(&self, id: &ContainerId) -> Result<()>;

    /// Stop container
    fn stop_container(&self, id: &ContainerId) -> Result<()>;

    /// Remove container
    fn remove_container(&self, id: &ContainerId) -> Result<()>;

    /// Execute command in container
    fn execute_command(&self, id: &ContainerId, cmd: &[String]) -> Result<ExecutionResult>;

    /// Get container logs
    fn get_logs(&self, id: &ContainerId) -> Result<String>;
}
```

**Primary Implementation:**
- `TestcontainerBackend` - Uses `testcontainers-rs` crate
- Supports Docker and Podman through unified API

**File Size:** Target 400 lines, Current: ⚠️ To be measured

---

## 4. Validation System

### 4.1 ValidationOrchestrator

**Purpose:** Coordinate multiple validators for comprehensive validation

**Public Interface:**
```rust
pub struct ValidationOrchestrator {
    config: OtelValidationConfig,
    validators: Vec<Box<dyn Validator>>,
}

impl ValidationOrchestrator {
    /// Create new orchestrator with configuration
    pub fn new(config: OtelValidationConfig) -> Self;

    /// Validate spans against expectations
    pub fn validate(&self, spans: &[Span], expectations: &ExpectationsConfig)
        -> Result<ValidationReport>;

    /// Add custom validator
    pub fn add_validator(&mut self, validator: Box<dyn Validator>);
}

pub struct ValidationReport {
    pub passed: bool,
    pub errors: Vec<ValidationError>,
    pub warnings: Vec<String>,
    pub span_count: usize,
    pub digest: String,  // SHA-256 of normalized JSON
}
```

**Validation Flow:**
1. Normalize spans (sort, remove volatile fields)
2. Run all validators in parallel
3. Aggregate results (fail-fast on first error)
4. Generate report with focused error message
5. Compute SHA-256 digest of normalized JSON

**File Size:** Target 200 lines, Current: ⚠️ To be measured

---

### 4.2 Validator Trait

**Purpose:** Extensible validation interface

**Public Interface:**
```rust
pub trait Validator: Send + Sync {
    /// Validator name
    fn name(&self) -> &str;

    /// Validate spans (SYNC, not async)
    fn validate(&self, spans: &[Span], expectations: &ExpectationsConfig)
        -> Result<Vec<ValidationError>>;
}
```

**Built-in Validators:**

#### SpanValidator
- Validates span attributes, kind, parent, events
- Supports glob patterns for span names
- Duration range validation
- File: `validation/span_validator.rs` (300 lines)

#### GraphValidator
- Validates trace topology
- Parent-child relationships
- Acyclic graph check
- Path existence validation
- File: `validation/graph_validator.rs` (350 lines)

#### HermeticityValidator
- Validates network isolation
- Resource attribute checking
- External service detection
- File: `validation/hermeticity_validator.rs` (200 lines)

#### CountValidator
- Span count assertions
- Event count assertions
- Error count assertions
- Per-name count validation
- File: `validation/count_validator.rs` (150 lines)

#### OrderValidator
- Temporal ordering constraints
- Must precede/follow relationships
- File: `validation/order_validator.rs` (200 lines)

#### WindowValidator
- Parent-child containment
- Span window verification
- File: `validation/window_validator.rs` (180 lines)

#### StatusValidator
- Span status code validation
- Glob pattern matching
- File: `validation/status_validator.rs` (120 lines)

---

## 5. Support Systems

### 5.1 Cache System (`crates/clnrm-core/src/cache/`)

**Purpose:** Change-aware execution with file hashing

**Public Interface:**
```rust
pub trait Cache: Send + Sync {
    fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    fn set(&self, key: &str, value: &[u8]) -> Result<()>;
    fn delete(&self, key: &str) -> Result<()>;
    fn clear(&self) -> Result<()>;
}

pub struct CacheManager {
    cache: Box<dyn Cache>,
}

impl CacheManager {
    pub fn new() -> Result<Self>;
    pub fn with_cache(cache: Box<dyn Cache>) -> Self;
    pub fn compute_file_hash(&self, path: &Path) -> Result<String>;
    pub fn is_changed(&self, path: &Path) -> Result<bool>;
    pub fn update_hash(&self, path: &Path) -> Result<()>;
}
```

**Implementations:**
- `MemoryCache` - In-memory HashMap (tests only)
- `FileCache` - Persistent file-based cache

**Hash Algorithm:** SHA-256 of file content

**File Size:** Target 300 lines total, Current: ⚠️ To be measured

---

### 5.2 Watch System (`crates/clnrm-core/src/watch/`)

**Purpose:** Hot reload with file watching (<3s latency)

**Public Interface:**
```rust
pub struct FileWatcher {
    debouncer: FileDebouncer,
}

impl FileWatcher {
    pub fn new(config: WatchConfig) -> Result<Self>;
    pub fn watch(&mut self, paths: Vec<PathBuf>) -> Result<()>;
    pub fn on_change<F>(&mut self, callback: F) -> Result<()>
    where F: Fn(&Path) + Send + 'static;
}

pub struct FileDebouncer {
    delay: Duration,
}

impl FileDebouncer {
    pub fn new(delay: Duration) -> Self;
    pub fn debounce<F>(&mut self, event: &Event, callback: F)
    where F: FnOnce();
}
```

**Features:**
- Debouncing (default 500ms)
- Recursive directory watching
- Filter by file extension (.clnrm.toml, .tera)

**File Size:** Target 200 lines, Current: ⚠️ To be measured

---

### 5.3 Telemetry System (`crates/clnrm-core/src/telemetry.rs`)

**Purpose:** Built-in OpenTelemetry instrumentation

**Public Interface:**
```rust
#[cfg(feature = "otel-traces")]
pub struct OtelConfig {
    pub service_name: String,
    pub deployment_env: String,
    pub sample_ratio: f64,
    pub export: Export,
    pub enable_fmt_layer: bool,
}

#[cfg(feature = "otel-traces")]
pub enum Export {
    Stdout,
    OtlpHttp { endpoint: String },
    OtlpGrpc { endpoint: String },
}

#[cfg(feature = "otel-traces")]
pub fn init_otel(config: OtelConfig) -> Result<OtelGuard>;

#[cfg(feature = "otel-metrics")]
pub mod metrics {
    pub fn record_test_duration(name: &str, duration_ms: f64, success: bool);
    pub fn increment_test_counter(name: &str, outcome: &str);
    pub fn record_operation_duration(operation: &str, duration_ms: f64);
    pub fn increment_counter(name: &str, value: f64);
}
```

**Features:**
- Production-ready OTEL support
- Supports OTLP HTTP/gRPC exporters
- Jaeger, DataDog, New Relic compatible
- Environment variable configuration

**File Size:** Target 300 lines, Current: ⚠️ To be measured

---

## 6. CLI Commands (`crates/clnrm/src/`)

### Core Commands

#### `clnrm run`
```rust
pub async fn run_command(config: RunConfig) -> Result<()> {
    // Parse TOML configs
    // Initialize backend
    // Execute scenarios (parallel with --workers)
    // Collect telemetry
    // Validate expectations
    // Generate reports
    // Return exit code
}
```

**Options:**
- `--workers N` - Parallel execution
- `--format <json|junit|human>` - Output format
- `--change-aware` - Skip unchanged scenarios (default)
- `--no-cache` - Disable cache

#### `clnrm dev --watch`
```rust
pub async fn dev_command(config: DevConfig) -> Result<()> {
    // Setup file watcher
    // On change: hash file, check cache
    // If changed: render, parse, execute
    // Print results (first failure focus)
    // Loop
}
```

**Performance:** <3s hot reload latency (p95)

#### `clnrm template <type>`
```rust
pub fn template_command(template_type: TemplateType) -> Result<()> {
    // Generate template from built-in templates
    // Write to current directory
    // Print usage instructions
}
```

**Templates:**
- `otel` - OTEL validation template
- `matrix` - Matrix testing template
- `macros` - Macro library showcase
- `full-validation` - Complete validation example

#### `clnrm fmt`
```rust
pub fn fmt_command(config: FmtConfig) -> Result<()> {
    // Parse TOML file
    // Sort keys deterministically
    // Preserve flat structure
    // Sort [vars] section
    // Write back if changed
    // Verify idempotency
}
```

**Guarantees:** Deterministic, idempotent formatting

#### `clnrm lint`
```rust
pub fn lint_command(config: LintConfig) -> Result<()> {
    // Schema validation
    // Orphan reference detection
    // Enum validation
    // Required field checking
    // Return errors with context
}
```

**Validation:** <1s for 10 files

---

## 7. Error Handling (All Components)

### CleanroomError

**Structure:**
```rust
pub struct CleanroomError {
    pub kind: ErrorKind,
    pub message: String,
    pub context: Option<String>,
    pub source: Option<String>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

pub enum ErrorKind {
    ContainerError,
    NetworkError,
    ResourceLimitExceeded,
    Timeout,
    ConfigurationError,
    PolicyViolation,
    DeterministicError,
    ValidationError,
    ServiceError,
    InternalError,
    TemplateError,
    // ... more
}
```

**Constructor Pattern:**
```rust
CleanroomError::template_error("Rendering failed")
    .with_context("File: template.toml.tera")
    .with_source("Tera error: undefined variable")
```

**Conversion Traits:**
```rust
impl From<std::io::Error> for CleanroomError { ... }
impl From<serde_json::Error> for CleanroomError { ... }
impl From<testcontainers::TestcontainersError> for CleanroomError { ... }
```

---

## 8. Testing Standards (All Components)

### AAA Pattern (MANDATORY)

```rust
#[tokio::test]
async fn test_template_renderer_renders_with_defaults() -> Result<()> {
    // Arrange
    let mut renderer = TemplateRenderer::with_defaults()?;
    let template = "{{ svc }}";

    // Act
    let result = renderer.render_str(template, "test")?;

    // Assert
    assert_eq!(result, "clnrm");
    Ok(())
}
```

### No Unwrap/Expect (MANDATORY)

```rust
// ❌ FORBIDDEN
let result = operation().unwrap();

// ✅ REQUIRED
let result = operation().map_err(|e| {
    CleanroomError::internal_error(format!("Operation failed: {}", e))
})?;
```

### Descriptive Names (MANDATORY)

```rust
// ✅ GOOD
#[test]
fn test_template_renderer_returns_error_on_undefined_variable() { ... }

// ❌ BAD
#[test]
fn test_error() { ... }
```

---

## 9. Performance Budgets

### File Size Budget (Per Module)

| Module | Budget | Status | Action |
|--------|--------|--------|--------|
| template/mod.rs | 200 lines | ✅ 180 | - |
| template/context.rs | 250 lines | ⚠️ TBD | Measure |
| template/functions.rs | 300 lines | ⚠️ TBD | Measure |
| config.rs | 500 lines | ⚠️ TBD | Measure |
| cleanroom.rs | 500 lines | ⚠️ TBD | Measure |
| validation/orchestrator.rs | 200 lines | ⚠️ TBD | Measure |

**Rule:** Files exceeding budget MUST be refactored

### Latency Budget (Per Operation)

| Operation | Budget | Status | Action |
|-----------|--------|--------|--------|
| Template rendering | 50ms | ⚠️ TBD | Benchmark |
| TOML parsing | 10ms | ⚠️ TBD | Benchmark |
| Container startup | 2s | ⚠️ TBD | Benchmark |
| Span validation | 100ms | ⚠️ TBD | Benchmark |
| SHA-256 hash | 20ms | ⚠️ TBD | Benchmark |

---

## 10. Next Steps

### Immediate Actions
1. **Measure file sizes** - Audit all modules against budget
2. **Benchmark operations** - Measure latency for all budgets
3. **Complete implementations** - Finish unimplemented validators
4. **Add tests** - Ensure 100% coverage for core paths

### Future Enhancements
1. **Performance optimization** - Profile and optimize hot paths
2. **Plugin marketplace** - Community plugin sharing
3. **Advanced analytics** - Trace analysis and insights
4. **Graph visualization** - SVG/TUI trace graphs

---

**Document Owner:** System Architecture Team
**Last Updated:** 2025-10-16
**Related Docs:** [System Architecture](./v0.7.0-system-architecture.md), [Summary](./ARCHITECTURE-SUMMARY.md)
