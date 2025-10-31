# Running Weaver Validation - Quick Start Guide

**Status:** Complete validation infrastructure ready to run

## Prerequisites

1. **Docker Desktop** - Must be fully started
2. **Weaver** - Install with `cargo install weaver`
3. **clnrm** - Build with `cargo build --release --features otel`

## Current Status

✅ **COMPLETED:**
- OTLP export configuration fix implemented
- Comprehensive validation script created
- All 5 failure modes documented with recovery
- 11 PlantUML architecture diagrams (3,096 lines)
- Docker integration tests ready

❌ **BLOCKING:**
- Docker daemon not running (needs to be started)

## Step 1: Start Docker Daemon

Docker Desktop GUI is running but the daemon hasn't fully started.

**Check Status:**
```bash
# This will show if daemon is ready
docker ps
```

**If you see "Cannot connect to the Docker daemon":**

1. Open Docker Desktop application (if not already open)
2. Look for the whale icon in your menu bar
3. Wait until it stops animating (solid/steady icon = ready)
4. This can take 30-60 seconds after opening Docker Desktop

**Or use the wait helper:**
```bash
./scripts/wait_for_docker.sh
```

## Step 2: Install Weaver (if not already installed)

```bash
cargo install weaver
```

Verify installation:
```bash
weaver --version
```

## Step 3: Run Complete Validation

Once Docker is ready, run the comprehensive validation script:

```bash
./scripts/run_weaver_validation.sh
```

This script will:
1. ✅ Check Docker daemon is running
2. ✅ Check port 4317 is available (clean up if needed)
3. ✅ Validate registry schemas (14 files, 0 warnings)
4. ✅ Start Weaver Live Check on :4317
5. ✅ Run Docker integration tests with OTLP export
6. ✅ Stop Weaver and generate JSON report
7. ✅ Validate report (check violations, coverage)
8. ✅ Display results summary

## What the Script Does

### Pre-flight Checks (Handles Failure Modes)

**Failure Mode #1 - Weaver Not Started:**
- Script starts Weaver automatically
- Verifies it's listening on :4317

**Failure Mode #2 - Tests Export to STDOUT:**
- ✅ FIXED: Tests now respect `OTEL_EXPORTER_OTLP_ENDPOINT`
- Script sets environment variable before running tests

**Failure Mode #3 - Docker Not Running:**
- Script checks Docker is running before proceeding
- Provides clear instructions if not

**Failure Mode #4 - Port Already in Use:**
- Script checks port 4317 availability
- Cleans up existing processes if needed

**Failure Mode #5 - Inactivity Timeout:**
- Script uses 300-second timeout (5 minutes)
- Plenty of time for all tests to complete

### Test Execution

The script runs:
```bash
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
cargo test -p clnrm-core --test docker_integration --features otel -- --test-threads=1
```

**Why this works now:**
- Tests check `OTEL_EXPORTER_OTLP_ENDPOINT` environment variable
- If set: use `Export::OtlpGrpc` to send to Weaver
- If not set: use `Export::StdoutNdjson` for local dev

**Root Cause Fix (in `docker_integration.rs:101-133`):**
```rust
fn init_test_otel() -> Result<OtelGuard> {
    // Read from environment variable set by validation script
    let export = if let Ok(endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
        Export::OtlpGrpc {
            endpoint: Box::leak(endpoint.into_boxed_str())
        }
    } else {
        Export::StdoutNdjson  // Fallback for local dev
    };
    // ...
}
```

### Report Validation

The script checks:

1. **Samples Received:** Must be > 0 (proves telemetry exported)
2. **Violations:** Must be 0 (proves no schema violations)
3. **Coverage:** Target 70%+ (proves adequate instrumentation)

**Expected Output (Success):**
```
===============================================================================
✅ WEAVER VALIDATION PASSED
===============================================================================

Results:
  ✓ Validated samples: 45
  ✓ Violations: 0
  ✓ Coverage: 0.85

Full report: validation_output/live_check.json

Summary:
{
  "violations": 0,
  "improvements": 2,
  "information": 5,
  "coverage": 0.85,
  "total_entities": 50
}
```

## Understanding the Results

### Zero Violations = Success

If violations = 0:
- All required attributes present (container.id, test.isolated, etc.)
- All types match schema definitions
- All naming conventions followed
- **Features proven to work by runtime telemetry**

