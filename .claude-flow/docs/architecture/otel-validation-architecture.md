# Architecture Design: OTEL Validation in Testcontainer Execution

**Author**: System Architect Agent
**Date**: 2025-10-16
**Status**: Design Phase
**Version**: 1.0.0

## Executive Summary

This document presents the architecture for integrating OpenTelemetry (OTEL) validation into the Cleanroom Testing Framework's testcontainer execution pipeline. The design follows the 80/20 principle, focusing on critical validation paths while maintaining production-quality standards.

## 1. Design Goals

### Primary Objectives (80%)
1. **Enable OTEL data capture within testcontainer runs** - Collect traces, metrics, and logs from containers spawned by `TestcontainerBackend`
2. **TOML-based assertions** - Allow declarative validation of OTEL data in `.clnrm.toml` test configurations
3. **Zero breaking changes** - Maintain backward compatibility with existing trait signatures and `dyn` compatibility
4. **Core team compliance** - No `.unwrap()`, proper `Result<T, CleanroomError>`, sync trait methods

### Secondary Objectives (20%)
- Advanced filtering/aggregation of OTEL data
- Cross-service trace correlation
- Custom OTEL exporters beyond OTLP

## 2. Current State Analysis

### Existing OTEL Infrastructure

**`src/telemetry.rs`** (Production-ready):
- Full OTLP HTTP/gRPC/Stdout exporter support
- Traces, metrics, and logs with feature flags
- Helper functions in `metrics` module
- `OtelConfig` and `OtelGuard` for lifecycle management

**`src/backend/testcontainer.rs`**:
- `TestcontainerBackend` implements `Backend` trait
- Already has OTEL tracing with `#[instrument]` macro
- Executes commands via `execute_in_container()` (sync method)
- Uses `testcontainers-rs` for container lifecycle

**`src/config.rs`**:
- `TestConfig` with `assertions: Option<HashMap<String, serde_json::Value>>`
- `StepConfig` for individual test steps
- Validation framework in place

**`src/cleanroom.rs`**:
- `CleanroomEnvironment` has `TelemetryState`
- Methods: `enable_tracing()`, `enable_metrics()`, `get_traces()`
- OTel meter integration with feature flag

### Gaps Identified

1. **No OTEL data collection from within containers** - Currently only framework-level tracing
2. **No assertion syntax for OTEL validation** - Assertions are generic `HashMap<String, serde_json::Value>`
3. **No bridge between container execution and OTEL exporters** - Containers run isolated from OTEL infrastructure
4. **No typed assertion structures** - Need structured types for OTEL assertions

## 3. Architecture Design

### 3.1 High-Level Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                     .clnrm.toml Test File                   │
│  ┌───────────────────────────────────────────────────────┐  │
│  │ [assertions.otel]                                     │  │
│  │ traces_count_min = 1                                  │  │
│  │ metrics_must_include = ["test.duration", ...]         │  │
│  │ [[assertions.otel.traces]]                            │  │
│  │   span_name = "container.exec.*"                      │  │
│  │   attributes = { "container.name" = "alpine" }        │  │
│  └───────────────────────────────────────────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│              TestConfig (config.rs)                          │
│  - Parse OTEL assertions into typed structs                 │
│  - Validate assertion syntax                                │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│        TestcontainerBackend (backend/testcontainer.rs)      │
│  - Configure container with OTEL collector sidecar          │
│  - Inject OTEL env vars (OTEL_EXPORTER_OTLP_ENDPOINT)       │
│  - Execute commands with OTEL context propagation           │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│           OtelCollector (new: telemetry/collector.rs)       │
│  - In-memory OTLP receiver (HTTP endpoint)                  │
│  - Stores traces/metrics/logs during test execution         │
│  - Query interface for assertions                           │
└─────────────────────────────────────────────────────────────┘
                            │
                            ▼
