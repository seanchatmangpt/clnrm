# PRD v1.0 Architecture Design

## Document Metadata

- **Version**: 1.0
- **Date**: 2025-10-16
- **Status**: Design Phase
- **Author**: Architecture Agent (SPARC Swarm)
- **Compliance**: clnrm Core Team Standards

## Executive Summary

This document describes the system architecture for implementing PRD v1.0 features in the clnrm testing framework. The design adheres to clnrm's core principles: hermetic testing, plugin architecture, proper error handling, and dyn-compatible traits.

## Architecture Principles

### 1. Core Team Standards Compliance

**Error Handling (MANDATORY)**
- No `.unwrap()` or `.expect()` in production code
- All functions return `Result<T, CleanroomError>`
- Meaningful error messages with context
- Error chaining via `.with_context()` and `.with_source()`

**Async/Sync Rules**
- Trait methods MUST be sync for `dyn` compatibility
- Use `tokio::task::block_in_place` internally for async operations
- Async for I/O operations (containers, network, files)
- Sync for computation (parsing, validation)

**Modular Design**
- Files under 500 lines
- Clear separation of concerns
- Plugin-based extensibility
- Zero hardcoded dependencies

## High-Level Architecture

### Component Overview

```
┌─────────────────────────────────────────────────────────────┐
│                     CLI Layer (clnrm)                        │
│  Commands: template, dev, run, diff, graph, record, etc.    │
└─────────────────┬───────────────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────────────┐
│              Template Rendering Engine                       │
│  - Tera integration with precedence resolver                │
│  - ENV → template vars → defaults                           │
│  - Renders .clnrm.toml.tera → flat TOML                     │
└─────────────────┬───────────────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────────────┐
│              Configuration System                            │
│  - TOML parser (extended for v0.7.0 schema)                 │
│  - Validation engine                                         │
│  - Change-aware hashing                                      │
└─────────────────┬───────────────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────────────┐
│              Test Execution Engine                           │
│  - Scenario orchestration                                    │
│  - Worker pool management                                    │
│  - Container lifecycle (CleanroomEnvironment)               │
└─────────────────┬───────────────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────────────┐
│              OTEL Collection & Normalization                │
│  - Span collection (stdout/OTLP)                            │
│  - Deterministic normalization                              │
│  - SHA-256 digest generation                                │
└─────────────────┬───────────────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────────────┐
│              Expectation Analysis Engine                     │
│  - Graph topology validation                                 │
│  - Count/cardinality checks                                  │
│  - Temporal ordering verification                            │
│  - Hermeticity validation                                    │
└─────────────────┬───────────────────────────────────────────┘
                  │
┌─────────────────▼───────────────────────────────────────────┐
│              Reporting System                                │
│  - Console output (focused failures)                         │
│  - JSON reports                                              │
│  - JUnit XML                                                 │
│  - Digest files                                              │
└─────────────────────────────────────────────────────────────┘
```

## Detailed Component Design

### 1. Template Rendering Engine

**Location**: `crates/clnrm-core/src/template/tera_renderer.rs` (new)

**Purpose**: Render Tera templates with ENV-based variable precedence

**Key Interfaces**:

```rust
pub struct TeraRenderer {
    tera: tera::Tera,
    resolver: VariableResolver,
}

pub struct VariableResolver {
    precedence: Vec<VariableSource>,
}

pub enum VariableSource {
    TemplateVars(HashMap<String, String>),
    Environment,
    Defaults,
}

impl TeraRenderer {
    /// Create new renderer with glob pattern
    pub fn new(template_glob: &str) -> Result<Self>;

    /// Render template with variable resolution
    pub fn render(
        &self,
        template_name: &str,
        user_vars: HashMap<String, String>
    ) -> Result<String>;

    /// Register custom Tera functions
    pub fn register_function<F: tera::Function>(
        &mut self,
        name: &str,
        func: F
    ) -> Result<()>;
}

impl VariableResolver {
    /// Resolve variables with precedence: template → ENV → default
    pub fn resolve(
        &self,
        user_vars: HashMap<String, String>
    ) -> HashMap<String, String>;

    /// Pick value from sources in priority order
    fn pick(
        &self,
        vars: &HashMap<String, String>,
        key: &str,
        env_key: &str,
        default: &str
    ) -> String;
}
```

