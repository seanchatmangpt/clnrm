# Fake-Green Detection Case Study

## Executive Summary

This case study demonstrates **clnrm's ability to detect "fake-green" tests**—tests that report success without actually executing the system under test. Traditional assertion-based testing frameworks would pass these tests because they only check exit codes. **OTEL-first validation** catches them by requiring complete execution evidence.

### Key Results

| Test Type | Traditional Testing | OTEL-First Validation |
|-----------|--------------------|-----------------------|
| **Honest** (actual execution) | ✅ PASS | ✅ PASS |
| **Fake-Green** (no execution) | ✅ PASS ❌ | ❌ FAIL ✅ |

**Conclusion**: OTEL-first validation is superior because it requires **proof of execution**, not just return codes.

---

## What is a Fake-Green Test?

A **fake-green test** is a test that:
1. Reports success (exit code 0)
2. Does NOT actually execute the system under test
3. Provides no real validation

### Example: The "echo Passed" Anti-Pattern

```bash
#!/bin/bash
# FAKE TEST - Does nothing but pretends to pass
echo "Passed"
exit 0

# No containers launched
# No services started
# No actual validation
# Traditional testing: PASS ✅ (exit code 0)
# OTEL-first testing: FAIL ❌ (no execution evidence)
```

This is a **critical failure mode** in testing infrastructure. If fake-green tests creep into your CI/CD pipeline, you get:
- ✅ Green builds without actual validation
- 🐛 Bugs making it to production
- 💥 False confidence in test coverage
- 🚨 Silent failures accumulating over time

---

## Why Traditional Testing Fails

Traditional assertion-based testing frameworks check:
- ✅ Exit code (0 = pass, non-zero = fail)
- ✅ Assertions within test code
- ✅ Expected vs actual output

**But they DON'T check**:
- ❌ Did containers actually launch?
- ❌ Were services actually started?
- ❌ Did operations happen in the correct order?
- ❌ Was execution truly hermetic?
- ❌ Were lifecycle events recorded?

**Result**: A script that just `echo "Passed" && exit 0` will PASS all checks.

---

## How OTEL-First Validation Catches Fake-Green

clnrm requires **complete execution evidence** through OpenTelemetry traces:

### 7 Detection Layers

Each layer independently catches fake-green tests:

#### 1️⃣ Lifecycle Events
**Requirement**: Container lifecycle events must be present
```toml
[expect.span.lifecycle_events]
required_events = ["container.start", "container.exec", "container.stop"]
```
**Fake-Green**: ❌ No events generated (no containers launched)

#### 2️⃣ Span Graph Structure
**Requirement**: Parent→child relationships must exist
```toml
[expect.graph.parent_child_edge]
parent_pattern = "clnrm\\.run"
child_pattern = "clnrm\\.step:.*"
```
**Fake-Green**: ❌ No spans, no edges (no execution hierarchy)

#### 3️⃣ Span Counts
**Requirement**: Minimum number of spans must be generated
```toml
[expect.counts.minimum_spans]
total_spans_min = 2  # At least run + step
```
**Fake-Green**: ❌ Zero spans (no execution occurred)

#### 4️⃣ Ordering Constraints
**Requirement**: Operations must happen in correct order
```toml
[expect.order.plugin_before_step]
before_pattern = "plugin\\.registry"
after_pattern = "clnrm\\.step:.*"
```
**Fake-Green**: ❌ No spans to order (no sequence to validate)

#### 5️⃣ Window Containment
**Requirement**: Child spans must be within parent time window
```toml
[expect.window.step_within_run]
container_pattern = "clnrm\\.run"
contained_pattern = "clnrm\\.step:.*"
```
**Fake-Green**: ❌ Empty window (no temporal containment)

#### 6️⃣ Status Validation
**Requirement**: All spans must have OK status
```toml
[expect.status.all_spans_ok]
expected_status = "OK"
max_errors = 0
```
**Fake-Green**: ❌ No status to check (no spans generated)

#### 7️⃣ Hermeticity Validation
**Requirement**: Hermetic attributes must be present
```toml
[expect.hermeticity.resource_attributes]
required_attributes = ["service.name", "deployment.environment"]
```
**Fake-Green**: ❌ No attributes (no execution context)

