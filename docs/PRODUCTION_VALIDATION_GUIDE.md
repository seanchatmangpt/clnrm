# Production Validation Guide

## Overview

This guide describes the comprehensive production validation suite for clnrm's Weaver live-check integration. The validation suite ensures production readiness across all deployment scenarios and failure modes.

## Quick Start

```bash
# Run all production validation tests
./scripts/production_validation.sh all

# Run specific category
./scripts/production_validation.sh performance
./scripts/production_validation.sh reliability
./scripts/production_validation.sh security
./scripts/production_validation.sh deployment
./scripts/production_validation.sh integration

# Quick smoke test
./scripts/production_validation.sh quick
```

## Test Categories

### 1. Performance Validation

**Validates:** Overhead, throughput, latency, streaming performance

**Key Tests:**
- CPU and memory overhead < 10%
- Handles 1000+ spans/sec without drops
- Startup time < 5 seconds
- Shutdown time < 10 seconds
- Sustained load performance

**Run:**
```bash
cargo test --test production_validation --features otel -- --ignored performance
```

**Success Criteria:**
- Memory overhead < 200 MB
- CPU overhead < 10%
- Throughput >= 1000 spans/sec
- Zero dropped spans under normal load
- Graceful degradation under extreme load

### 2. Reliability Validation

**Validates:** Crash recovery, network failures, resource exhaustion

**Key Tests:**
- Force kill recovery (no zombie processes)
- Network failure handling (OTLP unavailable)
- Disk full scenarios
- Timeout behavior
- Multiple start/stop cycles
- Concurrent controller instances

**Run:**
```bash
cargo test --test production_validation --features otel -- --ignored reliability
```

**Success Criteria:**
- Graceful degradation when dependencies fail
- No data loss on unexpected shutdown
- Informative error messages
- Automatic cleanup of resources
- Safe concurrent operation

### 3. Security Validation

**Validates:** Sensitive data handling, redaction, policies

**Key Tests:**
- Sensitive attributes not in output
- PII detection and redaction
- Custom security policies
- Secret masking in error messages
- Secure file permissions
- Data sanitization

**Run:**
```bash
cargo test --test production_validation --features otel -- --ignored security
```

**Success Criteria:**
- No sensitive data in validation reports
- Configurable redaction policies
- Files not world-readable (Unix)
- No API keys/secrets in logs
- SQL injection prevention in error messages

### 4. Deployment Validation

**Validates:** Platform compatibility, containerization, CI/CD

**Key Tests:**
- Docker container deployment
- Kubernetes pod deployment
- GitHub Actions environment
- Multi-platform compatibility (Linux, macOS, Windows)
- Docker Compose orchestration
- Cloud deployment scenarios (AWS, GCP, Azure)
- Bare metal deployment

**Run:**
```bash
cargo test --test production_validation --features otel -- --ignored deployment
```

**Success Criteria:**
- Works in all container runtimes
- Compatible with orchestrators (K8s)
- Runs in CI/CD pipelines
- Cross-platform (Linux, macOS, Windows)
- Cloud-native friendly

### 5. Integration Validation

**Validates:** Real-world usage, end-to-end workflows

**Key Tests:**
- Real clnrm tests with Weaver
- Multiple concurrent live-checks
- Different OTLP endpoints (Jaeger, Collector, etc.)
- Custom registries
- Docker OTLP Collector integration
- End-to-end validation workflow
- High-cardinality attributes

**Run:**
```bash
cargo test --test production_validation --features otel -- --ignored integration
```

**Success Criteria:**
- Real tests produce valid telemetry
- Weaver validates actual clnrm tests
- Works with industry-standard OTLP backends
- Custom schemas supported
- 100% pass rate in end-to-end workflow

## Validation Infrastructure

### Prerequisites

1. **Weaver Installation:**
   ```bash
   cargo install weaver-cli
   # Or download from: https://github.com/open-telemetry/weaver/releases
   ```

2. **Docker (for deployment tests):**
   ```bash
   docker --version  # Should be >= 20.10
   ```

