# Weaver Live-Check Validation Guide

**Version**: v1.3.0
**Status**: Production Ready
**Date**: 2025-10-31

---

## Table of Contents

1. [What is Live-Check?](#what-is-live-check)
2. [Why Live-Check Matters](#why-live-check-matters)
3. [Quick Start](#quick-start)
4. [Validation Modes](#validation-modes)
5. [TOML Configuration](#toml-configuration)
6. [CLI Reference](#cli-reference)
7. [Understanding Results](#understanding-results)
8. [Common Workflows](#common-workflows)
9. [Troubleshooting](#troubleshooting)
10. [Examples](#examples)

---

## What is Live-Check?

Weaver live-check is **schema-first telemetry validation** that proves your code emits correct OpenTelemetry data at runtime.

### Traditional Testing Problem

```bash
#!/bin/bash
# Test passes but does nothing
echo "✅ Test passed"
exit 0

# ❌ No container was created
# ❌ No database was queried
# ❌ Services never interacted
# ✅ Exit code 0 = "success"
```

Traditional tests can pass even when features are broken (false positives).

### Live-Check Solution

Live-check validates **actual runtime behavior** through telemetry:

```yaml
# Schema defines contract
[[expect.span]]
name = "container.lifecycle"
attrs.all = { "container.id" = "*", "container.destroyed_at" = "*" }

# Runtime validation
✅ Container ID present → Container actually created
✅ Destroyed timestamp → Container actually cleaned up
❌ Missing attribute → Test is fake-green
```

**Key Insight**: If your code doesn't emit the right telemetry, live-check fails. No false positives.

---

## Why Live-Check Matters

### False Positives Eliminated

| Traditional Testing | Weaver Live-Check |
|---------------------|-------------------|
| ✅ Exit code 0 = "pass" | ✅ Telemetry proves execution |
| ❌ Can pass without running feature | ❌ Must emit correct telemetry |
| ❌ Can pass with mocked behavior | ❌ Validates actual runtime |
| ❌ Tests test logic, not production | ✅ Validates production behavior |

### Real-World Example

```toml
# Test claims to start database
[[scenario]]
name = "start_database"
run = "echo 'Database started'"  # Fake implementation
exit_code = 0

# Traditional test: PASS (exit code 0)
# Live-check: FAIL (no database span emitted)
```

Live-check catches this fake-green test because no `db.connect` span was emitted.

---

## Quick Start

### 1. Install Weaver

```bash
# Install Weaver CLI
cargo install weaver-cli

# Verify installation
weaver --version
```

### 2. Enable Live-Check in TOML

```toml
# tests/my_test.clnrm.toml
[meta]
name = "my_first_live_check"
version = "1.0.0"

[weaver]
enabled = true
registry_path = "registry"  # Path to schema registry

[otel]
exporter = "otlp-http"

[service.api]
plugin = "generic_container"
image = "my-api:latest"

[[scenario]]
name = "api_handles_request"
service = "api"
run = "curl http://localhost:8080/health"

# Validate span was emitted
[[expect.span]]
name = "http.server.request"
attrs.all = { "http.method" = "GET", "http.route" = "/health" }
```

### 3. Run Test

```bash
clnrm run tests/my_test.clnrm.toml
```

Output:
```
✅ Weaver validation: PASS
   - 12 spans received
   - 0 violations
   - Registry coverage: 85%
```

---

## Validation Modes

clnrm v1.3.0 supports four validation modes, each optimized for different use cases.

### Strict Mode (100% Validation)

**When to use**: Production releases, compliance audits, security reviews

```toml
[weaver]
enabled = true

[weaver.validation]
mode = "strict"
fail_on_violations = true
```

**Characteristics:**
- ✅ Validates ALL spans and attributes
- ✅ Most thorough validation
- ⚠️ Slowest mode (~10-20% overhead)
- ✅ Use for: Production releases, final certification

**Example Output:**
```
🔍 Strict Mode Validation
   - Spans validated: 847/847 (100%)
   - Attributes validated: 3,421/3,421 (100%)
   - Duration: 2.3s
   - Result: PASS ✅
```

### 80/20 Mode (Recommended)

**When to use**: CI/CD pipelines, development, most testing

```toml
[weaver]
enabled = true

[weaver.validation]
mode = "80_20"

[weaver.eighty_twenty]
critical_spans = [
    "test.execute",
    "container.start",
    "container.stop",
    "service.health_check"
]
```

**Characteristics:**
- ✅ **6x faster** than strict mode
- ✅ **80% bug coverage** with 20% effort
- ✅ Validates critical telemetry only
- ✅ Use for: CI/CD, daily development

**Example Output:**
```
⚡ 80/20 Mode Validation
   - Critical spans: 124/124 (100%)
   - Total spans: 124/847 (15%)
   - Bug coverage: ~80%
   - Duration: 0.4s
   - Result: PASS ✅
```

**Performance Comparison:**
```
Strict:   2.3s  [████████████████████] 100%
80/20:    0.4s  [████                ] 20%
Speedup:  6x faster ⚡
```

### Lenient Mode (90% Validation)

**When to use**: Transitional periods, gradual migration

```toml
[weaver]
enabled = true

[weaver.validation]
mode = "lenient"
fail_on_violations = false  # Warnings only
```

**Characteristics:**
- ✅ Allows minor deviations
- ⚠️ Does not fail on style issues
- ✅ Good for migration from no validation
- ✅ Use for: Gradual adoption, legacy code

**Example Output:**
```
⚠️  Lenient Mode Validation
   - Spans validated: 847/847 (100%)
   - Violations: 0
   - Improvements: 12 (warnings)
   - Result: PASS ✅ (with warnings)
```

### Minimal Mode (60% Validation)

**When to use**: Quick smoke tests, rapid prototyping

```toml
[weaver]
enabled = true

[weaver.validation]
mode = "minimal"

[weaver.eighty_twenty]
critical_spans = ["test.execute"]  # Just the essentials
```

**Characteristics:**
- ✅ **Fastest mode** (~2% overhead)
- ⚠️ Validates only essential telemetry
- ⚠️ Lower bug coverage (~60%)
- ✅ Use for: Smoke tests, prototyping

**Example Output:**
```
💨 Minimal Mode Validation
   - Essential spans: 24/24 (100%)
   - Duration: 0.1s
   - Result: PASS ✅
```

### Mode Comparison Table

| Mode | Speed | Coverage | Use Case | Overhead |
|------|-------|----------|----------|----------|
| **Strict** | 1x | 100% | Production releases | ~20% |
| **80/20** ⭐ | 6x | 80% | CI/CD, development | ~5% |
| **Lenient** | 3x | 90% | Migration, legacy | ~10% |
| **Minimal** | 10x | 60% | Smoke tests | ~2% |

⭐ **Recommended for most users**

---

## TOML Configuration

### Basic Configuration

Minimal setup - good for getting started:

```toml
[meta]
name = "basic_test"
version = "1.0.0"

[weaver]
enabled = true  # That's it!

[service.app]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "test_scenario"
service = "app"
run = "echo hello"
```

### Advanced Configuration

Full control over validation behavior:

```toml
[weaver]
enabled = true
registry_path = "./registry"       # Path to schema registry
otlp_port = 0                      # 0 = auto-discover available port
admin_port = 0                     # 0 = auto-discover available port
output_dir = "./validation_output" # Validation report directory
stream = false                     # Enable real-time streaming
fail_fast = false                  # Stop on first violation

[weaver.validation]
mode = "80_20"                     # strict | 80_20 | lenient | minimal
fail_on_violations = true          # Fail build on violations
fail_on_improvements = false       # Fail on style warnings

[weaver.eighty_twenty]
critical_spans = [
    "test.execute",
    "container.start",
    "container.stop",
    "service.health_check"
]

[weaver.performance]
startup_timeout_ms = 5000          # Max time for Weaver startup
flush_timeout_ms = 2000            # Max time for telemetry flush
max_samples = 100000               # Max samples to collect
```

### Port Configuration

**Auto-Discovery (Recommended)**

```toml
[weaver]
otlp_port = 0      # Automatic port selection
admin_port = 0     # Automatic port selection
```

Weaver will find available ports automatically. Recommended for:
- Local development
- Parallel test execution
- CI/CD environments

**Fixed Ports**

```toml
[weaver]
otlp_port = 4317   # Fixed OTLP gRPC port
admin_port = 8080  # Fixed admin API port
```

Use fixed ports when:
- Debugging with known ports
- Integration with external systems
- Documentation/training examples

**Port Requirements:**
- Ports must be >= 1024 (non-privileged)
- OTLP and admin ports must differ
- Ports must be available (not in use)

### Integration with OTEL

Combine Weaver validation with OTEL export:

```toml
[weaver]
enabled = true

[otel]
exporter = "otlp-http"
endpoint = "http://localhost:4318"  # Weaver OTLP endpoint
protocol = "http/protobuf"
sample_ratio = 1.0

resources = {
  "service.name" = "my_service",
  "service.version" = "1.0.0",
  "deployment.environment" = "test"
}

headers = {
  "Authorization" = "Bearer ${AUTH_TOKEN}"
}

propagators.use = ["tracecontext", "baggage"]
```

**Key Points:**
- Weaver endpoint is configured automatically
- OTEL export goes directly to Weaver
- No external collector needed
- Telemetry validated in real-time

---

## CLI Reference

### Basic Usage

```bash
# Run test with live-check (enabled in TOML)
clnrm run tests/my_test.clnrm.toml

# Force live-check validation (override TOML)
clnrm run tests/my_test.clnrm.toml --validate

# Disable live-check (even if enabled in TOML)
clnrm run tests/my_test.clnrm.toml --no-validate
```

### Validation Mode Override

```bash
# Use strict mode (override TOML)
clnrm run tests/my_test.clnrm.toml --validate-mode strict

# Use 80/20 mode
clnrm run tests/my_test.clnrm.toml --validate-mode 80_20

# Use lenient mode
clnrm run tests/my_test.clnrm.toml --validate-mode lenient
```

### Output Options

```bash
# Streaming output (real-time feedback)
clnrm run tests/my_test.clnrm.toml --stream

# Fail fast (stop on first violation)
clnrm run tests/my_test.clnrm.toml --fail-fast

# Custom output directory
clnrm run tests/my_test.clnrm.toml --output-dir ./reports

# JSON format
clnrm run tests/my_test.clnrm.toml --format json
```

### Registry Commands

```bash
# Validate schemas (static check)
clnrm validate --registry ./registry

# List all schemas
clnrm registry list

# Show schema details
clnrm registry show core.test_execution

# Check schema coverage
clnrm registry coverage tests/
```

### Debug Options

```bash
# Verbose output
clnrm run tests/my_test.clnrm.toml --verbose

# Debug logs
clnrm run tests/my_test.clnrm.toml --log-level debug

# Keep Weaver running (for inspection)
clnrm run tests/my_test.clnrm.toml --keep-alive

# Save telemetry dump
clnrm run tests/my_test.clnrm.toml --dump-telemetry
```

---

## Understanding Results

### Success Output

```
🎉 Validation: PASS

✅ Weaver Live-Check Results:
   - Validation mode: 80/20
   - Samples received: 124
   - Violations: 0
   - Improvements: 0
   - Registry coverage: 87.5%
   - Duration: 0.42s

✅ Critical Spans Validated:
   ✓ test.execute (12 spans)
   ✓ container.start (4 spans)
   ✓ container.stop (4 spans)
   ✓ service.health_check (8 spans)

📊 Telemetry Summary:
   - Total spans: 847
   - Total events: 234
   - Total metrics: 56
   - Trace count: 12
```

### Failure Output

```
❌ Validation: FAIL

❌ Weaver Live-Check Results:
   - Validation mode: strict
   - Samples received: 124
   - Violations: 3
   - Improvements: 7
   - Registry coverage: 65.2%
   - Duration: 2.15s

❌ Violations (CRITICAL):

  1. Missing Required Attribute
     Span: test.execute
     Attribute: container.id
     Message: Required attribute 'container.id' not found
     Impact: Cannot prove test ran in container
     Fix: Add container.id to span attributes

  2. Invalid Attribute Type
     Span: container.lifecycle
     Attribute: container.destroyed_at
     Expected: string (ISO 8601 timestamp)
     Actual: int (unix timestamp)
     Fix: Use ISO 8601 format: "2025-10-31T14:23:45Z"

  3. Missing Span
     Expected: service.health_check
     Found: 0 instances
     Impact: Cannot prove health check executed
     Fix: Emit service.health_check span

⚠️  Improvements (Warnings):

  1. Attribute Naming Convention
     Span: test.execute
     Attribute: testName
     Suggestion: Use dot notation: test.name
     Severity: style

💡 Recommendations:
   - Fix 3 violations before release
   - Consider 7 improvements for consistency
   - Increase registry coverage (target: 80%+)
```

### Validation Report Structure

Reports are saved to `output_dir/validation_report/`:

```
validation_report/
├── summary.json              # High-level summary
├── violations.json           # Detailed violations
├── improvements.json         # Recommended improvements
├── telemetry_dump.json       # Raw telemetry (if --dump-telemetry)
└── coverage_report.json      # Registry coverage
```

**summary.json**:
```json
{
  "validation_mode": "80_20",
  "sample_count": 124,
  "violations": 0,
  "improvements": 0,
  "registry_coverage": 0.875,
  "duration_ms": 420,
  "result": "pass",
  "timestamp": "2025-10-31T14:23:45Z"
}
```

---

## Common Workflows

### Local Development

```bash
# Fast feedback with 80/20 mode
clnrm run tests/my_test.clnrm.toml --validate-mode 80_20

# Iterate quickly
cargo build && clnrm run tests/
```

Configuration:
```toml
[weaver.validation]
mode = "80_20"           # Fast validation
fail_on_improvements = false  # Ignore style warnings
```

### CI/CD Pipeline

```bash
# Comprehensive validation in CI
clnrm run tests/ --validate-mode strict --fail-fast
```

GitHub Actions:
```yaml
- name: Run Tests with Live-Check
  run: |
    clnrm run tests/ \
      --validate-mode strict \
      --format json \
      --output-dir ./reports

- name: Upload Reports
  if: always()
  uses: actions/upload-artifact@v3
  with:
    name: validation-reports
    path: ./reports/
```

### Pre-Release Validation

```bash
# Final validation before release
clnrm run tests/ \
  --validate-mode strict \
  --fail-on-improvements \
  --output-dir ./release-validation
```

Checklist:
- [ ] All tests pass
- [ ] Zero violations
- [ ] Zero improvement warnings
- [ ] Registry coverage > 80%
- [ ] Telemetry conforms to semantic conventions

### Debugging Failed Tests

```bash
# Run with debug output
clnrm run tests/failing_test.clnrm.toml \
  --validate \
  --verbose \
  --log-level debug \
  --keep-alive \
  --dump-telemetry

# Inspect telemetry
cat validation_report/telemetry_dump.json | jq '.spans[] | select(.name == "test.execute")'

# Check Weaver logs
cat validation_report/weaver.log
```

---

## Troubleshooting

### Zero Samples Received

**Problem**: Weaver reports success but `sample_count = 0`

**Diagnosis**:
```bash
# Check OTEL configuration
clnrm run tests/test.clnrm.toml --verbose

# Verify Weaver is listening
lsof -i :4317

# Check connectivity
curl http://localhost:4317
```

**Common Causes**:
1. ❌ OTEL export not configured
2. ❌ Wrong endpoint in OTEL config
3. ❌ Weaver not started before tests
4. ❌ Firewall blocking port 4317

**Fix**:
```toml
[weaver]
enabled = true  # Ensure enabled

[otel]
exporter = "otlp-http"  # Must match Weaver protocol
# endpoint configured automatically by clnrm
```

### Port Conflicts

**Problem**: "Address already in use" error

**Diagnosis**:
```bash
# Check ports in use
lsof -i :4317
lsof -i :8080

# Find conflicting process
ps aux | grep weaver
```

**Fix**:
```toml
[weaver]
otlp_port = 0    # Use auto-discovery
admin_port = 0   # Use auto-discovery
```

### Missing Required Attributes

**Problem**: Violation for missing required attribute

**Example**:
```
❌ Missing Required Attribute: container.id
```

**Fix**:
```rust
// Ensure all required attributes are set
let span = trace_span!(
    "test.execute",
    test.name = %test_name,
    container.id = %container_id,  // Add missing attribute
    test.isolated = true
);
```

### Schema Not Found

**Problem**: "Schema not found in registry"

**Diagnosis**:
```bash
# List available schemas
clnrm registry list

# Check registry path
ls -la registry/
```

**Fix**:
```toml
[weaver]
registry_path = "./registry"  # Correct path to schemas
```

### Validation Timeout

**Problem**: Weaver times out waiting for telemetry

**Diagnosis**:
```bash
# Check flush timeout
clnrm run tests/test.clnrm.toml --verbose
```

**Fix**:
```toml
[weaver.performance]
flush_timeout_ms = 5000  # Increase timeout
```

---

## Examples

### Example 1: Basic HTTP Service

```toml
[meta]
name = "http_service_test"
version = "1.0.0"

[weaver]
enabled = true

[service.api]
plugin = "generic_container"
image = "my-api:latest"

[[scenario]]
name = "health_check"
service = "api"
run = "curl http://localhost:8080/health"

[[expect.span]]
name = "http.server.request"
kind = "server"
attrs.all = {
  "http.method" = "GET",
  "http.route" = "/health",
  "http.status_code" = "200"
}
```

### Example 2: Database Integration

```toml
[meta]
name = "database_test"
version = "1.0.0"

[weaver]
enabled = true

[service.db]
plugin = "generic_container"
image = "postgres:15"

[[scenario]]
name = "query_users"
service = "db"
run = "psql -c 'SELECT * FROM users'"

[[expect.span]]
name = "db.query"
kind = "client"
attrs.all = {
  "db.system" = "postgresql",
  "db.operation" = "SELECT",
  "db.name" = "testdb"
}

# Ensure query completed successfully
[expect.status]
by_name = { "db.query" = "OK" }
```

### Example 3: Multi-Service with 80/20 Mode

```toml
[meta]
name = "microservices_test"
version = "1.0.0"

[weaver]
enabled = true

[weaver.validation]
mode = "80_20"

[weaver.eighty_twenty]
critical_spans = [
    "http.server.request",
    "db.query",
    "cache.get"
]

[service.api]
plugin = "generic_container"
image = "api:latest"

[service.db]
plugin = "generic_container"
image = "postgres:15"

[service.cache]
plugin = "generic_container"
image = "redis:7"

[[scenario]]
name = "api_with_cache"
service = "api"
run = "curl http://localhost:8080/api/users/123"

# Validate critical path only (80/20)
[[expect.span]]
name = "http.server.request"
attrs.all = { "http.route" = "/api/users/{id}" }

[[expect.span]]
name = "cache.get"
parent = "http.server.request"
attrs.all = { "cache.key" = "user:123" }

[[expect.span]]
name = "db.query"
parent = "http.server.request"
attrs.all = { "db.operation" = "SELECT" }

# Validate trace structure
[expect.graph]
must_include = [
    ["http.server.request", "cache.get"],
    ["http.server.request", "db.query"]
]
```

### Example 4: CI/CD with Strict Validation

```toml
[meta]
name = "production_validation"
version = "1.0.0"

[weaver]
enabled = true

[weaver.validation]
mode = "strict"
fail_on_violations = true
fail_on_improvements = true  # Enforce style in CI

[weaver.performance]
flush_timeout_ms = 10000  # Longer timeout for CI
max_samples = 1000000

[service.app]
plugin = "generic_container"
image = "app:${VERSION}"

[[scenario]]
name = "full_integration_test"
service = "app"
run = "./run_integration_tests.sh"

# Comprehensive validation for production
[[expect.span]]
name = "test.execute"
attrs.all = {
  "test.isolated" = "true",
  "container.id" = "*",
  "test.cleanup_performed" = "true"
}

[expect.counts]
spans_total = { gte = 100 }
errors_total = { eq = 0 }

[expect.status]
all = "OK"  # All spans must succeed
```

---

## Next Steps

### Learn More

- **[Best Practices](LIVE_CHECK_BEST_PRACTICES.md)** - Advanced patterns and recommendations
- **[Troubleshooting](LIVE_CHECK_TROUBLESHOOTING.md)** - Detailed problem resolution
- **[Migration Guide](MIGRATING_TO_V1_3_0.md)** - Upgrading from v1.2.x
- **[Tutorial](LIVE_CHECK_TUTORIAL.md)** - Step-by-step walkthrough
- **[TOML Reference](../book/src/reference/toml-schema.md)** - Complete configuration reference

### Get Help

- **GitHub Issues**: https://github.com/seanchatmangpt/clnrm/issues
- **Documentation**: https://github.com/seanchatmangpt/clnrm/docs
- **Examples**: https://github.com/seanchatmangpt/clnrm/examples

---

**Last Updated**: 2025-10-31
**Version**: v1.3.0
**Feedback**: Please report issues or suggestions on GitHub