### What Violations Mean

If violations > 0:
- Missing required attributes (e.g., no `container.id` = fake container)
- Type mismatches (e.g., exit_code should be int64)
- Invalid naming (e.g., attribute names don't follow conventions)
- **These prove false positives or incomplete implementation**

### Coverage Interpretation

- **Coverage = seen_registry_entities / total_registry_entities**
- Example: 42 / 50 = 0.84 (84%)

**clnrm v1.2.0 Targets:**
- Minimum: 70% (0.70)
- Target: 85% (0.85)
- Excellent: 95%+ (0.95)

## Manual Validation (Advanced)

If you want to run Weaver manually:

### Terminal 1: Start Weaver
```bash
weaver registry live-check \
    --registry registry/ \
    --otlp-grpc-port 4317 \
    --format json \
    --output validation_output/ \
    --inactivity-timeout 300
```

### Terminal 2: Run Tests
```bash
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
cargo test -p clnrm-core --test docker_integration --features otel -- --test-threads=1
```

### Terminal 1: Stop Weaver
```bash
# Press Ctrl+C or send SIGHUP
kill -HUP $(pgrep weaver)
```

### Check Report
```bash
cat validation_output/live_check.json | jq '.statistics'
```

## Troubleshooting

### "Cannot connect to the Docker daemon"

**Cause:** Docker daemon not started

**Fix:**
1. Open Docker Desktop
2. Wait for whale icon to stop animating
3. Run: `docker ps` to verify
4. Or use: `./scripts/wait_for_docker.sh`

### "Port 4317 already in use"

**Cause:** Previous Weaver process still running

**Fix:**
```bash
# Find and kill existing process
lsof -i :4317
kill -9 $(lsof -t -i :4317)
```

### "No telemetry received" (0 samples)

**Cause:** Tests not exporting to Weaver

**Fix:** Check environment variable is set:
```bash
echo $OTEL_EXPORTER_OTLP_ENDPOINT
# Should show: http://localhost:4317
```

**Verify in code:** `crates/clnrm-core/tests/docker_integration.rs:113`
```rust
let export = if let Ok(endpoint) = std::env::var("OTEL_EXPORTER_OTLP_ENDPOINT") {
    Export::OtlpGrpc { endpoint: Box::leak(endpoint.into_boxed_str()) }
} else {
    Export::StdoutNdjson
};
```

### "Registry validation failed"

**Cause:** Schema errors in registry/

**Fix:**
```bash
weaver registry check --registry registry/
# Fix any errors reported
```

## Architecture Documentation

Complete architectural documentation available in:
- `docs/architecture/ARCHITECTURE_SUMMARY.md` - Complete overview
- `docs/architecture/PUML_INDEX.md` - Diagram index
- `docs/architecture/*.puml` - 11 PlantUML diagrams (3,096 lines)

**Key Diagrams:**
- `weaver-live-check-complete.puml` - Full Weaver architecture
- `weaver-test-execution-flow.puml` - End-to-end test flow
- `weaver-failure-modes.puml` - All 5 failure modes with recovery
- `weaver-cicd-pipeline.puml` - CI/CD integration example

## Next Steps After Successful Validation

Once validation passes with 0 violations:

1. **CI/CD Integration:** Use the validation script in GitHub Actions
2. **Coverage Improvement:** Add telemetry to increase coverage to 85%+
3. **Production Deployment:** Deploy with confidence (no false positives)
4. **Continuous Validation:** Run on every PR to prevent regressions

## Summary

**The Critical Path:**
```
Test → Span Creation → OTel SDK → Batch Processor →
OTLP Exporter → gRPC :4317 → Weaver Listener →
Advisors → Violations Check → Exit Code → CI/CD Gate
```

**Any break in this chain = No validation**

**The Fix:**
✅ Tests now respect `OTEL_EXPORTER_OTLP_ENDPOINT`
✅ Comprehensive validation script handles all failure modes
✅ Complete documentation and recovery strategies

**To Run:**
1. Start Docker Desktop (wait for daemon to be ready)
2. Run: `./scripts/run_weaver_validation.sh`
3. Review results (target: 0 violations, 70%+ coverage)

---

**Last Updated:** 2025-10-30
**Status:** Ready to run (blocked only by Docker daemon startup)