┌─────────────────────────────────────────────────────────────┐
│         OtelAssertion Validator (new: assertions/otel.rs)   │
│  - Validates collected OTEL data against assertions         │
│  - Returns CleanroomError with detailed failure reasons     │
└─────────────────────────────────────────────────────────────┘
```

### 3.2 Component Design

#### 3.2.1 TOML Assertion Syntax (config.rs)

**New structures to add to `config.rs`**:

```rust
/// OTEL assertion configuration
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct OtelAssertions {
    /// Minimum number of traces expected
    pub traces_count_min: Option<usize>,
    /// Maximum number of traces expected
    pub traces_count_max: Option<usize>,
    /// Metrics that must be present
    pub metrics_must_include: Option<Vec<String>>,
    /// Metrics that must not be present
    pub metrics_must_not_include: Option<Vec<String>>,
    /// Specific trace assertions
    pub traces: Option<Vec<TraceAssertion>>,
    /// Specific metric assertions
    pub metrics: Option<Vec<MetricAssertion>>,
    /// Log assertions
    pub logs: Option<Vec<LogAssertion>>,
}

/// Trace assertion
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct TraceAssertion {
    /// Span name pattern (regex)
    pub span_name: String,
    /// Required attributes (exact match or regex)
    pub attributes: Option<HashMap<String, String>>,
    /// Minimum duration in milliseconds
    pub min_duration_ms: Option<u64>,
    /// Maximum duration in milliseconds
    pub max_duration_ms: Option<u64>,
    /// Required status (ok, error, unset)
    pub status: Option<String>,
}

/// Metric assertion
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct MetricAssertion {
    /// Metric name pattern (regex)
    pub name: String,
    /// Metric type (counter, histogram, gauge)
    pub metric_type: Option<String>,
    /// Minimum value
    pub min_value: Option<f64>,
    /// Maximum value
    pub max_value: Option<f64>,
    /// Required attributes
    pub attributes: Option<HashMap<String, String>>,
}

/// Log assertion
#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LogAssertion {
    /// Log message pattern (regex)
    pub message_pattern: String,
    /// Log level (debug, info, warn, error)
    pub level: Option<String>,
    /// Required attributes
    pub attributes: Option<HashMap<String, String>>,
}
```

**Integration with existing TestConfig**:

```rust
// Extend TestConfig to parse OTEL assertions
impl TestConfig {
    pub fn get_otel_assertions(&self) -> Option<OtelAssertions> {
        if let Some(assertions) = &self.assertions {
            if let Some(otel_value) = assertions.get("otel") {
                // Deserialize from serde_json::Value to OtelAssertions
                serde_json::from_value(otel_value.clone()).ok()
            } else {
                None
            }
        } else {
            None
        }
    }
}
```

#### 3.2.2 OTEL Collector (new: `src/telemetry/collector.rs`)

**Purpose**: In-memory OTLP receiver that collects telemetry data during test execution.

**Key features**:
- HTTP server that implements OTLP protocol
- Stores traces/metrics/logs in memory during test
- Query interface for assertion validation
- Thread-safe with `Arc<RwLock<...>>`

**Core struct**:

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory OTEL collector for test validation
pub struct OtelCollector {
    /// Collected traces
    traces: Arc<RwLock<Vec<TraceData>>>,
    /// Collected metrics
    metrics: Arc<RwLock<Vec<MetricData>>>,
    /// Collected logs
    logs: Arc<RwLock<Vec<LogData>>>,
    /// HTTP server handle
    server_handle: Option<tokio::task::JoinHandle<()>>,
    /// Collector endpoint (e.g., http://127.0.0.1:4318)
    endpoint: String,
}

impl OtelCollector {
    /// Start collector on a random available port
    pub fn start() -> Result<Self>;

    /// Stop collector and return collected data
    pub async fn stop(self) -> Result<CollectedData>;

    /// Get endpoint URL for containers to export to
    pub fn endpoint(&self) -> &str;

    /// Query collected traces
    pub async fn query_traces(&self) -> Vec<TraceData>;

    /// Query collected metrics
    pub async fn query_metrics(&self) -> Vec<MetricData>;

    /// Query collected logs
    pub async fn query_logs(&self) -> Vec<LogData>;
}
```

**Protocol handling**:
- Use `axum` or `warp` for HTTP server (already in dependency tree via testcontainers)
- Parse OTLP protobuf messages (use `opentelemetry_proto` crate)
- Store data in simple Rust structs for easy querying