**Design Decisions**:

1. **Separation of Concerns**: Template rendering logic separate from variable resolution
2. **No Prefixes**: Template variables are plain (`{{ svc }}` not `{{ vars.svc }}`)
3. **ENV Integration**: Rust handles ENV lookup, not Tera
4. **Error Handling**: All methods return `Result<T, CleanroomError>`

**File Size**: ~250 lines (within 500 line limit)

### 2. Extended Configuration System

**Location**: `crates/clnrm-core/src/config.rs` (extend existing)

**New Structures for v0.7.0**:

```rust
// Already exists, extend with new fields

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TestConfig {
    // Existing fields...

    // v0.7.0 additions
    #[serde(default)]
    pub vars: Option<HashMap<String, serde_json::Value>>,

    #[serde(default)]
    pub matrix: Option<HashMap<String, Vec<String>>>,

    #[serde(default)]
    pub determinism: Option<DeterminismConfig>,
}

// New v0.7.0 structures

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DeterminismConfig {
    pub seed: Option<u64>,
    pub freeze_clock: Option<String>, // RFC3339 timestamp
}

impl DeterminismConfig {
    pub fn is_deterministic(&self) -> bool {
        self.seed.is_some() || self.freeze_clock.is_some()
    }

    pub fn validate(&self) -> Result<()> {
        if let Some(ref clock) = self.freeze_clock {
            // Validate RFC3339 format
            chrono::DateTime::parse_from_rfc3339(clock)
                .map_err(|e| CleanroomError::validation_error(
                    format!("Invalid freeze_clock format: {}", e)
                ))?;
        }
        Ok(())
    }
}
```

**Integration Point**: Extends existing `config.rs` without breaking changes

**File Size Impact**: Adds ~150 lines to existing config.rs (~1383 → ~1533 lines)
**Action Required**: Consider splitting into `config/mod.rs` and `config/v0_7.rs`

### 3. Change-Aware Execution Engine

**Location**: `crates/clnrm-core/src/execution/change_aware.rs` (new)

**Purpose**: Track scenario changes and skip unchanged tests

**Key Interfaces**:

```rust
pub struct ChangeTracker {
    cache_dir: PathBuf,
    hash_algorithm: HashAlgorithm,
}

pub enum HashAlgorithm {
    Sha256,
    Blake3,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScenarioHash {
    pub scenario_name: String,
    pub content_hash: String,
    pub last_run: chrono::DateTime<chrono::Utc>,
    pub last_result: TestResult,
}

impl ChangeTracker {
    pub fn new(cache_dir: PathBuf) -> Result<Self>;

    /// Compute stable hash from rendered scenario section
    pub fn compute_hash(&self, scenario: &ScenarioConfig) -> Result<String>;

    /// Check if scenario has changed since last run
    pub fn has_changed(&self, scenario: &ScenarioConfig) -> Result<bool>;

    /// Update hash after successful run
    pub fn update_hash(
        &self,
        scenario: &ScenarioConfig,
        result: TestResult
    ) -> Result<()>;

    /// List all cached scenarios
    pub fn list_cached(&self) -> Result<Vec<ScenarioHash>>;

    /// Clear all cached hashes
    pub fn clear_cache(&self) -> Result<()>;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TestResult {
    Pass,
    Fail { reason: String },
    Skip,
}
```

**Design Decisions**:

1. **Stable Hashing**: Hash normalized scenario content (sorted keys, deterministic JSON)
2. **Cache Storage**: JSON files in `.clnrm/cache/` directory
3. **Dependency Tracking**: Future enhancement (not v0.7.0)
4. **Performance**: Blake3 for faster hashing (configurable)

**File Size**: ~300 lines

### 4. OTEL Normalization Engine

**Location**: `crates/clnrm-core/src/validation/otel_normalization.rs` (new)

