# gVisor Docker Elimination - Validation Checklist

> Comprehensive validation for complete Docker daemon elimination from clnrm

**Status**: In Progress
**Version**: 2.0.0
**Last Updated**: 2026-01-05

## Executive Summary

This document provides a complete validation framework for eliminating Docker dependencies from the clnrm project and replacing them with gVisor-based containerization. The validation ensures zero Docker daemon references, complete testcontainers replacement, and full functional parity.

## Validation Categories

### 1. Docker Daemon References (CRITICAL)

**Objective**: Verify NO Docker daemon dependencies exist in the codebase

#### 1.1 Docker CLI Usage
- [ ] No `docker` CLI calls in source code
- [ ] No `docker` CLI calls in shell scripts
- [ ] No `docker-compose` CLI calls
- [ ] No Docker API client libraries imported
- [ ] No Docker SDK usage

**Validation Command**:
```bash
# Search for docker CLI usage
grep -rn "docker\s" --include="*.rs" --include="*.sh" --include="*.toml" .

# Search for docker API clients
grep -rn "bollard\|shiplift\|dkregistry" --include="*.rs" --include="*.toml" .
```

**Expected Result**: Zero matches (excluding comments and documentation)

#### 1.2 Docker Socket Access
- [ ] No `/var/run/docker.sock` references
- [ ] No `DOCKER_HOST` environment variable usage
- [ ] No Docker daemon TCP connections
- [ ] No Docker context switching

**Validation Command**:
```bash
# Search for docker socket
grep -rn "/var/run/docker.sock\|DOCKER_HOST" --include="*.rs" --include="*.sh" .
```

**Expected Result**: Zero matches

#### 1.3 Docker Daemon Checks
- [ ] No `docker info` checks
- [ ] No `docker version` checks
- [ ] No Docker daemon availability validation
- [ ] No Docker daemon state queries

**Validation Command**:
```bash
# Search for docker daemon checks
grep -rn "docker\s\+info\|docker\s\+version\|docker\s\+ps" --include="*.rs" --include="*.sh" .
```

**Expected Result**: Zero matches

### 2. Testcontainers References (CRITICAL)

**Objective**: Verify ALL testcontainers usage is replaced with gVisor backend

#### 2.1 Testcontainers Dependencies
- [ ] No `testcontainers` crate in Cargo.toml
- [ ] No `testcontainers-modules` crate in Cargo.toml
- [ ] No testcontainers imports in source code
- [ ] No GenericImage usage
- [ ] No Container type usage from testcontainers

**Validation Command**:
```bash
# Check Cargo.toml files
grep -rn "testcontainers" --include="Cargo.toml" .

# Check source code
grep -rn "use testcontainers\|testcontainers::" --include="*.rs" .
```

**Expected Result**: Zero matches in Cargo.toml, zero imports in source

#### 2.2 Testcontainers API Usage
- [ ] No `GenericImage::new()` calls
- [ ] No `Container::start()` calls
- [ ] No `Container::exec()` calls
- [ ] No `SyncRunner` or `AsyncRunner` usage
- [ ] No testcontainers module usage (surrealdb, postgres, etc.)

**Validation Command**:
```bash
# Search for testcontainers API
grep -rn "GenericImage\|SyncRunner\|AsyncRunner" --include="*.rs" .
```

**Expected Result**: Zero matches

#### 2.3 Backend Implementation
- [ ] `TestcontainerBackend` removed or deprecated
- [ ] `AutoBackend` no longer uses testcontainers
- [ ] Backend trait fully implemented by gVisor
- [ ] No fallback to testcontainers in any code path

**Validation Command**:
```bash
# Check backend implementations
grep -rn "TestcontainerBackend\|testcontainer" crates/clnrm-core/src/backend/
```

**Expected Result**: Only in deprecated/removed files with clear warnings

### 3. OCI Image Loading (CRITICAL)

**Objective**: Verify gVisor can load and run OCI container images without Docker

#### 3.1 Image Sources
- [ ] Can load from Docker Hub (docker.io)
- [ ] Can load from GitHub Container Registry (ghcr.io)
- [ ] Can load from local OCI archive (tar)
- [ ] Can load from local OCI layout directory
- [ ] Can specify image by digest

**Test Cases**:
```rust
// Test 1: Load from Docker Hub
gvisor_backend.load_image("alpine:latest")?;

// Test 2: Load from GHCR
gvisor_backend.load_image("ghcr.io/my-org/my-image:v1")?;

// Test 3: Load from local tar
gvisor_backend.load_image_from_tar("/path/to/image.tar")?;

// Test 4: Load by digest
gvisor_backend.load_image("alpine@sha256:abc123...")?;
```

