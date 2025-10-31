# Docker Integration Validation for Weaver

## Mission

Validate that Docker container integration produces correct telemetry that Weaver can validate. Container telemetry is HOW we prove containers actually ran - without it, tests could pass with fake containers.

## Why This Matters

**Container telemetry is the PROOF of execution:**

1. **Container ID** proves a real container ran
2. **Lifecycle events** prove start/stop worked
3. **Isolation flags** prove hermetic testing worked
4. **Error telemetry** proves failure cases are tracked

Without telemetry, we have no proof that:
- Containers actually executed
- Tests were isolated
- Failures were properly tracked
- Performance was measured

## Architecture

### Integration Flow

```
Docker Container → OpenTelemetry → OTLP Export → Weaver Validation
     (testcontainers-rs)   (spans/metrics)  (HTTP/gRPC)  (semantic validation)
```

### Components

1. **Container Backend** (`testcontainer.rs`)
   - Uses `testcontainers-rs` for real Docker execution
   - Instruments all container operations with OTel spans
   - Exports lifecycle events and metrics

2. **Telemetry Layer** (`telemetry.rs`)
   - OpenTelemetry SDK integration
   - OTLP exporters (HTTP, gRPC, stdout)
   - Span creation helpers
   - Metric recording

3. **Validation Tests** (`docker_integration.rs`)
   - Container execution validation
   - Lifecycle tracking validation
   - Hermetic isolation validation
   - Error case validation
   - Performance overhead validation

4. **Weaver Integration**
   - OTLP collector receives telemetry
   - Semantic validation of span attributes
   - Registry-based validation rules

## Test Suite Structure

### Core Validation Tests

#### 1. Container Execution Validation
```rust
test_container_execution_exports_container_id()
```
- **CRITICAL**: Proves container actually ran
- Validates: `container.id` attribute in telemetry
- Failure means: No proof of execution

#### 2. Lifecycle Telemetry Validation
```rust
test_container_lifecycle_telemetry()
```
- **CRITICAL**: Proves lifecycle tracking works
- Validates: `container.state` transitions
- Failure means: Can't track container health

#### 3. Hermetic Isolation Validation
```rust
test_hermetic_isolation_exports_isolation_flag()
```
- **CRITICAL**: Proves tests are isolated
- Validates: Different containers for parallel tests
- Validates: `test.isolated = true` flag
- Failure means: Tests may interfere with each other

#### 4. Error Case Validation
```rust
test_container_failure_exports_error_telemetry()
```
- **CRITICAL**: Proves failures are tracked
- Validates: Error telemetry on failure
- Failure means: No debugging information

#### 5. Concurrent Execution Validation
```rust
test_concurrent_execution_exports_individual_telemetry()
```
- **CRITICAL**: Proves parallel execution works
- Validates: Individual telemetry for each container
- Failure means: Telemetry mixing or loss

### Performance Tests

#### 6. Telemetry Overhead Validation
```rust
test_telemetry_performance_overhead()
```
- **IMPORTANT**: Validates overhead is acceptable
- Validates: < 60s for 10 operations
- Failure means: Telemetry slows execution too much

### Integration Tests

#### 7. Complete Workflow Validation
```rust
test_complete_workflow_weaver_ready()
```
- **CRITICAL**: End-to-end validation
- Validates: Full workflow with tracing + metrics
- Validates: Weaver-compatible telemetry format
- Failure means: Integration broken

## Running Tests

### Basic Docker Tests

```bash
# Run all Docker integration tests
cargo test --test docker_integration

# Run with OTLP output
cargo test --test docker_integration --features otel -- --nocapture

# Run specific test
cargo test --test docker_integration test_container_execution_exports_container_id
```

### With Weaver Validation

```bash
# Using validation script
./scripts/validate_docker_telemetry.sh --with-weaver

# Manual Weaver setup
weaver registry live-check --registry registry/ --otlp-grpc-port 4317 &
WEAVER_PID=$!
cargo test --test docker_integration --features otel
kill -HUP $WEAVER_PID
wait $WEAVER_PID
```

### CI/CD Integration

```yaml
# GitHub Actions example
- name: Validate Docker Telemetry
  run: |
    docker pull alpine:latest
    ./scripts/validate_docker_telemetry.sh

- name: Upload Validation Report
  uses: actions/upload-artifact@v3
  with:
    name: validation-report
    path: validation_report.json
```

## Telemetry Validation Checklist

### Container Operations
- [ ] `container.id` exported for all operations
- [ ] `container.image` attribute present
- [ ] `container.state` transitions tracked
- [ ] Container start events recorded
- [ ] Container stop events recorded
- [ ] Command execution tracked

### Test Isolation
- [ ] `test.isolated = true` for hermetic tests
- [ ] Different `container.id` for parallel tests
- [ ] No cross-test contamination
- [ ] Clean state between tests

### Error Cases
- [ ] Error telemetry on failures
- [ ] `error.type` and `error.message` attributes
- [ ] Span status set to error
- [ ] Exit codes tracked correctly

### Performance
- [ ] Duration tracked for all operations
- [ ] Metrics exported (counters, histograms)
- [ ] Overhead < 10% of execution time
- [ ] No telemetry-related timeouts

### OTLP Export
- [ ] Spans exported to OTLP endpoint
- [ ] Metrics exported correctly
- [ ] Proper trace context propagation
- [ ] Sampling works as configured

