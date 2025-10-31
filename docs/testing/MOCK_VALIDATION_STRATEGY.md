# Mock Validation Strategy for Weaver Integration

## Overview

This document outlines the London School TDD validation strategy for the Weaver integration refactor. It explains the two-phase validation approach and why both phases are critical.

## The London School TDD Approach

### Core Principles

1. **Schema Defines Contract**: The Weaver schema (YAML) is the source of truth for interfaces
2. **Mocks Generated from Schema**: Mockall mocks generated from schema ensure type safety
3. **Red Phase**: Write failing tests using mocks
4. **Green Phase**: Implement features to satisfy mocks
5. **Proof Phase**: Weaver live-check validates runtime behavior

### Why London School?

London School TDD (mockist approach) is ideal for telemetry validation because:

- **Focus on Interactions**: We care HOW objects collaborate to emit telemetry
- **Contract-First**: Schema defines contracts, mocks enforce them
- **Isolation**: Test units in isolation from infrastructure
- **Early Feedback**: Discover interface issues before implementation

## Two-Phase Validation: Mocks + Weaver

### Phase 1: Mock Tests (Interface Contract)

**What Mocks Prove:**
- ✅ Required attributes are set
- ✅ Correct types are used
- ✅ Proper method calls occur
- ✅ Error cases handled
- ✅ Lifecycle sequences correct

**What Mocks DON'T Prove:**
- ❌ Containers actually ran
- ❌ Telemetry actually exported
- ❌ OTLP endpoints reachable
- ❌ Schema conformance at runtime

**Example Mock Test:**

```rust
#[tokio::test]
async fn test_execution_exports_required_telemetry() {
    // Arrange - Mock from schema
    let mut mock_span = MockTestExecutionSpanTrait::new();

    // Expect ALL required attributes from schema
    mock_span.expect_set_container_id()
        .with(eq("test-container-123"))
        .times(1)
        .returning(|_| ());

    mock_span.expect_set_test_name()
        .with(eq("my_test"))
        .times(1)
        .returning(|_| ());

    mock_span.expect_set_isolated()
        .with(eq(true))
        .times(1)
        .returning(|_| ());

    mock_span.expect_set_test_result()
        .with(eq(TestResult::Pass))
        .times(1)
        .returning(|_| ());

    // Act
    let result = execute_test_with_span(
        "my_test",
        "alpine:latest",
        mock_span
    ).await;

    // Assert
    assert!(result.is_ok());
    // Mock automatically verifies all expectations were met
}
```

### Phase 2: Weaver Live-Check (Runtime Validation)

**What Weaver Proves:**
- ✅ Actual runtime telemetry matches schema
- ✅ Containers did run (container.id exists in actual spans)
- ✅ Isolation did work (test.isolated = true in actual telemetry)
- ✅ ALL required attributes present in exported telemetry
- ✅ Enum values match schema definitions
- ✅ Span hierarchy correct
- ✅ Metrics recorded with correct units

**What Weaver Validates:**

```bash
# Run tests with OTEL export
clnrm run tests/ --otel-exporter file --otel-export-path telemetry.json

# Validate exported telemetry against schema
weaver registry live-check \
  --registry registry/ \
  --telemetry telemetry.json

# Output:
✅ All spans match schema
✅ All required attributes present
✅ No unexpected attributes
✅ Enum values valid
✅ Span hierarchy correct
```

## Why Both Are Required

### Mock Tests Without Weaver = False Positives

```
Scenario: Developer forgets to export span
Mock Test: ✅ PASSES (interface called correctly)
Reality: ❌ No telemetry exported
Result: FALSE POSITIVE
```

### Weaver Without Mock Tests = Late Feedback

```
Scenario: Wrong attribute type in implementation
Mock Test: Would catch immediately (compile error)
Weaver: Catches at runtime (integration test time)
Result: SLOWER FEEDBACK LOOP
```

### Both Together = True Validation

```
Scenario: Feature implementation
Mock Test: ✅ Interface contract correct (fast feedback)
Weaver: ✅ Runtime behavior correct (proof)
Result: FEATURE VALIDATED ✅
```

## Test Coverage Requirements

### 100% Mock Coverage of Schema

Every schema definition must have corresponding mock tests:

- **Spans**: Test all required attributes
- **Metrics**: Test all recorded values
- **Events**: Test all event types
- **Enums**: Test all enum values
- **Hierarchies**: Test parent-child relationships

### Example Coverage Matrix

| Schema Element | Mock Test | Weaver Validation |
|----------------|-----------|-------------------|
| `container.id` required | ✅ Mock expects call | ✅ Weaver finds in actual span |
| `test.isolated` required | ✅ Mock expects true | ✅ Weaver validates boolean |
| `test.result` enum | ✅ Mock tests all values | ✅ Weaver validates against schema |
| Container lifecycle | ✅ Mock validates sequence | ✅ Weaver validates state transitions |

## Development Workflow

### Step 1: Schema Definition

```yaml
# registry/core/test_execution.yaml
groups:
  - id: span.clnrm.test_execution
    type: span
    attributes:
      - ref: container.id
        requirement_level: required
      - id: test.isolated
        type: boolean
        requirement_level: required
```

### Step 2: Generate Mocks

