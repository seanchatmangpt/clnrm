# OpenTelemetry Validation Guide

## Overview

The OTEL validation module provides comprehensive validation of OpenTelemetry instrumentation, following the TTBD (Test That Backs Documentation) philosophy. This ensures that observability claims are backed by verifiable telemetry data, not just documentation.

## Architecture

### Core Components

1. **OtelValidator** - Main validation engine
2. **SpanAssertion** - Validates individual span creation and attributes
3. **TraceAssertion** - Validates complete traces with parent-child relationships
4. **OtelValidationConfig** - Configuration for validation behavior

### Design Principles

- **Zero Unwrap/Expect** - All operations return `Result<T, CleanroomError>`
- **Sync Trait Methods** - Maintains `dyn` compatibility for trait objects
- **AAA Test Pattern** - Arrange, Act, Assert structure in all tests
- **No False Positives** - Uses `unimplemented!()` for incomplete features instead of fake `Ok(())` returns

## Usage

### Basic Validation

```rust
use clnrm_core::{OtelValidator, OtelValidationConfig};

// Create validator with default configuration
let validator = OtelValidator::new();

// Validate performance overhead
let baseline_duration = 100.0; // ms
let telemetry_duration = 150.0; // ms
let result = validator.validate_performance_overhead(
    baseline_duration,
    telemetry_duration,
)?;
```

### Custom Configuration

```rust
use clnrm_core::{OtelValidator, OtelValidationConfig};
use std::collections::HashMap;

let config = OtelValidationConfig {
    validate_spans: true,
    validate_traces: true,
    validate_exports: false, // Requires external collector
    validate_performance: true,
    max_overhead_ms: 50.0, // Maximum 50ms overhead
    expected_attributes: HashMap::new(),
};

let validator = OtelValidator::with_config(config);
```

### Span Assertions

```rust
use clnrm_core::SpanAssertion;
use std::collections::HashMap;

let mut attributes = HashMap::new();
attributes.insert("http.method".to_string(), "GET".to_string());
attributes.insert("http.status_code".to_string(), "200".to_string());

let assertion = SpanAssertion {
    name: "http.request".to_string(),
    attributes,
    required: true,
    min_duration_ms: Some(1.0),
    max_duration_ms: Some(5000.0),
};

// Validate span (requires OTel SDK integration)
// let result = validator.validate_span(&assertion)?;
```

### Trace Assertions

```rust
use clnrm_core::{TraceAssertion, SpanAssertion};
use std::collections::HashMap;

let trace_assertion = TraceAssertion {
    trace_id: Some("trace-123".to_string()),
    expected_spans: vec![
        SpanAssertion {
            name: "http.request".to_string(),
            attributes: HashMap::new(),
            required: true,
            min_duration_ms: None,
            max_duration_ms: None,
        },
        SpanAssertion {
            name: "database.query".to_string(),
            attributes: HashMap::new(),
            required: true,
            min_duration_ms: None,
            max_duration_ms: None,
        },
    ],
    complete: true, // All spans must be present
    parent_child_relationships: vec![
        ("http.request".to_string(), "database.query".to_string()),
    ],
};

// Validate trace (requires OTel SDK integration)
// let result = validator.validate_trace(&trace_assertion)?;
```

## TOML Configuration

### Test Configuration with OTEL Validation

```toml
[test.metadata]
name = "otel_validation_test"
description = "Validate OpenTelemetry instrumentation"

[[steps]]
name = "execute_operation"
command = ["sh", "-c", "echo 'test'"]

[otel_validation]
enabled = true
validate_spans = true
validate_traces = true
validate_exports = false
validate_performance = true
max_overhead_ms = 100.0

[[otel_validation.expected_spans]]
name = "container.start"
required = true
min_duration_ms = 10.0
max_duration_ms = 30000.0

[otel_validation.expected_spans.attributes]
"container.image" = "alpine:latest"
"operation.type" = "startup"

[[otel_validation.expected_traces]]
trace_id = "trace-123"
span_names = ["container.start", "container.execute", "container.stop"]
complete = true

[[otel_validation.expected_traces.parent_child]]
["container.start", "container.execute"]
["container.start", "container.stop"]
```

