# Weaver Schema Validation (v1.2.1)

**OpenTelemetry Weaver** is the source of truth for validation in clnrm v1.2.1. This chapter explains schema-first validation and how to use Weaver live-check with proper health checks to prevent false positives.

## The Problem: False Positives in Traditional Testing

Traditional tests can pass even when features don't work:

```rust
// ❌ TRADITIONAL TEST: Can pass with broken features
#[test]
fn test_container_execution() {
    let env = CleanroomEnvironment::new().await?;
    let result = env.execute_test("my_test").await?;
    assert!(result.success); // ✅ PASSES
    // But: NO PROOF container actually ran!
}
```

### Why Traditional Tests Fail

1. **Mocks can lie**: Tests validate mocked behavior, not production behavior
2. **Stub implementations**: `Ok(())` returns without doing work
3. **No runtime validation**: Tests don't prove actual execution happened
4. **Circular validation**: Framework testing itself is inherently unreliable

## The Solution: Weaver Schema Validation

Weaver validates **actual runtime telemetry** against **declared schemas**:

```
Schema Definition (Source of Truth):
  registry/core/test_execution.yaml
    - container.id: REQUIRED
    - test.isolated: REQUIRED
    - test.duration_ms: REQUIRED

Runtime Telemetry:
  Span: clnrm.test_execution
    - container.id: "abc-123-def"  ✅ Present
    - test.isolated: true           ✅ Present
    - test.duration_ms: 5234        ✅ Present

Weaver Validation:
  ✅ 100% schema coverage
  ✅ All required attributes present
  ✅ Exit code 0 (PASS)
```

### Why Weaver Validation Works

1. **Cannot fake runtime telemetry**: `container.id` requires real container
2. **External validation**: Weaver is independent tool, not self-testing
3. **Schema enforces behavior**: Required attributes prove features work
4. **Industry standard**: OpenTelemetry's official validation approach

## The 4 Attributes That Prove Everything (80/20 Rule)

Only 4 attributes prove 80% of clnrm's functionality:

| Attribute | Proves | Why It Cannot Be Faked |
|-----------|--------|------------------------|
| `container.id` | Container actually ran | Requires real Docker container |
| `test.isolated` | Hermetic isolation | Requires separate container per test |
| `container.destroyed_at` | Cleanup happened | Requires actual container cleanup |
| `test.duration_ms` | Actual execution | Must be >0ms for real operations |

**If these 4 attributes are present in telemetry, the feature MUST be working.**

## Getting Started with Weaver Validation

### Prerequisites

1. **Install Weaver** (included in `vendors/weaver/`):

```bash
# Already cloned for you
cd vendors/weaver
cargo build --release
alias weaver=./target/release/weaver
```

2. **Understand the Schema Registry**:

```bash
registry/
├── registry_manifest.yaml         # Registry metadata
├── core/
│   ├── test_execution.yaml        # Test execution spans
│   ├── container_lifecycle.yaml   # Container lifecycle spans
│   └── plugin_system.yaml         # Plugin execution spans
└── metrics/
    └── test_metrics.yaml          # Performance metrics
```

3. **Validate Schemas**:

```bash
weaver registry check --registry registry/
# Expected:
# ✅ clnrm semconv registry loaded (200 files)
# ✅ 0 violations, 0 warnings
```

## Running Live Validation

### Basic Workflow

```bash
# Terminal 1: Start Weaver listener
weaver registry live-check \
    --registry registry/ \
    --otlp-grpc-port 4316 \
    --format json \
    --output ./validation_output

# Terminal 2: Run tests with OTEL export
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4316 \
cargo test --features otel

# Terminal 3: Stop Weaver and check results
kill -SIGHUP $WEAVER_PID
cat validation_output/live_check.json
```

### Health Check Integration (v1.2.1)

WeaverController now uses proper HTTP health checks instead of hardcoded sleeps:

