# CountValidator Usage Examples

## Real-World TOML Configuration Examples

### Example 1: Basic Test Validation

```toml
[test.metadata]
name = "basic_count_test"
description = "Validate basic span and error counts"

[expect.counts]
spans_total = { gte = 2, lte = 200 }
errors_total = { eq = 0 }
```

**What this validates**:
- At least 2 spans were created, but no more than 200
- No spans have error status
- Perfect for successful test execution validation

### Example 2: Complete PRD Example

```toml
[test.metadata]
name = "hello_world_test"
description = "Validate the hello world test produces expected telemetry"

[expect.counts]
spans_total = { gte = 2, lte = 200 }
errors_total = { eq = 0 }
events_total = { gte = 2 }

[expect.counts.by_name]
"clnrm.run" = { eq = 1 }
"clnrm.step:hello_world" = { gte = 1 }
```

**What this validates**:
- Total span count between 2-200 (reasonable range)
- No error spans (test passed)
- At least 2 events were recorded
- Exactly 1 "clnrm.run" span (top-level test execution)
- At least 1 "clnrm.step:hello_world" span (step execution)

### Example 3: Error Detection Test

```toml
[test.metadata]
name = "failure_test"
description = "Test that intentionally fails and validates error reporting"

[expect.counts]
spans_total = { gte = 1 }
errors_total = { eq = 1 }  # Expect exactly 1 error

[expect.counts.by_name]
"clnrm.step:failing_step" = { eq = 1 }
```

**What this validates**:
- At least 1 span was created
- Exactly 1 span has error status
- The failing step span exists
- Perfect for testing error handling

### Example 4: Multi-Step Test

```toml
[test.metadata]
name = "complex_workflow"
description = "Multi-step workflow with precise count requirements"

[expect.counts]
spans_total = { gte = 5, lte = 10 }
errors_total = { eq = 0 }
events_total = { gte = 10 }

[expect.counts.by_name]
"clnrm.run" = { eq = 1 }
"clnrm.step:setup" = { eq = 1 }
"clnrm.step:test_step_1" = { eq = 1 }
"clnrm.step:test_step_2" = { eq = 1 }
"clnrm.step:cleanup" = { eq = 1 }
```

**What this validates**:
- Total of 5-10 spans (accounts for internal framework spans)
- No errors in entire workflow
- At least 10 events across all spans
- Each step runs exactly once
- Linear workflow execution

### Example 5: Range Validation

```toml
[test.metadata]
name = "load_test"
description = "Test that processes variable number of items"

[expect.counts]
spans_total = { gte = 10, lte = 100 }
errors_total = { lte = 5 }  # Allow up to 5 errors

[expect.counts.by_name]
"clnrm.step:process_item" = { gte = 10, lte = 50 }
"clnrm.step:validate_item" = { gte = 10, lte = 50 }
```

**What this validates**:
- Between 10-100 total spans
- Up to 5 error spans allowed (some failures expected)
- Each item processes 10-50 times (variable load)
- Validation step runs same number of times as processing

## Rust API Examples

### Example 1: Programmatic Validation

```rust
use clnrm_core::validation::{CountExpectation, CountBound};

fn validate_test_execution(spans: &[SpanData]) -> Result<()> {
    let expectations = CountExpectation::new()
        .with_spans_total(CountBound::range(2, 200)?)
        .with_errors_total(CountBound::eq(0))
        .with_events_total(CountBound::gte(2))
        .with_name_count("clnrm.run".to_string(), CountBound::eq(1))
        .with_name_count("clnrm.step:hello_world".to_string(), CountBound::gte(1));

    expectations.validate(spans)?;

    println!("✅ All count validations passed!");
    Ok(())
}
```

### Example 2: Detailed Error Reporting