## Integration with TestcontainerBackend

### OTEL Validation in Container Operations

```rust
use clnrm_core::backend::testcontainer::TestcontainerBackend;

// Create backend
let backend = TestcontainerBackend::new("alpine:latest")?;

// Check if OTEL validation is enabled
if backend.otel_validation_enabled() {
    // OTEL features are enabled
    #[cfg(feature = "otel-traces")]
    {
        let validation_result = backend.validate_otel_instrumentation()?;
        assert!(validation_result, "OTEL instrumentation validated");
    }
}
```

## Validation Workflow

### 1. Span Creation Validation

Validates that operations create expected spans with correct attributes:

```rust
// 1. Execute operation (creates spans)
execute_traced_operation()?;

// 2. Validate span was created
let span_assertion = SpanAssertion {
    name: "operation.name",
    attributes: expected_attributes,
    required: true,
    min_duration_ms: Some(10.0),
    max_duration_ms: Some(1000.0),
};

// 3. Validate span matches assertion
let result = validator.validate_span(&span_assertion)?;
assert!(result.passed, "Span validation failed");
```

### 2. Trace Completeness Validation

Validates that complete traces contain all expected spans:

```rust
// 1. Execute operation chain (creates trace)
execute_operation_chain()?;

// 2. Define expected trace structure
let trace_assertion = TraceAssertion {
    trace_id: Some("trace-id"),
    expected_spans: vec![span1, span2, span3],
    complete: true,
    parent_child_relationships: vec![
        ("parent", "child1"),
        ("parent", "child2"),
    ],
};

// 3. Validate trace completeness
let result = validator.validate_trace(&trace_assertion)?;
assert!(result.passed, "Trace validation failed");
```

### 3. Performance Overhead Validation

Validates that telemetry overhead is within acceptable limits:

```rust
// 1. Measure baseline performance (without telemetry)
let baseline = measure_without_telemetry()?;

// 2. Measure with telemetry enabled
let with_telemetry = measure_with_telemetry()?;

// 3. Validate overhead
let overhead_ok = validator.validate_performance_overhead(
    baseline,
    with_telemetry,
)?;
assert!(overhead_ok, "Telemetry overhead too high");
```

## Current Implementation Status

### Implemented ✓

- **Core Validation Structures**
  - `OtelValidator` with configuration management
  - `SpanAssertion` for span validation requirements
  - `TraceAssertion` for trace validation requirements
  - `OtelValidationConfig` for validation behavior

- **TOML Configuration Support**
  - `OtelValidationSection` for TOML parsing
  - `ExpectedSpanConfig` for span assertions
  - `ExpectedTraceConfig` for trace assertions
  - Integration with `TestConfig`

- **Performance Validation**
  - `validate_performance_overhead()` - Fully functional
  - Configurable overhead thresholds
  - Proper error handling and reporting

- **Integration Points**
  - `TestcontainerBackend.validate_otel_instrumentation()`
  - `TestcontainerBackend.otel_validation_enabled()`
  - `telemetry::validation` module with helper functions

- **Comprehensive Tests**
  - 20+ unit tests covering all validation scenarios
  - AAA pattern test structure
  - Edge case coverage (zero baseline, negative overhead, etc.)
  - Feature flag testing (with/without `otel-traces`)

### Pending Implementation (Marked with `unimplemented!()`)

These features require integration with OpenTelemetry SDK's span processor:

1. **`validate_span()`** - Requires in-memory span exporter
   - Query spans by operation name
   - Validate span attributes
   - Check span duration constraints

2. **`validate_trace()`** - Requires span processor integration
   - Query spans by trace ID
   - Validate parent-child relationships
   - Check trace completeness

3. **`validate_export()`** - Requires mock OTLP collector
   - Capture exported telemetry
   - Verify data integrity
   - Validate export destinations

4. **`telemetry::validation::span_exists()`** - Needs span exporter
   - Query captured spans
   - Check span existence by operation name

5. **`telemetry::validation::capture_test_spans()`** - Needs exporter
   - Configure in-memory span exporter
   - Capture all spans during test execution

## Future Implementation Roadmap

### Phase 1: In-Memory Span Exporter (Priority: High)