#### 3.2.3 TestcontainerBackend Integration

**Modification to `execute_in_container()` method**:

```rust
// In TestcontainerBackend
impl TestcontainerBackend {
    /// Execute command with OTEL collection enabled
    fn execute_in_container_with_otel(
        &self,
        cmd: &Cmd,
        otel_endpoint: Option<&str>,
    ) -> Result<RunResult> {
        // Existing container creation logic...

        // If OTEL endpoint provided, inject environment variables
        if let Some(endpoint) = otel_endpoint {
            container_request = container_request
                .with_env_var("OTEL_EXPORTER_OTLP_ENDPOINT", endpoint)
                .with_env_var("OTEL_SERVICE_NAME", "clnrm-test-container")
                .with_env_var("OTEL_TRACES_EXPORTER", "otlp")
                .with_env_var("OTEL_METRICS_EXPORTER", "otlp")
                .with_env_var("OTEL_LOGS_EXPORTER", "otlp");
        }

        // Rest of execution logic...
    }
}
```

**Backend trait extension** (optional - can be done without trait changes):

Add optional OTEL support without breaking `dyn Backend`:

```rust
// In backend/mod.rs
pub trait Backend {
    // Existing methods remain unchanged...

    // Optional: Get backend capabilities
    fn capabilities(&self) -> BackendCapabilities {
        BackendCapabilities::default()
    }
}

pub struct BackendCapabilities {
    pub supports_otel: bool,
    pub supports_network_isolation: bool,
}
```

#### 3.2.4 OTEL Assertion Validator (new: `src/assertions/otel.rs`)

**Purpose**: Validate collected OTEL data against assertions defined in TOML.

```rust
use crate::config::{OtelAssertions, TraceAssertion, MetricAssertion};
use crate::telemetry::collector::{CollectedData, TraceData, MetricData};
use crate::error::{CleanroomError, Result};

pub struct OtelValidator;

impl OtelValidator {
    /// Validate collected OTEL data against assertions
    pub fn validate(
        assertions: &OtelAssertions,
        collected: &CollectedData,
    ) -> Result<()> {
        // Validate trace count
        if let Some(min_count) = assertions.traces_count_min {
            if collected.traces.len() < min_count {
                return Err(CleanroomError::assertion_error(
                    format!(
                        "Expected at least {} traces, found {}",
                        min_count,
                        collected.traces.len()
                    )
                ));
            }
        }

        // Validate each trace assertion
        if let Some(trace_assertions) = &assertions.traces {
            for assertion in trace_assertions {
                Self::validate_trace(assertion, &collected.traces)?;
            }
        }

        // Validate metrics
        if let Some(required_metrics) = &assertions.metrics_must_include {
            for metric_name in required_metrics {
                if !collected.metrics.iter().any(|m| &m.name == metric_name) {
                    return Err(CleanroomError::assertion_error(
                        format!("Required metric '{}' not found", metric_name)
                    ));
                }
            }
        }

        Ok(())
    }

    fn validate_trace(
        assertion: &TraceAssertion,
        traces: &[TraceData],
    ) -> Result<()> {
        use regex::Regex;
        let pattern = Regex::new(&assertion.span_name).map_err(|e| {
            CleanroomError::validation_error(
                format!("Invalid span name pattern: {}", e)
            )
        })?;

        let matching_traces: Vec<_> = traces
            .iter()
            .filter(|t| pattern.is_match(&t.span_name))
            .collect();

        if matching_traces.is_empty() {
            return Err(CleanroomError::assertion_error(
                format!("No traces found matching pattern '{}'", assertion.span_name)
            ));
        }

        // Validate attributes, duration, status...
        for trace in matching_traces {
            if let Some(required_attrs) = &assertion.attributes {
                for (key, expected_value) in required_attrs {
                    match trace.attributes.get(key) {
                        Some(actual_value) => {
                            if actual_value != expected_value {
                                return Err(CleanroomError::assertion_error(
                                    format!(
                                        "Trace attribute '{}' has value '{}', expected '{}'",
                                        key, actual_value, expected_value
                                    )
                                ));
                            }
                        }
                        None => {
                            return Err(CleanroomError::assertion_error(
                                format!("Trace missing required attribute '{}'", key)
                            ));
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
```

