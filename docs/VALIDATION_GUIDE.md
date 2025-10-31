# Validation Guide

**clnrm v1.2.0** - Complete guide to validating clnrm features using Weaver schema validation

## Overview

clnrm uses **OpenTelemetry Weaver schema validation** as the source of truth for feature validation. This guide covers everything you need to validate clnrm features end-to-end.

**Why Weaver Validation?**
- Validates actual runtime telemetry (not just test logic)
- Detects false positives (tests can pass when features are broken)
- Schema defines exact behavior contract
- Cannot pass with fake implementation

---

## Quick Start

### One-Command Validation

```bash
# Run complete validation pipeline
./scripts/validation_pipeline.sh
```

This single command:
- ✅ Starts Docker if needed
- ✅ Configures OTLP environment
- ✅ Starts Weaver live-check
- ✅ Runs integration tests
- ✅ Generates validation report
- ✅ Validates schema compliance

### Manual Validation

```bash
# Terminal 1: Start Weaver
weaver registry live-check --registry registry/ --otlp-grpc-port 4317 --format json --output validation_output/ --inactivity-timeout 300

# Terminal 2: Run tests
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
cargo test -p clnrm-core --test docker_integration --features otel -- --test-threads=1

# Terminal 1: Stop Weaver (Ctrl+C)
# Check results
cat validation_output/live_check.json | jq '.statistics'
```

---

## Prerequisites

1. **Docker Desktop** - Must be fully started
2. **Weaver** - Install with `cargo install weaver`
3. **clnrm** - Build with `cargo build --release --features otel`

---

## Validation Pipeline Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Validation Pipeline                        │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │   Docker    │───▶│    OTLP     │───▶│   Weaver    │     │
│  │   Startup   │    │   Config    │    │   Startup   │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│         │                   │                   │            │
│         └───────────────────┴───────────────────┘            │
│                             │                                │
│                             ▼                                │
│                     ┌─────────────┐                          │
│                     │    Tests    │                          │
│                     │   Execute   │                          │
│                     └─────────────┘                          │
│                             │                                │
│                             ▼                                │
│                     ┌─────────────┐                          │
│                     │   Report    │                          │
│                     │  Validate   │                          │
│                     └─────────────┘                          │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

**The Critical Path:**
```
Test → Span Creation → OTel SDK → Batch Processor →
OTLP Exporter → gRPC :4317 → Weaver Listener →
Advisors → Violations Check → Exit Code → CI/CD Gate
```

Any break in this chain = No validation

---

## Understanding Validation Results

### Success Criteria

Validation passes when:
1. ✅ **Docker Ready** - Daemon responsive and functional
2. ✅ **Weaver Started** - Listening on OTLP port
3. ✅ **Tests Passed** - All integration tests succeed
4. ✅ **Telemetry Received** - Samples > 0
5. ✅ **Zero Violations** - No schema compliance issues
6. ✅ **Coverage Target** - Registry coverage ≥ 70%

### Reading the Report

```json
{
  "advice_level_counts": {
    "violation": 0,       // MUST be 0 for release
    "improvement": 5,     // Suggestions, not blocking
    "information": 12     // FYI only
  },
  "registry_coverage": 0.92,  // MUST be >= 0.85
  "all_advice": [
    {
      "advice_level": "violation",
      "advice_type": "missing_required_attribute",
      "message": "Missing required attribute: container.id",
      "signal_name": "clnrm.test_execution",
      "signal_type": "span"
    }
  ],
  "seen_registry_attributes": {
    "container.id": 10,
    "test.isolated": 8,
    "test.result": 8,
    "container.destroyed_at": 10
  }
}
```

### Decision Matrix

```
Violations = 0? → Continue, else → ❌ BLOCK
Coverage >= 85%? → Continue, else → ❌ BLOCK
Critical attributes present? → ✅ APPROVE, else → ❌ BLOCK
```

**Critical Attributes (MUST be present):**
- `container.id` - Proves container actually created
- `test.isolated` - Proves hermetic isolation working
- `test.result` - Proves test executed to completion
- `container.destroyed_at` - Proves cleanup happened

---

## Script Reference

### validation_pipeline.sh

**Purpose:** Unified end-to-end validation orchestrator

```bash
# Full pipeline
./scripts/validation_pipeline.sh

# Skip Docker startup
./scripts/validation_pipeline.sh --skip-docker

# Skip test execution
./scripts/validation_pipeline.sh --skip-tests

# No cleanup (for debugging)
./scripts/validation_pipeline.sh --no-cleanup
```

### docker_startup.sh

**Purpose:** Cross-platform Docker daemon startup

```bash
./scripts/docker_startup.sh
```

### otlp_config.sh

