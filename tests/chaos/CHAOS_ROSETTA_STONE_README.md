# Chaos Engineering Test Suite - Rosetta Stone Extension

## Overview

This chaos engineering test suite extends the Rosetta Stone validation framework with comprehensive failure scenario testing. It validates that the clnrm framework maintains **hermetic isolation**, **deterministic behavior**, and **proper cleanup** even under extreme failure conditions.

## Philosophy: OTEL-First Chaos Validation

Unlike traditional chaos tests that rely on exit codes or logs, these tests use **OpenTelemetry spans as the source of truth**:

- **No False Positives**: Fake green detection via span validation
- **Hermetic Verification**: Cleanup spans prove resource leakage detection
- **Deterministic Chaos**: Seeded random failures for reproducibility
- **Self-Validating**: Tests prove themselves using OTEL trace graphs

## Test Suites

### 1. Container Failures (`container_failures.clnrm.toml`)

**Purpose**: Validate resilience when containers fail unexpectedly

**Scenarios**:
- Container killed mid-step (SIGKILL)
- OOM kills with memory limits
- Network disconnection during execution
- Non-zero exit codes
- Concurrent container failures

**Critical Validation**:
```toml
[[expect.span]]
name = "clnrm.cleanup"
attrs.all = { "cleanup.success" = "true" }
status = "OK"
# MUST exist even when container is killed!
```

**Key Insight**: The cleanup span MUST be emitted even on catastrophic failure, proving no container leakage.

### 2. Network Partitions (`network_partitions.clnrm.toml`)

**Purpose**: Simulate network failures and validate service resilience

**Scenarios**:
- Network latency injection (500ms delay)
- Packet loss simulation (30% loss rate)
- Complete network partition (iptables DROP)
- Split-brain scenarios (node isolation)
- Connection timeouts (various durations)
- Connection refused handling

**Critical Validation**:
```toml
[[expect.span]]
name = "clnrm.network.partition"
attrs.all = { "partition.detected" = "true" }

[[expect.span]]
name = "clnrm.network.recovery"
parent = "clnrm.network.partition"
attrs.all = { "recovery.success" = "true" }
```

**Key Insight**: Network chaos requires `capabilities = ["NET_ADMIN"]` for traffic control.

### 3. Resource Exhaustion (`resource_exhaustion.clnrm.toml`)

**Purpose**: Validate graceful degradation under resource pressure

**Scenarios**:
- Memory exhaustion (hitting memory limits)
- CPU throttling (10% CPU allocation)
- Disk space exhaustion (tmpfs limits)
- File descriptor exhaustion (ulimit enforcement)
- Concurrent resource pressure (all at once)
- Resource recovery validation

**Critical Validation**:
```toml
[[expect.span]]
name = "clnrm.resource.memory_pressure"
attrs.all = { "pressure.level" = "high" }

[[expect.span]]
name = "clnrm.step:memory_exhaustion"
attrs.all = { "result" = "pass", "degraded" = "true" }
# System continues working even under pressure!
```

**Key Insight**: Graceful degradation means tests pass with `degraded = true`, not catastrophic failure.

### 4. Timeout Scenarios (`timeout_scenarios.clnrm.toml`)

**Purpose**: Validate deadline enforcement and timeout handling

**Scenarios**:
- Step timeout enforcement (hard kill after deadline)
- Graceful vs hard timeout (SIGTERM → SIGKILL)
- Cascading timeouts (multi-step timeout chain)
- Deadline-based scheduling (absolute deadlines)
- Timeout with resource cleanup
- Exponential backoff retries
- Timeout race conditions

**Critical Validation**:
```toml
[[expect.span]]
name = "clnrm.timeout.enforced"
attrs.all = { "timeout.ms" = "1000" }

[[expect.span]]
name = "clnrm.cleanup"
parent = "clnrm.timeout.enforced"
attrs.all = { "cleanup.triggered" = "timeout" }
# Cleanup MUST happen after timeout!
```

**Key Insight**: Timeout enforcement must include cleanup verification to prevent zombie processes.

### 5. Concurrent Chaos (`concurrent_chaos.clnrm.toml`)