### 3.3 Integration Flow

**Step-by-step execution with OTEL validation**:

1. **Test Parser** (`cli/commands/run.rs`):
   ```rust
   let config = load_config_from_file(test_file)?;
   let otel_assertions = config.get_otel_assertions();
   ```

2. **Start OTEL Collector** (if assertions exist):
   ```rust
   let collector = if otel_assertions.is_some() {
       Some(OtelCollector::start()?)
   } else {
       None
   };
   ```

3. **Execute Test Steps** (with OTEL endpoint):
   ```rust
   for step in &config.steps {
       let result = if let Some(ref collector) = collector {
           backend.execute_in_container_with_otel(
               &cmd,
               Some(collector.endpoint())
           )?
       } else {
           backend.execute_in_container(&cmd)?
       };
   }
   ```

4. **Validate OTEL Assertions**:
   ```rust
   if let (Some(assertions), Some(collector)) = (otel_assertions, collector) {
       let collected = collector.stop().await?;
       OtelValidator::validate(&assertions, &collected)?;
   }
   ```

5. **Report Results**:
   ```rust
   // Include OTEL validation in test report
   report.otel_validation_passed = true;
   ```

## 4. Example Usage

### 4.1 TOML Test Configuration

```toml
[test.metadata]
name = "otel_validation_test"
description = "Test that validates OTEL data from container execution"

[[steps]]
name = "execute_with_traces"
command = ["sh", "-c", "echo 'Starting'; sleep 1; echo 'Done'"]

[assertions]
container_should_have_executed_commands = 1

[assertions.otel]
traces_count_min = 1
metrics_must_include = ["container.command.duration"]

[[assertions.otel.traces]]
span_name = "container\\.exec\\..*"
attributes = { "command" = "sh -c echo 'Starting'; sleep 1; echo 'Done'" }
min_duration_ms = 1000

[[assertions.otel.metrics]]
name = "container\\.command\\.duration"
metric_type = "histogram"
min_value = 1.0
```

### 4.2 Rust Test Code

```rust
#[cfg(feature = "otel-traces")]
#[tokio::test]
async fn test_container_execution_produces_otel_data() -> Result<()> {
    // Arrange
    let config = load_config_from_file("tests/otel_validation.clnrm.toml")?;
    let otel_assertions = config.get_otel_assertions().unwrap();

    // Start OTEL collector
    let collector = OtelCollector::start()?;

    // Create backend
    let backend = TestcontainerBackend::new("alpine:latest")?;

    // Act
    let cmd = Cmd::new("sh").arg("-c").arg("echo 'test'");
    backend.execute_in_container_with_otel(&cmd, Some(collector.endpoint()))?;

    // Assert
    let collected = collector.stop().await?;
    OtelValidator::validate(&otel_assertions, &collected)?;

    Ok(())
}
```

## 5. Implementation Strategy (80/20)

### Phase 1: Core Infrastructure (80% of value)

1. **Add OTEL assertion structs to `config.rs`** (2 hours)
   - `OtelAssertions`, `TraceAssertion`, `MetricAssertion`
   - Validation methods
   - Parser integration

2. **Implement `OtelCollector`** (4 hours)
   - Basic HTTP server with OTLP endpoint
   - In-memory storage for traces/metrics
   - Query interface
   - Tests

3. **Extend `TestcontainerBackend`** (2 hours)
   - Add OTEL env var injection
   - Modify `execute_in_container()` to accept optional endpoint
   - Maintain `dyn` compatibility

4. **Implement `OtelValidator`** (3 hours)
   - Trace validation
   - Metric validation
   - Error messages with context

5. **Integration in CLI** (2 hours)
   - Detect OTEL assertions in test config
   - Start/stop collector lifecycle
   - Run validation after test execution

6. **Tests** (3 hours)
   - Unit tests for each component
   - Integration test with real container
   - Self-test validation