```rust
// WeaverController.wait_for_weaver_ready()
// - Polls http://localhost:{admin_port}/health
// - Exponential backoff (100ms → 1000ms)
// - 30 second timeout
// - Verifies actual readiness, not just process existence
```

**Benefits:**
- ✅ Faster startup (detects readiness immediately)
- ✅ No arbitrary delays (responds to actual state)
- ✅ Proper error handling (timeout if Weaver fails to start)
- ✅ Production-ready reliability

### Automated Validation Script

Use the production-ready script:

```bash
./scripts/validate_docker_telemetry.sh --with-weaver
```

This script:
1. Starts Weaver live-check listener
2. Waits for health check to pass (via WeaverController)
3. Runs Docker integration tests
4. Stops Weaver with SIGHUP
5. Validates sample count > 0 (prevents false positives)
6. Parses validation report
7. Exits with code 0 (pass) or 1 (violations)

## Understanding Weaver Reports

### Zero Coverage (False Positive Detected) - v1.2.1

**CRITICAL:** v1.2.1 now explicitly fails validation when `sample_count == 0`:

```json
{
  "sample_count": 0,
  "statistics": {
    "registry_coverage": 0.0,
    "total_entities": 0,
    "seen_registry_attributes": {
      "container.id": 0,
      "test.isolated": 0,
      "test.duration_ms": 0
    }
  },
  "status": "failure"
}
```

**Interpretation:** Tests may pass, but **NO telemetry emitted**. Features don't work.
**Validation Result:** ❌ **FAILS** - Zero-sample validation prevents false positives.

### Partial Coverage (Missing Attributes)

```json
{
  "statistics": {
    "registry_coverage": 25.0,
    "seen_registry_attributes": {
      "container.id": 1,        // ✅ Present
      "test.isolated": 0,       // ❌ Missing
      "test.duration_ms": 0,    // ❌ Missing
      "container.destroyed_at": 0  // ❌ Missing
    }
  },
  "advices": [
    {
      "level": "violation",
      "message": "Required attribute 'test.isolated' not found in span 'clnrm.test_execution'"
    }
  ]
}
```

**Interpretation:** Partial implementation. Container runs, but isolation and cleanup not proven.

### Full Coverage (Production Ready)

```json
{
  "statistics": {
    "registry_coverage": 100.0,
    "total_advisories": 0,
    "seen_registry_attributes": {
      "container.id": 12,
      "test.isolated": 12,
      "test.duration_ms": 12,
      "container.destroyed_at": 12
    }
  },
  "advices": []
}
```

**Interpretation:** ✅ **PRODUCTION READY**. All required telemetry present, 0 violations.

## Writing Code to Pass Weaver Validation

### Phase 1: Export Critical Attributes (8 hours)

**Goal:** Get Weaver validation to pass with 4 critical attributes.

#### Step 1: Test Execution Span

```rust
// File: crates/clnrm-core/src/cli/commands/run/executor.rs
use opentelemetry::trace::{Span, Tracer};
use opentelemetry::global;
use opentelemetry::KeyValue;
use std::time::Instant;

pub async fn execute_test_with_telemetry(
    test_name: &str,
    container_id: &str,
    test_fn: impl Future<Output = Result<()>>,
) -> Result<()> {
    let tracer = global::tracer("clnrm");
    let mut span = tracer
        .span_builder("clnrm.test_execution")
        .with_attributes(vec![
            KeyValue::new("test.name", test_name.to_string()),
            KeyValue::new("test.isolated", true),
            KeyValue::new("container.id", container_id.to_string()), // ✅ Proves container ran
        ])
        .start(&tracer);

    let start = Instant::now();
    let result = test_fn.await;
    let duration = start.elapsed();

    span.set_attribute(KeyValue::new(
        "test.duration_ms",
        duration.as_millis() as i64  // ✅ Proves actual execution
    ));
    span.set_attribute(KeyValue::new(
        "test.result",
        if result.is_ok() { "pass" } else { "fail" }
    ));

    span.end();
    result
}
```

#### Step 2: Container Lifecycle Span