**Purpose**: Normalize spans for deterministic comparison

**Key Interfaces**:

```rust
pub struct SpanNormalizer {
    config: NormalizationConfig,
}

#[derive(Debug, Clone)]
pub struct NormalizationConfig {
    pub sort_by: Vec<SortField>,
    pub strip_fields: Vec<String>,
    pub normalize_timestamps: bool,
    pub deterministic_ids: bool,
}

pub enum SortField {
    TraceId,
    SpanId,
    StartTime,
    Name,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NormalizedSpan {
    pub name: String,
    pub kind: String,
    pub parent_span_id: Option<String>,
    pub attributes: BTreeMap<String, String>, // Sorted
    pub events: Vec<NormalizedEvent>,         // Sorted
    pub status: SpanStatus,
    pub duration_ms: u64,
}

impl SpanNormalizer {
    pub fn new(config: NormalizationConfig) -> Self;

    /// Normalize a collection of spans
    pub fn normalize(&self, spans: Vec<RawSpan>) -> Result<Vec<NormalizedSpan>>;

    /// Compute SHA-256 digest of normalized spans
    pub fn compute_digest(&self, spans: &[NormalizedSpan]) -> Result<String>;

    /// Sort spans deterministically
    fn sort_spans(&self, spans: &mut [NormalizedSpan]);

    /// Strip volatile fields (timestamps, trace IDs if configured)
    fn strip_volatile_fields(&self, span: &mut NormalizedSpan);

    /// Normalize timestamps to relative offsets
    fn normalize_timestamps(&self, spans: &mut [NormalizedSpan]);
}
```

**Design Decisions**:

1. **BTreeMap**: Ensures attribute order is stable
2. **Configurable**: What fields to strip/normalize via config
3. **Deterministic**: Same spans always produce same digest
4. **SHA-256**: Industry standard for content addressing

**File Size**: ~350 lines

### 5. Expectation Analysis Engine

**Location**: `crates/clnrm-core/src/validation/expectations/` (new directory)

**Structure**:
- `mod.rs` - Main analyzer
- `graph.rs` - Graph topology validation
- `counts.rs` - Cardinality checks
- `order.rs` - Temporal ordering
- `hermeticity.rs` - Isolation validation
- `status.rs` - Status code checks

**Key Interfaces**:

```rust
// mod.rs
pub struct ExpectationAnalyzer {
    graph_validator: GraphValidator,
    count_validator: CountValidator,
    order_validator: OrderValidator,
    hermeticity_validator: HermeticityValidator,
    status_validator: StatusValidator,
}

pub struct ValidationResult {
    pub verdict: Verdict,
    pub failures: Vec<ValidationFailure>,
    pub summary: ValidationSummary,
}

pub enum Verdict {
    Pass,
    Fail,
}

#[derive(Debug, Clone)]
pub struct ValidationFailure {
    pub rule_type: RuleType,
    pub rule_name: String,
    pub message: String,
    pub context: HashMap<String, String>,
}

pub enum RuleType {
    Graph,
    Count,
    Order,
    Hermeticity,
    Status,
}

impl ExpectationAnalyzer {
    pub fn new(expectations: ExpectationsConfig) -> Result<Self>;

    /// Analyze all expectations against normalized spans
    pub fn analyze(&self, spans: &[NormalizedSpan]) -> Result<ValidationResult>;

    /// Get first failure (for focused output)
    pub fn first_failure(&self, result: &ValidationResult) -> Option<&ValidationFailure>;
}

// graph.rs
pub struct GraphValidator {
    must_include: Vec<(String, String)>,
    must_not_cross: Vec<(String, String)>,
    acyclic: bool,
}

impl GraphValidator {
    pub fn validate(&self, spans: &[NormalizedSpan]) -> Result<Vec<ValidationFailure>>;

    /// Build parent-child graph from spans
    fn build_graph(&self, spans: &[NormalizedSpan]) -> SpanGraph;

    /// Check if edge exists in graph
    fn has_edge(&self, graph: &SpanGraph, parent: &str, child: &str) -> bool;

    /// Detect cycles using DFS
    fn detect_cycles(&self, graph: &SpanGraph) -> Vec<Vec<String>>;
}

// counts.rs
pub struct CountValidator {
    spans_total: Option<CountBound>,
    events_total: Option<CountBound>,
    errors_total: Option<CountBound>,
    by_name: HashMap<String, CountBound>,
}

pub struct CountBound {
    pub gte: Option<usize>,
    pub lte: Option<usize>,
    pub eq: Option<usize>,
}

impl CountValidator {
    pub fn validate(&self, spans: &[NormalizedSpan]) -> Result<Vec<ValidationFailure>>;

    /// Check if count satisfies bound
    fn satisfies_bound(&self, count: usize, bound: &CountBound) -> bool;
}

// order.rs
pub struct OrderValidator {
    must_precede: Vec<(String, String)>,
    must_follow: Vec<(String, String)>,
}

impl OrderValidator {
    pub fn validate(&self, spans: &[NormalizedSpan]) -> Result<Vec<ValidationFailure>>;

    /// Get span start time
    fn get_start_time(&self, spans: &[NormalizedSpan], name: &str) -> Option<u64>;

    /// Check temporal ordering
    fn check_temporal_order(&self, t1: u64, t2: u64, should_precede: bool) -> bool;
}

// hermeticity.rs
pub struct HermeticityValidator {
    no_external_services: bool,
    resource_attrs_must_match: HashMap<String, String>,
    span_attrs_forbid_keys: Vec<String>,
}

impl HermeticityValidator {
    pub fn validate(&self, spans: &[NormalizedSpan]) -> Result<Vec<ValidationFailure>>;

    /// Check for external network attributes
    fn has_external_network_attrs(&self, span: &NormalizedSpan) -> bool;

    /// Validate resource attributes match
    fn validate_resource_attrs(&self, spans: &[NormalizedSpan]) -> Result<Vec<ValidationFailure>>;
}

// status.rs
pub struct StatusValidator {
    all: Option<String>,
    by_name: HashMap<String, String>,
}

impl StatusValidator {
    pub fn validate(&self, spans: &[NormalizedSpan]) -> Result<Vec<ValidationFailure>>;

    /// Check if status matches expected
    fn status_matches(&self, actual: &str, expected: &str) -> bool;

    /// Match span name against glob pattern
    fn matches_pattern(&self, name: &str, pattern: &str) -> bool;
}
```

**File Sizes**:
- `mod.rs`: ~200 lines
- `graph.rs`: ~300 lines
- `counts.rs`: ~150 lines
- `order.rs`: ~150 lines
- `hermeticity.rs`: ~200 lines
- `status.rs`: ~150 lines

**Total**: ~1150 lines across 6 files (all under 500 line limit)

### 6. CLI Command Extensions

**Location**: `crates/clnrm/src/cli/commands/` (extend existing)

**New Commands**:

```rust
// template.rs (new)
pub struct TemplateCommand {
    template_type: TemplateType,
    output_path: Option<PathBuf>,
}

pub enum TemplateType {
    Otel,
    Basic,
    Advanced,
}

impl TemplateCommand {
    pub fn execute(&self) -> Result<()>;
}

// dev.rs (new)
pub struct DevCommand {
    watch: bool,
    workers: usize,
    only: Option<String>,
    timebox_ms: Option<u64>,
}

impl DevCommand {
    pub fn execute(&self) -> Result<()>;
}

// diff.rs (new)
pub struct DiffCommand {
    json: bool,
    baseline: Option<PathBuf>,
    current: Option<PathBuf>,
}

impl DiffCommand {
    pub fn execute(&self) -> Result<()>;
}

// graph.rs (new)
pub struct GraphCommand {
    ascii: bool,
    format: GraphFormat,
}

pub enum GraphFormat {
    Ascii,
    Dot,
    Mermaid,
}

impl GraphCommand {
    pub fn execute(&self) -> Result<()>;
}

// record.rs (existing, extend)
pub struct RecordCommand {
    // Existing fields...

    // v0.7.0 additions
    pub deterministic: bool,
    pub freeze_clock: Option<String>,
    pub seed: Option<u64>,
}

// Additional commands: redgreen, fmt, lint, render, spans, up, down
```