```rust
use clnrm_core::validation::{CountExpectation, CountBound};

fn validate_with_details(spans: &[SpanData]) {
    let expectations = CountExpectation::new()
        .with_spans_total(CountBound::range(5, 10).unwrap())
        .with_errors_total(CountBound::eq(0));

    match expectations.validate(spans) {
        Ok(()) => {
            println!("✅ Validation passed");
            println!("   - Total spans: {}", spans.len());
            println!("   - Error spans: 0");
        }
        Err(e) => {
            println!("❌ Validation failed");
            println!("   Error: {}", e);
            println!("   Actual span count: {}", spans.len());
        }
    }
}
```

### Example 3: Custom Constraint Builder

```rust
use clnrm_core::validation::{CountExpectation, CountBound};

struct TestExpectations {
    min_spans: usize,
    max_spans: usize,
    allow_errors: bool,
}

impl TestExpectations {
    fn to_count_expectation(&self) -> Result<CountExpectation> {
        let mut expectation = CountExpectation::new()
            .with_spans_total(CountBound::range(self.min_spans, self.max_spans)?);

        if !self.allow_errors {
            expectation = expectation.with_errors_total(CountBound::eq(0));
        }

        Ok(expectation)
    }
}

// Usage
let test_config = TestExpectations {
    min_spans: 2,
    max_spans: 200,
    allow_errors: false,
};

let expectations = test_config.to_count_expectation()?;
expectations.validate(&spans)?;
```

### Example 4: Validation Pipeline

```rust
use clnrm_core::validation::{CountExpectation, CountBound};

fn validate_test_lifecycle(spans: &[SpanData]) -> Result<()> {
    // Phase 1: Basic count validation
    CountExpectation::new()
        .with_spans_total(CountBound::gte(1))
        .validate(spans)?;

    // Phase 2: Error validation
    CountExpectation::new()
        .with_errors_total(CountBound::eq(0))
        .validate(spans)?;

    // Phase 3: Specific span validation
    CountExpectation::new()
        .with_name_count("clnrm.run".to_string(), CountBound::eq(1))
        .with_name_count("clnrm.step:test".to_string(), CountBound::gte(1))
        .validate(spans)?;

    println!("✅ All lifecycle validations passed");
    Ok(())
}
```

## Error Messages Reference

### Spans Total Errors

```
❌ Total span count: expected exactly 5 items, found 3
❌ Total span count: expected at least 2 items, found 1
❌ Total span count: expected at most 10 items, found 15
```

### Errors Total Errors

```
❌ Total error count: expected exactly 0 items, found 3
❌ Total error count: expected at least 1 items, found 0
❌ Total error count: expected at most 5 items, found 8
```

### Events Total Errors

```
❌ Total event count: expected exactly 10 items, found 7
❌ Total event count: expected at least 2 items, found 1
❌ Total event count: expected at most 20 items, found 25
```

### By Name Errors

```
❌ Count for span name 'clnrm.run': expected exactly 1 items, found 2
❌ Count for span name 'clnrm.step:hello_world': expected at least 1 items, found 0
❌ Count for span name 'clnrm.step:test': expected at most 3 items, found 5
```

## Best Practices

### 1. Use Ranges for Flexible Tests

```toml
# ✅ GOOD: Allows for framework overhead
spans_total = { gte = 2, lte = 200 }

# ❌ BAD: Too restrictive, may break with framework changes
spans_total = { eq = 3 }
```

### 2. Always Validate Error Counts

```toml
# ✅ GOOD: Explicitly checks for no errors
errors_total = { eq = 0 }

# ❌ BAD: Missing error validation
# (no errors_total specified)
```

### 3. Use Specific Span Names

```toml
# ✅ GOOD: Validates specific spans
[expect.counts.by_name]
"clnrm.step:my_specific_step" = { eq = 1 }

# ❌ BAD: Generic names are hard to validate
[expect.counts.by_name]
"step" = { gte = 1 }
```

### 4. Combine Multiple Constraints