**Purpose**: Validate resilience under simultaneous multiple failures

**Scenarios**:
- Concurrent container failures (3+ simultaneous)
- Network + resource chaos combined
- Cascading failure propagation (domino effect)
- Chaos Monkey (40% random failure rate)
- Maximum concurrent stress (5+ containers)
- Recovery under concurrent failures

**Critical Validation**:
```toml
[[expect.span]]
name = "clnrm.cleanup.concurrent"
attrs.all = {
    "containers.cleaned" = "3",
    "cleanup.parallel" = "true"
}
# Parallel cleanup for concurrent failures!
```

**Key Insight**: Concurrent failures test the framework's ability to handle multiple simultaneous chaos events without deadlocks.

## Validation Strategy

### OTEL-First Validation

Every chaos test validates behavior through OpenTelemetry spans:

1. **Span Existence**: Cleanup spans MUST exist for all failures
2. **Span Attributes**: Capture failure type, recovery strategy
3. **Span Relationships**: Parent-child graphs prove execution flow
4. **Span Timing**: Duration constraints validate timeout enforcement
5. **Span Status**: `ERROR` for failures, `OK` for cleanup/recovery

### Hermetic Isolation

All tests verify hermetic isolation:

```toml
[expect.hermeticity]
no_container_leakage = true
no_network_leakage = true
no_volume_leakage = true
no_process_leakage = true
```

**Validation**: Cleanup verification spans prove all resources freed.

### Deterministic Chaos

All chaos tests use deterministic seeds:

```toml
[determinism]
seed = 666  # Chaos seed for reproducibility
freeze_clock = "2025-01-01T00:00:00Z"
```

**Validation**: Same chaos sequence every run, enabling reproducible failures.

## Running Chaos Tests

### Prerequisites

1. **Homebrew Installation**:
   ```bash
   brew install clnrm
   ```

2. **Docker/Podman**: Required for container chaos
3. **NET_ADMIN Capability**: For network chaos (privileged containers)

### Run Individual Suites

```bash
# Container failure chaos
clnrm run tests/chaos/container_failures.clnrm.toml

# Network partition chaos
clnrm run tests/chaos/network_partitions.clnrm.toml

# Resource exhaustion chaos
clnrm run tests/chaos/resource_exhaustion.clnrm.toml

# Timeout scenarios
clnrm run tests/chaos/timeout_scenarios.clnrm.toml

# Concurrent multi-failure chaos
clnrm run tests/chaos/concurrent_chaos.clnrm.toml
```

### Run Complete Chaos Suite

```bash
# Run all chaos tests
clnrm run tests/chaos/*.clnrm.toml

# With OTEL validation
clnrm run tests/chaos/*.clnrm.toml --otel-exporter stdout
```

### Integration Testing

```bash
# Run Rust integration tests
cargo test --test chaos_integration

# Run with OTEL features
cargo test --test chaos_integration --features otel
```

## Success Criteria

### Per-Test Criteria

- ✅ **Cleanup Spans**: ALL failures produce cleanup spans
- ✅ **No Leakage**: Zero container/network/volume leakage
- ✅ **Graceful Degradation**: System continues under pressure
- ✅ **Recovery Validated**: Recovery spans confirm restoration
- ✅ **Determinism**: Identical chaos sequence every run

### Suite-Level Criteria

- ✅ **All Tests Pass**: Complete chaos suite passes
- ✅ **Zero False Greens**: Span validation prevents fake passes
- ✅ **Hermetic Verified**: Isolation maintained under chaos
- ✅ **Performance**: Cleanup completes within 30s
- ✅ **Documentation**: All scenarios documented

## Architecture

### Chaos Injection Points

1. **Container Level**: Kill, OOM, exit codes
2. **Network Level**: Latency, partition, packet loss
3. **Resource Level**: Memory, CPU, disk, FD limits
4. **Time Level**: Timeouts, deadlines, backoff
5. **Concurrency Level**: Multiple simultaneous failures

### Validation Layers