---

## Case Study Implementation

### File Structure

```
examples/case-studies/
├── fake-green-detection.toml      # Main test configuration
├── run-case-study.sh              # Execution script
├── scripts/
│   ├── honest-test.sh             # Honest implementation
│   └── fake-green.sh              # Fake-green implementation
└── README.md                      # This file
```

### Test Configuration

The TOML file defines **two service implementations** for the same test:

#### Honest Implementation
```toml
[service.honest]
plugin = "generic_container"
image = "alpine:latest"
args = ["sh", "/scripts/honest-test.sh"]
env = { "OTEL_TRACES_EXPORTER" = "otlp" }
wait_for_span = "clnrm.run"  # Will succeed
```

**Behavior**:
- ✅ Launches containers
- ✅ Generates OTEL spans
- ✅ Records lifecycle events
- ✅ Creates parent→child edges
- ✅ Sets hermetic attributes
- ✅ Completes within time window

**Result**: PASS (all evidence present)

#### Fake-Green Implementation
```toml
[service.fake]
plugin = "generic_container"
image = "alpine:latest"
args = ["sh", "/scripts/fake-green.sh"]
env = {}
wait_for_span = "clnrm.run"  # Will timeout
```

**Behavior**:
- ❌ No container lifecycle
- ❌ No OTEL spans
- ❌ No lifecycle events
- ❌ No span edges
- ❌ No hermetic attributes
- ❌ No execution evidence

**Result**: FAIL (all evidence missing)

---

## Running the Case Study

### Prerequisites

1. **clnrm installed**: `cargo build --release`
2. **OTEL collector running** (optional): `http://localhost:4318`
3. **Docker available**: For container execution

### Execution

```bash
cd examples/case-studies
chmod +x run-case-study.sh scripts/*.sh
./run-case-study.sh
```

### Expected Output

```
╔════════════════════════════════════════════════════════════╗
║  Fake-Green Detection Case Study                           ║
║  Demonstrating OTEL-First Validation Superiority           ║
╚════════════════════════════════════════════════════════════╝

═══════════════════════════════════════════════════════════
[TEST 1] Honest Implementation (should PASS)
═══════════════════════════════════════════════════════════
Running: clnrm run fake-green-detection.toml --service honest

✅ SUCCESS: Honest implementation PASSED (as expected)
   - All OTEL spans generated
   - Lifecycle events recorded
   - Parent→child edges established
   - Hermetic attributes present
   - All detection layers satisfied

═══════════════════════════════════════════════════════════
[TEST 2] Fake-Green Implementation (should FAIL)
═══════════════════════════════════════════════════════════
Running: clnrm run fake-green-detection.toml --service fake

✅ SUCCESS: Analyzer correctly detected fake-green!

   Expected failures detected:
   ├─ Missing lifecycle events (container.start, exec, stop)
   ├─ Missing parent→child edge (clnrm.run → step)
   ├─ Span count mismatch (0 spans, expected ≥2)
   ├─ No ordering validation possible (no spans)
   ├─ Empty time window (no containment)
   ├─ No status to validate
   └─ No hermetic attributes

   Traditional assertion-based testing would have PASSED
   because exit code was 0, but OTEL-first validation
   correctly identified missing execution evidence.

═══════════════════════════════════════════════════════════
[CASE STUDY COMPLETE]
═══════════════════════════════════════════════════════════

KEY FINDINGS:
  1. Honest implementation: PASSED (all evidence present)
  2. Fake implementation: FAILED (all evidence missing)
  3. Detection layers caught fake-green independently

CONCLUSION:
  OTEL-first validation is SUPERIOR to traditional assertion-based
  testing because it requires PROOF OF EXECUTION, not just exit codes.

  Traditional testing: ❌ Checks only return value (fake-green PASSES)
  OTEL-first testing: ✅ Requires complete execution evidence (fake-green FAILS)
```

---

## Detailed Detection Layer Analysis

### Layer 1: Lifecycle Events

