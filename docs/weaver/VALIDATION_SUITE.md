# Weaver Live-Check Validation Suite

**Version:** 1.2.0
**Date:** 2025-10-30
**Status:** ✅ READY FOR EXECUTION
**Agent:** tester (hive-mind swarm-1761867489536-uaq0b78ke)

---

## Executive Summary

This validation suite proves that **clnrm's Weaver integration works** by validating actual runtime telemetry against schemas. This is the **ONLY source of truth** for feature validation because:

- ✅ **Traditional tests can lie** (false positives)
- ✅ **Weaver validation cannot lie** (requires actual telemetry)
- ✅ **Schema-first approach** (features must match declared behavior)

### 80/20 Principle Applied

This suite focuses on the **critical 20% of validation scenarios** that deliver **80% confidence** in the system:

| Category | Coverage | Scenarios | Rationale |
|----------|----------|-----------|-----------|
| **Container Isolation** | 95% | 3 | Core feature, highest risk |
| **OTLP Export** | 90% | 2 | Critical integration point |
| **Schema Conformance** | 85% | 2 | Source of truth validation |
| **Plugin System** | 70% | 1 | Lower risk, stable code |
| **CI/CD Integration** | 100% | 1 | Production deployment gate |
| **Edge Cases** | 0% | 0 | NOT tested (low value) |

**Total Scenarios:** 9 (down from 20 possible)
**Pass Rate Target:** 100% (9/9 must pass)
**Execution Time:** <5 minutes

---

## Critical Validation Hierarchy

```
┌─────────────────────────────────────────────────────────────┐
│  LEVEL 1: Weaver Schema Validation (SOURCE OF TRUTH)       │
│  ├─ weaver registry check (schemas valid)                  │
│  └─ weaver registry live-check (runtime telemetry valid)   │
│                                                              │
│  Exit Code 0 = Features work, ship it                      │
│  Exit Code 1 = Features broken, DO NOT SHIP                │
└─────────────────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────────────────┐
│  LEVEL 2: Compilation & Code Quality (BASELINE)            │
│  ├─ cargo build --features otel (compiles)                 │
│  └─ cargo clippy (zero warnings)                           │
└─────────────────────────────────────────────────────────────┘
         ↓
┌─────────────────────────────────────────────────────────────┐
│  LEVEL 3: Traditional Tests (SUPPORTING EVIDENCE)          │
│  ├─ cargo test (unit tests)                                │
│  ├─ cargo test --test docker_integration (integration)     │
│  └─ clnrm self-test (framework self-test)                  │
│                                                              │
│  ⚠️  Can have false positives, NOT source of truth         │
└─────────────────────────────────────────────────────────────┘
```

**CRITICAL RULE:** If Weaver validation fails, the feature DOES NOT WORK, regardless of test results.

---

## Test Coverage Matrix (80/20 Focused)

### Scenario 1: Container Lifecycle Validation (CRITICAL)

**Priority:** P0 (must pass)
**Why:** Core feature - container isolation is clnrm's primary value proposition

**Schema Coverage:**
- `registry/core/container_lifecycle.yaml`
- Required attributes: `container.id`, `container.image`, `container.runtime`

**Test:**
```bash
# Run Docker integration tests with OTLP export
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
cargo test -p clnrm-core --test docker_integration --features otel
```

**Success Criteria:**
- ✅ `container.id` present (proves container actually ran)
- ✅ `container.runtime` = "docker" or "podman"
- ✅ `container.exit_code` present
- ❌ Zero violations

**What This Proves:**
- Container isolation is real (not mocked)
- Lifecycle telemetry accurate
- Schema matches runtime behavior

**What We DON'T Test:**
- ❌ Exotic container runtimes (containerd, cri-o) - low usage
- ❌ Container pause/resume - edge case
- ❌ Multi-arch containers - platform-specific

---

### Scenario 2: Test Execution Telemetry (CRITICAL)

**Priority:** P0 (must pass)
**Why:** Validates test orchestration is working correctly

**Schema Coverage:**
- `registry/core/test_execution.yaml`
- Required attributes: `test.name`, `test.result`, `test.duration_ms`

**Test:**
```bash
# Run core tests with OTLP export
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
cargo test --lib --features otel
```

**Success Criteria:**
- ✅ `test.name` matches actual test names
- ✅ `test.result` = "pass" or "fail" (not "unknown")
- ✅ `test.duration_ms` > 0 (proves test ran)
- ❌ Zero violations

