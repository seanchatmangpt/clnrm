# Red-Team OTLP Validation Case Study

## Executive Summary

This case study demonstrates **clnrm's red-team approach to detecting fake-green tests** using OpenTelemetry Protocol (OTLP) validation with environment variable configuration. Unlike traditional assertion-based testing, OTEL-first validation requires **complete execution evidence** through 7 independent detection layers.

### Key Innovation

**Traditional Testing Problem:**
```bash
#!/bin/bash
echo "Passed"
exit 0
# ✅ PASSES traditional testing (exit code 0)
# ❌ NO actual execution occurred
```

**OTEL-First Solution:**
```bash
# ❌ FAILS OTEL validation - multiple detection layers catch it:
# - Layer 1: No lifecycle events
# - Layer 2: No span graph
# - Layer 3: Zero spans
# - Layer 4: No temporal windows
# - Layer 5: No ordering constraints
# - Layer 6: No status codes
# - Layer 7: Missing SDK resources
```

---

## Table of Contents

1. [Overview](#overview)
2. [Detection Layers](#detection-layers)
3. [Environment Variable Configuration](#environment-variable-configuration)
4. [Usage Examples](#usage-examples)
5. [Detection Strategy](#detection-strategy)
6. [Test Results](#test-results)
7. [Implementation Details](#implementation-details)

---

## Overview

### What is Red-Team Validation?

Red-team validation is an adversarial testing approach where we actively try to create tests that appear to pass but don't actually execute the system under test. **The goal is to prove that our validation system cannot be fooled.**

### The Fake-Green Problem

A "fake-green" test is a test that:
- ✅ Reports success (exit code 0)
- ❌ Does NOT execute the system under test
- ❌ Provides NO real validation

**This is catastrophic** because:
- 🐛 Bugs make it to production
- 💥 False confidence in test coverage
- 🚨 Silent failures accumulate over time
- 📉 Test suite becomes meaningless

### Why OTEL-First Validation?

OTEL-first validation requires **proof of execution** at multiple levels:

| Aspect | Traditional Testing | OTEL-First Validation |
|--------|--------------------|-----------------------|
| **Exit Code** | ✅ Checked | ✅ Checked |
| **Lifecycle Events** | ❌ Not checked | ✅ Required |
| **Parent-Child Relationships** | ❌ Not checked | ✅ Validated |
| **Temporal Containment** | ❌ Not checked | ✅ Enforced |
| **Execution Order** | ❌ Not checked | ✅ Validated |
| **SDK Resources** | ❌ Not checked | ✅ Required |
| **Hermetic Isolation** | ❌ Not checked | ✅ Validated |

**Result:** Fake-green tests have nowhere to hide.

---

## Detection Layers

The red-team validation uses **7 independent detection layers**. Each layer can independently catch fake-green tests:

### Layer 1: Span Validator

**Requirement:** Specific spans must exist with required attributes and lifecycle events.

**Configuration:**
```toml
[[expect.span]]
name = "clnrm.run"
kind = "internal"

[expect.span.attrs]
all = {
  "clnrm.version" = "0.7.0",
  "test.hermetic" = "true"
}

[[expect.span]]
name = "clnrm.step:run_test"
parent = "clnrm.run"

[expect.span.events]
any = ["container.start", "container.exec", "container.stop"]
```

**Catches:**
- ❌ Echo-based tests (no spans)
- ❌ Spoofed spans without lifecycle events
- ❌ Missing required attributes

### Layer 2: Graph Validator

**Requirement:** Parent-child relationships must form valid causality graph.

**Configuration:**
```toml
[expect.graph]
must_include = [
  ["clnrm.run", "clnrm.step:run_test"]
]
acyclic = true
max_depth = 10
```

**Catches:**
- ❌ Orphaned spans (no parent)
- ❌ Cyclic dependencies
- ❌ Missing edges
- ❌ Invalid causality

### Layer 3: Count Validator

**Requirement:** Minimum number of spans and events must be present.

**Configuration:**
```toml
[expect.counts]
[expect.counts.spans_total]
gte = 2  # At least run + step

[expect.counts.events_total]
gte = 3  # At least start, exec, stop

[expect.counts.by_name]
"clnrm.run" = { eq = 1 }
"clnrm.step:run_test" = { eq = 1 }
```

**Catches:**
- ❌ Zero spans (no execution)
- ❌ Insufficient events
- ❌ Wrong span counts

### Layer 4: Window Validator

**Requirement:** Spans must be temporally contained within parent spans.

**Configuration:**
```toml
[[expect.window]]
outer = "clnrm.run"
contains = ["clnrm.step:run_test"]
```

**Catches:**
- ❌ Impossible timing relationships
- ❌ Spans outside parent window
- ❌ Time travel paradoxes

### Layer 5: Order Validator

**Requirement:** Operations must occur in correct order.

**Configuration:**
```toml
[expect.order]
must_precede = [
  ["clnrm.plugin.registry", "clnrm.step:run_test"]
]

must_follow = [
  ["clnrm.step:run_test", "clnrm.init"]
]
```

**Catches:**
- ❌ Out-of-order execution
- ❌ Initialization after use
- ❌ Temporal violations

### Layer 6: Status Validator

**Requirement:** All spans must have correct status codes.

**Configuration:**
```toml
[expect.status]
all = "OK"
none = "ERROR"
```

**Catches:**
- ❌ Spans with ERROR status
- ❌ Partial failures
- ❌ Unreported errors

### Layer 7: Hermeticity Validator

**Requirement:** Spans must have SDK resource attributes proving real OTEL SDK usage.

**Configuration:**
```toml
[expect.hermeticity]
no_external_services = true

[expect.hermeticity.resource_attrs]
must_match = {
  "service.name" = "clnrm",
  "deployment.environment" = "ci"
}

[expect.hermeticity.span_attrs]
forbid_keys = [
  "net.peer.name",
  "http.url",
  "db.connection_string"
]
```

**Catches:**
- ❌ Spoofed spans without SDK resources
- ❌ External service access
- ❌ Non-hermetic execution
- ❌ Hand-crafted fake spans

---

## Environment Variable Configuration

The red-team validation template uses environment variables for flexible configuration:

### Required Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `OTEL_SERVICE_NAME` | Service name for OTEL SDK | `clnrm` |
| `OTEL_DEPLOYMENT_ENV` | Deployment environment | `ci`, `staging`, `prod` |
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP collector endpoint | `http://localhost:4318` |
| `OTEL_EXPORTER_TYPE` | Exporter type | `otlp`, `stdout` |

### Optional Variables

| Variable | Description | Default |
|----------|-------------|---------|
| `CONTAINER_IMAGE` | Container image to test | `registry/clnrm:latest` |
| `FREEZE_CLOCK` | Freeze time for determinism | `2025-01-01T00:00:00Z` |
| `TEST_SEED` | Random seed for determinism | `42` |

### Configuration Precedence

1. **Command-line ENV flags** (highest priority)
   ```bash
   clnrm template render \
     -e OTEL_SERVICE_NAME=myservice \
     -e OTEL_DEPLOYMENT_ENV=staging
   ```

2. **System environment variables**
   ```bash
   export OTEL_SERVICE_NAME=myservice
   clnrm template render
   ```

3. **Template defaults** (lowest priority)
   ```jinja2
   {{ OTEL_SERVICE_NAME | default(value="clnrm") }}
   ```

---

## Usage Examples

### Example 1: Basic Validation

**Render template with default values:**
```bash
cd examples/case-studies
clnrm template render redteam-otlp-env.clnrm.toml.tera \
  -o test-config.clnrm.toml
```

**Run validation:**
```bash
clnrm run test-config.clnrm.toml
```

**Expected Result:** PASS (with real OTEL spans)

### Example 2: Custom Environment

**Render with custom environment:**
```bash
clnrm template render redteam-otlp-env.clnrm.toml.tera \
  -e OTEL_SERVICE_NAME=payment-service \
  -e OTEL_DEPLOYMENT_ENV=production \
  -e OTEL_EXPORTER_OTLP_ENDPOINT=http://otel-collector.prod:4318 \
  -o payment-validation.clnrm.toml
```

**Run validation:**
```bash
clnrm run payment-validation.clnrm.toml
```

### Example 3: Detect Fake-Green

**Create fake test:**
```bash
cat > fake-test.sh <<'EOF'
#!/bin/bash
echo "Test passed!"
exit 0
EOF
chmod +x fake-test.sh
```

**Run with validation:**
```bash
clnrm run redteam-otlp-env.clnrm.toml \
  --command ./fake-test.sh
```

**Expected Result:** FAIL (multiple validators catch it)
```
❌ Validation FAILED
  ❌ Span Validator: Missing required span: clnrm.run
  ❌ Graph Validator: No graph structure (zero spans)
  ❌ Count Validator: Insufficient spans: expected >=2, got 0
  ❌ Window Validator: No temporal containment (zero spans)
  ❌ Hermeticity Validator: Missing SDK resources
```

### Example 4: Detect Spoofed Spans

**Create spoofed test:**
```bash
cat > spoofed-test.sh <<'EOF'
#!/bin/bash
# Try to fake spans by echoing JSON
echo '{"name": "clnrm.run", "status": "OK"}'
exit 0
EOF
chmod +x spoofed-test.sh
```

**Run with validation:**
```bash
clnrm run redteam-otlp-env.clnrm.toml \
  --command ./spoofed-test.sh
```

**Expected Result:** FAIL (caught by hermeticity validator)
```
❌ Validation FAILED
  ❌ Hermeticity Validator: Missing SDK resource attributes
    (service.name, deployment.environment) - not from real OTEL SDK
  ❌ Span Validator: Missing lifecycle events
    (container.start, container.exec, container.stop)
```

---

## Detection Strategy

### How Each Layer Catches Fakes

| Fake Type | Layer 1 | Layer 2 | Layer 3 | Layer 4 | Layer 5 | Layer 6 | Layer 7 |
|-----------|---------|---------|---------|---------|---------|---------|---------|
| **Echo-based** | ❌ No spans | ❌ No graph | ❌ Zero count | ✅ N/A | ✅ N/A | ✅ N/A | ❌ No SDK |
| **Spoofed spans** | ❌ No events | ❌ No edges | ✅ Has spans | ❌ Wrong timing | ❌ Wrong order | ✅ Status OK | ❌ No SDK |
| **Partial exec** | ✅ Has spans | ❌ Missing edges | ❌ Low count | ❌ Orphaned | ❌ Out of order | ❌ Errors | ✅ Has SDK |
| **External calls** | ✅ Has spans | ✅ Valid graph | ✅ Valid count | ✅ Valid timing | ✅ Valid order | ✅ Status OK | ❌ Non-hermetic |

**Key Insight:** Multiple layers catch each type of fake. This provides **defense in depth**.

### Why 7 Layers?

Each layer validates a different aspect of execution:

1. **Span Validator** → "Did the operations happen?"
2. **Graph Validator** → "Did they happen in the right relationships?"
3. **Count Validator** → "Did enough operations happen?"
4. **Window Validator** → "Did they happen at the right times?"
5. **Order Validator** → "Did they happen in the right sequence?"
6. **Status Validator** → "Did they succeed?"
7. **Hermeticity Validator** → "Were they truly isolated?"

**No single layer is sufficient.** All 7 are required for comprehensive validation.

---

## Test Results

### Comparison Matrix

| Test Type | Exit Code | Span Count | Events | SDK Resources | Verdict |
|-----------|-----------|------------|--------|---------------|---------|
| **Honest (real exec)** | 0 | 2+ | 3+ | ✅ Present | ✅ PASS |
| **Fake-green (echo)** | 0 | 0 | 0 | ❌ Missing | ❌ FAIL |
| **Spoofed (fake spans)** | 0 | 1 | 0 | ❌ Missing | ❌ FAIL |
| **Partial (incomplete)** | 0 | 1 | 1 | ✅ Present | ❌ FAIL |
| **External (non-hermetic)** | 0 | 2+ | 3+ | ✅ Present | ❌ FAIL |

### Validation Report Example

**Honest Implementation:**
```json
{
  "verdict": "PASS",
  "validators": {
    "span_validator": {"passed": true, "message": "All required spans present"},
    "graph_validator": {"passed": true, "message": "Valid causality graph"},
    "count_validator": {"passed": true, "message": "2 spans, 5 events"},
    "window_validator": {"passed": true, "message": "Temporal containment valid"},
    "order_validator": {"passed": true, "message": "Execution order correct"},
    "status_validator": {"passed": true, "message": "All spans OK"},
    "hermeticity_validator": {"passed": true, "message": "SDK resources present"}
  }
}
```

**Fake-Green Implementation:**
```json
{
  "verdict": "FAIL",
  "validators": {
    "span_validator": {"passed": false, "message": "Missing required span: clnrm.run"},
    "graph_validator": {"passed": false, "message": "No graph structure"},
    "count_validator": {"passed": false, "message": "Insufficient spans: 0"},
    "window_validator": {"passed": false, "message": "No temporal containment"},
    "order_validator": {"passed": true, "message": "N/A - no spans"},
    "status_validator": {"passed": true, "message": "N/A - no spans"},
    "hermeticity_validator": {"passed": false, "message": "Missing SDK resources"}
  }
}
```

---

## Implementation Details

### File Structure

```
examples/case-studies/
├── redteam-otlp-env.clnrm.toml.tera    # Template with ENV vars
├── redteam-otlp-env.clnrm.toml         # Rendered example
└── REDTEAM_OTLP.md                     # This documentation

crates/clnrm-core/tests/
└── redteam_otlp_validation.rs          # Validation tests
```

### Template Structure

The template uses Tera syntax for variable substitution:

```toml
[vars]
svc = "{{ OTEL_SERVICE_NAME | default(value="clnrm") }}"
env = "{{ OTEL_DEPLOYMENT_ENV | default(value="ci") }}"
```

Variables are replaced during rendering:
```bash
clnrm template render template.tera -e OTEL_SERVICE_NAME=myservice
```

### Test Structure

Tests follow AAA pattern (Arrange, Act, Assert):

```rust
#[test]
fn test_fake_green_detection() -> Result<()> {
    // Arrange
    let execution = create_fake_green_execution();

    // Act
    let verdict = execution.verdict;
    let results = execution.validation_results;

    // Assert
    assert!(matches!(verdict, TestVerdict::Fail(_)));
    assert!(!results.span_validator.passed);
    assert!(!results.count_validator.passed);

    Ok(())
}
```

### Validator Interface

Each validator follows the same interface:

```rust
fn validate(spans: &[SpanData]) -> LayerResult {
    // Validation logic
    if condition_failed {
        return LayerResult::fail("Descriptive error message");
    }
    LayerResult::pass("Success message")
}
```

---

## Conclusion

Red-team OTLP validation with 7 detection layers provides **defense in depth** against fake-green tests. By requiring complete execution evidence at multiple levels, we ensure that tests cannot report success without actually executing the system under test.

**Key Takeaways:**

1. **Traditional testing is insufficient** - exit codes alone are not enough
2. **OTEL-first validation is superior** - requires proof of execution
3. **Multiple layers provide defense in depth** - no single point of failure
4. **Environment variables enable flexibility** - easy configuration per environment
5. **Determinism ensures reproducibility** - same input = same output

**Try it yourself:**
```bash
cd examples/case-studies
clnrm template render redteam-otlp-env.clnrm.toml.tera
clnrm run redteam-otlp-env.clnrm.toml
```

---

## References

- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/otel/)
- [OTLP Protocol](https://opentelemetry.io/docs/specs/otlp/)
- [Fake-Green Detection Case Study](./README.md)
- [clnrm Documentation](../../docs/)