**Success Criteria**: All image sources work without Docker daemon

#### 3.2 Image Caching
- [ ] Images are cached locally (not re-pulled every run)
- [ ] Cache invalidation works correctly
- [ ] Cache location is configurable
- [ ] Cache cleanup/garbage collection exists

**Validation Command**:
```bash
# Run test twice, verify second run is faster
time cargo test gvisor_image_load_test
time cargo test gvisor_image_load_test  # Should be faster
```

**Expected Result**: Second run 10x faster (cached)

#### 3.3 Image Metadata
- [ ] Can inspect image layers
- [ ] Can read image manifest
- [ ] Can extract environment variables from image
- [ ] Can read image entrypoint/cmd

**Test**: Verify image inspection API works

### 4. Network Isolation (CRITICAL)

**Objective**: Verify gVisor provides network isolation equivalent to Docker

#### 4.1 Network Namespace
- [ ] Each container gets isolated network namespace
- [ ] Containers cannot access host network by default
- [ ] Network isolation configurable per test
- [ ] IPv4 and IPv6 support

**Test Cases**:
```rust
// Test: Container should not see host network
let result = gvisor_backend.run_cmd("ip addr show");
assert!(!result.stdout.contains("docker0"));
assert!(!result.stdout.contains("eth0")); // host interface
```

#### 4.2 Port Mapping
- [ ] Can map container ports to host ports
- [ ] Port conflicts detected and prevented
- [ ] Dynamic port allocation works
- [ ] Port mapping persists for container lifetime

**Test Cases**:
```rust
// Test: Port mapping
let backend = gvisor_backend
    .with_port_mapping(8080, 80)?;
let result = backend.run_cmd("nc -l 80")?;
// Verify host can connect to localhost:8080
```

#### 4.3 DNS Resolution
- [ ] Container has working DNS resolution
- [ ] Can configure custom DNS servers
- [ ] DNS search domains configurable
- [ ] No DNS leakage to host

**Test Cases**:
```rust
// Test: DNS works
let result = gvisor_backend.run_cmd("nslookup google.com");
assert!(result.exit_code == 0);
```

#### 4.4 Network Performance
- [ ] Network latency < 1ms (local)
- [ ] Throughput > 1 Gbps (local)
- [ ] TCP connection establishment < 100ms
- [ ] No packet loss in normal conditions

**Benchmark**: Run iperf3 between containers

### 5. Filesystem Isolation (CRITICAL)

**Objective**: Verify gVisor provides filesystem isolation equivalent to Docker

#### 5.1 Root Filesystem
- [ ] Container has isolated root filesystem
- [ ] Cannot access host filesystem by default
- [ ] Root filesystem is from OCI image
- [ ] Filesystem changes don't persist after container stop

**Test Cases**:
```rust
// Test: Isolation
let result = gvisor_backend.run_cmd("ls /host");
assert!(result.exit_code != 0); // Should fail - /host doesn't exist
```

#### 5.2 Volume Mounts
- [ ] Can mount host directories into container
- [ ] Read-only mounts enforced
- [ ] Read-write mounts work correctly
- [ ] Volume paths validated for security

**Test Cases**:
```rust
// Test: Volume mount
let backend = gvisor_backend
    .with_volume("/tmp/host", "/container", true)?; // read-only
let result = backend.run_cmd("touch /container/test.txt");
assert!(result.exit_code != 0); // Should fail - read-only
```

#### 5.3 Filesystem Performance
- [ ] File I/O latency < 10ms
- [ ] Sequential read throughput > 500 MB/s
- [ ] Sequential write throughput > 300 MB/s
- [ ] Small file operations < 5ms

**Benchmark**: Run fio benchmark suite

#### 5.4 Temp Directories
- [ ] Each container gets isolated /tmp
- [ ] /tmp cleaned up after container stop
- [ ] /tmp size limits enforced
- [ ] No /tmp pollution to host

### 6. Service Management (CRITICAL)

**Objective**: Verify gVisor can manage long-running services (SurrealDB, OTEL Collector)

#### 6.1 Service Lifecycle
- [ ] Can start service in background
- [ ] Can stop service gracefully
- [ ] Can restart service
- [ ] Can check service health/status
- [ ] Service cleanup on test failure