**Purpose:** OpenTelemetry Protocol environment configuration

```bash
# Export variables to current shell
source ./scripts/otlp_config.sh

# Validate config
./scripts/otlp_config.sh validate
```

### weaver_startup.sh

**Purpose:** Weaver live-check process lifecycle management

```bash
./scripts/weaver_startup.sh start
./scripts/weaver_startup.sh status
./scripts/weaver_startup.sh stop
```

---

## Troubleshooting

### "Cannot connect to the Docker daemon"

**Fix:**
1. Open Docker Desktop
2. Wait for whale icon to stop animating
3. Run: `docker ps` to verify

### "Port 4317 already in use"

**Fix:**
```bash
lsof -i :4317
kill -9 $(lsof -t -i :4317)
```

### "No telemetry received" (0 samples)

**Fix:**
```bash
# Check environment variable
echo $OTEL_EXPORTER_OTLP_ENDPOINT
# Should show: http://localhost:4317

# Verify Weaver is listening
lsof -i :4317
```

### Coverage too low

**Fix:**
1. Review schemas: `ls registry/core/`
2. Find missing span types
3. Add span creation to code

---

## Common Violations and Fixes

### Missing Required Attribute

**Violation:**
```
Missing required attribute: container.id
```

**Fix:**
```rust
let span = span!(
    Level::INFO,
    "clnrm.test",
    test.name = "my_test",
    container.id = %container_id,  // ← Add this
);
```

### Attribute Type Mismatch

**Violation:**
```
Attribute 'test.duration_ms' expected type 'double', got 'int'
```

**Fix:**
```rust
// Wrong
span.record("test.duration_ms", 125);

// Right
span.record("test.duration_ms", 125.0);  // ← Use f64
```

---

## CI/CD Integration

### GitHub Actions

```yaml
name: Weaver Validation

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install Weaver
        run: cargo install weaver
      - name: Run Validation Pipeline
        run: ./scripts/validation_pipeline.sh
      - name: Upload Report
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: validation-report
          path: validation_output/
```

---

## Querying Validation Reports

### Get All Violations
```bash
cat validation_output/validation_report.json | jq '.all_advice[] | select(.advice_level == "violation")'
```

### Get Coverage Percentage
```bash
cat validation_output/validation_report.json | jq '.registry_coverage * 100'
```

### Get Critical Attributes
```bash
cat validation_output/validation_report.json | jq '.seen_registry_attributes | {container.id, test.isolated, test.result, container.destroyed_at}'
```

---

## Migration Guide

### For Existing Features

1. **Define Schema** - Create schema for feature's telemetry
2. **Add Telemetry** - Emit spans matching schema
3. **Run Validation** - Verify 0 violations
4. **Update Tests** - Add Weaver validation checks

See `docs/archive/validation-system/MIGRATING_TO_WEAVER_VALIDATION.md` for detailed migration steps.

---

## Best Practices

### Development Workflow

```bash
# 1. Start infrastructure once
./scripts/docker_startup.sh
./scripts/weaver_startup.sh start

# 2. Configure environment
source ./scripts/otlp_config.sh

# 3. Iterate on tests
cargo test -p clnrm-core --test docker_integration --features otel

# 4. Generate final report
./scripts/weaver_startup.sh stop
```

### Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit
./scripts/validation_pipeline.sh --skip-docker
```

---

## Performance Benchmarks

| Phase | Duration | Notes |
|-------|----------|-------|
| Docker Startup | 5-30s | Depends on cold start |
| OTLP Config | <1s | Instant |
| Weaver Startup | 3-5s | Registry validation |
| Test Execution | 10-60s | Varies by test suite |
| Report Generation | 1-2s | JSON export |
| **Total** | **20-100s** | Full pipeline |

---

## Additional Resources

- **Weaver User Guide:** `docs/weaver/WEAVER_USER_GUIDE.md`
- **Schema Writing Guide:** `docs/SCHEMA_WRITING_GUIDE.md`
- **OpenTelemetry Integration:** `docs/OPENTELEMETRY_INTEGRATION_GUIDE.md`
- **Documentation Validation:** `docs/DOCUMENTATION_VALIDATION_GUIDE.md`

**Archived detailed guides:**
- Pipeline details: `docs/archive/validation-system/VALIDATION_PIPELINE_GUIDE.md`
- Results interpretation: `docs/archive/validation-system/VALIDATION_RESULTS_GUIDE.md`
- Migration steps: `docs/archive/validation-system/MIGRATING_TO_WEAVER_VALIDATION.md`
- Quick reference: `docs/archive/validation-system/QUICK_VALIDATION_GUIDE.md`

---

**Last Updated:** 2025-10-30
**Status:** Complete guide for v1.2.0+ Weaver validation

