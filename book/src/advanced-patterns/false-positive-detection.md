# False Positive Detection

**False positives** are tests that pass even when features don't work. clnrm v1.2.1 solves this problem using Weaver schema validation with zero-sample detection. This chapter explains false positive detection and prevention.

## The False Positive Problem

### What is a False Positive?

A **false positive** occurs when:
1. Test executes and reports success ✅
2. Feature doesn't actually work ❌
3. No one notices until production 💥

### Real-World Example

```rust
// ❌ FALSE POSITIVE: Test passes, feature broken
#[test]
async fn test_container_execution() {
    let env = CleanroomEnvironment::new().await?;
    let result = env.execute_test("my_test").await?;

    assert!(result.success); // ✅ PASSES
    // But: Container never actually ran!
    // But: No cleanup happened!
    // But: No telemetry emitted!
}
```

**What actually happened:**
```rust
// Broken implementation
pub async fn execute_test(&self, name: &str) -> Result<TestResult> {
    // Stub implementation
    Ok(TestResult { success: true }) // Always returns success!
}
```

**Result:**
- Test: ✅ PASSED
- Feature: ❌ BROKEN
- Production: 💥 FAILED

## Why False Positives Happen

### Reason 1: Mocked Behavior

```rust
// ❌ Test validates mocked behavior, not production behavior
#[test]
async fn test_with_mock() {
    let mock_container = MockContainer::new();
    mock_container.expect_run()
        .returning(|| Ok(())); // Mock always succeeds

    let result = execute_in_container(&mock_container).await?;
    assert!(result.is_ok()); // ✅ PASSES

    // But: Real container might fail!
    // But: No proof real container works!
}
```

### Reason 2: Stub Implementations

```rust
// ❌ Stub returns success without doing work
pub async fn execute_command(&self, cmd: &[&str]) -> Result<Output> {
    println!("Executing: {:?}", cmd); // Just logs
    Ok(Output::default()) // Returns fake success
}
```

### Reason 3: Circular Validation

```rust
// ❌ Framework testing itself (circular logic)
#[test]
async fn test_framework() {
    let env = CleanroomEnvironment::new().await?;
    let result = env.self_test().await?;
    assert!(result.passed()); // Framework validates itself (?!)
}
```

### Reason 4: Missing Assertions

```rust
// ❌ Test doesn't validate actual behavior
#[test]
async fn test_container_cleanup() {
    let container = create_container().await?;
    cleanup_container(container).await?;
    // Missing: Assert container was actually removed!
}
```

## The Hive Mind Validation: Real False Positive Detected

**From the Hive Mind validation (2025-10-30):**

```
Test Result:     ✅ PASSED (cargo test docker_integration)
Weaver Result:   ❌ 0.0% COVERAGE (NO telemetry emitted)

Conclusion: Tests pass, but features don't emit telemetry.
```

**Weaver report:**
```json
{
  "samples": [],
  "statistics": {
    "registry_coverage": 0.0,
    "total_entities": 0,
    "seen_registry_attributes": {
      "container.id": 0,
      "test.isolated": 0,
      "test.duration_ms": 0,
      "container.destroyed_at": 0
    }
  }
}
```

**This is a FALSE POSITIVE:**
- Tests claim feature works (test passed)
- Weaver proves feature doesn't work (0% telemetry)
- Without Weaver, we would ship broken code

## How Weaver Detects False Positives

### Detection Mechanism

```
Schema Definition (Source of Truth):
  registry/core/test_execution.yaml
    - container.id: REQUIRED
    - test.isolated: REQUIRED
    - test.duration_ms: REQUIRED

Test Claims:
  ✅ Test passed
  ✅ Feature works
  ✅ Container executed

Runtime Telemetry:
  ❌ container.id: NOT EMITTED
  ❌ test.isolated: NOT EMITTED
  ❌ test.duration_ms: NOT EMITTED

Weaver Validation:
  ❌ VIOLATION: Required attributes missing
  ❌ registry_coverage: 0.0%
  ❌ Exit code: 1 (FAIL BUILD)

Result: FALSE POSITIVE DETECTED
```

### Why Weaver Works

1. **External Validation**: Weaver is independent tool, not self-testing
2. **Cannot Fake Telemetry**: `container.id` requires real container
3. **Schema Enforces Behavior**: Required attributes prove features work
4. **Runtime Validation**: Validates actual execution, not test code

## The 4 Attributes That Cannot Be Faked

These attributes **prove** features work and **cannot be faked**:

### 1. `container.id` - Proves Container Ran