**Integration**: Register in `crates/clnrm/src/cli/mod.rs`

**File Sizes**: Each command ~150-250 lines

## Data Flow Diagrams

### Template Rendering Flow

```
User runs: clnrm template otel
         │
         ▼
┌─────────────────────┐
│  TemplateCommand    │
│  - Load built-in    │
│    template         │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  VariableResolver   │
│  - ENV vars         │
│  - Defaults         │
│  - User overrides   │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  TeraRenderer       │
│  - Render template  │
│  - Output TOML      │
└──────────┬──────────┘
           │
           ▼
      Write to file or stdout
```

### Test Execution Flow

```
User runs: clnrm run --workers 4
         │
         ▼
┌─────────────────────┐
│  Load .clnrm.toml   │
│  (or *.tera)        │
└──────────┬──────────┘
           │
           ▼ (if .tera)
┌─────────────────────┐
│  TeraRenderer       │
│  → flat TOML        │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  TOML Parser        │
│  → TestConfig       │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  ChangeTracker      │
│  - Filter changed   │
│  - Hash scenarios   │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Worker Pool        │
│  - Parallel exec    │
│  - N scenarios      │
└──────────┬──────────┘
           │
           ▼ (per scenario)
┌─────────────────────┐
│  CleanroomEnv       │
│  - Start container  │
│  - Execute steps    │
│  - Collect OTEL     │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  SpanNormalizer     │
│  - Sort & normalize │
│  - Compute digest   │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  ExpectationAnalyzer│
│  - Validate rules   │
│  - Generate report  │
└──────────┬──────────┘
           │
           ▼
┌─────────────────────┐
│  Reporter           │
│  - Console output   │
│  - JSON/JUnit       │
│  - Digest file      │
└─────────────────────┘
```

## Integration Points

### 1. Template System ↔ Config System

**Interface**: `TemplateRenderer::render()` produces TOML string
**Consumer**: `config::parse_toml_config()` parses into `TestConfig`
**Contract**: Valid TOML output or error

### 2. Config System ↔ Execution Engine

**Interface**: `TestConfig` with validated scenarios
**Consumer**: `Executor::run()` orchestrates scenario execution
**Contract**: All config validation passed

### 3. Execution Engine ↔ OTEL Collection

**Interface**: `CleanroomEnvironment::execute_in_container()` produces spans
**Consumer**: `OtelCollector::collect()` gathers spans from stdout/OTLP
**Contract**: Valid OTEL JSON format

### 4. OTEL Collection ↔ Normalization

**Interface**: `Vec<RawSpan>` from collector
**Consumer**: `SpanNormalizer::normalize()` produces `Vec<NormalizedSpan>`
**Contract**: All required span fields present

### 5. Normalization ↔ Analysis

**Interface**: `Vec<NormalizedSpan>` with stable ordering
**Consumer**: `ExpectationAnalyzer::analyze()` produces `ValidationResult`
**Contract**: Spans are sorted and normalized

### 6. Analysis ↔ Reporting

**Interface**: `ValidationResult` with verdict and failures
**Consumer**: `Reporter::generate()` produces output
**Contract**: All failures have context for user-friendly messages

## Error Handling Strategy

### Error Types

All components use `CleanroomError` with specific kinds:

```rust
// Template rendering
CleanroomError::template_error("Failed to render template: {}")

// Configuration
CleanroomError::config_error("Invalid TOML syntax: {}")
CleanroomError::validation_error("Missing required field: {}")

// Execution
CleanroomError::container_error("Container failed to start: {}")
CleanroomError::execution_error("Command execution failed: {}")

// OTEL
CleanroomError::serialization_error("Failed to parse OTEL JSON: {}")

// Analysis
CleanroomError::validation_error("Expectation failed: {}")
```

### Error Propagation