**Total Phase 1: ~16 hours**

### Phase 2: Advanced Features (20% of value)

- Log assertions
- Cross-service trace correlation
- Advanced filtering (attribute wildcards, regex)
- Performance optimization (async collector)
- Custom OTEL exporters

## 6. Architecture Decision Records (ADRs)

### ADR-001: Use In-Memory Collector Instead of External Collector

**Context**: Need to collect OTEL data from containers during tests.

**Decision**: Implement lightweight in-memory collector rather than requiring external OTEL collector.

**Rationale**:
- **Simplicity**: No external dependencies to install/configure
- **Speed**: In-process, no network latency
- **Isolation**: Each test has its own collector instance
- **CI-friendly**: Works in restricted environments

**Consequences**:
- (+) Zero external dependencies
- (+) Fast data collection
- (-) Not suitable for production monitoring
- (-) Limited to single-node scenarios

### ADR-002: TOML Assertion Syntax Over Code-Based Assertions

**Context**: Need to define OTEL validation criteria.

**Decision**: Use declarative TOML syntax rather than programmatic assertions.

**Rationale**:
- **Consistency**: Matches existing `.clnrm.toml` pattern
- **Accessibility**: Non-programmers can write assertions
- **Versioning**: Assertions tracked in version control
- **Testability**: Can validate assertion syntax independently

**Consequences**:
- (+) Lower barrier to entry
- (+) Version-controllable test definitions
- (-) Less flexibility than code
- (-) Requires parser maintenance

### ADR-003: Sync Trait Methods with `tokio::task::block_in_place`

**Context**: Need to maintain `dyn` compatibility while supporting async operations.

**Decision**: Keep trait methods synchronous, use `block_in_place` internally for async operations.

**Rationale**:
- **Compatibility**: Maintains `dyn ServicePlugin` pattern
- **Core team standard**: No async trait methods
- **Proven pattern**: Already used in `MockDatabasePlugin`

**Consequences**:
- (+) No breaking changes to trait definitions
- (+) Consistent with existing codebase
- (-) Slightly more complex internal implementation
- (-) Potential runtime context issues (mitigated by `block_in_place`)

### ADR-004: Optional OTEL Validation via Feature Flag

**Context**: OTEL validation adds dependencies and complexity.

**Decision**: Gate OTEL validation behind existing `otel-traces` feature flag.

**Rationale**:
- **Optional**: Users can opt-out if not needed
- **Lightweight**: Minimal code when disabled
- **Consistent**: Uses existing feature flag pattern

**Consequences**:
- (+) No impact on users without OTEL
- (+) Reduced binary size when disabled
- (-) Need conditional compilation guards
- (-) More testing surface area

## 7. Risk Assessment

### High Risks

1. **Container Network Isolation**
   - **Risk**: Containers may not be able to reach collector endpoint
   - **Mitigation**: Use host.docker.internal or bridge network
   - **Validation**: Test in CI environment early

2. **OTLP Protocol Complexity**
   - **Risk**: Incorrect protobuf parsing leads to data loss
   - **Mitigation**: Use official `opentelemetry_proto` crate, extensive testing
   - **Validation**: Compare with real OTEL collector

3. **Performance Impact**
   - **Risk**: OTEL collection slows down test execution
   - **Mitigation**: In-memory collector, async processing, optional feature
   - **Validation**: Benchmark with/without OTEL validation

### Medium Risks

1. **TOML Syntax Complexity**
   - **Risk**: Assertion syntax becomes too complex for users
   - **Mitigation**: Start simple, iterate based on feedback
   - **Validation**: User testing with sample assertions

2. **Error Message Quality**
   - **Risk**: Validation failures produce unclear error messages
   - **Mitigation**: Rich error context, examples in messages
   - **Validation**: Test with intentionally failing assertions

## 8. Success Metrics

### Functional Metrics

- [ ] Can parse OTEL assertions from TOML
- [ ] Can collect traces from containers
- [ ] Can validate trace count, span names, attributes
- [ ] Can validate metrics presence and values
- [ ] Zero test failures due to OTEL validation bugs

### Non-Functional Metrics

