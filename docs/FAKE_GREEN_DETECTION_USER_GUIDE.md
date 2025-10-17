# Fake-Green Detection User Guide

## Table of Contents

1. [Introduction](#introduction)
2. [What is Fake-Green Detection?](#what-is-fake-green-detection)
3. [Why It Matters](#why-it-matters)
4. [How to Use It](#how-to-use-it)
5. [The 7 Detection Layers](#the-7-detection-layers)
6. [Example Workflows](#example-workflows)
7. [Troubleshooting](#troubleshooting)
8. [Best Practices](#best-practices)

---

## Introduction

Fake-green detection is a powerful feature in the Cleanroom Testing Framework that uses OpenTelemetry (OTEL) traces to verify that your tests are genuinely executing code, not just returning false positives. This guide will help you understand and use this critical feature effectively.

**Version:** 1.0.0
**Status:** Production Ready

---

## What is Fake-Green Detection?

### The Problem

A "fake-green" test is a test that:
- Reports **PASS** (green status)
- **But never actually executed** the code under test
- Provides **zero confidence** about system correctness

### Common Causes

```bash
# ❌ Example 1: Stubbed implementation
def run_test():
    print("✅ Test passed!")  # Lies! No actual test ran
    return True

# ❌ Example 2: Early return
def execute_container_test():
    if os.getenv("SKIP_SLOW_TESTS"):
        return "PASS"  # Skipped without notice
    # ... actual test code never runs

# ❌ Example 3: Exception swallowing
try:
    run_integration_test()
except Exception:
    pass  # Silently fails, returns success
return "PASS"
```

### The Solution: OTEL-First Validation

Instead of trusting test assertions, clnrm validates that **observable evidence** exists in OpenTelemetry traces:

```toml
# Tests must PROVE they executed by generating telemetry
[expect.span]
name = "container.exec"
events.any = ["container.start", "container.exec", "container.stop"]

[expect.graph]
must_include = [["test.run", "container.exec"]]

[expect.counts]
spans_total.gte = 2
```

**Key Insight:** If a test claims "PASS" but produces no telemetry evidence, it's fake-green.

---

## Why It Matters

### False Confidence is Worse Than No Tests

```
Traditional Testing:
┌─────────────────┐
│ Test: ✅ PASS   │ ← High confidence
│ Code: 🐛 Broken │ ← Reality: Broken
└─────────────────┘
   💣 Ship to production with bugs
```

```
OTEL-First Validation:
┌─────────────────────────────┐
│ Test: ✅ Claims PASS        │
│ OTEL: ❌ No evidence found  │ ← Detected fake-green!
└─────────────────────────────┘
   🛡️ Prevented deployment of broken code
```

### Real-World Impact

**Without Fake-Green Detection:**
- Production incidents from "passing" tests
- Wasted debugging time
- Loss of team confidence in test suite

**With Fake-Green Detection:**
- 100% confidence: PASS means code executed
- Zero false positives in production
- Team trusts the test suite

---

## How to Use It

### Step 1: Write a Test with OTEL Expectations

Create a `.clnrm.toml` file with telemetry expectations:

```toml
# test.clnrm.toml
[meta]
name = "api_integration_test"
version = "1.0.0"

[otel]
exporter = "otlp"
endpoint = "http://localhost:4318"

[service.api_server]
plugin = "generic_container"
image = "myapi:latest"
env.OTEL_EXPORTER_OTLP_ENDPOINT = "http://localhost:4318"

[[scenario]]
name = "api_health_check"
service = "api_server"
run = "curl http://localhost:8080/health"

# OTEL expectations catch fake-green tests
[[expect.span]]
name = "http.request"
kind = "server"
attrs.all = { "http.status_code" = "200" }

[expect.graph]
must_include = [["api.server", "http.request"]]

[expect.status]
all = "OK"
```

### Step 2: Run the Test

```bash
# Execute test with OTEL validation
clnrm run test.clnrm.toml
```

### Step 3: Analyze Results

**Honest Implementation (PASS):**
```
📊 OTEL Validation Report
========================

Test: api_integration_test
Traces: 5 spans, 12 events

Validators:
  ✅ Span Expectations (2/2 passed)
  ✅ Graph Structure (all 1 edges present)
  ✅ Counts (spans_total: 5)
  ✅ Window Containment (all 1 windows satisfied)
  ✅ Ordering (all constraints satisfied)
  ✅ Status (all spans OK)
  ✅ Hermeticity (no external services detected)

Result: PASS (7/7 validators passed)
```

**Fake-Green Implementation (FAIL):**
```
📊 OTEL Validation Report
========================

Test: api_integration_test
Traces: 0 spans, 0 events

Validators:
  ❌ Span Expectations (Expected span 'http.request' not found)
  ❌ Graph Structure (FAIL: required edge not found)
  ❌ Counts (FAIL: expected at least 2 items, found 0)

Result: FAIL (5/7 validators failed)
```

### Step 4: Fix Fake-Green Tests

If validation fails, the test is fake-green. Fix it by ensuring actual execution:

```bash
# Before (fake-green)
#!/bin/bash
echo "✅ Test passed!"
exit 0

# After (honest implementation)
#!/bin/bash
# Actually run the test with OTEL instrumentation
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 \
  cargo test --features otel
```

---

## The 7 Detection Layers

Fake-green detection uses **7 independent validation layers**. Each layer catches different types of fake-green tests.

### Layer 1: Lifecycle Events

**What it validates:** Container lifecycle events were generated.

**TOML Configuration:**
```toml
[[expect.span]]
name = "container.exec"
events.any = ["container.start", "container.exec", "container.stop"]
```

**Catches:**
- Tests that skip container execution
- Stubbed container operations
- Early returns before container launch

**Example Failure:**
```
❌ Lifecycle Events: Missing lifecycle event: container.start
   Span: container.exec
   Expected events: [container.start, container.exec, container.stop]
   Found events: []
```

---

### Layer 2: Span Graph Structure

**What it validates:** Parent-child relationships exist between spans.

**TOML Configuration:**
```toml
[expect.graph]
must_include = [
    ["test.run", "container.exec"],
    ["container.exec", "process.spawn"]
]
acyclic = true
```

**Catches:**
- Tests that skip orchestration logic
- Missing integration points
- Broken call chains

**Example Failure:**
```
❌ Graph Structure: FAIL: required edge 'test.run' -> 'container.exec' not found
   Expected: Parent span 'test.run' with child 'container.exec'
   Found: No parent-child relationship exists
```

---

### Layer 3: Span Counts

**What it validates:** Expected number of operations occurred.

**TOML Configuration:**
```toml
[expect.counts]
spans_total.gte = 3
by_name."http.request".eq = 5
errors_total.eq = 0
```

**Catches:**
- Tests that execute fewer operations than expected
- Loop bodies that never run
- Batches that process zero items

**Example Failure:**
```
❌ Counts: FAIL: Total span count: expected at least 3 items, found 0
```

---

### Layer 4: Temporal Ordering

**What it validates:** Operations occurred in correct sequence.

**TOML Configuration:**
```toml
[expect.order]
must_precede = [
    ["plugin.registry", "container.start"],
    ["container.start", "test.execute"]
]
```

**Catches:**
- Tests that skip initialization steps
- Race conditions
- Out-of-order execution

**Example Failure:**
```
❌ Ordering: FAIL: 'plugin.registry' must precede 'container.start' but no valid ordering found
   Expected: plugin.registry.end <= container.start.start
   Found: No ordering relationship
```

---

### Layer 5: Window Containment

**What it validates:** Child operations completed within parent timeframes.

**TOML Configuration:**
```toml
[[expect.window]]
outer = "test.run"
contains = ["container.start", "test.execute", "container.stop"]
```

**Catches:**
- Tests that skip cleanup
- Leaked background processes
- Incomplete execution

**Example Failure:**
```
❌ Window Containment: FAIL: window 'test.run': child span 'container.stop' not found
```

---

### Layer 6: Status Validation

**What it validates:** All operations completed successfully.

**TOML Configuration:**
```toml
[expect.status]
all = "OK"
by_name."http.*" = "OK"
```

**Catches:**
- Tests that ignore errors
- Silent failures
- Exception swallowing

**Example Failure:**
```
❌ Status: FAIL: span 'http.request' has status ERROR but expected OK
   Span: http.request
   Status: ERROR (expected OK)
```

---

### Layer 7: Hermeticity Validation

**What it validates:** Tests run in isolation without external dependencies.

**TOML Configuration:**
```toml
[expect.hermeticity]
no_external_services = true
resource_attrs.must_match = { "service.name" = "clnrm", "env" = "test" }
span_attrs.forbid_keys = ["net.peer.name", "http.url"]
```

**Catches:**
- Tests that call production APIs
- Network access to external services
- Missing test isolation

**Example Failure:**
```
❌ Hermeticity: FAIL: Span 'api.call' contains external network attribute 'net.peer.name'
   Attribute: net.peer.name
   Value: api.production.com
   Violation: Non-hermetic execution detected
```

---

## Example Workflows

### Workflow 1: Add Fake-Green Detection to Existing Test

**Before:**
```toml
# basic-test.clnrm.toml
[test.metadata]
name = "basic_container_test"

[services.app]
type = "generic_container"
image = "alpine:latest"

[[steps]]
name = "run_test"
command = ["echo", "hello"]
```

**After (with fake-green detection):**
```toml
# basic-test.clnrm.toml
[meta]
name = "basic_container_test"
version = "1.0.0"

[otel]
exporter = "otlp"
endpoint = "http://localhost:4318"

[service.app]
plugin = "generic_container"
image = "alpine:latest"

[[scenario]]
name = "run_test"
service = "app"
run = "echo hello"

# Fake-green detection layers
[[expect.span]]
name = "container.exec"
kind = "internal"
events.any = ["container.start", "container.exec"]

[expect.graph]
must_include = [["test.run", "container.exec"]]

[expect.counts]
spans_total.gte = 2

[expect.status]
all = "OK"
```

---

### Workflow 2: Validate Test Suite Against Fake-Greens

```bash
# Run all tests with OTEL validation
clnrm run tests/

# Check for fake-green failures
if [ $? -ne 0 ]; then
    echo "❌ Fake-green tests detected!"
    clnrm analyze --verbose
    exit 1
fi

echo "✅ All tests have valid OTEL evidence"
```

---

### Workflow 3: CI/CD Integration

```yaml
# .github/workflows/test.yml
name: Test with Fake-Green Detection

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest

    services:
      otel-collector:
        image: otel/opentelemetry-collector:latest
        ports:
          - 4318:4318

    steps:
      - uses: actions/checkout@v3

      - name: Run tests with OTEL validation
        run: |
          clnrm run tests/ \
            --otel-endpoint http://localhost:4318 \
            --format junit \
            > test-results.xml

      - name: Check for fake-green tests
        run: |
          if grep -q "fake-green detected" test-results.xml; then
            echo "❌ FAIL: Fake-green tests found"
            exit 1
          fi
          echo "✅ PASS: All tests validated"
```

---

## Troubleshooting

### Issue 1: "Span not found" errors

**Symptom:**
```
❌ Span Expectations: Expected span 'container.exec' not found
```

**Cause:** OTEL instrumentation is not enabled or traces aren't being collected.

**Solution:**
```bash
# 1. Verify OTEL exporter is configured
cat test.clnrm.toml | grep -A 3 "\[otel\]"

# 2. Check OTEL collector is running
curl http://localhost:4318/v1/traces

# 3. Enable OTEL features
cargo build --features otel
clnrm run --features otel test.clnrm.toml
```

---

### Issue 2: "Graph structure validation failed"

**Symptom:**
```
❌ Graph Structure: FAIL: required edge 'parent' -> 'child' not found
```

**Cause:** Parent-child span relationships are not being created.

**Solution:**
```rust
// Ensure spans are properly nested
use tracing::{span, Level};

#[tracing::instrument]
fn parent_operation() {
    let child_span = span!(Level::INFO, "child_operation");
    let _enter = child_span.enter();

    // Child work happens here
    actual_work();
}
```

---

### Issue 3: "All validators failed"

**Symptom:**
```
Result: FAIL (7/7 validators failed)
Traces: 0 spans, 0 events
```

**Cause:** Test is completely fake-green - no code executed at all.

**Solution:**
1. Review test implementation for early returns
2. Check for environment variables that skip tests
3. Verify container actually launches
4. Add logging to confirm execution path

```bash
# Debug test execution
RUST_LOG=debug clnrm run test.clnrm.toml

# Check if containers were created
docker ps -a | grep test
```

---

### Issue 4: "Hermeticity validation failed"

**Symptom:**
```
❌ Hermeticity: external network attribute 'http.url' found
```

**Cause:** Test is calling external services (not hermetic).

**Solution:**
```toml
# Option 1: Allow external services (not recommended)
[expect.hermeticity]
no_external_services = false

# Option 2: Use mocks/stubs instead (recommended)
[service.mock_api]
plugin = "generic_container"
image = "mockserver:latest"
ports = ["8080:1080"]
```

---

## Best Practices

### 1. Start with Comprehensive Expectations

**Good:**
```toml
# All 7 detection layers configured
[[expect.span]]
name = "container.exec"
events.any = ["container.start", "container.exec", "container.stop"]

[expect.graph]
must_include = [["run", "exec"]]

[expect.counts]
spans_total.gte = 2

[expect.order]
must_precede = [["registry", "exec"]]

[[expect.window]]
outer = "run"
contains = ["exec"]

[expect.status]
all = "OK"

[expect.hermeticity]
no_external_services = true
```

**Why:** Each layer provides independent validation. More layers = higher confidence.

---

### 2. Use Specific Span Names

**Bad:**
```toml
[[expect.span]]
name = "span"  # Too generic
```

**Good:**
```toml
[[expect.span]]
name = "clnrm.container.exec"  # Specific and namespaced
```

**Why:** Specific names prevent false positives from unrelated spans.

---

### 3. Validate Error Paths Too

```toml
# Test that errors are properly reported
[[scenario]]
name = "error_handling_test"
service = "app"
run = "false"  # Intentional failure

[[expect.span]]
name = "container.exec"
attrs.all = { "error" = "true" }

[expect.status]
by_name."container.exec" = "ERROR"

[expect.counts]
errors_total.eq = 1
```

**Why:** Fake-green tests often ignore errors. Validate error handling too.

---

### 4. Use Deterministic Configuration

```toml
[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"

[report]
json = "report.json"
digest = "trace.sha256"  # Reproducible digest
```

**Why:** Deterministic tests produce identical telemetry, making validation reliable.

---

### 5. Document Expected Behavior

```toml
# docs/test-expectations.md
#
# Test: api_integration_test
# Expected spans: 5
# Expected events: 12
# Critical edges: ["api.server" -> "http.request"]
#
# If this test fails, check:
# 1. Is the OTEL collector running?
# 2. Are environment variables set correctly?
# 3. Did the API server start successfully?
```

**Why:** Documentation helps debug failures faster.

---

## Summary

Fake-green detection gives you **mathematical certainty** that your tests actually execute code:

```
Without Fake-Green Detection:
  Test says "PASS" → Trust it? 🤷

With Fake-Green Detection:
  Test says "PASS" + OTEL proves execution → Trust it! ✅
```

**Key Takeaways:**
1. Use all 7 detection layers for maximum confidence
2. Configure OTEL expectations in every test
3. Treat validation failures as critical bugs
4. Run tests in CI/CD with OTEL validation
5. Document expected telemetry patterns

**Next Steps:**
- Read the [Developer Guide](FAKE_GREEN_DETECTION_DEV_GUIDE.md) to add custom validators
- See [TOML Schema Reference](FAKE_GREEN_TOML_SCHEMA.md) for complete configuration options
- Review [CLI Reference](CLI_ANALYZE_REFERENCE.md) for command-line usage

---

**Questions?** See [docs/](.) for complete documentation.