**What it checks**: Container operations must generate events
```toml
[expect.span.lifecycle_events]
required_events = ["container.start", "container.exec", "container.stop"]
```

**Honest execution**:
```json
{
  "span_name": "container.exec",
  "events": [
    {"name": "container.start", "timestamp": "2025-01-15T10:00:00Z"},
    {"name": "container.exec", "timestamp": "2025-01-15T10:00:01Z"},
    {"name": "container.stop", "timestamp": "2025-01-15T10:00:05Z"}
  ]
}
```
✅ PASS: All events present

**Fake-green execution**:
```json
{
  "spans": []
}
```
❌ FAIL: No spans, no events

---

### Layer 2: Span Graph Structure

**What it checks**: Parent-child relationships must exist
```toml
[expect.graph.parent_child_edge]
parent_pattern = "clnrm\\.run"
child_pattern = "clnrm\\.step:.*"
```

**Honest execution**:
```
clnrm.run
    └── clnrm.step:run_self_test
            └── container.exec
```
✅ PASS: Edge exists (run → step)

**Fake-green execution**:
```
(empty graph)
```
❌ FAIL: No spans, no edges

---

### Layer 3: Span Counts

**What it checks**: Minimum span count threshold
```toml
[expect.counts.minimum_spans]
total_spans_min = 2
```

**Honest execution**: 5+ spans generated
✅ PASS: Count ≥ 2

**Fake-green execution**: 0 spans generated
❌ FAIL: Count < 2

---

### Layer 4: Ordering Constraints

**What it checks**: Operations happen in correct sequence
```toml
[expect.order.plugin_before_step]
before_pattern = "plugin\\.registry"
after_pattern = "clnrm\\.step:.*"
```

**Honest execution**:
```
T0: plugin.registry (start=0ms, end=10ms)
T1: clnrm.step:run_self_test (start=15ms, end=100ms)
```
✅ PASS: Registry completes before step starts

**Fake-green execution**: No spans to order
❌ FAIL: Cannot validate ordering

---

### Layer 5: Window Containment

**What it checks**: Child spans within parent time bounds
```toml
[expect.window.step_within_run]
container_pattern = "clnrm\\.run"
contained_pattern = "clnrm\\.step:.*"
```

**Honest execution**:
```
clnrm.run:       [================] (0-100ms)
  step:           [----------]       (10-90ms)
```
✅ PASS: Step contained in run window

**Fake-green execution**: No windows to check
❌ FAIL: Empty window

---

### Layer 6: Status Validation

**What it checks**: All spans have OK status
```toml
[expect.status.all_spans_ok]
expected_status = "OK"
max_errors = 0
```

**Honest execution**: All spans OK
✅ PASS: Status valid

**Fake-green execution**: No spans to validate
❌ FAIL: No status present

---

### Layer 7: Hermeticity Validation

**What it checks**: Hermetic attributes must be set
```toml
[expect.hermeticity.resource_attributes]
required_attributes = ["service.name", "deployment.environment"]
```

**Honest execution**:
```json
{
  "resource": {
    "attributes": {
      "service.name": "clnrm-self-test",
      "deployment.environment": "case-study"
    }
  }
}
```
✅ PASS: Hermetic attributes present

**Fake-green execution**: No attributes
❌ FAIL: No hermetic context

---

## Reproduction Steps

### Step 1: Run Honest Implementation

```bash
cd examples/case-studies
clnrm run fake-green-detection.toml --service honest --format json > honest-run.json
```

**Expected**: PASS with JSON output containing:
- Multiple spans (run, step, container ops)
- Lifecycle events in span data
- Parent→child relationships
- Hermetic attributes
- Proper timestamps and ordering

### Step 2: Run Fake-Green Implementation

```bash
clnrm run fake-green-detection.toml --service fake --format json > fake-run.json
```

**Expected**: FAIL with error messages indicating:
- Missing lifecycle events
- Missing parent→child edges
- Span count mismatch (0 vs ≥2)
- No ordering validation possible
- Empty time window
- No status data
- No hermetic attributes

### Step 3: Compare Outputs

```bash
clnrm diff honest-run.json fake-run.json
```