- [ ] OTEL validation adds <10% overhead to test execution
- [ ] Error messages clearly explain validation failures
- [ ] Feature works in CI environments (GitHub Actions, GitLab CI)
- [ ] All code passes `cargo clippy -- -D warnings`
- [ ] 90%+ test coverage for new code

## 9. Dependencies

### New Crate Dependencies

```toml
# Cargo.toml additions
[dependencies]
# For OTLP protocol parsing
opentelemetry_proto = { version = "0.31", optional = true }

# For HTTP server (may already be in tree via testcontainers)
axum = { version = "0.7", optional = true }
tokio = { version = "1.35", features = ["rt-multi-thread", "net", "sync"] }

# For regex in assertions
regex = "1.10"

[features]
otel-validation = ["otel-traces", "opentelemetry_proto", "axum"]
```

### Internal Dependencies

- `telemetry.rs` - Existing OTEL infrastructure
- `config.rs` - TOML parsing
- `backend/testcontainer.rs` - Container execution
- `error.rs` - Error handling

## 10. Testing Strategy

### Unit Tests

```rust
// tests/unit/otel_assertions.rs
#[test]
fn test_parse_otel_assertions_from_toml() { }

#[test]
fn test_validate_trace_count() { }

#[test]
fn test_validate_trace_attributes() { }

#[test]
fn test_validate_metric_presence() { }
```

### Integration Tests

```rust
// tests/integration/otel_validation.rs
#[cfg(feature = "otel-traces")]
#[tokio::test]
async fn test_container_produces_expected_traces() { }

#[cfg(feature = "otel-traces")]
#[tokio::test]
async fn test_validation_fails_on_missing_metric() { }
```

### Self-Tests

```toml
# tests/self/otel_validation.clnrm.toml
[test.metadata]
name = "framework_otel_validation_self_test"
description = "Cleanroom tests its own OTEL validation feature"

[[steps]]
name = "execute_command"
command = ["echo", "test"]

[assertions.otel]
traces_count_min = 1
```

## 11. Documentation Requirements

### User-Facing Documentation

1. **TOML Reference** (`docs/TOML_REFERENCE.md`):
   - Add `[assertions.otel]` section
   - Document all assertion types
   - Provide examples

2. **Testing Guide** (`docs/TESTING.md`):
   - Add "OTEL Validation" section
   - Usage patterns
   - Common pitfalls

3. **CLI Guide** (`docs/CLI_GUIDE.md`):
   - Mention `--features otel-traces` requirement

### Developer Documentation

1. **Architecture Guide** (this document)
2. **API Documentation** (inline Rustdoc)
3. **Integration Examples** (`examples/otel-validation/`)

## 12. Open Questions

1. **Should we support external OTEL collectors?**
   - Answer: Not in Phase 1 (80%), consider for Phase 2

2. **How to handle containers that don't emit OTEL data?**
   - Answer: Validation should gracefully handle zero traces/metrics

3. **Should assertions be AND or OR logic?**
   - Answer: All assertions are AND (all must pass)

4. **How to handle sampling?**
   - Answer: Use sample_ratio=1.0 for tests (always sample)

## 13. Future Enhancements

- **Log assertions** - Validate log output via OTEL logs
- **Distributed tracing** - Validate trace propagation across services
- **Custom matchers** - User-defined validation functions
- **JSON schema** - Validate OTEL data against JSON schema
- **Performance benchmarks** - Built-in performance regression detection
- **OTEL collector sidecar** - Optionally use real collector

## 14. Conclusion

This architecture provides a production-quality foundation for OTEL validation in testcontainer runs while maintaining the framework's core principles:

- **Hermetic testing** - Each test has isolated OTEL collector
- **Self-testing** - Framework validates its own OTEL capabilities
- **Core team standards** - No unwrap, proper errors, sync traits
- **80/20 principle** - Focus on critical validation paths

The design is extensible for future enhancements while delivering immediate value through basic trace and metric validation.

---

**Next Steps**:
1. Review architecture with team
2. Implement Phase 1 components
3. Write integration tests
4. Update documentation
5. Run framework self-tests
