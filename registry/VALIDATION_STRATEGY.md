# CLNRM Schema Registry - Validation Strategy

## The Problem We're Solving

**clnrm exists to eliminate false positives in testing.**

But how do we validate clnrm itself without falling into the same trap?

### The False Positive Problem

Traditional test validation:
```rust
#[test]
fn test_container_creation() -> Result<()> {
    let env = CleanroomEnvironment::new().await?;
    env.create_container("alpine:latest").await?;
    Ok(())  // Test passes... but did container actually run?
}
```

This test can pass with stub implementations:
- `create_container` could just return `Ok(())`
- No container actually created
- No isolation actually happened
- Test result: **FALSE POSITIVE**

## The Solution: Weaver Live-Check

Instead of trusting test method results, we validate **actual runtime telemetry** against schemas.

### How It Works

1. **Instrumentation**: Every critical operation emits telemetry
2. **Schema Definition**: This registry defines what MUST be emitted
3. **Live Checking**: Weaver validates telemetry matches schemas in real-time
4. **Proof**: Presence of required attributes proves behavior happened

### Example: Proving Container Creation

**Schema Requirements** (from `core/test_execution.yaml`):
```yaml
- id: container.id
  requirement_level: required
  note: CANNOT exist without a real container

- id: test.isolated
  requirement_level: required
  note: Must be true, proves hermetic isolation

- id: test.result
  requirement_level: required
  note: Must be set, proves execution completed
```

**Runtime Validation**:
```bash
weaver live-check \
  --schema registry/core/test_execution.yaml \
  --endpoint http://localhost:4318/v1/traces

# Checks every span.clnrm.test_execution span has:
# ✓ container.id exists (proves container created)
# ✓ test.isolated = true (proves isolation)
# ✓ test.result in [pass, fail, error] (proves completion)
# ✓ test.duration_ms > 0 (proves actual execution time)
```

If ANY required attribute is missing or invalid, validation FAILS.

**This cannot be faked.** A stub implementation cannot produce a real container.id.

## Schema Registry Architecture

### Core Schemas (registry/core/)

**1. test_execution.yaml**
- **PURPOSE**: Prove tests actually run in containers with isolation
- **KEY ATTRIBUTES**:
  - `container.id` - Cannot exist without real container
  - `test.isolated` - Must be true
  - `test.result` - Must be pass/fail/error
  - `test.duration_ms` - Must be > 0
- **VALIDATES**: Core testing promise works

**2. container_lifecycle.yaml**
- **PURPOSE**: Prove containers are created and cleaned up
- **KEY ATTRIBUTES**:
  - `container.created_at` - Proves creation
  - `container.destroyed_at` - Proves cleanup
  - `cleanup.success` - Must be true
- **VALIDATES**: No resource leaks
- **DETECTS**: Leaked containers show as missing destroyed_at

**3. plugin_system.yaml**
- **PURPOSE**: Prove plugin architecture works
- **KEY ATTRIBUTES**:
  - `plugin.state` transitions - Prove lifecycle
  - `plugin.health_check.performed` - Prove health checking
  - `service.name` + `container.id` - Link plugins to containers
- **VALIDATES**: Plugin system operational

### Metrics (registry/metrics/)

**Aggregate Behavior Validation**:
- `clnrm.test.duration` - Distribution proves consistent performance
- `clnrm.container.count` - created MUST equal destroyed (no leaks)
- `clnrm.isolation.score` - Must be 1.0 (perfect isolation)

### Events (registry/events/)

**Lifecycle Transition Validation**:
- Every `test.started` must have matching `test.completed` or `test.failed`
- `container.leaked` events should NEVER occur
- `isolation.violation` events should NEVER occur

## Critical Attributes

These attributes PROVE core functionality:

### 1. container.id
- **PRESENCE PROVES**: Container actually created
- **CANNOT FAKE**: Requires real container backend
- **VALIDATES**: Backend integration works

### 2. test.isolated
- **VALUE MUST BE**: true
- **PROVES**: Hermetic isolation worked
- **DETECTS**: Shared state violations

### 3. container.destroyed_at
- **PRESENCE PROVES**: Cleanup happened
- **ABSENCE DETECTS**: Resource leaks
- **VALIDATES**: Container lifecycle management

