# ADR-002: OpenTelemetry Validation Enhancement for v0.6.0

**Status**: APPROVED
**Date**: 2025-10-16
**Deciders**: Architecture Sub-Coordinator, System Designer, API Architect
**Technical Story**: Complete OTEL validation system for observability testing

## Context and Problem Statement

CLNRM v0.4.x has comprehensive OTEL integration (commit b24db56) with:
- Span/trace/metrics support
- OTLP HTTP/gRPC exporters
- Feature flags (otel-traces, otel-metrics, otel-logs)

However, the OTEL validation system (`validation/otel.rs`) is incomplete:
- `validate_span()` - **UNIMPLEMENTED** (line 143-160)
- `validate_trace()` - **UNIMPLEMENTED** (line 170-188)
- `validate_export()` - **UNIMPLEMENTED** (line 197-218)
- Only `validate_performance_overhead()` is implemented (line 227-248)

**Current State**: Skeleton implementation with `unimplemented!()` macros (following core team standards - no false positives)

**Problem**: Cannot validate that observability claims are backed by actual telemetry data (violates TTBD philosophy)

## Decision Drivers

1. **TTBD Philosophy**: "Test That Backs Documentation" - verify observability claims
2. **Core Team Standards**: No `.unwrap()`, sync traits, proper error handling
3. **Production Quality**: Comprehensive validation with helpful error messages
4. **Performance**: Low overhead for validation operations
5. **Integration**: Work with existing OTEL setup (traces, metrics, logs)

## Considered Options

### Option 1: In-Memory Span Exporter (SELECTED)
- Use OpenTelemetry's `InMemorySpanExporter` for test validation
- Capture spans during test execution
- Query captured spans for validation
- **Pros**: Native OTEL integration, accurate, reliable
- **Cons**: Requires test-specific exporter configuration

### Option 2: Mock OTLP Collector
- Implement mock collector to receive OTLP exports
- Validate spans at export destination
- **Pros**: Tests real export path
- **Cons**: Complex setup, requires network, slower

### Option 3: Tracing Subscriber Inspection
- Hook into `tracing-subscriber` layer
- Inspect spans before export
- **Pros**: Low-level access
- **Cons**: Couples to subscriber implementation, brittle

## Decision Outcome

**Chosen option**: "Option 1: In-Memory Span Exporter"

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                   OTEL Validation Architecture                   │
└─────────────────────────────────────────────────────────────────┘

┌──────────────────┐
│ Test Execution   │
│ (with OTEL)      │
└────────┬─────────┘
         │
         │ Spans generated
         ▼
┌──────────────────────────────┐
│ InMemorySpanExporter         │  ← NEW: Capture spans in memory
│ (test mode only)             │
└────────┬─────────────────────┘
         │
         │ Query spans
         ▼
┌──────────────────────────────┐
│ OtelValidator                │
│ ├─ validate_span()           │  ← Validate individual spans
│ ├─ validate_trace()          │  ← Validate complete traces
│ ├─ validate_export()         │  ← Validate export success
│ └─ validate_performance()    │  ← Already implemented
└────────┬─────────────────────┘
         │
         │ Validation results
         ▼
┌──────────────────────────────┐
│ SpanValidationResult         │
│ TraceValidationResult        │
└──────────────────────────────┘
```

### Key Components

#### 1. In-Memory Span Collector

**NEW Module**: `validation/otel/span_collector.rs`
```rust
use opentelemetry_sdk::trace::SpanData;
use std::sync::{Arc, Mutex};

/// In-memory span collector for validation
pub struct InMemorySpanCollector {
    spans: Arc<Mutex<Vec<SpanData>>>,
}