Integrate OpenTelemetry's in-memory span exporter for testing:

```rust
// Example future implementation
use opentelemetry_sdk::testing::trace::InMemorySpanExporter;

pub fn setup_test_exporter() -> InMemorySpanExporter {
    let exporter = InMemorySpanExporter::default();
    // Configure tracer provider with test exporter
    // Return exporter for span capture
    exporter
}
```

### Phase 2: Span Validation (Priority: High)

Implement `validate_span()` with real span querying:

```rust
pub fn validate_span(&self, assertion: &SpanAssertion) -> Result<SpanValidationResult> {
    // 1. Query in-memory exporter for spans
    let spans = query_spans_by_name(&assertion.name)?;

    // 2. Validate attributes
    let attribute_errors = validate_attributes(&spans, &assertion.attributes);

    // 3. Validate duration constraints
    let duration_errors = validate_duration_constraints(&spans, assertion);

    // 4. Return validation result
    Ok(SpanValidationResult {
        passed: attribute_errors.is_empty() && duration_errors.is_empty(),
        span_name: assertion.name.clone(),
        errors: [attribute_errors, duration_errors].concat(),
        actual_attributes: extract_attributes(&spans),
        actual_duration_ms: extract_duration(&spans),
    })
}
```

### Phase 3: Trace Validation (Priority: Medium)

Implement `validate_trace()` with relationship checking:

```rust
pub fn validate_trace(&self, assertion: &TraceAssertion) -> Result<TraceValidationResult> {
    // 1. Query spans by trace ID
    let spans = query_spans_by_trace_id(&assertion.trace_id)?;

    // 2. Validate each expected span
    let span_results = assertion.expected_spans
        .iter()
        .map(|span| self.validate_span(span))
        .collect::<Result<Vec<_>>>()?;

    // 3. Validate parent-child relationships
    let relationship_errors = validate_relationships(
        &spans,
        &assertion.parent_child_relationships,
    );

    // 4. Check trace completeness
    let completeness_errors = if assertion.complete {
        validate_completeness(&spans, &assertion.expected_spans)
    } else {
        Vec::new()
    };

    // 5. Return comprehensive result
    Ok(TraceValidationResult {
        passed: relationship_errors.is_empty() && completeness_errors.is_empty(),
        trace_id: assertion.trace_id.clone(),
        expected_span_count: assertion.expected_spans.len(),
        actual_span_count: spans.len(),
        span_results,
        errors: [relationship_errors, completeness_errors].concat(),
    })
}
```

### Phase 4: Export Validation (Priority: Low)

Implement mock OTLP collector for export testing:

```rust
pub fn validate_export(&self, endpoint: &str) -> Result<bool> {
    // 1. Start mock OTLP collector
    let collector = start_mock_collector(endpoint)?;

    // 2. Generate test spans
    generate_test_spans()?;

    // 3. Wait for export
    tokio::time::sleep(Duration::from_secs(1)).await;

    // 4. Verify spans received
    let received_spans = collector.get_received_spans();

    // 5. Validate data integrity
    for span in received_spans {
        validate_span_data(&span)?;
    }

    Ok(true)
}
```

## Integration with clnrm CLI

### Planned CLI Commands

```bash
# Validate OTEL instrumentation in tests
clnrm validate-otel tests/

# Validate specific test with OTEL assertions
clnrm run tests/otel-test.clnrm.toml --validate-otel

# Generate OTEL validation report
clnrm report --otel-validation

# Self-test with OTEL validation
clnrm self-test --observability
```

## Best Practices

### 1. Always Use Result Types

```rust
// ✓ CORRECT - Returns Result for error handling
pub fn validate_span(&self, assertion: &SpanAssertion) -> Result<SpanValidationResult> {
    // ...
}

// ✗ WRONG - Can panic on error
pub fn validate_span(&self, assertion: &SpanAssertion) -> SpanValidationResult {
    // ...unwrap() // NEVER DO THIS
}
```

### 2. Use unimplemented!() for Incomplete Features