3. **OTLP Backend (optional, for integration tests):**
   ```bash
   # Start Jaeger
   docker run -d --name jaeger \
     -p 4317:4317 \
     -p 4318:4318 \
     -p 16686:16686 \
     jaegertracing/all-in-one:latest

   # Or start OTEL Collector
   docker run -d --name otel-collector \
     -p 4317:4317 \
     -p 4318:4318 \
     otel/opentelemetry-collector-contrib:latest
   ```

### Test Data

Tests use:
- **Real registry:** `registry/` (14 schemas, 200+ entities)
- **Temporary output:** `/tmp/clnrm_*_test/`
- **Mock telemetry:** Simulated spans/metrics
- **Real clnrm tests:** `cargo test --lib --features otel`

## Failure Modes and Recovery

### Common Failure Scenarios

#### 1. Weaver Not Found

**Symptom:** `Failed to start Weaver (is it installed?)`

**Recovery:**
```bash
cargo install weaver-cli
# Or
brew install weaver  # If available
```

#### 2. Port Already In Use

**Symptom:** `Address already in use`

**Recovery:**
```bash
# Find process using port
lsof -i :4317
kill <PID>

# Or use different port
export OTLP_PORT=4327
```

#### 3. Registry Not Found

**Symptom:** `Failed to load registry`

**Recovery:**
```bash
# Ensure registry exists
ls -la registry/
weaver registry check --registry registry/
```

#### 4. Timeout on Shutdown

**Symptom:** `Weaver did not stop within timeout`

**Recovery:**
- Check for hung processes: `ps aux | grep weaver`
- Force kill: `killall -9 weaver`
- Increase timeout in `WeaverController::wait_with_timeout()`

#### 5. Disk Full

**Symptom:** `No space left on device`

**Recovery:**
```bash
# Clean up test artifacts
rm -rf /tmp/clnrm_*_test/
rm -rf ./validation_output/

# Check disk space
df -h
```

### Automatic Recovery

The validation suite includes automatic recovery for:

1. **Zombie Processes:** `Drop` impl kills orphaned Weaver processes
2. **Temporary Files:** Tests clean up `/tmp/clnrm_*` on success
3. **Port Conflicts:** Tests use different ports (4317, 4327, 4337)
4. **Container Cleanup:** Docker tests remove containers on completion

## Performance Benchmarks

### Baseline Metrics (Reference System)

**System:** MacBook Pro M1, 16GB RAM, macOS 14.x

| Metric | Value | Threshold |
|--------|-------|-----------|
| Startup Time | ~1.5s | < 5s |
| Shutdown Time | ~2.0s | < 10s |
| Memory Overhead | ~100 MB | < 200 MB |
| CPU Overhead | ~5% | < 10% |
| Throughput | 1200 spans/sec | >= 1000 spans/sec |

### Running Benchmarks

```bash
# Run benchmark suite
./scripts/production_validation.sh benchmark

# Or specific benchmark
cargo test --test production_validation --features otel -- \
  --ignored \
  benchmark_weaver_latency
```

**Benchmark Output:**
```
📊 Benchmark Results:
   Average startup:  1.498s
   Max startup:      2.103s
   Average shutdown: 2.045s
   Max shutdown:     3.201s
✅ Benchmark completed successfully
```

## CI/CD Integration

### GitHub Actions

```yaml
name: Production Validation

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install Weaver
        run: cargo install weaver-cli

      - name: Schema Validation
        run: weaver registry check --registry registry/

      - name: Quick Validation
        run: ./scripts/production_validation.sh quick

      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: validation-results
          path: validation_output/
```

### Production Deployment Gate

```bash
#!/bin/bash
# deploy.sh - Production deployment script

# GATE 1: Schema validation
if ! weaver registry check --registry registry/; then
    echo "❌ BLOCK: Schema validation failed"
    exit 1
fi

# GATE 2: Live-check validation
./scripts/production_validation.sh integration
if [ $? -ne 0 ]; then
    echo "❌ BLOCK: Live-check validation failed"
    exit 1
fi

# GATE 3: Security validation
./scripts/production_validation.sh security
if [ $? -ne 0 ]; then
    echo "❌ BLOCK: Security validation failed"
    exit 1
fi

# All gates passed
echo "✅ APPROVED: Deploying to production"
./deploy_production.sh
```