**What This Proves:**
- Test execution is tracked
- Results are accurate
- Telemetry exported correctly

**What We DON'T Test:**
- ❌ Flaky test detection - analysis feature, not core
- ❌ Test retry logic - edge case
- ❌ Parallel test execution telemetry - optimization, not correctness

---

### Scenario 3: OTLP gRPC Export (CRITICAL)

**Priority:** P0 (must pass)
**Why:** Primary integration mechanism for production observability

**Schema Coverage:**
- All schemas (OTLP is the transport)

**Test:**
```bash
# Start Weaver on :4317
weaver registry live-check --registry registry/ --otlp-grpc-port 4317 &

# Export telemetry
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
cargo test --features otel -- --test-threads=1

# Stop Weaver, check report
kill -SIGHUP $WEAVER_PID
cat validation_output/live_check.json
```

**Success Criteria:**
- ✅ Weaver receives telemetry (samples > 0)
- ✅ Zero violations
- ✅ Coverage > 70%
- ❌ Network errors = 0

**What This Proves:**
- OTLP export works
- Network connectivity established
- Batch processing successful

**What We DON'T Test:**
- ❌ OTLP HTTP export - alternative transport, same behavior
- ❌ Custom exporters - plugin territory
- ❌ Compression options - optimization, not correctness

---

### Scenario 4: Schema Conformance Validation (CRITICAL)

**Priority:** P0 (must pass)
**Why:** Validates schemas are correct and comprehensive

**Schema Coverage:**
- All 14 schema files in `registry/`

**Test:**
```bash
# Validate registry
weaver registry check --registry registry/

# Generate Rust code (validates templates)
weaver registry generate rust --registry registry/ --templates templates/registry/rust/
```

**Success Criteria:**
- ✅ Zero schema warnings
- ✅ Zero policy violations
- ✅ Code generation successful
- ❌ No deprecated fields used

**What This Proves:**
- Schemas are syntactically valid
- Semantic conventions followed
- Templates generate valid Rust

**What We DON'T Test:**
- ❌ Schema versioning migrations - future feature
- ❌ Cross-registry dependencies - advanced use case
- ❌ Custom semantic conventions - org-specific

---

### Scenario 5: Plugin Execution Telemetry (HIGH)

**Priority:** P1 (should pass)
**Why:** Validates plugin system instrumentation

**Schema Coverage:**
- `registry/core/plugin_system.yaml`
- Required attributes: `plugin.name`, `plugin.type`, `plugin.execution_time_ms`

**Test:**
```bash
# Run integration test with plugin execution
cargo test -p clnrm-core --test docker_integration --features otel -- test_generic_plugin
```

**Success Criteria:**
- ✅ `plugin.name` present
- ✅ `plugin.execution_time_ms` > 0
- ✅ `plugin.success` = true/false
- ❌ Zero violations

**What This Proves:**
- Plugin execution tracked
- Performance metrics accurate
- Success/failure captured

**What We DON'T Test:**
- ❌ Plugin dependency resolution - framework internal
- ❌ Plugin versioning - not implemented yet
- ❌ Hot plugin reload - advanced feature

---

### Scenario 6: Metrics Export Validation (HIGH)

**Priority:** P1 (should pass)
**Why:** Validates metrics telemetry (complementary to spans)

**Schema Coverage:**
- `registry/metrics/test_metrics.yaml`
- Required metrics: `test.count`, `test.duration.histogram`

**Test:**
```bash
# Run tests with metrics export
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
cargo test --features otel-metrics
```

**Success Criteria:**
- ✅ Metrics received (metric samples > 0)
- ✅ Histogram buckets configured correctly
- ✅ Counter increments match test executions
- ❌ Zero violations

**What This Proves:**
- Metrics instrumentation works
- Aggregations are correct
- Metrics schema valid

**What We DON'T Test:**
- ❌ Custom metric types - low usage
- ❌ Metric cardinality limits - platform-specific
- ❌ Exemplars - advanced feature

---

### Scenario 7: Event Export Validation (MEDIUM)

**Priority:** P2 (nice to have)
**Why:** Validates event telemetry (logs)

**Schema Coverage:**
- `registry/events/test_events.yaml`
- Required attributes: `event.name`, `event.severity`

**Test:**
```bash
# Run tests with event export
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
cargo test --features otel-logs
```

**Success Criteria:**
- ✅ Events received (log samples > 0)
- ✅ Severity levels correct
- ✅ Event names match schema
- ❌ Zero violations

**What This Proves:**
- Log export works
- Structured logging correct
- Event schema valid