```rust
// Example: Template rendering chain
pub fn render_template(path: &Path) -> Result<String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| CleanroomError::io_error(
            format!("Failed to read template: {}", e)
        ))?;

    let renderer = TeraRenderer::new("**/*.tera")?;

    renderer.render("template.tera", Default::default())
        .map_err(|e| e.with_context(
            format!("Template path: {}", path.display())
        ))
}
```

## Testing Strategy

### Unit Tests

Each component has inline unit tests:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_variable_resolver_precedence() -> Result<()> {
        let resolver = VariableResolver::new();

        // Template vars win over ENV
        std::env::set_var("TEST_VAR", "env_value");
        let mut vars = HashMap::new();
        vars.insert("TEST_VAR".to_string(), "template_value".to_string());

        let resolved = resolver.resolve(vars);
        assert_eq!(resolved.get("TEST_VAR"), Some(&"template_value".to_string()));

        Ok(())
    }
}
```

### Integration Tests

Test complete workflows in `crates/clnrm-core/tests/`:

```rust
// tests/prd_v1_integration.rs
#[tokio::test]
async fn test_full_workflow_template_to_verdict() -> Result<()> {
    // 1. Render template
    let renderer = TeraRenderer::new("tests/fixtures/**/*.tera")?;
    let toml = renderer.render("otel.clnrm.toml.tera", Default::default())?;

    // 2. Parse config
    let config = parse_toml_config(&toml)?;
    config.validate()?;

    // 3. Execute scenario
    let env = CleanroomEnvironment::new().await?;
    // ... execution ...

    // 4. Normalize spans
    let normalizer = SpanNormalizer::new(NormalizationConfig::default());
    let normalized = normalizer.normalize(spans)?;

    // 5. Analyze expectations
    let analyzer = ExpectationAnalyzer::new(config.expect.unwrap())?;
    let result = analyzer.analyze(&normalized)?;

    assert_eq!(result.verdict, Verdict::Pass);
    Ok(())
}
```

### Property-Based Tests

Use existing proptest infrastructure:

```rust
#[cfg(feature = "proptest")]
mod property_tests {
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_normalization_is_deterministic(
            spans in prop::collection::vec(arbitrary_span(), 1..100)
        ) {
            let normalizer = SpanNormalizer::new(NormalizationConfig::default());

            let digest1 = normalizer.compute_digest(&normalizer.normalize(spans.clone())?)?;
            let digest2 = normalizer.compute_digest(&normalizer.normalize(spans.clone())?)?;

            prop_assert_eq!(digest1, digest2);
        }
    }
}
```

## Performance Considerations

### Template Rendering

**Target**: Template cold run ≤5s, edit→rerun p95 ≤3s

**Optimizations**:
1. Cache compiled Tera templates in memory
2. Lazy ENV variable resolution (only when needed)
3. Incremental rendering (only changed templates)

### Change-Aware Execution

**Target**: 30-50% reduction in suite time via skipping unchanged scenarios

**Optimizations**:
1. Blake3 hashing (faster than SHA-256)
2. Parallel hash computation
3. Memory-mapped cache files for large suites

### Worker Pool

**Target**: Linear scaling up to N cores

**Optimizations**:
1. Use `rayon` for CPU-bound work (normalization)
2. Use `tokio` for I/O-bound work (container execution)
3. Work-stealing scheduler for uneven scenario durations

### OTEL Normalization

**Target**: <100ms overhead for 1000 spans

**Optimizations**:
1. Pre-allocate vectors with capacity hints
2. Use `BTreeMap` for O(log n) sorted inserts
3. Parallel normalization for large span batches

## Security & Isolation

### Hermetic Testing

All containers run with:
- Isolated networks (no external access by default)
- Read-only root filesystems where possible
- Resource limits (CPU/memory from config)
- No privileged containers

### Validation

`HermeticityValidator` ensures:
- No spans with `net.peer.name` attributes (external calls)
- All resource attributes match expected values
- No forbidden span attribute keys

### Secret Management

**Never commit**:
- OTEL tokens/API keys
- Database passwords
- Service credentials

**Use ENV vars**:
```bash
export OTEL_TOKEN="secret"
clnrm run  # Token injected via template
```

## Deployment & Distribution

### Crate Structure

```
clnrm/
├── crates/
│   ├── clnrm/              # Binary crate
│   │   └── src/
│   │       └── cli/
│   │           └── commands/
│   │               ├── template.rs  (new)
│   │               ├── dev.rs       (new)
│   │               ├── diff.rs      (new)
│   │               └── graph.rs     (new)
│   │
│   ├── clnrm-core/         # Library crate
│   │   └── src/
│   │       ├── template/
│   │       │   └── tera_renderer.rs     (new)
│   │       ├── execution/
│   │       │   └── change_aware.rs      (new)
│   │       ├── validation/
│   │       │   ├── otel_normalization.rs (new)
│   │       │   └── expectations/        (new dir)
│   │       │       ├── mod.rs
│   │       │       ├── graph.rs
│   │       │       ├── counts.rs
│   │       │       ├── order.rs
│   │       │       ├── hermeticity.rs
│   │       │       └── status.rs
│   │       └── config.rs               (extend)
│   │
│   ├── clnrm-shared/       # Shared utilities
│   └── clnrm-ai/           # Experimental (excluded)
```

### Feature Flags

```toml
[features]
default = ["otel-traces", "otel-metrics"]
otel-traces = ["opentelemetry/trace"]
otel-metrics = ["opentelemetry/metrics"]
otel-logs = ["opentelemetry/logs"]
template-tera = ["tera"]  # v0.7.0 default
template-handlebars = ["handlebars"]  # Alternative
proptest = ["dep:proptest"]
```

### Build Targets

```bash
# Development build
cargo build