```rust
// ✓ CORRECT - Honest about incompleteness
pub fn validate_trace(&self, assertion: &TraceAssertion) -> Result<TraceValidationResult> {
    unimplemented!(
        "validate_trace: Requires integration with OpenTelemetry span processor"
    )
}

// ✗ WRONG - Lies about success
pub fn validate_trace(&self, assertion: &TraceAssertion) -> Result<TraceValidationResult> {
    Ok(TraceValidationResult {
        passed: true, // FALSE POSITIVE!
        // ...
    })
}
```

### 3. Follow AAA Test Pattern

```rust
#[test]
fn test_performance_overhead_validation() {
    // Arrange
    let validator = OtelValidator::new();
    let baseline_duration = 100.0;
    let telemetry_duration = 150.0;

    // Act
    let result = validator.validate_performance_overhead(
        baseline_duration,
        telemetry_duration,
    );

    // Assert
    assert!(result.is_ok());
}
```

### 4. Keep Trait Methods Sync

```rust
// ✓ CORRECT - Sync method maintains dyn compatibility
pub fn validate_span(&self, assertion: &SpanAssertion) -> Result<SpanValidationResult> {
    // Use tokio::task::block_in_place for async operations if needed
}

// ✗ WRONG - Async trait method breaks dyn compatibility
pub async fn validate_span(&self, assertion: &SpanAssertion) -> Result<SpanValidationResult> {
    // Cannot use dyn OtelValidator
}
```

## Error Handling

### Validation Errors

All validation methods return `Result<T, CleanroomError>`:

```rust
use clnrm_core::error::CleanroomError;

// Validation error example
let result = validator.validate_performance_overhead(100.0, 300.0);

match result {
    Ok(_) => println!("Validation passed"),
    Err(e) => match e.kind {
        ErrorKind::ValidationError => {
            eprintln!("Validation failed: {}", e.message);
        }
        _ => {
            eprintln!("Unexpected error: {}", e);
        }
    }
}
```

### Common Error Scenarios

1. **Performance Overhead Exceeded**
   ```
   CleanroomError::validation_error(
       "Telemetry performance overhead 150ms exceeds maximum allowed 100ms"
   )
   ```

2. **OTEL Not Initialized**
   ```
   CleanroomError::validation_error(
       "OpenTelemetry is not initialized. Enable OTEL features and call init_otel()"
   )
   ```

3. **Incomplete Implementation**
   ```
   panic!("validate_span: Requires integration with OpenTelemetry span processor. ...")
   ```

## Testing

### Running OTEL Validation Tests

```bash
# Run all validation tests
cargo test -p clnrm-core --test otel_validation

# Run with OTEL features enabled
cargo test -p clnrm-core --test otel_validation --features otel-traces

# Run specific test
cargo test -p clnrm-core --test otel_validation test_performance_overhead_validation

# Run with verbose output
cargo test -p clnrm-core --test otel_validation -- --nocapture
```

### Test Coverage

- **20+ unit tests** covering all validation scenarios
- **Edge case testing** (zero baseline, negative overhead)
- **Feature flag testing** (with/without OTEL features)
- **AAA pattern** in all tests
- **No .unwrap() or .expect()** in production code

## Contributing

When adding new validation features:

1. **Follow Core Team Standards**
   - No `.unwrap()` or `.expect()` in production code
   - Return `Result<T, CleanroomError>` from all fallible functions
   - Keep trait methods sync (use `tokio::task::block_in_place` for async)
   - Use `unimplemented!()` for incomplete features

2. **Write Comprehensive Tests**
   - Follow AAA pattern (Arrange, Act, Assert)
   - Test edge cases and error conditions
   - Add integration tests for complex scenarios

3. **Document Implementation**
   - Update this guide with new features
   - Add inline documentation to code
   - Provide usage examples

## References

- [OpenTelemetry Rust SDK](https://github.com/open-telemetry/opentelemetry-rust)
- [TTBD Philosophy](./tests-to-be-done/10-observability-validation.md)
- [Core Team Standards](../CLAUDE.md)
- [Telemetry Module](../crates/clnrm-core/src/telemetry.rs)

---

**Status**: Core infrastructure implemented. Span/trace validation pending OpenTelemetry SDK integration.

**Version**: 0.4.0

**Last Updated**: 2025-10-16