```toml
# ✅ GOOD: Comprehensive validation
[expect.counts]
spans_total = { gte = 2, lte = 200 }
errors_total = { eq = 0 }
events_total = { gte = 2 }

[expect.counts.by_name]
"clnrm.run" = { eq = 1 }
"clnrm.step:test" = { gte = 1 }

# ❌ BAD: Only validates one thing
[expect.counts]
spans_total = { gte = 1 }
```

## Common Patterns

### Pattern: Successful Test Validation

```toml
[expect.counts]
spans_total = { gte = 2, lte = 200 }
errors_total = { eq = 0 }
```

### Pattern: Failure Test Validation

```toml
[expect.counts]
spans_total = { gte = 1 }
errors_total = { gte = 1 }  # Expect at least one error
```

### Pattern: Repeated Step Validation

```toml
[expect.counts.by_name]
"clnrm.step:process" = { gte = 5, lte = 10 }  # Run 5-10 times
```

### Pattern: Single Entry Point Validation

```toml
[expect.counts.by_name]
"clnrm.run" = { eq = 1 }  # Only one top-level execution
```

## Troubleshooting

### Issue: "Expected exactly 1, found 0"

**Cause**: Span name mismatch or span not created

**Solution**:
1. Check span name spelling
2. Verify step actually executed
3. Use `spans_total` to check if any spans exist

### Issue: "Expected exactly 0 errors, found 1"

**Cause**: Test had an error but expects success

**Solution**:
1. Check test logs for actual error
2. Fix the underlying test failure
3. Or update expectation if error is expected

### Issue: "Expected at least 2 items, found 1"

**Cause**: Not enough spans/events created

**Solution**:
1. Check if test steps actually ran
2. Verify OTEL instrumentation is active
3. Lower minimum if expectation is too high

## Integration Testing

### Complete Integration Example

```rust
#[tokio::test]
async fn test_count_validator_integration() -> Result<()> {
    // Arrange
    let spans = vec![
        SpanData {
            name: "clnrm.run".to_string(),
            attributes: HashMap::new(),
            trace_id: "trace1".to_string(),
            span_id: "span1".to_string(),
            parent_span_id: None,
            start_time_unix_nano: Some(1000000),
            end_time_unix_nano: Some(2000000),
            kind: None,
            events: None,
            resource_attributes: HashMap::new(),
        },
        SpanData {
            name: "clnrm.step:hello_world".to_string(),
            attributes: {
                let mut attrs = HashMap::new();
                attrs.insert("event.count".to_string(), json!(2));
                attrs
            },
            trace_id: "trace1".to_string(),
            span_id: "span2".to_string(),
            parent_span_id: Some("span1".to_string()),
            start_time_unix_nano: Some(1500000),
            end_time_unix_nano: Some(2500000),
            kind: None,
            events: None,
            resource_attributes: HashMap::new(),
        },
    ];

    // Act
    let expectations = CountExpectation::new()
        .with_spans_total(CountBound::range(2, 200)?)
        .with_errors_total(CountBound::eq(0))
        .with_events_total(CountBound::gte(2))
        .with_name_count("clnrm.run".to_string(), CountBound::eq(1))
        .with_name_count("clnrm.step:hello_world".to_string(), CountBound::eq(1));

    let result = expectations.validate(&spans);

    // Assert
    assert!(result.is_ok(), "Validation should pass: {:?}", result);
    Ok(())
}
```

## Summary

The CountValidator provides powerful cardinality validation for OTEL traces:

- ✅ **Flexible Constraints**: `eq`, `gte`, `lte` support
- ✅ **Multiple Dimensions**: Total spans, errors, events, by-name
- ✅ **Clear Errors**: Descriptive messages with context
- ✅ **TOML Integration**: Easy configuration in `.clnrm.toml`
- ✅ **Rust API**: Programmatic validation support

Perfect for validating test execution completeness and correctness through telemetry.