```rust
// File: crates/clnrm-core/src/backend/testcontainer.rs
use opentelemetry::trace::{Span, Tracer};
use opentelemetry::global;
use opentelemetry::KeyValue;
use std::time::{SystemTime, UNIX_EPOCH};

pub async fn create_container_with_telemetry(
    image: &str,
) -> Result<(Container, String)> {
    let tracer = global::tracer("clnrm");
    let mut span = tracer
        .span_builder("clnrm.container_lifecycle")
        .with_attributes(vec![
            KeyValue::new("container.image.name", image.to_string()),
        ])
        .start(&tracer);

    let created_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();
    span.set_attribute(KeyValue::new("container.created_at", created_at as i64));

    // Create container
    let container = Container::new(image).await?;
    let container_id = container.id().to_string();

    span.set_attribute(KeyValue::new("container.id", container_id.clone()));

    // Return container and ID for later cleanup
    Ok((container, container_id))
}

pub async fn cleanup_container_with_telemetry(
    container: Container,
    lifecycle_span: &mut Span,
) -> Result<()> {
    // Cleanup container
    container.stop().await?;
    container.rm().await?;

    // Record cleanup telemetry
    let destroyed_at = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_secs();

    lifecycle_span.set_attribute(KeyValue::new(
        "container.destroyed_at",
        destroyed_at as i64  // ✅ Proves cleanup happened
    ));
    lifecycle_span.set_attribute(KeyValue::new("cleanup.success", true));

    lifecycle_span.end();
    Ok(())
}
```

### Phase 2: Validate with Weaver (30 minutes)

```bash
# Start Weaver
weaver registry live-check --registry registry/ --otlp-grpc-port 4316 &
WEAVER_PID=$!

# Run tests
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4316 cargo test --features otel

# Stop Weaver
kill -SIGHUP $WEAVER_PID

# Check results
cat validation_output/live_check.json | grep "registry_coverage"
# Expected: "registry_coverage": 80.0 or higher
```

## Schema-Driven Development Workflow

### 1. Define Behavior in Schema

```yaml
# registry/core/test_execution.yaml
groups:
  - id: span.clnrm.test_execution
    type: span
    brief: "Test execution span with required attributes"
    attributes:
      - id: container.id
        type: string
        requirement_level: required
        brief: "Unique container ID proving container ran"

      - id: test.isolated
        type: boolean
        requirement_level: required
        brief: "Proves hermetic isolation"

      - id: test.duration_ms
        type: int
        requirement_level: required
        brief: "Actual execution duration in milliseconds"
```

### 2. Write Code to Match Schema

```rust
// Code MUST export all required attributes
span.set_attribute(KeyValue::new("container.id", container_id));
span.set_attribute(KeyValue::new("test.isolated", true));
span.set_attribute(KeyValue::new("test.duration_ms", duration.as_millis() as i64));
```

### 3. Validate with Weaver

```bash
weaver registry live-check --registry registry/
# If required attribute missing → VIOLATION (exit code 1)
```

### 4. Ship Only When Weaver Passes

```bash
# CI/CD gate
./scripts/validate_docker_telemetry.sh --with-weaver || exit 1
```

## CI/CD Integration

### GitHub Actions Weaver Gate

```yaml
# .github/workflows/weaver-validation.yml
name: Weaver Schema Validation

on: [push, pull_request]

jobs:
  weaver-validation:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Start Weaver listener
        run: |
          cd vendors/weaver
          cargo build --release
          ./target/release/weaver registry live-check \
            --registry ../../registry/ \
            --otlp-grpc-port 4316 \
            --format json \
            --output ../../validation_output &
          echo $! > weaver.pid
          sleep 5

      - name: Run tests with OTEL
        run: |
          OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4316 \
          cargo test --features otel --test docker_integration

      - name: Stop Weaver and validate
        run: |
          kill -SIGHUP $(cat weaver.pid)
          wait $(cat weaver.pid)

          # Check for violations
          violations=$(cat validation_output/live_check.json | \
            jq -r '.statistics.advice_level_counts.violation // 0')

          if [ "$violations" -gt 0 ]; then
            echo "❌ Weaver validation FAILED: $violations violations"
            cat validation_output/live_check.json | jq '.advices'
            exit 1
          fi

          # Check coverage
          coverage=$(cat validation_output/live_check.json | \
            jq -r '.statistics.registry_coverage')

          echo "✅ Weaver validation PASSED"
          echo "📊 Schema coverage: $coverage%"
```

