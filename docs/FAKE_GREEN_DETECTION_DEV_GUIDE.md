# Fake-Green Detection Developer Guide

## Table of Contents

1. [Architecture Overview](#architecture-overview)
2. [How Validators Work](#how-validators-work)
3. [Adding New Validators](#adding-new-validators)
4. [Testing Strategy](#testing-strategy)
5. [Debugging Tips](#debugging-tips)
6. [Performance Considerations](#performance-considerations)
7. [API Reference](#api-reference)

---

## Architecture Overview

### System Components

```
┌─────────────────────────────────────────────────────────────┐
│                    Test Execution                           │
│  ┌───────────┐       ┌────────────┐       ┌──────────┐     │
│  │ Container │  -->  │    Code    │  -->  │   OTEL   │     │
│  │  Launch   │       │ Execution  │       │  Traces  │     │
│  └───────────┘       └────────────┘       └──────────┘     │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│            OTEL Trace Collection (JSON)                     │
│  spans: [                                                   │
│    { name: "clnrm.run", span_id: "abc", ... },             │
│    { name: "container.exec", parent_id: "abc", ... }       │
│  ]                                                          │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
┌─────────────────────────────────────────────────────────────┐
│            Validation Orchestrator                          │
│  ┌──────────────────────────────────────────────────┐      │
│  │  1. Load Test Config (.clnrm.toml)              │      │
│  │  2. Load OTEL Traces (spans.json)               │      │
│  │  3. Extract Expectations from Config             │      │
│  │  4. Run Each Validator                           │      │
│  │  5. Aggregate Results                            │      │
│  └──────────────────────────────────────────────────┘      │
└─────────────────────────────────────────────────────────────┘
                              │
          ┌───────────────────┼───────────────────┐
          ▼                   ▼                   ▼
    ┌──────────┐       ┌──────────┐       ┌──────────┐
    │ Validator│       │ Validator│  ...  │ Validator│
    │    #1    │       │    #2    │       │    #7    │
    └──────────┘       └──────────┘       └──────────┘
          │                   │                   │
          └───────────────────┼───────────────────┘
                              ▼
┌─────────────────────────────────────────────────────────────┐
│                 Validation Report                           │
│  {                                                          │
│    test_name: "my_test",                                   │
│    validators: [                                           │
│      { name: "Span Expectations", passed: true },         │
│      { name: "Graph Structure", passed: false },          │
│    ],                                                      │
│    is_success: false                                       │
│  }                                                          │
└─────────────────────────────────────────────────────────────┘
```

### Key Modules

**Location:** `crates/clnrm-core/src/`

```
validation/
├── span_validator.rs       # SpanData model and span loading
├── graph_validator.rs      # Layer 2: Graph topology
├── count_validator.rs      # Layer 3: Cardinality
├── order_validator.rs      # Layer 4: Temporal ordering
├── window_validator.rs     # Layer 5: Time window containment
├── status_validator.rs     # Layer 6: Status code validation
└── hermeticity_validator.rs  # Layer 7: Isolation validation

cli/commands/v0_7_0/
└── analyze.rs              # CLI command and orchestrator
```

---

## How Validators Work

### Validator Interface

All validators follow a consistent pattern:

```rust
/// Generic validator interface (conceptual)
pub trait Validator {
    /// Configuration for this validator
    type Config;

    /// Create validator from configuration
    fn new(config: Self::Config) -> Self;

    /// Validate spans against expectations
    fn validate(&self, spans: &[SpanData]) -> Result<()>;
}
```

### SpanData Model

All validators operate on the `SpanData` structure:

```rust
/// OTEL span data extracted from traces
pub struct SpanData {
    /// Span name (e.g., "clnrm.run", "container.exec")
    pub name: String,

    /// Unique span ID
    pub span_id: String,

    /// Parent span ID (if any)
    pub parent_span_id: Option<String>,

    /// Trace ID (all spans in trace share same ID)
    pub trace_id: String,

    /// Span attributes (key-value pairs)
    pub attributes: HashMap<String, serde_json::Value>,

    /// Resource attributes (environment metadata)
    pub resource_attributes: HashMap<String, serde_json::Value>,

    /// Start time (nanoseconds since Unix epoch)
    pub start_time_unix_nano: Option<u64>,

    /// End time (nanoseconds since Unix epoch)
    pub end_time_unix_nano: Option<u64>,

    /// Span kind (client, server, internal, etc.)
    pub kind: Option<String>,

    /// Span events (log entries within span)
    pub events: Option<Vec<String>>,
}
```

### Validation Flow

**1. Load Spans:**
```rust
// In analyze.rs
let validator = SpanValidator::from_file(traces_file)?;
let spans = validator.spans();  // Vec<SpanData>
```

**2. Extract Config:**
```rust
// Parse TOML to get expectations
let config: TestConfig = toml::from_str(&config_str)?;
let expect = config.expect.unwrap();
```

**3. Run Validators:**
```rust
// Example: Graph validator
if let Some(ref graph_config) = expect.graph {
    let graph = GraphExpectation::new(
        graph_config.must_include.clone()
    );

    match graph.validate(&spans) {
        Ok(_) => /* validation passed */,
        Err(e) => /* validation failed: e */,
    }
}
```

**4. Aggregate Results:**
```rust
let report = AnalysisReport {
    test_name: "my_test".to_string(),
    validators: vec![
        ValidatorResult {
            name: "Graph Structure".to_string(),
            passed: true,
            details: "all edges present".to_string(),
        },
    ],
    // ...
};
```

---

## Adding New Validators

### Step 1: Define Expectation Structure

Create `src/validation/my_validator.rs`:

```rust
use crate::error::{CleanroomError, Result};
use crate::validation::span_validator::SpanData;
use serde::{Deserialize, Serialize};

/// Expectation for my custom validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyExpectation {
    /// Configuration fields
    pub some_field: String,
    pub another_field: Option<usize>,
}

impl MyExpectation {
    /// Create a new expectation
    pub fn new(some_field: String) -> Self {
        Self {
            some_field,
            another_field: None,
        }
    }

    /// Builder pattern for optional fields
    pub fn with_another_field(mut self, value: usize) -> Self {
        self.another_field = Some(value);
        self
    }

    /// Main validation logic
    pub fn validate(&self, spans: &[SpanData]) -> Result<()> {
        // Implement validation logic here
        for span in spans {
            // Check your condition
            if !self.check_condition(span) {
                return Err(CleanroomError::validation_error(format!(
                    "My validation failed for span '{}'",
                    span.name
                )));
            }
        }
        Ok(())
    }

    /// Helper methods
    fn check_condition(&self, span: &SpanData) -> bool {
        // Your logic here
        span.attributes.contains_key(&self.some_field)
    }
}
```

### Step 2: Add TOML Configuration

Update `src/config/otel.rs` (or relevant config module):

```rust
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectationConfig {
    // Existing validators...
    pub span: Option<Vec<SpanExpectationConfig>>,
    pub graph: Option<GraphExpectationConfig>,

    // Your new validator
    pub my_validator: Option<MyExpectationConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MyExpectationConfig {
    pub some_field: String,
    pub another_field: Option<usize>,
}
```

### Step 3: Integrate with Analyzer

Update `src/cli/commands/v0_7_0/analyze.rs`:

```rust
// Import your validator
use crate::validation::my_validator::MyExpectation;

pub fn analyze_traces(test_file: &Path, traces_file: &Path) -> Result<AnalysisReport> {
    // ... existing setup ...

    // Run validators based on expectations in config
    if let Some(ref expect) = config.expect {
        // ... existing validators ...

        // 8. Your New Validator
        if let Some(ref my_config) = expect.my_validator {
            let result = validate_my_expectation(my_config, spans);
            report.validators.push(result);
        }
    }

    Ok(report)
}

/// Validate custom expectation
fn validate_my_expectation(
    config: &MyExpectationConfig,
    spans: &[SpanData],
) -> ValidatorResult {
    let expectation = MyExpectation::new(config.some_field.clone())
        .with_another_field(config.another_field.unwrap_or(0));

    match expectation.validate(spans) {
        Ok(_) => ValidatorResult {
            name: "My Validator".to_string(),
            passed: true,
            details: "all checks passed".to_string(),
        },
        Err(e) => ValidatorResult {
            name: "My Validator".to_string(),
            passed: false,
            details: format!("FAIL: {}", e),
        },
    }
}
```

### Step 4: Write Tests

Add comprehensive tests in `my_validator.rs`:

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn create_test_span(name: &str) -> SpanData {
        let mut attrs = HashMap::new();
        attrs.insert("test_attr".to_string(), json!("value"));

        SpanData {
            name: name.to_string(),
            span_id: "span123".to_string(),
            parent_span_id: None,
            trace_id: "trace123".to_string(),
            attributes: attrs,
            start_time_unix_nano: Some(1000),
            end_time_unix_nano: Some(2000),
            kind: None,
            events: None,
            resource_attributes: HashMap::new(),
        }
    }

    #[test]
    fn test_my_expectation_passes() {
        // Arrange
        let spans = vec![create_test_span("test_span")];
        let expectation = MyExpectation::new("test_attr".to_string());

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_ok());
    }

    #[test]
    fn test_my_expectation_fails() {
        // Arrange
        let spans = vec![create_test_span("test_span")];
        let expectation = MyExpectation::new("missing_attr".to_string());

        // Act
        let result = expectation.validate(&spans);

        // Assert
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("failed"));
    }
}
```

### Step 5: Document in TOML Schema

Update `docs/FAKE_GREEN_TOML_SCHEMA.md`:

````markdown
### `[expect.my_validator]`

Custom validator description.

**Fields:**
- `some_field` (string, required): Description
- `another_field` (integer, optional): Description

**Example:**
```toml
[expect.my_validator]
some_field = "value"
another_field = 42
```
````

---

## Testing Strategy

### Unit Tests

Test validators in isolation:

```rust
#[test]
fn test_validator_specific_behavior() {
    // Arrange: Create minimal test data
    let span = create_test_span("test");
    let expectation = MyExpectation::new("test");

    // Act: Run validation
    let result = expectation.validate(&[span]);

    // Assert: Verify behavior
    assert!(result.is_ok());
}
```

### Integration Tests

Test validators with realistic spans:

```rust
#[test]
fn test_validator_with_real_traces() {
    // Arrange: Load actual OTEL traces
    let traces_path = "tests/fixtures/real_traces.json";
    let validator = SpanValidator::from_file(Path::new(traces_path)).unwrap();
    let spans = validator.spans();

    // Act: Run your validator
    let expectation = MyExpectation::new("clnrm");
    let result = expectation.validate(spans);

    // Assert: Should pass with real data
    assert!(result.is_ok());
}
```

### Property-Based Tests

Use `proptest` for exhaustive testing:

```rust
#[cfg(feature = "proptest")]
mod property_tests {
    use super::*;
    use proptest::prelude::*;

    proptest! {
        #[test]
        fn test_validator_never_panics(
            span_count in 0usize..100,
            attr_key in "\\w{1,20}",
        ) {
            // Generate random spans
            let spans: Vec<SpanData> = (0..span_count)
                .map(|i| create_test_span(&format!("span_{}", i)))
                .collect();

            // Validator should never panic
            let expectation = MyExpectation::new(attr_key);
            let _ = expectation.validate(&spans);
        }
    }
}
```

### Test Coverage Checklist

- [ ] Empty spans array
- [ ] Single span
- [ ] Multiple spans
- [ ] Missing required attributes
- [ ] Invalid attribute values
- [ ] Missing parent relationships
- [ ] Missing timestamps
- [ ] Duplicate span IDs
- [ ] Null/None values
- [ ] Large span counts (10,000+)

---

## Debugging Tips

### Enable Tracing

```rust
// Add to your validator
use tracing::{debug, trace};

pub fn validate(&self, spans: &[SpanData]) -> Result<()> {
    debug!("Starting validation with {} spans", spans.len());

    for span in spans {
        trace!("Validating span: {}", span.name);
        // ... validation logic
    }

    debug!("Validation complete");
    Ok(())
}
```

Run with logging:
```bash
RUST_LOG=debug clnrm analyze test.toml traces.json
```

### Print Span Details

```rust
/// Debug helper to pretty-print span
fn debug_span(span: &SpanData) {
    eprintln!("Span Debug Info:");
    eprintln!("  Name: {}", span.name);
    eprintln!("  ID: {}", span.span_id);
    eprintln!("  Parent: {:?}", span.parent_span_id);
    eprintln!("  Attributes: {:#?}", span.attributes);
    eprintln!("  Start: {:?}", span.start_time_unix_nano);
    eprintln!("  End: {:?}", span.end_time_unix_nano);
}
```

### Inspect OTEL Traces

```bash
# Pretty-print collected traces
jq '.' traces.json

# Count spans
jq '.spans | length' traces.json

# List span names
jq '.spans[].name' traces.json

# Show span with specific name
jq '.spans[] | select(.name == "clnrm.run")' traces.json

# Show parent-child relationships
jq '.spans[] | {name, parent: .parent_span_id}' traces.json
```

### Common Issues

**Issue:** Spans not found
```rust
// Check if spans were collected
if spans.is_empty() {
    eprintln!("WARNING: No spans collected!");
    eprintln!("Check: OTEL exporter, endpoint, and instrumentation");
}
```

**Issue:** Attributes have wrong format
```rust
// OTEL attributes can be nested objects
// Wrong:
let value = span.attributes.get("key").as_str();

// Right:
let value = span.attributes.get("key")
    .and_then(|v| match v {
        serde_json::Value::String(s) => Some(s.as_str()),
        serde_json::Value::Object(obj) => {
            // Handle OTEL format: {"stringValue": "..."}
            obj.get("stringValue").and_then(|v| v.as_str())
        }
        _ => None,
    });
```

---

## Performance Considerations

### Optimization Strategies

**1. Use HashMap for Lookups:**
```rust
// Slow: O(n) for each lookup
fn find_span_slow(spans: &[SpanData], span_id: &str) -> Option<&SpanData> {
    spans.iter().find(|s| s.span_id == span_id)
}

// Fast: O(1) after initial O(n) build
fn find_span_fast<'a>(
    span_map: &'a HashMap<String, &'a SpanData>,
    span_id: &str
) -> Option<&'a SpanData> {
    span_map.get(span_id).copied()
}
```

**2. Early Returns:**
```rust
// Stop validation on first failure
for span in spans {
    if !validate_span(span)? {
        return Err(/* error */);  // Don't check remaining spans
    }
}
```

**3. Lazy Evaluation:**
```rust
// Don't compute unless needed
if let Some(ref config) = my_config {
    // Only run expensive computation if config exists
    let result = expensive_validation(spans)?;
}
```

### Benchmarking

Add benchmarks in `benches/`:

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_my_validator(c: &mut Criterion) {
    let spans: Vec<SpanData> = (0..1000)
        .map(|i| create_test_span(&format!("span_{}", i)))
        .collect();

    let expectation = MyExpectation::new("test".to_string());

    c.bench_function("my_validator_1000_spans", |b| {
        b.iter(|| {
            expectation.validate(black_box(&spans)).unwrap();
        });
    });
}

criterion_group!(benches, benchmark_my_validator);
criterion_main!(benches);
```

Run benchmarks:
```bash
cargo bench --bench my_validator
```

---

## API Reference

### Core Types

#### `SpanData`

```rust
pub struct SpanData {
    pub name: String,
    pub span_id: String,
    pub parent_span_id: Option<String>,
    pub trace_id: String,
    pub attributes: HashMap<String, serde_json::Value>,
    pub resource_attributes: HashMap<String, serde_json::Value>,
    pub start_time_unix_nano: Option<u64>,
    pub end_time_unix_nano: Option<u64>,
    pub kind: Option<String>,
    pub events: Option<Vec<String>>,
}
```

#### `AnalysisReport`

```rust
pub struct AnalysisReport {
    pub test_name: String,
    pub traces_file: String,
    pub span_count: usize,
    pub event_count: usize,
    pub digest: String,
    pub validators: Vec<ValidatorResult>,
}

impl AnalysisReport {
    pub fn is_success(&self) -> bool;
    pub fn failure_count(&self) -> usize;
    pub fn pass_count(&self) -> usize;
    pub fn format_report(&self) -> String;
}
```

#### `ValidatorResult`

```rust
pub struct ValidatorResult {
    pub name: String,
    pub passed: bool,
    pub details: String,
}
```

### Error Handling

```rust
use crate::error::{CleanroomError, Result};

// Create validation errors
CleanroomError::validation_error("message")

// Create config errors
CleanroomError::config_error("message")

// Create internal errors
CleanroomError::internal_error("message")
```

### Common Patterns

**Iterate over spans by name:**
```rust
let matching_spans: Vec<&SpanData> = spans
    .iter()
    .filter(|s| s.name == "container.exec")
    .collect();
```

**Build span lookup map:**
```rust
let span_by_id: HashMap<String, &SpanData> = spans
    .iter()
    .map(|s| (s.span_id.clone(), s))
    .collect();
```

**Check attribute exists:**
```rust
if span.attributes.contains_key("my_attr") {
    // Attribute exists
}
```

**Extract string attribute:**
```rust
let value = span.attributes
    .get("my_attr")
    .and_then(|v| v.as_str())
    .unwrap_or("default");
```

---

## Summary

**Key Principles:**
1. All validators operate on `SpanData`
2. Use `Result<()>` for validation results
3. Provide detailed error messages
4. Write comprehensive tests
5. Document TOML schema
6. Optimize for common case (validation passes)

**Next Steps:**
- Review existing validators in `src/validation/`
- Read [TOML Schema Reference](FAKE_GREEN_TOML_SCHEMA.md)
- Check [User Guide](FAKE_GREEN_DETECTION_USER_GUIDE.md) for context
- See [CLI Reference](CLI_ANALYZE_REFERENCE.md) for usage

**Questions?** See [project documentation](.) or file an issue.