**Test Cases**:
```rust
// Test: Service lifecycle
let service = gvisor_backend.start_service("surrealdb")?;
assert!(service.is_running());
service.stop()?;
assert!(!service.is_running());
```

#### 6.2 Service Communication
- [ ] Tests can connect to service ports
- [ ] Service-to-service communication works
- [ ] Service discovery mechanism exists
- [ ] Connection pooling works

**Test Cases**:
```rust
// Test: Connect to SurrealDB
let db_service = gvisor_backend.start_service("surrealdb")?;
let client = surrealdb::Surreal::new(&db_service.endpoint()).await?;
assert!(client.health().await.is_ok());
```

#### 6.3 Service Configuration
- [ ] Can pass environment variables to service
- [ ] Can mount config files into service
- [ ] Can override service entrypoint
- [ ] Can set resource limits for service

**Test**: Verify config options work

#### 6.4 Service Monitoring
- [ ] Can retrieve service logs
- [ ] Can monitor service resource usage
- [ ] Can detect service crashes
- [ ] Can trigger alerts on service failure

**Test**: Verify monitoring API works

### 7. OTLP Telemetry (HIGH PRIORITY)

**Objective**: Verify OpenTelemetry telemetry export works with gVisor

#### 7.1 Trace Export
- [ ] Spans exported to OTLP collector
- [ ] Trace context propagated correctly
- [ ] Span attributes populated correctly
- [ ] No trace data loss

**Test Cases**:
```rust
// Test: Trace export
let collector = gvisor_backend.start_service("otel-collector")?;
let result = gvisor_backend.run_cmd("echo test"); // Traced operation
// Verify span appears in collector
assert!(collector.has_span("clnrm.container.exec"));
```

#### 7.2 Metrics Export
- [ ] Metrics exported to OTLP collector
- [ ] Counter metrics work
- [ ] Histogram metrics work
- [ ] Gauge metrics work
- [ ] Metric labels/attributes correct

**Test**: Verify metrics appear in collector

#### 7.3 Log Export
- [ ] Logs exported to OTLP collector
- [ ] Log levels preserved
- [ ] Log attributes populated
- [ ] Structured logging works

**Test**: Verify logs appear in collector

#### 7.4 Collector Integration
- [ ] OTEL Collector runs in gVisor
- [ ] Collector receives telemetry
- [ ] Collector exports to backends (Jaeger, Prometheus)
- [ ] No telemetry loss during container transitions

**Validation**: Run full telemetry pipeline

### 8. Performance Metrics (HIGH PRIORITY)

**Objective**: Verify gVisor performance meets or exceeds testcontainers baseline

#### 8.1 Container Startup Time
- [ ] Cold start (no cache): < 5 seconds
- [ ] Warm start (cached): < 500ms
- [ ] Startup time variance < 10%
- [ ] Parallel container creation scales linearly

**Baseline (testcontainers)**:
- Cold start: ~3-5s
- Warm start: ~1-2s

**Target (gVisor)**:
- Cold start: < 3s (40% improvement)
- Warm start: < 500ms (75% improvement)

**Benchmark**:
```bash
# Run benchmark suite
cargo bench --bench container_startup_benchmark
```

#### 8.2 Memory Usage
- [ ] Container memory overhead < 100MB
- [ ] Memory usage stable over time (no leaks)
- [ ] Memory limits enforced correctly
- [ ] Memory usage scales linearly with containers

**Baseline (testcontainers)**:
- Overhead: ~150-200MB per container

**Target (gVisor)**:
- Overhead: < 100MB per container (50% improvement)

**Benchmark**:
```bash
# Measure memory usage
./scripts/validate_memory_usage.sh
```

#### 8.3 Network Performance
- [ ] Latency < 1ms (localhost)
- [ ] Throughput > 1 Gbps
- [ ] No performance degradation over time
- [ ] Performance comparable to native Docker

**Baseline (testcontainers/Docker)**:
- Latency: ~0.5-1ms
- Throughput: ~2-5 Gbps

**Target (gVisor)**:
- Latency: < 2ms (acceptable for testing)
- Throughput: > 1 Gbps

**Benchmark**:
```bash
# Network performance
cargo bench --bench network_performance_benchmark
```

#### 8.4 Disk I/O Performance
- [ ] Sequential read: > 500 MB/s
- [ ] Sequential write: > 300 MB/s
- [ ] Random read IOPS: > 10k
- [ ] Random write IOPS: > 5k

**Benchmark**: Use fio for disk I/O testing

### 9. Test Suite Validation (CRITICAL)