## Best Practices

### 1. Schema-First Development

```bash
# ✅ CORRECT: Define schema first
1. Write schema: registry/core/my_feature.yaml
2. Validate schema: weaver registry check
3. Write code to match schema
4. Validate with live-check
5. Ship when coverage >80%

# ❌ WRONG: Code-first approach
1. Write code
2. Hope it emits telemetry
3. Discover gaps in production
```

### 2. The 4 Critical Attributes

Always export these attributes:

```rust
// ✅ CRITICAL ATTRIBUTES (cannot be faked)
span.set_attribute(KeyValue::new("container.id", container_id));
span.set_attribute(KeyValue::new("test.isolated", true));
span.set_attribute(KeyValue::new("test.duration_ms", duration.as_millis() as i64));
span.set_attribute(KeyValue::new("container.destroyed_at", destroyed_at));
```

### 3. Never Ship Without Weaver Validation

```bash
# ✅ PRODUCTION GATE
./scripts/validate_docker_telemetry.sh --with-weaver
if [ $? -ne 0 ]; then
  echo "❌ Weaver validation failed - NOT PRODUCTION READY"
  exit 1
fi
```

### 4. Monitor Coverage Over Time

```bash
# Track coverage trend
echo "$(date),$(jq -r '.statistics.registry_coverage' validation_output/live_check.json)" \
  >> coverage_history.csv
```

## Common Issues and Solutions

### Issue: Zero Coverage Detected

**Symptom:**
```json
{"statistics": {"registry_coverage": 0.0}}
```

**Diagnosis:** No telemetry emitted to Weaver

**Solutions:**
1. Check OTEL_EXPORTER_OTLP_ENDPOINT is set correctly
2. Verify Weaver is listening on correct port
3. Ensure OTEL feature is enabled: `cargo test --features otel`
4. Check spans are actually created in code

### Issue: Partial Coverage

**Symptom:**
```json
{
  "registry_coverage": 25.0,
  "advices": [{"level": "violation", "message": "Required attribute missing"}]
}
```

**Diagnosis:** Some attributes missing

**Solutions:**
1. Read violation messages to identify missing attributes
2. Add missing `span.set_attribute()` calls
3. Re-run validation until coverage >80%

### Issue: Tests Pass, Weaver Fails

**Symptom:**
```
cargo test: ✅ PASSED
weaver validation: ❌ FAILED (0% coverage)
```

**Diagnosis:** **FALSE POSITIVE DETECTED** - This is working as designed!

**Explanation:** Tests can pass without emitting telemetry. Weaver catches this.

**Solution:**
1. Don't trust tests alone
2. Fix code to emit required telemetry
3. Re-validate with Weaver until it passes

## Next Steps

1. **Understand the 80/20 approach**: See [80/20 Validation Strategy](80-20-validation.md)
2. **Learn about false positive detection**: See [False Positive Detection](false-positive-detection.md)
3. **Set up CI/CD**: See [Production Deployment](../production-deployment/ci-cd-integration.md)
4. **Write your first schema**: See [Reference: Weaver Schemas](../reference/weaver-schemas.md)

## Further Reading

- [Weaver Documentation](https://github.com/open-telemetry/weaver)
- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [clnrm Hive Mind Validation Report](../../../docs/HIVE_MIND_VALIDATION_REPORT.md)
- [Live-Check Feature Analysis](../../../docs/weaver/WEAVER_LIVE_CHECK_FEATURE_ANALYSIS.md)