**Expected**: Diff showing all OTEL evidence present in honest run but missing in fake run.

### Step 4: Record Baseline

```bash
clnrm record fake-green-detection.toml --service honest -o baseline.json
```

**Expected**: Baseline recorded for regression testing.

---

## Integration with CI/CD

### GitHub Actions Example

```yaml
name: Fake-Green Detection
on: [push, pull_request]

jobs:
  validate-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Install clnrm
        run: cargo install --path .
      - name: Run case study
        run: |
          cd examples/case-studies
          ./run-case-study.sh
      - name: Verify fake-green detection
        run: |
          # Ensure fake implementation failed
          ! clnrm run fake-green-detection.toml --service fake
```

### Expected CI Behavior

1. ✅ Honest tests PASS (CI green)
2. ❌ Fake-green tests FAIL (CI red)
3. 🔒 No fake-green can slip through to production

---

## Real-World Impact

### Without OTEL-First Validation

❌ Developer accidentally writes wrapper script that doesn't actually run tests
❌ Script exits 0, CI goes green
❌ Bug ships to production
❌ Incident discovered by customers
❌ Post-mortem reveals tests weren't actually running

**Cost**: Downtime, customer impact, trust erosion

### With OTEL-First Validation

✅ Developer writes wrapper script
✅ clnrm detects missing OTEL spans
✅ CI fails with specific error: "Missing lifecycle events"
✅ Developer fixes script to actually run tests
✅ Bug caught before merge

**Cost**: 5 minutes to fix wrapper script

---

## Comparison with Other Frameworks

| Framework | Detection Method | Fake-Green Vulnerability |
|-----------|-----------------|-------------------------|
| **JUnit** | Assertions + exit codes | ❌ Vulnerable (checks only assertions) |
| **pytest** | Assertions + exit codes | ❌ Vulnerable (checks only assertions) |
| **RSpec** | Expectations + exit codes | ❌ Vulnerable (checks only expectations) |
| **TestContainers** | Assertions + containers | ⚠️ Partially protected (no evidence enforcement) |
| **clnrm** | OTEL-first validation | ✅ Protected (requires execution proof) |

---

## Advanced Use Cases

### Detecting Partial Execution

Fake-green isn't always binary. Sometimes tests run **partially**:

```bash
#!/bin/bash
# Runs SOME tests but not all
clnrm run tests/unit/  # Runs unit tests
# Skips integration tests!
exit 0
```

OTEL-first validation catches this:
- ❌ Missing spans for integration tests
- ❌ Span count lower than expected
- ❌ Missing service lifecycle events for integration services

### Detecting Mock Abuse

Tests that mock everything and test nothing:

```python
def test_critical_feature():
    # Mock EVERYTHING
    with mock.patch('service.execute'):
        with mock.patch('database.query'):
            with mock.patch('api.call'):
                result = run_feature()
                assert result == True  # Mocked to always return True
```

OTEL-first validation catches this:
- ❌ No actual service startup events
- ❌ No database connection spans
- ❌ No API call traces
- ❌ Execution happens in <1ms (impossible for real ops)

---

## Conclusion

**clnrm's OTEL-first validation is fundamentally superior to traditional assertion-based testing** because it:

1. ✅ Requires **proof of execution**, not just return codes
2. ✅ Validates **observable behavior**, not just internal state
3. ✅ Enforces **hermetic isolation** through span attributes
4. ✅ Detects **timing and ordering** issues automatically
5. ✅ Catches **partial execution** and mock abuse
6. ✅ Provides **complete audit trail** of test execution
7. ✅ Prevents **fake-green tests** from reaching production

**This case study proves that OTEL-first validation is not just an enhancement—it's a paradigm shift in testing methodology.**

---

## Further Reading

- [OTEL Specification](https://opentelemetry.io/docs/specs/otel/)
- [clnrm Documentation](../../README.md)
- [Hermetic Testing Best Practices](../../docs/TESTING.md)
- [TOML Configuration Reference](../../docs/TOML_REFERENCE.md)

---

## Questions?

Open an issue: https://github.com/seanchatmangpt/clnrm/issues