**Objective**: Verify 100% of existing tests pass with gVisor backend

#### 9.1 Unit Tests
- [ ] All unit tests pass with gVisor
- [ ] No Docker-specific test failures
- [ ] Test execution time acceptable (< 2x slowdown)
- [ ] No flaky tests introduced

**Validation Command**:
```bash
# Run full unit test suite
CLNRM_BACKEND=gvisor cargo test --all --lib
```

**Expected Result**: 100% pass rate

#### 9.2 Integration Tests
- [ ] All integration tests pass with gVisor
- [ ] SurrealDB integration works
- [ ] OTEL collector integration works
- [ ] Service discovery works
- [ ] Multi-container scenarios work

**Validation Command**:
```bash
# Run integration tests
CLNRM_BACKEND=gvisor cargo test --all --test '*'
```

**Expected Result**: 100% pass rate

#### 9.3 End-to-End Tests
- [ ] All E2E tests pass with gVisor
- [ ] Full workflow tests pass
- [ ] No regressions in functionality
- [ ] Performance acceptable

**Validation Command**:
```bash
# Run E2E tests
CLNRM_BACKEND=gvisor ./scripts/run_e2e_tests.sh
```

**Expected Result**: 100% pass rate

#### 9.4 Performance Tests
- [ ] All performance benchmarks pass
- [ ] Performance meets baseline requirements
- [ ] No performance regressions
- [ ] Scalability tests pass

**Validation Command**:
```bash
# Run performance tests
CLNRM_BACKEND=gvisor cargo bench --all
```

**Expected Result**: Performance within 20% of baseline

### 10. Configuration & Compatibility (MEDIUM PRIORITY)

**Objective**: Verify gVisor backend is properly configurable and compatible

#### 10.1 Configuration
- [ ] Backend selectable via environment variable
- [ ] Backend selectable via config file
- [ ] Backend-specific options configurable
- [ ] Default backend is gVisor
- [ ] Configuration validation works

**Test**:
```bash
# Test environment variable
CLNRM_BACKEND=gvisor cargo test

# Test config file
cat > .clnrm.toml <<EOF
[backend]
type = "gvisor"
image_cache_dir = "/tmp/clnrm-cache"
EOF
cargo test
```

#### 10.2 Platform Support
- [ ] Works on Linux x86_64
- [ ] Works on macOS (if applicable)
- [ ] Works in CI/CD (GitHub Actions)
- [ ] Works in Docker (for nested scenarios)

**Validation**: Run tests on all platforms

#### 10.3 Error Handling
- [ ] Clear error messages when gVisor unavailable
- [ ] Graceful degradation if applicable
- [ ] Error messages include remediation steps
- [ ] No panics or unwrap() failures

**Test**: Simulate various failure conditions

#### 10.4 Logging & Debugging
- [ ] gVisor backend logs are clear
- [ ] Debug mode provides detailed information
- [ ] Logs include container IDs
- [ ] Performance metrics logged

**Validation**: Review logs for clarity

## Automated Validation Scripts

### Script 1: Docker Reference Checker

**Location**: `/scripts/validate_docker_elimination.sh`

```bash
#!/bin/bash
# Validates zero Docker references in codebase
# Exit code: 0 = success, 1 = Docker references found

set -e

ERRORS=0

echo "Checking for Docker CLI usage..."
if grep -rn "docker\s" --include="*.rs" --include="*.sh" . | grep -v "^#" | grep -v "//"; then
    echo "❌ Found Docker CLI usage"
    ERRORS=$((ERRORS + 1))
fi

echo "Checking for Docker socket..."
if grep -rn "/var/run/docker.sock" --include="*.rs" --include="*.sh" .; then
    echo "❌ Found Docker socket references"
    ERRORS=$((ERRORS + 1))
fi

echo "Checking for testcontainers..."
if grep -rn "testcontainers" --include="Cargo.toml" .; then
    echo "❌ Found testcontainers dependencies"
    ERRORS=$((ERRORS + 1))
fi

if [ $ERRORS -eq 0 ]; then
    echo "✅ No Docker references found"
    exit 0
else
    echo "❌ Found $ERRORS categories of Docker references"
    exit 1
fi
```

### Script 2: Test Suite Validator

**Location**: `/scripts/validate_gvisor_tests.sh`

```bash
#!/bin/bash
# Validates all tests pass with gVisor backend
# Exit code: 0 = success, 1 = test failures

set -e

export CLNRM_BACKEND=gvisor

echo "Running unit tests..."
cargo test --all --lib

echo "Running integration tests..."
cargo test --all --test '*'

echo "Running benchmarks..."
cargo bench --no-run

echo "✅ All tests passed with gVisor backend"
```