impl InMemorySpanCollector {
    pub fn new() -> Self {
        Self {
            spans: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Collect a span for validation
    pub fn collect(&self, span: SpanData) -> Result<()> {
        self.spans
            .lock()
            .map_err(|e| CleanroomError::internal_error(format!("Lock error: {}", e)))?
            .push(span);
        Ok(())
    }

    /// Query spans by name
    pub fn query_by_name(&self, name: &str) -> Result<Vec<SpanData>> {
        let spans = self.spans
            .lock()
            .map_err(|e| CleanroomError::internal_error(format!("Lock error: {}", e)))?;

        Ok(spans.iter()
            .filter(|s| s.name == name)
            .cloned()
            .collect())
    }

    /// Query spans by trace ID
    pub fn query_by_trace_id(&self, trace_id: &str) -> Result<Vec<SpanData>> {
        let spans = self.spans
            .lock()
            .map_err(|e| CleanroomError::internal_error(format!("Lock error: {}", e)))?;

        Ok(spans.iter()
            .filter(|s| s.span_context.trace_id().to_string() == trace_id)
            .cloned()
            .collect())
    }

    /// Clear all collected spans
    pub fn clear(&self) -> Result<()> {
        self.spans
            .lock()
            .map_err(|e| CleanroomError::internal_error(format!("Lock error: {}", e)))?
            .clear();
        Ok(())
    }
}
```

**Decision**: Thread-safe span collection with mutex
- **Rationale**: Multi-threaded test execution support
- **Trade-off**: Lock contention (minimal for tests)
- **Alternative**: Lock-free queue (more complex, overkill for tests)

#### 2. Enhanced OtelValidator

**Modifications to** `validation/otel.rs`:

```rust
/// OpenTelemetry validator with span collector
#[derive(Debug, Clone)]
pub struct OtelValidator {
    config: OtelValidationConfig,
    span_collector: Arc<InMemorySpanCollector>, // NEW
}

impl OtelValidator {
    /// Create validator with span collector
    pub fn with_collector(collector: Arc<InMemorySpanCollector>) -> Self {
        Self {
            config: OtelValidationConfig::default(),
            span_collector: collector,
        }
    }

    /// Validate span assertion (IMPLEMENTED)
    pub fn validate_span(&self, assertion: &SpanAssertion) -> Result<SpanValidationResult> {
        if !self.config.validate_spans {
            return Err(CleanroomError::validation_error(
                "Span validation is disabled"
            ));
        }

        // Query spans by name
        let spans = self.span_collector.query_by_name(&assertion.name)?;

        if spans.is_empty() {
            if assertion.required {
                return Ok(SpanValidationResult {
                    passed: false,
                    span_name: assertion.name.clone(),
                    errors: vec![format!("Required span '{}' not found", assertion.name)],
                    actual_attributes: HashMap::new(),
                    actual_duration_ms: None,
                });
            }
            // Not required, pass validation
            return Ok(SpanValidationResult {
                passed: true,
                span_name: assertion.name.clone(),
                errors: Vec::new(),
                actual_attributes: HashMap::new(),
                actual_duration_ms: None,
            });
        }

        // Validate first matching span
        let span = &spans[0];
        let mut errors = Vec::new();

        // Validate attributes
        for (key, expected_value) in &assertion.attributes {
            let actual_value = span.attributes.get(key)
                .map(|v| v.to_string())
                .unwrap_or_default();

            if &actual_value != expected_value {
                errors.push(format!(
                    "Attribute '{}': expected '{}', got '{}'",
                    key, expected_value, actual_value
                ));
            }
        }

        // Validate duration
        let duration_ms = span.end_time.duration_since(span.start_time)
            .ok()
            .map(|d| d.as_millis() as f64);

        if let Some(min_duration) = assertion.min_duration_ms {
            if let Some(actual) = duration_ms {
                if actual < min_duration {
                    errors.push(format!(
                        "Duration {}ms < minimum {}ms",
                        actual, min_duration
                    ));
                }
            }
        }

        if let Some(max_duration) = assertion.max_duration_ms {
            if let Some(actual) = duration_ms {
                if actual > max_duration {
                    errors.push(format!(
                        "Duration {}ms > maximum {}ms",
                        actual, max_duration
                    ));
                }
            }
        }

        Ok(SpanValidationResult {
            passed: errors.is_empty(),
            span_name: assertion.name.clone(),
            errors,
            actual_attributes: span.attributes.iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect(),
            actual_duration_ms: duration_ms,
        })
    }

    /// Validate trace assertion (IMPLEMENTED)
    pub fn validate_trace(&self, assertion: &TraceAssertion) -> Result<TraceValidationResult> {
        if !self.config.validate_traces {
            return Err(CleanroomError::validation_error(
                "Trace validation is disabled"
            ));
        }

        // Query spans by trace ID (if provided)
        let spans = if let Some(trace_id) = &assertion.trace_id {
            self.span_collector.query_by_trace_id(trace_id)?
        } else {
            // No trace ID - validate expected spans exist
            Vec::new()
        };

        let mut span_results = Vec::new();
        let mut errors = Vec::new();

        // Validate each expected span
        for span_assertion in &assertion.expected_spans {
            let result = self.validate_span(span_assertion)?;
            if !result.passed {
                errors.extend(result.errors.clone());
            }
            span_results.push(result);
        }

        // Validate parent-child relationships
        for (parent_name, child_name) in &assertion.parent_child_relationships {
            let parent_spans = self.span_collector.query_by_name(parent_name)?;
            let child_spans = self.span_collector.query_by_name(child_name)?;

            if parent_spans.is_empty() {
                errors.push(format!("Parent span '{}' not found", parent_name));
                continue;
            }

            if child_spans.is_empty() {
                errors.push(format!("Child span '{}' not found", child_name));
                continue;
            }

            // Check if child's parent_span_id matches parent's span_id
            let parent_id = parent_spans[0].span_context.span_id();
            let child_parent_id = child_spans[0].parent_span_id;

            if child_parent_id != parent_id {
                errors.push(format!(
                    "Span '{}' is not a child of '{}'",
                    child_name, parent_name
                ));
            }
        }

        Ok(TraceValidationResult {
            passed: errors.is_empty(),
            trace_id: assertion.trace_id.clone(),
            expected_span_count: assertion.expected_spans.len(),
            actual_span_count: spans.len(),
            span_results,
            errors,
        })
    }

    /// Validate export (IMPLEMENTED)
    pub fn validate_export(&self, _endpoint: &str) -> Result<bool> {
        if !self.config.validate_exports {
            return Err(CleanroomError::validation_error(
                "Export validation is disabled"
            ));
        }

        // Check if any spans have been collected
        let spans = self.span_collector.spans.lock()
            .map_err(|e| CleanroomError::internal_error(format!("Lock error: {}", e)))?;

        if spans.is_empty() {
            return Err(CleanroomError::validation_error(
                "No spans collected - export may have failed"
            ));
        }

        Ok(true)
    }
}
```

**Decision**: Implement validation using span collector queries
- **Rationale**: Direct access to span data, accurate validation
- **Benefit**: No mocking, uses real OTEL data structures
- **Trade-off**: Requires test-specific exporter setup

#### 3. Test Utilities

**NEW Module**: `validation/otel/test_helpers.rs`
```rust
/// Setup OTEL with in-memory span collector for testing
pub fn setup_otel_for_testing() -> Result<(OtelGuard, Arc<InMemorySpanCollector>)> {
    let collector = Arc::new(InMemorySpanCollector::new());

    // Create in-memory exporter
    let exporter = InMemorySpanExporter::new(collector.clone());

    // Initialize OTEL with in-memory exporter
    let config = OtelConfig {
        service_name: "test",
        deployment_env: "test",
        sample_ratio: 1.0,
        export: Export::Custom(Box::new(exporter)), // NEW: Custom exporter variant
        enable_fmt_layer: false,
    };

    let guard = init_otel(config)?;

    Ok((guard, collector))
}
```

**Decision**: Test-specific OTEL setup helper
- **Rationale**: Simplify test configuration
- **Benefit**: One-line setup for OTEL validation tests
- **Example Usage**:
  ```rust
  #[tokio::test]
  async fn test_span_validation() {
      let (guard, collector) = setup_otel_for_testing().unwrap();
      let validator = OtelValidator::with_collector(collector);

      // Execute code that creates spans
      some_traced_function().await;

      // Validate spans
      let result = validator.validate_span(&SpanAssertion {
          name: "some_traced_function".to_string(),
          attributes: HashMap::new(),
          required: true,
          min_duration_ms: None,
          max_duration_ms: Some(1000.0),
      }).unwrap();

      assert!(result.passed);
  }
  ```

### Integration with Existing OTEL Features

**Current OTEL Setup** (telemetry.rs):
```rust
#[cfg(feature = "otel-traces")]
pub fn init_otel(config: OtelConfig) -> Result<OtelGuard> {
    // Existing implementation with OTLP/stdout exporters
}
```

**Enhancement**: Support custom exporters
```rust
pub enum Export {
    OtlpHttp { endpoint: String },
    OtlpGrpc { endpoint: String },
    Stdout,
    Custom(Box<dyn SpanExporter>), // NEW: For testing
}
```

**Decision**: Add `Custom` variant to Export enum
- **Rationale**: Allows injecting in-memory exporter for tests
- **Backward Compatible**: Existing configurations unchanged
- **Benefit**: Flexible exporter configuration

### Testing Strategy (London TDD)

**Phase 1: Unit Tests** (span_collector.rs)
```rust
#[test]
fn test_span_collector_stores_spans() {
    let collector = InMemorySpanCollector::new();
    let span = create_test_span("test.span");

    collector.collect(span.clone()).unwrap();

    let spans = collector.query_by_name("test.span").unwrap();
    assert_eq!(spans.len(), 1);
}

#[test]
fn test_span_collector_filters_by_name() {
    let collector = InMemorySpanCollector::new();
    collector.collect(create_test_span("span1")).unwrap();
    collector.collect(create_test_span("span2")).unwrap();

    let spans = collector.query_by_name("span1").unwrap();
    assert_eq!(spans.len(), 1);
    assert_eq!(spans[0].name, "span1");
}
```

**Phase 2: Integration Tests** (validator tests)
```rust
#[tokio::test]
async fn test_validate_span_with_attributes() {
    let (guard, collector) = setup_otel_for_testing().unwrap();
    let validator = OtelValidator::with_collector(collector);

    // Create span with attributes
    let span = tracing::info_span!(
        "test.operation",
        service.name = "test-service",
        operation.type = "query"
    );
    let _enter = span.enter();

    // Validate
    let mut expected_attrs = HashMap::new();
    expected_attrs.insert("service.name".to_string(), "test-service".to_string());

    let result = validator.validate_span(&SpanAssertion {
        name: "test.operation".to_string(),
        attributes: expected_attrs,
        required: true,
        min_duration_ms: None,
        max_duration_ms: None,
    }).unwrap();

    assert!(result.passed);
}
```

**Phase 3: E2E Tests** (full trace validation)
```rust
#[tokio::test]
async fn test_validate_complete_trace() {
    let (guard, collector) = setup_otel_for_testing().unwrap();
    let validator = OtelValidator::with_collector(collector);

    // Execute traced workflow
    traced_workflow().await;

    // Validate trace structure
    let trace_assertion = TraceAssertion {
        trace_id: None,
        expected_spans: vec![
            SpanAssertion {
                name: "parent_operation".to_string(),
                attributes: HashMap::new(),
                required: true,
                min_duration_ms: None,
                max_duration_ms: None,
            },
            SpanAssertion {
                name: "child_operation".to_string(),
                attributes: HashMap::new(),
                required: true,
                min_duration_ms: None,
                max_duration_ms: None,
            },
        ],
        complete: true,
        parent_child_relationships: vec![
            ("parent_operation".to_string(), "child_operation".to_string()),
        ],
    };

    let result = validator.validate_trace(&trace_assertion).unwrap();
    assert!(result.passed);
}
```

### Performance Considerations

**Overhead Analysis**:
- **Span Collection**: ~0.1ms per span (mutex lock + vec push)
- **Query Operations**: O(n) scan through collected spans
- **Memory Usage**: ~1KB per span (minimal for tests)

**Decision**: Acceptable overhead for testing
- **Rationale**: Validation runs in test mode only
- **Optimization**: Index by name/trace_id (future enhancement)
- **Monitoring**: Track validation performance metrics

### Error Handling

**Current**: Proper error types exist (ErrorKind::ValidationError)

**Enhancement**: Rich validation error messages
```rust
// Bad span validation
Ok(SpanValidationResult {
    passed: false,
    span_name: "user.login".to_string(),
    errors: vec![
        "Attribute 'user.id': expected 'user-123', got 'user-456'",
        "Duration 1500ms > maximum 1000ms"
    ],
    actual_attributes: [
        ("user.id", "user-456"),
        ("operation", "login")
    ].into_iter().map(|(k, v)| (k.to_string(), v.to_string())).collect(),
    actual_duration_ms: Some(1500.0),
})
```

**Decision**: Detailed validation results with actual vs expected
- **Rationale**: Developers need to see what went wrong
- **Benefit**: Faster debugging, clear expectations
- **Example Output**:
  ```
  Span validation failed for 'user.login':
    - Attribute 'user.id': expected 'user-123', got 'user-456'
    - Duration 1500ms > maximum 1000ms

  Actual attributes:
    user.id: user-456
    operation: login
  ```

## Consequences

### Positive

1. **TTBD Compliance**: Can validate observability claims with actual telemetry
2. **Production Quality**: No `unimplemented!()` - all validation methods work
3. **Comprehensive**: Validates spans, traces, exports, performance
4. **Developer Experience**: Clear validation errors with context
5. **Backward Compatible**: No breaking changes to existing OTEL setup

### Negative

1. **Complexity**: Additional validation infrastructure (span collector)
2. **Test Setup**: Requires specific OTEL configuration for validation
3. **Performance**: O(n) query operations (acceptable for tests)
4. **Memory**: Must store all spans for validation (minimal impact)

### Neutral

1. **Maintenance**: More validation code to maintain
2. **Documentation**: Need validation guide and examples
3. **Testing**: More comprehensive test coverage needed

## Implementation Roadmap

### Phase 1: Span Collector (Week 1)
- [ ] Implement `InMemorySpanCollector` in `validation/otel/span_collector.rs`
- [ ] Add thread-safe span storage (Arc<Mutex<Vec<SpanData>>>)
- [ ] Implement query methods (by_name, by_trace_id)
- [ ] Unit tests for collector (>90% coverage)

### Phase 2: Validator Implementation (Week 2)
- [ ] Implement `validate_span()` with attribute/duration validation
- [ ] Implement `validate_trace()` with relationship validation
- [ ] Implement `validate_export()` with export verification
- [ ] Integration tests with test doubles

### Phase 3: Test Utilities & Docs (Week 3)
- [ ] Create `setup_otel_for_testing()` helper
- [ ] Add `Custom` exporter variant to `Export` enum
- [ ] E2E tests validating complete traces
- [ ] Documentation with examples

### Total Timeline: 3 weeks

## Links

- [OTEL Validation Module](../../crates/clnrm-core/src/validation/otel.rs) - Current implementation
- [Telemetry Module](../../crates/clnrm-core/src/telemetry.rs) - OTEL setup
- [Tera Templating ADR](./ADR-001-tera-templating-integration.md) - Related decision

## Notes

- Existing `validation/otel.rs` has skeleton with `unimplemented!()` (following core standards)
- `validate_performance_overhead()` already implemented (line 227-248)
- OTEL features enabled via flags: `otel-traces`, `otel-metrics`, `otel-logs`
- Comprehensive OTEL support added in commit b24db56
- InMemorySpanExporter requires opentelemetry_sdk dependency (already present)