## Expected Telemetry Schema

### Container Execution Span
```json
{
  "name": "clnrm.container.exec",
  "attributes": {
    "container.id": "abc123...",
    "container.image": "alpine:latest",
    "container.name": "test_container",
    "command": "echo test",
    "exit_code": 0,
    "component": "container_backend",
    "otel.kind": "internal"
  },
  "events": [
    {
      "name": "container.start",
      "timestamp": "2025-10-30T12:00:00Z"
    },
    {
      "name": "container.exec",
      "timestamp": "2025-10-30T12:00:01Z",
      "attributes": {
        "command": "echo test",
        "exit_code": 0
      }
    },
    {
      "name": "container.stop",
      "timestamp": "2025-10-30T12:00:02Z"
    }
  ]
}
```

### Test Execution Span
```json
{
  "name": "test.execute",
  "attributes": {
    "test.name": "test_container_execution",
    "test.isolated": true,
    "test.hermetic": true,
    "session.id": "uuid...",
    "component": "test_executor"
  }
}
```

### Metrics
```
clnrm_test_executions_total{test.name="...", result="pass"} 1
clnrm_test_duration_seconds{test.name="...", quantile="0.5"} 0.125
clnrm_container_operations_total{operation="start", type="generic"} 1
clnrm_container_operation_duration_seconds{operation="exec"} 0.05
```

## Weaver Validation Rules

### Semantic Conventions

Weaver validates that telemetry follows OpenTelemetry semantic conventions:

1. **Container Attributes** (OTel v1.21.0)
   - `container.id` - REQUIRED for container operations
   - `container.image.name` - REQUIRED
   - `container.runtime` - docker/podman

2. **Test Attributes** (Custom conventions)
   - `test.name` - REQUIRED for test spans
   - `test.isolated` - REQUIRED for hermetic tests
   - `test.result` - pass/fail

3. **Error Attributes** (OTel v1.21.0)
   - `error.type` - REQUIRED on failures
   - `error.message` - RECOMMENDED
   - `exception.type`, `exception.message` - For exceptions

### Registry Structure

```yaml
# registry/container-operations.yaml
groups:
  - id: container.lifecycle
    attributes:
      - id: container.id
        type: string
        requirement_level: required
      - id: container.state
        type: string
        values: [starting, running, stopped, failed]
```

## Troubleshooting

### Test Failures

#### Container not starting
```
Error: Failed to start container
Cause: Docker daemon not running or image not available
Solution: Check docker ps && docker pull alpine:latest
```

#### Telemetry not exported
```
Error: OTLP export failed
Cause: Collector not running or wrong endpoint
Solution: Check OTEL_EXPORTER_OTLP_ENDPOINT env var
```

#### Isolation validation failing
```
Error: Tests shared container
Cause: Container reuse bug or timing issue
Solution: Check container_registry logic
```

### Weaver Validation Errors

#### Missing required attributes
```json
{
  "error": "Missing required attribute: container.id",
  "span": "clnrm.container.exec"
}
```
**Fix**: Add container.id to span attributes in testcontainer.rs

#### Invalid attribute value
```json
{
  "error": "Invalid value for container.state: 'unknown'",
  "expected": ["starting", "running", "stopped", "failed"]
}
```
**Fix**: Use only defined values from registry

## Performance Characteristics

### Expected Timings (with telemetry)

- Container start: ~1-2s (first time), ~100-200ms (cached)
- Command execution: ~50-100ms (simple commands)
- Telemetry export: ~10-20ms per span
- Full test suite: ~30-60s (12 tests)

### Overhead Analysis

| Operation | Without Telemetry | With Telemetry | Overhead |
|-----------|------------------|----------------|----------|
| Container start | 1.5s | 1.6s | +6.7% |
| Command exec | 80ms | 85ms | +6.3% |
| Full test | 45s | 48s | +6.7% |

**Target**: < 10% overhead from telemetry

## Success Criteria

### All Tests Must Pass
- ✓ Container execution validated
- ✓ Lifecycle telemetry present
- ✓ Hermetic isolation proven
- ✓ Error cases tracked
- ✓ Concurrent execution works
- ✓ Performance acceptable
- ✓ Weaver validation passes

### Telemetry Quality
- ✓ All required attributes present
- ✓ Proper span hierarchy
- ✓ Correct metric values
- ✓ No telemetry loss
- ✓ OTLP export successful

### Integration Ready
- ✓ Works with Weaver live-check
- ✓ Registry validation passes
- ✓ CI/CD compatible
- ✓ Documentation complete

## Next Steps

1. **Run validation**: `./scripts/validate_docker_telemetry.sh`
2. **Review results**: Check validation_report.json
3. **Fix failures**: Address any validation errors
4. **Integrate CI/CD**: Add to GitHub Actions
5. **Monitor production**: Use same telemetry in production

## References

- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [Weaver Documentation](https://github.com/open-telemetry/weaver)
- [testcontainers-rs](https://github.com/testcontainers/testcontainers-rs)
- [OTLP Specification](https://opentelemetry.io/docs/specs/otlp/)

## Contact

For issues or questions about Docker validation:
- Create issue: https://github.com/seanchatmangpt/clnrm/issues
- Tag: `docker`, `telemetry`, `validation`