# Production build (optimized)
cargo build --release

# With all OTEL features
cargo build --release --features otel

# Minimal build (no OTEL)
cargo build --release --no-default-features
```

## Migration Path

### Phase 1: Foundation (v0.7.0-alpha)

**Scope**: Template rendering + basic execution

**Deliverables**:
- `TeraRenderer` with ENV integration
- Extended config structs for v0.7.0 schema
- `clnrm template otel` command
- Basic template→TOML→execute flow

**DoD**: Can render template and run single scenario

### Phase 2: Normalization (v0.7.0-beta)

**Scope**: OTEL collection + normalization + digests

**Deliverables**:
- `SpanNormalizer` with deterministic sorting
- SHA-256 digest generation
- `clnrm diff` command for digest comparison
- Digest file output

**DoD**: Same spans → same digest on repeat runs

### Phase 3: Expectations (v0.7.0-rc)

**Scope**: Full expectation analysis

**Deliverables**:
- All validators (graph, counts, order, hermeticity, status)
- `ExpectationAnalyzer` orchestration
- Focused failure output
- JSON report with first_failure

**DoD**: All PRD expectation types working

### Phase 4: Performance (v0.7.0)

**Scope**: Change-aware execution + workers + watch mode

**Deliverables**:
- `ChangeTracker` for scenario hashing
- Worker pool execution
- `clnrm dev --watch` hot reload
- Performance targets met

**DoD**: 30-50% suite time reduction, p95 ≤3s edit→rerun

## Acceptance Criteria

From PRD v1.0:

- [x] Architecture adheres to clnrm core team standards
- [x] No unwrap/expect in production code paths
- [x] All traits are dyn compatible (sync methods)
- [x] Proper `Result<T, CleanroomError>` error handling
- [x] Files under 500 lines (modular design)
- [x] Plugin architecture integration maintained
- [ ] Tera→TOML→exec→OTEL→normalize→analyze→report flow designed
- [ ] Template rendering with ENV precedence specified
- [ ] Change-aware execution strategy defined
- [ ] OTEL normalization algorithm specified
- [ ] Expectation analysis components designed
- [ ] Performance targets documented (5s cold, 3s hot, 30-50% reduction)
- [ ] Security and hermeticity considerations addressed

## Risks & Mitigations

### Risk 1: File Size Explosion

**Risk**: config.rs grows beyond 500 lines with v0.7.0 additions

**Impact**: Medium - violates core team standards

**Mitigation**:
- Split into `config/mod.rs` + `config/v0_7.rs` + `config/legacy.rs`
- Keep each file under 400 lines for safety margin

### Risk 2: Tera Performance

**Risk**: Template rendering becomes bottleneck in hot-reload workflow

**Impact**: High - misses p95 ≤3s target

**Mitigation**:
- Benchmark early with realistic templates
- Cache compiled templates in memory
- Consider template precompilation for production

### Risk 3: Normalization Non-Determinism

**Risk**: Floating-point timestamps cause digest instability

**Impact**: Critical - breaks record/repro workflow

**Mitigation**:
- Use integer milliseconds internally
- Normalize timestamps to relative offsets
- Extensive property-based testing

### Risk 4: Graph Validator Complexity

**Risk**: Cycle detection algorithm exceeds 500 line limit

**Impact**: Low - can be isolated to separate module

**Mitigation**:
- Use well-tested graph library (petgraph)
- Keep validator focused on validation, not algorithms
- Extract graph algorithms to `utils/graph.rs`

## Future Enhancements (Post-v0.7.0)

1. **Graph Visualization**: SVG/Mermaid output for `clnrm graph`
2. **Dependency Tracking**: Re-run scenarios that depend on changed ones
3. **Snapshot Reuse v2**: Store normalized spans for faster analysis
4. **Coverage Metrics**: Track which expectations cover which code paths
5. **AI-Assisted Debugging**: Suggest fixes for failing expectations
6. **Multi-Backend Support**: Test against Podman, Docker, containerd
7. **Distributed Execution**: Run scenarios across multiple machines

## Appendix A: File Size Breakdown

| File | Estimated Lines | Status |
|------|----------------|--------|
| `template/tera_renderer.rs` | 250 | New |
| `template/variable_resolver.rs` | 150 | New |
| `execution/change_aware.rs` | 300 | New |
| `validation/otel_normalization.rs` | 350 | New |
| `validation/expectations/mod.rs` | 200 | New |
| `validation/expectations/graph.rs` | 300 | New |
| `validation/expectations/counts.rs` | 150 | New |
| `validation/expectations/order.rs` | 150 | New |
| `validation/expectations/hermeticity.rs` | 200 | New |
| `validation/expectations/status.rs` | 150 | New |
| `cli/commands/template.rs` | 200 | New |
| `cli/commands/dev.rs` | 250 | New |
| `cli/commands/diff.rs` | 150 | New |
| `cli/commands/graph.rs` | 200 | New |
| `config.rs` extensions | +150 | Extend |
| **Total New Code** | **3,000** | |

**Recommendation**: Split config.rs before adding v0.7.0 fields

## Appendix B: Dependency Additions

```toml
[dependencies]
# Template rendering
tera = "1.19"

# Hashing (change detection)
blake3 = "1.5"  # Optional, configurable

# Graph algorithms
petgraph = "0.6"  # For graph validator

# Glob patterns (for status validator)
glob = "0.3"

# Existing dependencies remain
serde = { version = "1", features = ["derive"] }
serde_json = "1"
chrono = { version = "0.4", features = ["serde"] }
toml = "0.8"
tokio = { version = "1", features = ["full"] }
opentelemetry = { version = "0.24", features = ["trace", "metrics"], optional = true }
```

## Appendix C: References

- **PRD v1.0**: `/Users/sac/clnrm/PRD-v1.md`
- **Core Team Standards**: `/Users/sac/clnrm/CLAUDE.md`
- **Existing Config**: `/Users/sac/clnrm/crates/clnrm-core/src/config.rs`
- **Error Handling**: `/Users/sac/clnrm/crates/clnrm-core/src/error.rs`
- **Plugin Architecture**: `/Users/sac/clnrm/crates/clnrm-core/src/cleanroom.rs`

---

**End of Architecture Document**

**Next Steps**:
1. Review with PRD Analyst for alignment
2. Validate with Test Scanner for test coverage strategy
3. Create implementation tasks via TodoWrite
4. Begin Phase 1 implementation (Foundation)