```rust
// ❌ Cannot fake without real container
span.set_attribute(KeyValue::new("container.id", "fake-id"));
// Weaver will detect: Invalid UUID format, no lifecycle events

// ✅ Must come from real container
let container = create_container().await?;
span.set_attribute(KeyValue::new("container.id", container.id()));
// Weaver validates: Valid UUID, correlates with lifecycle
```

### 2. `test.isolated` - Proves Hermetic Isolation

```rust
// ❌ Cannot claim isolation without proof
span.set_attribute(KeyValue::new("test.isolated", true));
// Weaver will detect: Same container.id across tests

// ✅ Must use unique container per test
let container1 = create_container().await?;
let container2 = create_container().await?;
assert_ne!(container1.id(), container2.id());
span.set_attribute(KeyValue::new("test.isolated", true));
// Weaver validates: Unique container.id per test
```

### 3. `container.destroyed_at` - Proves Cleanup

```rust
// ❌ Cannot claim cleanup without proof
span.set_attribute(KeyValue::new("cleanup.success", true));
// Weaver will detect: Missing destroyed_at timestamp

// ✅ Must record actual cleanup
let destroyed_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();
container.stop().await?;
container.rm().await?;
span.set_attribute(KeyValue::new("container.destroyed_at", destroyed_at as i64));
// Weaver validates: Timestamp present, after created_at
```

### 4. `test.duration_ms` - Proves Actual Execution

```rust
// ❌ Stub returns immediately (0ms)
fn execute_test() -> Result<()> {
    Ok(()) // duration = 0ms
}
// Weaver will detect: duration = 0 indicates stub

// ✅ Must measure actual work
let start = Instant::now();
container.exec(&["echo", "hello"]).await?;
let duration = start.elapsed();
span.set_attribute(KeyValue::new("test.duration_ms", duration.as_millis() as i64));
// Weaver validates: duration >0, reasonable for operation
```

## False Positive Prevention Workflow

### Step 1: Define Expected Behavior in Schema

```yaml
# registry/core/test_execution.yaml
groups:
  - id: span.clnrm.test_execution
    type: span
    brief: "Test execution span"
    attributes:
      - id: container.id
        type: string
        requirement_level: required
        brief: "Container ID proving execution"

      - id: test.duration_ms
        type: int
        requirement_level: required
        brief: "Execution duration >0ms"
```

### Step 2: Write Code to Match Schema

```rust
pub async fn execute_test(&self, name: &str) -> Result<TestResult> {
    let tracer = global::tracer("clnrm");
    let mut span = tracer
        .span_builder("clnrm.test_execution")
        .start(&tracer);

    let start = Instant::now();

    // ACTUAL WORK (not stub)
    let container = self.create_container("alpine").await?;
    let output = container.exec(&["echo", "test"]).await?;

    // REQUIRED TELEMETRY
    span.set_attribute(KeyValue::new("container.id", container.id()));
    span.set_attribute(KeyValue::new(
        "test.duration_ms",
        start.elapsed().as_millis() as i64
    ));

    span.end();
    Ok(TestResult { success: output.status.success() })
}
```

### Step 3: Validate with Weaver

```bash
# Start Weaver
weaver registry live-check --registry registry/ --otlp-grpc-port 4316 &

# Run tests
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4316 cargo test

# Check results
kill -SIGHUP $WEAVER_PID
cat validation_output/live_check.json
```

### Step 4: Interpret Results

**FALSE POSITIVE (Feature Broken):**
```json
{
  "registry_coverage": 0.0,
  "seen_registry_attributes": {
    "container.id": 0,  // ❌ Missing
    "test.duration_ms": 0  // ❌ Missing
  }
}
```

**TRUE POSITIVE (Feature Works):**
```json
{
  "registry_coverage": 100.0,
  "seen_registry_attributes": {
    "container.id": 12,  // ✅ Present
    "test.duration_ms": 12  // ✅ Present
  }
}
```

## CI/CD Gate to Prevent False Positives

### GitHub Actions Example

```yaml
# .github/workflows/false-positive-gate.yml
name: False Positive Detection

on: [push, pull_request]

jobs:
  detect-false-positives:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run traditional tests
        id: tests
        run: cargo test --features otel
        continue-on-error: true

      - name: Run Weaver validation
        id: weaver
        run: ./scripts/validate_docker_telemetry.sh --with-weaver
        continue-on-error: true

      - name: Detect false positives
        run: |
          if [ "${{ steps.tests.outcome }}" == "success" ] && \
             [ "${{ steps.weaver.outcome }}" == "failure" ]; then
            echo "🚨 FALSE POSITIVE DETECTED!"
            echo "Tests passed but Weaver validation failed."
            echo "This means features claim to work but don't emit telemetry."
            exit 1
          fi

          if [ "${{ steps.weaver.outcome }}" == "failure" ]; then
            echo "❌ Weaver validation failed - NOT PRODUCTION READY"
            exit 1
          fi

          echo "✅ No false positives detected"
```