### Script 3: Performance Baseline Checker

**Location**: `/scripts/validate_gvisor_performance.sh`

```bash
#!/bin/bash
# Validates gVisor performance meets baseline requirements
# Exit code: 0 = success, 1 = performance regression

set -e

export CLNRM_BACKEND=gvisor

echo "Running container startup benchmark..."
STARTUP_TIME=$(cargo bench --bench container_startup_benchmark | grep "time:" | awk '{print $2}')

if [ $(echo "$STARTUP_TIME > 5000" | bc) -eq 1 ]; then
    echo "❌ Startup time ${STARTUP_TIME}ms exceeds 5000ms threshold"
    exit 1
fi

echo "Running memory usage benchmark..."
MEMORY_MB=$(cargo bench --bench memory_usage_benchmark | grep "memory:" | awk '{print $2}')

if [ $(echo "$MEMORY_MB > 100" | bc) -eq 1 ]; then
    echo "❌ Memory usage ${MEMORY_MB}MB exceeds 100MB threshold"
    exit 1
fi

echo "✅ Performance meets baseline requirements"
```

## Success Criteria

### Phase 1: Foundation (Week 1-2)
- [ ] gVisor backend trait implementation complete
- [ ] Basic OCI image loading works
- [ ] Single container execution works
- [ ] Network isolation functional

### Phase 2: Feature Parity (Week 3-4)
- [ ] All Docker features replicated in gVisor
- [ ] Volume mounts working
- [ ] Port mapping working
- [ ] Service management working
- [ ] 50% of tests passing

### Phase 3: Validation (Week 5-6)
- [ ] 100% test pass rate
- [ ] Performance meets baseline
- [ ] OTLP telemetry working
- [ ] Documentation complete
- [ ] Zero Docker references

### Phase 4: Production (Week 7-8)
- [ ] CI/CD integration complete
- [ ] Migration guide published
- [ ] Troubleshooting guide complete
- [ ] User feedback incorporated
- [ ] Release candidate ready

## Key Metrics

| Metric | Baseline (Docker) | Target (gVisor) | Status |
|--------|------------------|-----------------|--------|
| Container startup (cold) | 3-5s | < 3s | ⏳ |
| Container startup (warm) | 1-2s | < 500ms | ⏳ |
| Memory overhead | 150-200MB | < 100MB | ⏳ |
| Network latency | 0.5-1ms | < 2ms | ⏳ |
| Test pass rate | 100% | 100% | ⏳ |
| Docker references | N/A | 0 | ⏳ |
| OTLP export success | 100% | 100% | ⏳ |

## Risk Mitigation

### Risk 1: gVisor Performance
- **Mitigation**: Benchmark early, optimize hot paths
- **Fallback**: Hybrid approach (gVisor + fallback)

### Risk 2: OCI Image Compatibility
- **Mitigation**: Test with wide variety of images
- **Fallback**: Maintain compatibility layer

### Risk 3: Platform Support
- **Mitigation**: Test on all target platforms early
- **Fallback**: Platform-specific backends

### Risk 4: Migration Complexity
- **Mitigation**: Comprehensive documentation and examples
- **Fallback**: Gradual migration path

## Next Steps

1. **Immediate** (This Week):
   - [ ] Create gVisor backend skeleton
   - [ ] Implement basic OCI image loading
   - [ ] Set up automated validation scripts

2. **Short Term** (Next 2 Weeks):
   - [ ] Complete gVisor backend implementation
   - [ ] Achieve 50% test pass rate
   - [ ] Document architecture

3. **Medium Term** (Next Month):
   - [ ] Achieve 100% test pass rate
   - [ ] Performance optimization
   - [ ] Complete documentation

4. **Long Term** (Next Quarter):
   - [ ] Production deployment
   - [ ] User feedback integration
   - [ ] Continuous optimization

## References

- [gVisor Documentation](https://gvisor.dev/)
- [OCI Image Spec](https://github.com/opencontainers/image-spec)
- [OCI Runtime Spec](https://github.com/opencontainers/runtime-spec)
- [clnrm Architecture](/docs/V2_0_0_ARCHITECTURE.md)
- [Performance Baselines](/docs/PERFORMANCE_BASELINE.md)

---

**Document Ownership**: Platform Team
**Review Cycle**: Weekly
**Status Updates**: Every Sprint