**What We DON'T Test:**
- ❌ Log filtering - runtime config
- ❌ Log sampling - optimization
- ❌ Custom log processors - advanced

---

### Scenario 8: CI/CD Integration Test (CRITICAL)

**Priority:** P0 (must pass)
**Why:** Production deployment gate

**Schema Coverage:**
- All schemas (full pipeline validation)

**Test:**
```bash
# Run full validation pipeline
./scripts/validation_pipeline.sh
```

**Success Criteria:**
- ✅ Exit code = 0 (all validations passed)
- ✅ Report generated (`validation_output/live_check.json`)
- ✅ Zero violations
- ✅ Coverage > 70%
- ❌ Pipeline duration < 5 minutes

**What This Proves:**
- End-to-end pipeline works
- Production-ready validation
- Automated quality gate functional

**What We DON'T Test:**
- ❌ Multi-platform CI (Linux, macOS, Windows) - platform-specific testing
- ❌ Matrix builds - CI optimization, not validation
- ❌ Nightly builds - separate concern

---

### Scenario 9: Performance Baseline (HIGH)

**Priority:** P1 (should pass)
**Why:** Validates telemetry overhead is acceptable

**Schema Coverage:**
- All schemas (overhead measurement)

**Test:**
```bash
# Run performance benchmarks
./scripts/run_telemetry_benchmarks.sh
```

**Success Criteria:**
- ✅ Telemetry overhead < 5% (vs no telemetry)
- ✅ Memory increase < 10MB
- ✅ No deadlocks or race conditions
- ❌ Throughput regression < 2%

**What This Proves:**
- Telemetry is production-ready
- Performance impact minimal
- No critical bugs introduced

**What We DON'T Test:**
- ❌ Extreme load scenarios (1M+ tests) - edge case
- ❌ Memory leak detection over days - long-running test
- ❌ Network latency simulation - integration test territory

---

## Validation Execution Plan

### Quick Validation (2 minutes)

**Use Case:** Development iteration, PR checks

```bash
# Run critical scenarios only (P0)
./scripts/quick_validate.sh

# Validates:
# - Container lifecycle (Scenario 1)
# - Test execution (Scenario 2)
# - OTLP export (Scenario 3)
# - Schema conformance (Scenario 4)
```

**Success:** 4/4 scenarios pass (100%)

---

### Comprehensive Validation (5 minutes)

**Use Case:** Pre-release, production deployment

```bash
# Run all 9 scenarios
./scripts/validation_pipeline.sh

# Validates:
# - All P0 scenarios (1-4, 8)
# - All P1 scenarios (5, 6, 9)
# - All P2 scenarios (7)
```

**Success:** 9/9 scenarios pass (100%)

---

### Live-Check Test Suite (10 minutes)

**Use Case:** Weaver capability testing, debugging

```bash
# Run comprehensive Weaver live-check tests
cd tests/weaver/live-check
./run_all_scenarios.sh

# Validates:
# - All input sources (OTLP gRPC, HTTP, file, stdin)
# - All output formats (ANSI, JSON)
# - All advisors (builtin, OTel policies, custom Rego)
# - All stop conditions (SIGINT, SIGHUP, HTTP, timeout)
# - Statistics (coverage, severity)
```

**Success:** 20/20 scenarios pass (100%)

---

## Performance Benchmarks

### Telemetry Overhead Benchmarks

**Location:** `benches/telemetry_performance.rs`

**Scenarios:**

1. **Baseline (No Telemetry)**
   - Run tests with telemetry disabled
   - Measure: execution time, memory usage
   - Purpose: Establish baseline

2. **Spans Only**
   - Enable span telemetry
   - Measure: overhead vs baseline
   - Target: <3% overhead

3. **Spans + Metrics**
   - Enable spans and metrics
   - Measure: overhead vs baseline
   - Target: <5% overhead

4. **Full Telemetry (Spans + Metrics + Logs)**
   - Enable all telemetry
   - Measure: overhead vs baseline
   - Target: <7% overhead

5. **OTLP Export Overhead**
   - Measure network export time
   - Target: <50ms per batch

6. **Batch Processing Efficiency**
   - Measure batch size vs latency
   - Target: 512 spans/batch optimal

7. **Memory Usage**
   - Measure heap allocation
   - Target: <10MB increase

8. **Concurrent Tests**
   - Run 100 parallel tests
   - Measure: throughput, contention
   - Target: Linear scaling up to 16 threads