```
┌─────────────────────────────────────┐
│     OTEL Span Validation Layer      │  ← Source of truth
├─────────────────────────────────────┤
│    Cleanup Verification Layer       │  ← Resource leak detection
├─────────────────────────────────────┤
│    Hermetic Isolation Layer         │  ← State isolation
├─────────────────────────────────────┤
│   Deterministic Chaos Layer         │  ← Reproducible failures
├─────────────────────────────────────┤
│      Container Execution Layer      │  ← Actual failures
└─────────────────────────────────────┘
```

## Comparison with Existing Chaos Tests

### Existing Rust Chaos Tests

Located in `/tests/chaos/*.rs`:
- Unit tests validating chaos engine implementation
- Resilience benchmarks measuring performance
- Recovery validation testing recovery strategies

### New TOML Chaos Tests

Located in `/tests/chaos/*.clnrm.toml`:
- **Integration tests** using production clnrm binary
- **OTEL-first validation** via span analysis
- **Hermetic verification** with cleanup detection
- **Dogfooding**: Tests validate the testing framework itself

### Complementary Design

```
Rust Tests           →  Validate chaos engine internals
TOML Tests           →  Validate end-to-end resilience
Together             →  Complete chaos validation coverage
```

## Best Practices

### 1. Always Use `expect_failure = true`

```toml
[[scenario]]
name = "container_killed_mid_step"
expect_failure = true  # Chaos WILL cause failures
```

### 2. Validate Cleanup Spans

```toml
[[expect.span]]
name = "clnrm.cleanup"
attrs.all = { "cleanup.success" = "true" }
# NEVER skip cleanup validation!
```

### 3. Use Deterministic Seeds

```toml
[determinism]
seed = 666  # Reproducible chaos
```

### 4. Set Resource Limits

```toml
[limits]
cpu_millicores = 1000
memory_mb = 512
max_duration_seconds = 60
```

### 5. Verify Cleanup Always

```toml
[cleanup]
containers = "always"
verify_cleanup = true  # CRITICAL
```

## Troubleshooting

### Container Cleanup Failures

**Symptom**: Tests report container leakage

**Solution**:
```bash
# Check for orphaned containers
docker ps -a | grep clnrm

# Force cleanup
docker rm -f $(docker ps -aq -f "label=clnrm")
```

### Network Chaos Requires Privileges

**Symptom**: `tc` or `iptables` commands fail

**Solution**: Add `capabilities = ["NET_ADMIN"]` to service config

### Resource Limits Not Enforced

**Symptom**: Memory/CPU limits ignored

**Solution**: Ensure Docker daemon has resource limits enabled

### Timeout Tests Hang

**Symptom**: Timeout tests exceed max duration

**Solution**: Check `timeout_ms` and `max_duration_seconds` settings

## Metrics and Reporting

Each chaos test generates:

- **JSON Report**: Detailed test results
- **JUnit XML**: CI/CD integration
- **SHA256 Digest**: Trace fingerprint for determinism
- **Cleanup Verification**: Resource cleanup report

Example:
```bash
clnrm run tests/chaos/container_failures.clnrm.toml
# Generates:
# - chaos_container_failures.report.json
# - chaos_container_failures.junit.xml
# - chaos_container_failures.trace.sha256
# - chaos_container_failures.cleanup.json
```

## Contributing

When adding new chaos scenarios:

1. **Follow Naming Convention**: `<failure_type>_<scenario>.clnrm.toml`
2. **Add OTEL Validation**: Always validate via spans
3. **Verify Cleanup**: Include cleanup span validation
4. **Document Thoroughly**: Update this README
5. **Test with Homebrew**: Validate via production installation

## References

- [Rosetta Stone Test Suite](../v1.0.1_release/rosetta_stone_suite.clnrm.toml)
- [OTEL Validation Integration](../../crates/clnrm-core/tests/otel_validation_integration.rs)
- [Chaos Engineering Principles](https://principlesofchaos.org/)
- [clnrm Architecture](../../docs/ARCHITECTURE.md)

## License

Same as clnrm project license.

---

**Remember**: Chaos tests don't just validate failure handling - they prove the framework's core guarantees (hermetic isolation, determinism, cleanup) hold even under extreme conditions.