```bash
# Generator Coder creates mocks from schema
weaver registry generate rust \
  --registry registry/ \
  --template templates/mocks.j2 \
  --output src/telemetry/generated/mocks.rs
```

### Step 3: Write Failing Tests (Red)

```rust
#[tokio::test]
async fn test_execution_exports_required_telemetry() {
    let mut mock = MockTestExecutionSpan::new();
    mock.expect_set_container_id().times(1);

    let result = execute_test(mock).await;
    assert!(result.is_ok());
}

// Expected: FAIL (execute_test not implemented)
```

### Step 4: Implement to Satisfy Mocks (Green)

```rust
async fn execute_test(span: impl TestExecutionSpan) -> Result<()> {
    let container_id = start_container().await?;
    span.set_container_id(&container_id);
    // ... rest of implementation
    Ok(())
}

// Expected: PASS (mocks satisfied)
```

### Step 5: Weaver Validation (Proof)

```bash
# Run with actual OTEL export
clnrm run tests/ --otel-exporter otlp

# Validate
weaver registry live-check --registry registry/

# Expected: PASS (runtime behavior correct)
```

## When to Ship

A feature is **SAFE TO SHIP** when:

1. ✅ Mock tests pass (interface contract correct)
2. ✅ Weaver validation passes (runtime behavior correct)
3. ✅ Both validations in CI/CD pipeline
4. ✅ No warnings from Weaver about schema drift

A feature is **NOT SAFE TO SHIP** if:

- ❌ Only mock tests pass (no runtime proof)
- ❌ Only Weaver passes (no contract validation)
- ❌ Either validation fails
- ❌ Schema not defined

## Common Pitfalls

### Pitfall 1: Trusting Mock Tests Alone

```rust
// Mock test passes ✅
mock.expect_export_span().times(1);

// But actual implementation forgets to call:
fn execute_test() {
    // Oops, forgot to call export_span()
}

// Weaver catches this: ❌ No spans found
```

### Pitfall 2: Skipping Mock Tests

```rust
// Only Weaver validation, no mocks
// Developer changes interface, long feedback loop
// Discovers at runtime instead of compile time
```

### Pitfall 3: Mock-Implementation Drift

```rust
// Mock tests schema v1
mock.expect_set_isolated().with(eq(true));

// Implementation uses schema v2
span.set_isolation_status(IsolationStatus::Hermetic);

// Mock passes (wrong contract) ✅
// Weaver fails (actual != schema) ❌
```

**Solution**: Generate mocks from schema, keep in sync.

## Metrics and Monitoring

### Mock Test Metrics

- **Coverage**: % of schema elements with mock tests
- **Pass Rate**: % of mock tests passing
- **Drift Detection**: Schema changes without mock updates

### Weaver Validation Metrics

- **Conformance**: % of spans matching schema
- **Completeness**: % of required attributes present
- **Accuracy**: % of attribute types correct

### Combined Health

```
Feature Health Score = (Mock Coverage × Mock Pass Rate) × Weaver Conformance

Example:
  Mock Coverage: 100%
  Mock Pass Rate: 100%
  Weaver Conformance: 100%
  → Health Score: 100% ✅ SAFE TO SHIP

Example:
  Mock Coverage: 90%
  Mock Pass Rate: 100%
  Weaver Conformance: 100%
  → Health Score: 90% ⚠️ INCOMPLETE COVERAGE

Example:
  Mock Coverage: 100%
  Mock Pass Rate: 100%
  Weaver Conformance: 70%
  → Health Score: 70% ❌ RUNTIME ISSUES
```

## Integration with CI/CD

### GitHub Actions Workflow

```yaml
name: Telemetry Validation

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - name: Schema Validation
        run: weaver registry check --registry registry/

      - name: Generate Mocks
        run: weaver registry generate rust --template mocks.j2

      - name: Mock Tests (Red/Green)
        run: cargo test --lib

      - name: Integration Tests with OTEL
        run: |
          clnrm run tests/ --otel-exporter file
          weaver registry live-check --registry registry/

      - name: Validate Both Passed
        run: |
          if [ "$MOCK_TESTS" = "pass" ] && [ "$WEAVER_CHECK" = "pass" ]; then
            echo "✅ Feature validated"
          else
            echo "❌ Validation failed"
            exit 1
          fi
```

## Summary

**London TDD Mocks:**
- Prove interface contract correct
- Fast feedback (compile-time)
- Catch design issues early

**Weaver Live-Check:**
- Prove runtime behavior correct
- Actual telemetry validation
- Catch implementation issues

**Both Required:**
- Mocks alone → false positives (interface correct, behavior wrong)
- Weaver alone → slow feedback (runtime-only validation)
- Both together → true validation (interface + behavior correct)

**Golden Rule:**
```
✓ Mock tests pass + ✓ Weaver validation passes = SAFE TO SHIP
Any other combination = NOT SAFE TO SHIP
```

## References

- [Weaver Registry Documentation](https://github.com/open-telemetry/weaver)
- [London School TDD](https://github.com/testdouble/contributing-tests/wiki/London-School-TDD)
- [OTel Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [clnrm Integration Plan](/docs/WEAVER_INTEGRATION_PLAN.md)