9. **Large Payload Handling**
   - Export 10,000 spans
   - Measure: processing time, memory
   - Target: <500ms, <50MB

**Execution:**

```bash
# Run all benchmarks
cargo bench --bench telemetry_performance

# Run specific benchmark
cargo bench --bench telemetry_performance -- baseline

# Generate comparison report
./scripts/run_telemetry_benchmarks.sh --compare
```

**Success Criteria:**
- ✅ All benchmarks complete without errors
- ✅ Overhead targets met
- ✅ No performance regressions vs v1.1.0

---

## CI/CD Integration

### GitHub Actions Pipeline

**File:** `.github/workflows/weaver-validation.yml`

```yaml
name: Weaver Validation

on:
  pull_request:
    branches: [main, master]
  push:
    branches: [main, master]

jobs:
  validate:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy

      - name: Install Weaver
        run: cargo install weaver

      - name: Start Docker
        run: |
          sudo systemctl start docker
          docker ps

      - name: Run Validation Pipeline
        run: ./scripts/validation_pipeline.sh
        env:
          RUST_LOG: info

      - name: Upload Validation Report
        if: always()
        uses: actions/upload-artifact@v4
        with:
          name: weaver-validation-report
          path: validation_output/

      - name: Check Violations
        run: |
          violations=$(jq '.statistics.advice_level_counts.violation // 0' validation_output/live_check.json)
          if [ "$violations" -gt 0 ]; then
            echo "❌ Weaver validation failed: $violations violations"
            exit 1
          fi
          echo "✅ Weaver validation passed"

      - name: Comment PR
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v7
        with:
          script: |
            const fs = require('fs');
            const report = JSON.parse(fs.readFileSync('validation_output/live_check.json', 'utf8'));
            const violations = report.statistics?.advice_level_counts?.violation || 0;
            const coverage = report.statistics?.registry_coverage || 0;

            const body = violations === 0
              ? `✅ **Weaver Validation PASSED**\n\n- Violations: 0\n- Coverage: ${(coverage * 100).toFixed(1)}%`
              : `❌ **Weaver Validation FAILED**\n\n- Violations: ${violations}\n- Coverage: ${(coverage * 100).toFixed(1)}%`;

            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: body
            });
```

**What This Does:**
1. Runs on every PR and push to main/master
2. Executes full validation pipeline
3. Uploads validation reports as artifacts
4. Comments on PR with pass/fail status
5. Blocks merge if violations detected

---

### Quick Validation Script

**File:** `scripts/quick_validate.sh`

```bash
#!/bin/bash
# Quick validation for development iteration (2 minutes)

set -e

echo "🚀 Quick Validation (P0 scenarios only)"
echo "========================================"

# Scenario 1: Schema validation
echo "1/4 Validating schemas..."
weaver registry check --registry registry/ || exit 1
echo "✅ Schemas valid"

# Scenario 2: Container lifecycle
echo "2/4 Testing container lifecycle..."
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
cargo test -p clnrm-core --test docker_integration --features otel -- --test-threads=1 || exit 1
echo "✅ Container tests passed"

# Scenario 3: Test execution telemetry
echo "3/4 Testing execution telemetry..."
cargo test --lib --features otel -- --test-threads=1 || exit 1
echo "✅ Execution tests passed"

# Scenario 4: Live-check validation
echo "4/4 Running Weaver live-check..."
./scripts/comprehensive_weaver_validation.sh || exit 1
echo "✅ Weaver validation passed"

echo ""
echo "✅ Quick validation PASSED (4/4)"
echo "Safe to continue development"
```

---

## Pass/Fail Criteria

### Overall Suite Success

**MUST PASS (P0 - Blocking):**
- ✅ Scenario 1: Container Lifecycle
- ✅ Scenario 2: Test Execution
- ✅ Scenario 3: OTLP gRPC Export
- ✅ Scenario 4: Schema Conformance
- ✅ Scenario 8: CI/CD Integration

**SHOULD PASS (P1 - Warning):**
- ✅ Scenario 5: Plugin Execution
- ✅ Scenario 6: Metrics Export
- ✅ Scenario 9: Performance Baseline

**NICE TO HAVE (P2 - Optional):**
- ✅ Scenario 7: Event Export

**Pass Rate:**
- **Production Release:** 9/9 (100%) or 8/9 (P2 may fail)
- **Development:** 5/9 minimum (all P0 must pass)

---

### Individual Scenario Success

Each scenario must meet:

1. **Zero Violations:** Weaver reports 0 schema violations
2. **Required Attributes:** All mandatory attributes present
3. **Type Conformance:** All attribute types match schema
4. **Coverage:** >70% registry coverage for that scenario
5. **Performance:** Within overhead targets

**Failure Indicators:**
- ❌ Violations > 0 = **Schema mismatch, DO NOT SHIP**
- ❌ Required attribute missing = **Feature broken, DO NOT SHIP**
- ❌ Type mismatch = **Instrumentation bug, DO NOT SHIP**
- ⚠️ Coverage < 70% = **Incomplete telemetry, WARN**
- ⚠️ Overhead > target = **Performance issue, WARN**

---

## Validation Report Format

### Report Location

**File:** `validation_output/live_check.json`

### Report Structure

```json
{
  "registry": "registry/",
  "execution_date": "2025-10-30T23:45:00Z",
  "samples": [
    {
      "resource_attributes": {...},
      "scope_attributes": {...},
      "span_name": "container.create",
      "span_attributes": {
        "container.id": "abc123",
        "container.image": "alpine:latest",
        "container.runtime": "docker"
      }
    }
  ],
  "statistics": {
    "total_samples": 142,
    "total_entities": 50,
    "seen_entities": 42,
    "registry_coverage": 0.84,
    "advice_level_counts": {
      "violation": 0,
      "improvement": 3,
      "information": 7
    }
  },
  "all_advice": [
    {
      "signal_type": "span",
      "signal_name": "test.execute",
      "advice_level": "improvement",
      "message": "Consider adding test.tags attribute for better filtering"
    }
  ]
}
```

### Interpreting Results

**Zero Violations (PASS):**
```json
{
  "advice_level_counts": {
    "violation": 0,
    "improvement": 2,
    "information": 5
  }
}
```
✅ All required attributes present
✅ All types match schemas
✅ Feature works as declared

**Schema Violations (FAIL):**
```json
{
  "advice_level_counts": {
    "violation": 3,
    "improvement": 1,
    "information": 4
  },
  "all_advice": [
    {
      "advice_level": "violation",
      "message": "Required attribute 'container.id' is missing"
    }
  ]
}
```
❌ Container didn't actually run (fake/mock)
❌ Feature broken or not implemented
❌ DO NOT SHIP

**Coverage Analysis:**
```json
{
  "registry_coverage": 0.72,
  "total_entities": 50,
  "seen_entities": 36
}
```
- Coverage = 72% (36/50 schemas validated)
- Target: >70% = PASS
- Excellent: >85% = EXCELLENT
- Missing: 14 schemas not validated (low-priority features)

---

## What We DON'T Test (Intentionally)

### Edge Cases (Low Value)

**NOT Tested:**
- ❌ Container pause/resume/kill signals
- ❌ Exotic container runtimes (containerd, cri-o)
- ❌ Multi-arch container images
- ❌ Container resource limits (CPU, memory pinning)
- ❌ Network mode variations (bridge, host, none)
- ❌ Volume mount edge cases (SELinux, AppArmor)

**Why:** These are container runtime features, not clnrm features. Docker/Podman handle these. Testing them would be testing Docker, not clnrm.

---

### Plugin Edge Cases (Low Usage)

**NOT Tested:**
- ❌ Plugin dependency cycles
- ❌ Plugin versioning conflicts
- ❌ Plugin hot reload
- ❌ Plugin crash recovery
- ❌ Plugin timeout handling

**Why:** These are advanced plugin system features not yet implemented. Will test when implemented.

---

### Performance Edge Cases (Extreme Scenarios)

**NOT Tested:**
- ❌ 1 million+ tests in single run
- ❌ Multi-day continuous test runs
- ❌ Extreme network latency (>1s)
- ❌ Disk space exhaustion
- ❌ OOM scenarios

**Why:** These are operational concerns, not functional correctness. Monitor in production, don't test in CI.

---

### OTLP Protocol Variations (Low Impact)

**NOT Tested:**
- ❌ OTLP HTTP export (only gRPC tested)
- ❌ OTLP compression options
- ❌ OTLP retry policies
- ❌ Custom OTLP headers
- ❌ OTLP batching strategies

**Why:** These are OTLP SDK features, not clnrm features. OTLP SDK already tested by OTel project.

---

## Troubleshooting Guide

### Problem: "No telemetry received" (0 samples)

**Root Causes:**
1. Tests not exporting OTLP
2. Environment variable not set
3. Network connection failed