### 4. test.duration_ms
- **MUST BE**: > 0
- **PROVES**: Actual execution occurred
- **DETECTS**: Stub implementations (return 0 or don't track time)

### 5. plugin.state transitions
- **SEQUENCE PROVES**: Plugin lifecycle works
- **DETECTS**: Stuck states, missing transitions
- **VALIDATES**: Plugin system operational

## Validation Workflows

### 1. Unit Test Validation (Fast)

```bash
# Run tests with OTEL enabled
OTEL_EXPORTER=stdout cargo test --features otel 2> telemetry.json

# Validate telemetry matches schemas
weaver registry check -r registry/
weaver validate --schema registry/ --input telemetry.json

# Check for required attributes
jq '.spans[] | select(.name == "clnrm.test_execution") | .attributes.container.id' telemetry.json
```

**PASS CRITERIA**:
- All required attributes present
- No schema violations
- Every span has container.id

### 2. Integration Test Validation (Comprehensive)

```bash
# Run integration tests with live OTEL collector
docker run -d -p 4318:4318 otel/opentelemetry-collector

# Run tests pointing to collector
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 cargo test --features otel

# Live check during execution
weaver live-check --schema registry/ --endpoint http://localhost:4318/v1/traces
```

**PASS CRITERIA**:
- All spans conform to schemas
- No missing required attributes
- Resource counts balanced (created == destroyed)

### 3. Production Validation (Continuous)

```bash
# clnrm self-test with validation
clnrm self-test --suite full --validate-telemetry

# Internally runs:
# 1. Execute tests with OTEL enabled
# 2. Export telemetry
# 3. Validate against schemas
# 4. Check critical attributes
# 5. Fail if any violations
```

## Detecting False Positives

### Scenario 1: Stub Implementation

**Code**:
```rust
async fn create_container(&self, image: &str) -> Result<ContainerId> {
    println!("Creating container with {}", image);
    Ok(ContainerId::new("fake-id"))  // STUB
}
```

**Test Result**: PASS (test method succeeds)

**Weaver Validation**: FAIL
```
ERROR: span.clnrm.container_lifecycle missing required attribute: container.created_at
ERROR: span.clnrm.container_lifecycle missing required attribute: container.destroyed_at
ERROR: No container lifecycle spans emitted

VERDICT: Implementation is stubbed, not real.
```

### Scenario 2: Resource Leak

**Code**:
```rust
async fn cleanup(&self) -> Result<()> {
    // Forgot to actually destroy container
    Ok(())
}
```

**Test Result**: PASS (cleanup method succeeds)

**Weaver Validation**: FAIL
```
ERROR: Metric clnrm.container.count shows:
  - created: 10
  - destroyed: 7
  - leak_count: 3

ERROR: 3 containers missing container.destroyed_at timestamp

VERDICT: Resource leak detected.
```

### Scenario 3: Isolation Failure

**Code**:
```rust
// Accidentally reusing containers between tests
static SHARED_CONTAINER: OnceCell<Container> = OnceCell::new();
```

**Test Result**: PASS (tests run successfully)

**Weaver Validation**: FAIL
```
ERROR: Multiple test_execution spans share same container.id
ERROR: Isolation score: 0.45 (expected 1.0)
ERROR: event.clnrm.isolation.violation emitted

VERDICT: Hermetic isolation violated.
```

## Schema Evolution

### Adding New Validation

When adding new features, update schemas:

1. **Identify provable behavior**
   - What attribute can ONLY exist if feature works?

2. **Add to appropriate schema**
   - Core behavior → registry/core/
   - Metrics → registry/metrics/
   - Events → registry/events/

3. **Mark as required**
   - If critical: `requirement_level: required`
   - If optional: `requirement_level: recommended`

4. **Document validation strategy**
   - What does presence prove?
   - What does absence detect?
   - How can it be faked?

### Schema Versioning

```yaml
# registry_manifest.yaml
semconv_version: 1.0.0  # Schema version
schema_base_url: https://github.com/seanchatmangpt/clnrm/schemas/v1.0.0/
```

Breaking changes require new major version.

## Integration with CI/CD

### GitHub Actions Workflow

```yaml
name: Validate with Weaver

jobs:
  test-and-validate:
    steps:
      - name: Run tests with OTEL
        run: cargo test --features otel
        env:
          OTEL_EXPORTER: file
          OTEL_OUTPUT: telemetry.json

      - name: Validate schemas
        run: weaver registry check -r registry/

      - name: Validate telemetry
        run: weaver validate --schema registry/ --input telemetry.json

      - name: Check critical attributes
        run: |
          # Ensure every test has container.id
          jq -e '.spans[] | select(.name == "clnrm.test_execution") | .attributes.container.id' telemetry.json

          # Ensure no leaks
          created=$(jq '[.metrics[] | select(.name == "clnrm.container.count" and .attributes.container.state == "created") | .value] | add' telemetry.json)
          destroyed=$(jq '[.metrics[] | select(.name == "clnrm.container.count" and .attributes.container.state == "destroyed") | .value] | add' telemetry.json)
          [ "$created" -eq "$destroyed" ] || exit 1

      - name: Upload telemetry
        uses: actions/upload-artifact@v3
        with:
          name: telemetry
          path: telemetry.json
```

## Success Criteria

Schema registry is successful when:

1. **Zero False Positives**
   - Stub implementations fail validation
   - Incomplete features fail validation
   - Tests cannot pass without real behavior

2. **Comprehensive Coverage**
   - All core behaviors have schemas
   - All critical attributes are required
   - All failure modes detected

3. **Continuous Validation**
   - CI/CD validates every commit
   - Production validates every execution
   - Regressions caught immediately

4. **Developer Experience**
   - Clear error messages when validation fails
   - Easy to understand what's missing
   - Simple to add new validations

## Next Steps

1. **Implement Instrumentation** (Instrumentation Engineer)
   - Add telemetry emission to match schemas
   - Ensure all required attributes populated

2. **Integrate Weaver** (DevOps Agent)
   - Setup validation in CI/CD
   - Configure live checking

3. **Create Test Suite** (Test Engineer)
   - Tests that validate telemetry
   - Negative tests (ensure stubs fail validation)

4. **Documentation** (Documentation Writer)
   - Guide for adding new schemas
   - Examples of validation workflows

## Conclusion

This schema registry transforms clnrm validation from "trust test methods" to "prove runtime behavior."

**Before**: Tests pass, but did containers actually run? Who knows.

**After**: Tests pass AND telemetry proves containers ran, isolation worked, cleanup happened.

**This is how we eat our own dogfood without false positives.**