## Validation Artifacts

### Output Files

After running validation, artifacts are in `validation_output/production/`:

```
validation_output/production/
├── production_validation_report.md    # Summary report
├── performance/
│   ├── benchmark_results.json
│   └── load_test_metrics.json
├── security/
│   ├── pii_scan_results.json
│   └── secret_detection.log
└── integration/
    ├── e2e_validation_report.json
    └── telemetry_samples/
```

### Report Format

**production_validation_report.md:**
```markdown
# Production Validation Report

**Generated:** 2025-10-30 14:30:00 UTC
**Category:** all
**Registry:** registry/

## Test Execution Summary

- Performance: ✅ PASS (5/5 tests)
- Reliability: ✅ PASS (7/7 tests)
- Security: ✅ PASS (7/7 tests)
- Deployment: ⚠️  PARTIAL (6/8 tests, K8s skipped)
- Integration: ✅ PASS (7/7 tests)

## Performance Metrics

- CPU Overhead: 5.2% ✅
- Memory Overhead: 95 MB ✅
- Throughput: 1205 spans/sec ✅
- Startup Time: 1.6s ✅

## Violations Detected

None

## Recommendations

- Enable Kubernetes tests when cluster available
- Consider increasing OTLP buffer size for high-throughput scenarios

## Conclusion

✅ PRODUCTION READY - All critical validations passed
```

## Troubleshooting

### Debug Mode

Enable verbose logging:

```bash
export RUST_LOG=debug
cargo test --test production_validation --features otel -- --ignored --nocapture
```

### Individual Test Execution

Run single test:

```bash
cargo test --test production_validation --features otel -- \
  --ignored \
  --exact \
  test_weaver_overhead_cpu_memory
```

### Manual Weaver Lifecycle

Test manually:

```bash
# Terminal 1: Start Weaver
weaver registry live-check \
  --registry registry/ \
  --otlp-grpc-port 4317 \
  --output /tmp/manual_test \
  --format json

# Terminal 2: Run tests
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317 \
cargo test --features otel

# Terminal 3: Stop and view report
curl -X POST http://localhost:8080/stop
cat /tmp/manual_test/validation_report.json
```

## Production Criteria

### Definition of Production Ready

ALL must be true:

- [ ] Schema validation passes (`weaver registry check`)
- [ ] Performance benchmarks meet targets
- [ ] Zero security violations
- [ ] Works in target deployment environment
- [ ] End-to-end integration validated
- [ ] Failure modes documented
- [ ] Recovery procedures tested

### Deployment Checklist

Before production deployment:

1. **Validate Schema:**
   ```bash
   weaver registry check --registry registry/
   ```

2. **Run Integration Tests:**
   ```bash
   ./scripts/production_validation.sh integration
   ```

3. **Verify Performance:**
   ```bash
   ./scripts/production_validation.sh performance
   ```

4. **Security Audit:**
   ```bash
   ./scripts/production_validation.sh security
   ```

5. **Platform Compatibility:**
   ```bash
   ./scripts/production_validation.sh deployment
   ```

6. **Review Artifacts:**
   ```bash
   cat validation_output/production/production_validation_report.md
   ```

## Next Steps

After validation passes:

1. **Tag Release:**
   ```bash
   git tag -a v1.2.0-validated -m "Production validation complete"
   git push origin v1.2.0-validated
   ```

2. **Deploy to Staging:**
   ```bash
   ./deploy_staging.sh
   ```

3. **Monitor in Staging:**
   - Check Weaver validation reports
   - Monitor performance metrics
   - Verify telemetry quality

4. **Production Deployment:**
   ```bash
   ./deploy_production.sh
   ```

5. **Post-Deployment Validation:**
   ```bash
   # Run live-check against production telemetry
   weaver registry live-check \
     --registry registry/ \
     --otlp-grpc-port 4317 \
     --input-source production_telemetry.json
   ```

## Support

- **Issues:** https://github.com/seanchatmangpt/clnrm/issues
- **Weaver Docs:** https://github.com/open-telemetry/weaver
- **OpenTelemetry:** https://opentelemetry.io/docs/

---

**Last Updated:** 2025-10-30
**Validation Suite Version:** 1.2.0