**Fix:**
```bash
# Verify environment variable
echo $OTEL_EXPORTER_OTLP_ENDPOINT
# Should show: http://localhost:4317

# Check Weaver is listening
lsof -i :4317
# Should show weaver process

# Check test code
grep -r "init_test_otel" crates/clnrm-core/tests/
# Should use OTEL_EXPORTER_OTLP_ENDPOINT
```

---

### Problem: "Violations detected"

**Root Causes:**
1. Missing required attributes
2. Type mismatches
3. Schema out of sync with code

**Fix:**
```bash
# View violations
jq '.all_advice[] | select(.advice_level == "violation")' validation_output/live_check.json

# Common violations:
# - Missing container.id → Container mock, not real
# - Wrong type for exit_code → Using string instead of int64
# - Invalid enum value → Using undeclared status value

# Fix in code, re-run validation
```

---

### Problem: "Low coverage" (<70%)

**Root Causes:**
1. Not all code paths executed
2. Schemas defined but features not implemented
3. Conditional telemetry not triggered

**Fix:**
```bash
# Check which schemas not validated
jq '.statistics | {total: .total_entities, seen: .seen_entities, coverage: .registry_coverage}' validation_output/live_check.json

# Add tests to cover missing schemas
# OR remove unused schemas from registry
```

---

### Problem: "Port already in use"

**Root Cause:** Previous Weaver process still running

**Fix:**
```bash
# Find process using port 4317
lsof -i :4317

# Kill it
lsof -ti :4317 | xargs kill -9

# Re-run validation
```

---

### Problem: "Docker daemon not running"

**Root Cause:** Docker not started

**Fix:**
```bash
# Start Docker daemon
sudo systemctl start docker  # Linux
# OR open Docker Desktop      # macOS/Windows

# Wait for ready
docker ps

# Re-run validation
```

---

## Continuous Improvement

### Adding New Scenarios

1. **Identify Need:**
   - New feature requires validation
   - Existing scenario doesn't cover behavior

2. **Apply 80/20:**
   - Does this validate critical 20% functionality?
   - What's the impact if this breaks?
   - How often will this code path execute?

3. **Create Schema:**
   - Define required attributes in `registry/`
   - Run `weaver registry check`

4. **Write Test:**
   - Add to appropriate scenario (1-9)
   - Export OTLP telemetry
   - Verify required attributes present

5. **Update Documentation:**
   - Add to this file
   - Update coverage matrix
   - Document what's NOT tested

---

### Removing Scenarios

**When to Remove:**
- ❌ Feature deprecated/removed
- ❌ Scenario no longer provides value
- ❌ Better validation method available

**How to Remove:**
1. Remove from validation scripts
2. Remove schema from registry (if unused)
3. Update documentation
4. Update coverage targets

---

## Summary

### What This Validation Suite Proves

✅ **Container isolation works** (not mocked)
✅ **Test execution telemetry accurate**
✅ **OTLP export functional**
✅ **Schemas match runtime behavior**
✅ **Plugin system instrumented**
✅ **Metrics exported correctly**
✅ **Events logged properly**
✅ **CI/CD pipeline functional**
✅ **Performance overhead acceptable**

### What It Doesn't Prove

❌ **Edge case handling** (intentionally not tested)
❌ **Extreme load scenarios** (operational concern)
❌ **Platform-specific behavior** (tested separately)
❌ **Future features** (test when implemented)

### Success Metrics

**Production Release:**
- 9/9 scenarios pass (100%)
- Zero violations
- Coverage >70%
- Overhead <5%

**Development:**
- 5/9 scenarios pass minimum (all P0)
- Violations decreasing
- Coverage increasing
- No regressions

---

## Execution Command Summary

```bash
# Quick validation (2 min, P0 only)
./scripts/quick_validate.sh

# Comprehensive validation (5 min, all scenarios)
./scripts/validation_pipeline.sh

# Full live-check suite (10 min, 20 scenarios)
cd tests/weaver/live-check && ./run_all_scenarios.sh

# Performance benchmarks (15 min)
./scripts/run_telemetry_benchmarks.sh

# CI/CD integration (automatic on PR)
# See .github/workflows/weaver-validation.yml
```

---

**Validation Suite Status:** ✅ READY FOR EXECUTION
**Next Action:** Run `./scripts/validation_pipeline.sh` to execute validation
**Expected Result:** 9/9 scenarios pass, 0 violations, 70%+ coverage

---

**Document Version:** 1.0
**Last Updated:** 2025-10-30
**Maintainer:** TESTER agent (hive-mind swarm)