## Common False Positive Patterns

### Pattern 1: Optimistic Return Values

```rust
// ❌ FALSE POSITIVE: Always returns success
pub fn validate_config(&self) -> Result<()> {
    Ok(()) // Fake success - no actual validation
}

// ✅ CORRECT: Honest about incomplete implementation
pub fn validate_config(&self) -> Result<()> {
    unimplemented!("validate_config: needs schema parsing, attribute validation, and error reporting")
}
```

**Fix:**
```rust
// ✅ Implement actual validation
pub fn validate_config(&self) -> Result<()> {
    if self.required_field.is_none() {
        return Err(CleanroomError::config_error("Missing required field"));
    }
    Ok(())
}
```

### Pattern 2: Ignored Errors

```rust
// ❌ FALSE POSITIVE: Ignores errors
pub async fn cleanup(&self) -> Result<()> {
    let _ = self.container.stop().await; // Ignores error
    Ok(()) // Claims success
}
```

**Fix:**
```rust
// ✅ Propagate errors
pub async fn cleanup(&self) -> Result<()> {
    self.container.stop().await?; // Propagates error
    Ok(())
}
```

### Pattern 3: Test-Only Paths

```rust
// ❌ FALSE POSITIVE: Different behavior in tests
#[cfg(test)]
pub async fn execute(&self) -> Result<()> {
    Ok(()) // Test version: stub
}

#[cfg(not(test))]
pub async fn execute(&self) -> Result<()> {
    self.container.exec(&["cmd"]).await // Production version: real
}
```

**Fix:**
```rust
// ✅ Same code path for tests and production
pub async fn execute(&self) -> Result<()> {
    self.container.exec(&["cmd"]).await
}
```

### Pattern 4: Missing Cleanup Verification

```rust
// ❌ FALSE POSITIVE: Doesn't verify cleanup
#[test]
async fn test_cleanup() {
    let container = create_container().await?;
    cleanup(container).await?;
    // Missing: Verify container actually removed
}
```

**Fix:**
```rust
// ✅ Verify cleanup with Weaver
#[test]
async fn test_cleanup() {
    let container = create_container().await?;
    let container_id = container.id().to_string();

    cleanup(container).await?;

    // Weaver validates container.destroyed_at is present
}
```

## Best Practices

### 1. Never Trust Tests Alone

```bash
# ❌ WRONG: Ship based on test results only
cargo test && git push

# ✅ CORRECT: Require Weaver validation
cargo test && ./scripts/validate_docker_telemetry.sh --with-weaver && git push
```

### 2. Use Schema as Contract

```yaml
# Schema = Contract between code and runtime
groups:
  - id: span.clnrm.test_execution
    attributes:
      - id: container.id
        requirement_level: required  # MUST be present
```

### 3. Fail Builds on Weaver Violations

```yaml
# CI/CD: Fail if Weaver detects violations
- run: weaver registry live-check
  if: failure()
    exit 1
```

### 4. Monitor False Positive Rate

```bash
# Track false positives over time
echo "$(date),$(cat validation_output/live_check.json | \
  jq -r '.statistics.advice_level_counts.violation // 0')" \
  >> false_positive_tracking.csv
```

## Next Steps

1. **Understand 80/20 validation**: See [80/20 Validation Strategy](80-20-validation.md)
2. **Learn Weaver validation**: See [Weaver Schema Validation](weaver-validation.md)
3. **Set up CI/CD gates**: See [Production Deployment](../production-deployment/ci-cd-integration.md)
4. **Review Hive Mind findings**: See [Hive Mind Validation Report](../../../docs/HIVE_MIND_VALIDATION_REPORT.md)

## Further Reading

- [False Positive in Software Testing](https://en.wikipedia.org/wiki/False_positives_and_false_negatives)
- [Weaver Documentation](https://github.com/open-telemetry/weaver)
- [clnrm Hive Mind Report](../../../docs/HIVE_MIND_VALIDATION_REPORT.md)
- [Code Analyzer OTEL Analysis](../../../docs/CODE_ANALYZER_OTEL_EMISSION_ANALYSIS.md)
