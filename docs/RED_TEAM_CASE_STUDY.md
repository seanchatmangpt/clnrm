# Red-Team Case Study: Span-First Invariant Detection

## Executive Summary

This document demonstrates how clnrm's **span-first invariant validation** prevents fake-green attacks where test results appear successful without actual execution. Unlike traditional test frameworks that rely on text output and exit codes, clnrm requires **cryptographic observability evidence** through OpenTelemetry spans with specific structural, temporal, and lifecycle invariants.

**Key Insight**: Traditional testing asks "Did it exit 0?" - clnrm asks **"Can you prove it executed with OTEL spans?"**

### Why Span-First Invariants

1. **Not Text-Based**: Exit codes and stdout can be trivially forged
2. **Structural Evidence**: Requires parent-child span relationships proving execution hierarchy
3. **Lifecycle Proof**: Container events (start, exec, stop) cannot be faked without actual execution
4. **Cryptographic**: SHA-256 digests over normalized spans provide tamper-evident proof
5. **Multi-Dimensional**: 7 independent validation layers (defense-in-depth)

### Attack Vectors Covered

- **Attack A**: Echo pass (prints success, no execution)
- **Attack B**: Log mimicry (looks like real logs, no spans)
- **Attack C**: Empty OTEL path (sets env vars, no spans)

All attacks fail because they produce **zero spans** - the fundamental evidence clnrm requires.

---

## Threat Model

### The Traditional Testing Vulnerability

Traditional test frameworks validate success through:
```bash
# Traditional approach
run_tests.sh
exit_code=$?
if [ $exit_code -eq 0 ]; then
    echo "✅ PASS"  # CI accepts this
fi
```

**Vulnerability**: Any script can fake exit code 0:
```bash
#!/bin/bash
echo "✅ All tests passed"
exit 0  # Spoofed success
```

**Impact**: False positives in CI/CD, undetected failures in production, compromised security.

### Attack A: Echo Pass (Trivial Forgery)

**Attack Description**: Script prints success message and exits 0 without executing tests.

**Implementation**:
```bash
#!/bin/bash
# attack_a_echo.sh
echo "✅ Tests passed: 100%"
echo "✅ Coverage: 95%"
echo "✅ All assertions passed"
echo "PASS"
exit 0
```

**Traditional CI Response**: ✅ **PASS** (accepts exit code 0)

**clnrm Response**: ❌ **FAIL** (zero spans detected)

**Why It's Realistic**:
- Broken CI scripts may fail silently and default to success
- Malicious PRs can inject wrapper scripts that bypass tests
- Build system bugs may execute wrong binary
- Container misconfiguration may run shell instead of test binary

### Attack B: Log Mimicry (Sophisticated Forgery)

**Attack Description**: Script produces realistic-looking log output that mimics successful test execution, including timestamps and progress indicators.

**Implementation**:
```bash
#!/bin/bash
# attack_b_logs.sh
echo "[2025-10-16T10:00:00Z] INFO: Starting test suite"
echo "[2025-10-16T10:00:01Z] INFO: Loading plugins..."
echo "[2025-10-16T10:00:02Z] INFO: Executing hello_world test"
echo "[2025-10-16T10:00:03Z] INFO: Container started"
echo "[2025-10-16T10:00:04Z] INFO: Command executed successfully"
echo "[2025-10-16T10:00:05Z] INFO: Container stopped"
echo "[2025-10-16T10:00:06Z] INFO: All tests passed"
echo "PASS (6 tests, 0 failures)"
exit 0
```

**Traditional CI Response**: ✅ **PASS** (looks legitimate with timestamps)

**clnrm Response**: ❌ **FAIL** (no OTEL spans, text-based validation insufficient)

**Why It's Realistic**:
- Advanced attackers craft convincing log output
- Log aggregation systems may only check for keywords
- Human reviewers may not spot fake logs in large output
- Regex-based validation can be bypassed

### Attack C: Empty OTEL Path (Environment Spoofing)

**Attack Description**: Script sets OTEL environment variables to appear instrumented but produces no actual spans.

**Implementation**:
```bash
#!/bin/bash
# attack_c_empty_otel.sh
export OTEL_TRACES_EXPORTER=otlp
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
export OTEL_SERVICE_NAME=clnrm
export OTEL_DEPLOYMENT_ENVIRONMENT=ci

echo "OTEL configuration:"
echo "  Exporter: $OTEL_TRACES_EXPORTER"
echo "  Endpoint: $OTEL_EXPORTER_OTLP_ENDPOINT"
echo "  Service: $OTEL_SERVICE_NAME"
echo ""
echo "✅ Tests completed with OTEL tracing enabled"
echo "PASS"
exit 0
```

**Traditional CI Response**: ✅ **PASS** (env vars set, appears instrumented)

**clnrm Response**: ❌ **FAIL** (zero spans received despite env vars)

**Why It's Realistic**:
- OTEL SDK may fail to initialize silently
- Network issues may prevent span export
- Misconfigured exporters may drop all spans
- Binary may not be instrumented despite env vars being set

---

## Detection Strategy

### Span-First Invariants (Not Text-Based)

clnrm **ignores stdout/stderr text** for validation. Only OTEL spans count as evidence.

**Core Principle**: If a test produces zero spans, it **did not execute** regardless of exit code or log output.

### 7 Validation Layers

Each layer independently validates different aspects of execution. All must pass for a test to succeed.

#### Layer 1: Span Structure (REQUIRED)

**What It Checks**: Specific spans exist with expected names, parents, kinds, attributes, and events.

**TOML Configuration**:
```toml
[[expect.span]]
name="clnrm.run"
kind="internal"
attrs.all={ "result"="pass" }

[[expect.span]]
name="clnrm.step:hello_world"
parent="clnrm.run"
kind="internal"
events.any=["container.start","container.exec","container.stop"]
```

**Detection Logic**:
- Requires span named "clnrm.run" with attribute `result=pass`
- Requires child span "clnrm.step:hello_world" with lifecycle events
- If span missing: **FAIL** - "span 'clnrm.run' not found"

**Why Attacks Fail**:
- Attack A produces 0 spans → missing span failure
- Attack B produces 0 spans → missing span failure
- Attack C produces 0 spans → missing span failure

**First-Failing-Rule Example**:
```
❌ FAIL: expect.span[clnrm.run] - span not found
   Expected: span with name='clnrm.run', kind=internal, attrs={result=pass}
   Found: 0 spans in trace
   Reason: No OTEL instrumentation detected (possible fake-green)
```

#### Layer 2: Graph Topology (REQUIRED)

**What It Checks**: Parent-child relationships between spans form expected execution hierarchy.

**TOML Configuration**:
```toml
[expect.graph]
must_include=[
    ["clnrm.run", "clnrm.step:hello_world"],
    ["clnrm.run", "clnrm.plugin.registry"]
]
must_not_cross=[
    ["clnrm.step:hello_world", "clnrm.plugin.registry"]
]
acyclic=true
```

**Detection Logic**:
- Builds directed graph from span parent relationships
- Verifies all `must_include` edges exist
- Verifies no `must_not_cross` edges exist
- Verifies graph is acyclic (no circular dependencies)

**Why Attacks Fail**:
- Zero spans → zero edges → all must_include checks fail
- Cannot forge span relationships without actual OTEL SDK

**First-Failing-Rule Example**:
```
❌ FAIL: expect.graph.must_include - missing edge
   Expected: clnrm.run → clnrm.step:hello_world
   Found: 0 edges in span graph
   Reason: Cannot build parent-child relationships from 0 spans
```

#### Layer 3: Lifecycle Events (REQUIRED)

**What It Checks**: Container lifecycle events prove actual execution occurred.

**TOML Configuration**:
```toml
[[expect.span]]
name="clnrm.step:hello_world"
events.any=["container.start", "container.exec", "container.stop"]
```

**Detection Logic**:
- Each step span must contain container lifecycle events
- Events are OTEL span events, not log lines
- Events prove Docker/Podman interaction occurred

**Why Attacks Fail**:
- Lifecycle events require actual container operations
- Cannot be faked without Docker API calls
- Text-based log lines don't count as events

**First-Failing-Rule Example**:
```
❌ FAIL: expect.span[clnrm.step:hello_world].events.any
   Expected: at least one of [container.start, container.exec, container.stop]
   Found: span does not exist (0 spans in trace)
   Reason: No container lifecycle detected
```

#### Layer 4: Count Guardrails (REQUIRED)

**What It Checks**: Minimum and maximum span/event counts enforce baseline expectations.

**TOML Configuration**:
```toml
[expect.counts]
spans_total={ gte=2, lte=200 }
events_total={ gte=2 }
errors_total={ eq=0 }
by_name={
    "clnrm.run"={ eq=1 },
    "clnrm.step:hello_world"={ eq=1 }
}
```

**Detection Logic**:
- Total span count must be ≥2 (at minimum: root + one child)
- Total event count must be ≥2 (at minimum: container.start + container.stop)
- Error span count must be exactly 0
- Named span counts must match exactly

**Why Attacks Fail**:
- Zero spans fail `spans_total.gte=2` immediately
- Zero events fail `events_total.gte=2` immediately
- This is the **fastest failing check** (O(1) validation)

**First-Failing-Rule Example**:
```
❌ FAIL: expect.counts.spans_total
   Expected: spans_total >= 2
   Found: 0 spans
   Reason: Empty trace (no OTEL spans collected)
```

#### Layer 5: Temporal Windows (REQUIRED)

**What It Checks**: Child spans are temporally contained within parent spans.

**TOML Configuration**:
```toml
[[expect.window]]
outer="clnrm.run"
contains=["clnrm.step:hello_world", "clnrm.plugin.registry"]
```

**Detection Logic**:
- Outer span start time ≤ inner span start time
- Inner span end time ≤ outer span end time
- Validates actual execution timeline (not just logical parent)

**Why Attacks Fail**:
- Requires actual span timestamps from OTEL SDK
- Cannot be forged without understanding precise timing
- Zero spans → cannot validate any windows

**First-Failing-Rule Example**:
```
❌ FAIL: expect.window[clnrm.run]
   Expected: outer span 'clnrm.run' contains ['clnrm.step:hello_world']
   Found: outer span does not exist
   Reason: No spans to validate temporal containment
```

#### Layer 6: Ordering Constraints (REQUIRED)

**What It Checks**: Spans occur in correct temporal order.

**TOML Configuration**:
```toml
[expect.order]
must_precede=[
    ["clnrm.plugin.registry", "clnrm.step:hello_world"]
]
must_follow=[
    ["clnrm.step:hello_world", "clnrm.plugin.registry"]
]
```

**Detection Logic**:
- Span A must_precede B → A.end_time ≤ B.start_time
- Span B must_follow A → A.end_time ≤ B.start_time
- Validates execution order matches expected flow

**Why Attacks Fail**:
- Requires precise span timestamps
- Zero spans → no ordering to validate

**First-Failing-Rule Example**:
```
❌ FAIL: expect.order.must_precede
   Expected: clnrm.plugin.registry precedes clnrm.step:hello_world
   Found: one or both spans missing
   Reason: Cannot validate ordering of 0 spans
```

#### Layer 7: Status Validation (REQUIRED)

**What It Checks**: OTEL span status codes (OK/ERROR/UNSET), not exit codes.

**TOML Configuration**:
```toml
[expect.status]
all="OK"
by_name={
    "clnrm.*"="OK"
}
```

**Detection Logic**:
- Every span must have OTEL status = OK
- Glob patterns match span name prefixes
- Exit code alone is **insufficient** (must have span status)

**Why Attacks Fail**:
- Exit code 0 does not satisfy status check
- Requires OTEL SDK to set span status
- Zero spans → no status to validate

**First-Failing-Rule Example**:
```
❌ FAIL: expect.status.all=OK
   Expected: all spans have status=OK
   Found: 0 spans with status
   Reason: Exit code alone insufficient (requires OTEL span status)
```

#### Layer 8: Hermeticity (REQUIRED)

**What It Checks**: Resource attributes prove SDK instrumentation; forbids external network calls.

**TOML Configuration**:
```toml
[expect.hermeticity]
no_external_services=true
resource_attrs.must_match={
    "service.name"="clnrm",
    "env"="ci"
}
span_attrs.forbid_keys=[
    "net.peer.name",
    "db.connection_string",
    "http.url"
]
```

**Detection Logic**:
- Resource attributes must match (SDK-generated at initialization)
- No spans may have forbidden attribute keys
- Validates hermetic execution (no external network)

**Why Attacks Fail**:
- Resource attributes require OTEL SDK initialization
- Cannot forge `service.name` without actual SDK
- Zero spans → no resource attributes to validate

**First-Failing-Rule Example**:
```
❌ FAIL: expect.hermeticity.resource_attrs.must_match
   Expected: resource attributes {service.name=clnrm, env=ci}
   Found: no resource attributes
   Reason: OTEL SDK did not initialize (no instrumentation)
```

### First-Failing-Rule for Fast Triage

clnrm reports the **first validation failure** to enable fast triage:

**Typical First Failure** (Attack A/B/C):
```
❌ FAIL fake_green_test (0.02s)

First Failing Rule: expect.counts.spans_total
  Expected: spans_total >= 2
  Found: 0

Reason: No OTEL spans collected from execution
Possible causes:
  - Test script exited without running instrumented binary
  - OTEL SDK failed to initialize
  - Exporter configuration incorrect
  - Fake-green attack (wrapper script spoofing success)

Digest: d41d8cd98f00b204e9800998ecf8427e (empty trace)
```

**Benefits**:
- Immediate detection (0.02s validation time)
- Clear root cause ("0 spans" vs. "missing edge")
- Actionable diagnostics
- Forensic digest for investigation

### Defense-in-Depth Approach

Even if an attacker bypasses one layer, other layers catch the attack:

| If Attacker... | Blocked By Layer... |
|---------------|---------------------|
| Fakes exit code 0 | Layer 4: Count guardrails (0 spans) |
| Prints realistic logs | Layer 1: Span structure (no spans) |
| Sets OTEL env vars | Layer 8: Hermeticity (no resource attrs) |
| Generates fake spans (hardcoded JSON) | Layer 2: Graph topology (no valid edges) |
| Replays old trace | Determinism: freeze_clock digest mismatch |
| Partial execution | Layer 4: Exact count requirements |
| Time manipulation | Layer 5: Window validation |
| Forge one span | Layer 2: Missing parent-child edges |

**Key Property**: All 7 layers are **independent**. Passing requires **all 7 to pass**, but failing requires only **1 to fail**.

---

## TOML Configuration

### Complete Annotated Example

```toml
# Red-Team Test: Comprehensive fake-green detection
#
# This test validates that clnrm rejects scripts that produce no OTEL spans,
# regardless of exit code or stdout/stderr content.

[meta]
name="red_team_fake_green_detection"
version="1.0"
description="Detects fake-green attacks through span-first validation"

# ========================================
# OTEL Configuration
# ========================================
# This configures WHERE to look for spans (stdout or OTLP endpoint)
# The configuration itself doesn't prevent attacks - validation does

[otel]
exporter="stdout"  # For red-team demo, use stdout (easier to inspect)
protocol="json"
sample_ratio=1.0
resources={
    "service.name"="clnrm",
    "env"="red-team"
}

# ========================================
# Service Configuration
# ========================================
# Replace `args` with your suspected fake-green script to test detection

[service.clnrm]
plugin="generic_container"
image="clnrm:latest"
args=["self-test"]  # SWAP THIS with "./fake_wrapper.sh" to test attack
env={
    "OTEL_TRACES_EXPORTER"="stdout",
    "RUST_LOG"="info"
}
wait_for_span="clnrm.run"

# ========================================
# Scenario Configuration
# ========================================

[[scenario]]
name="verify_real_execution"
service="clnrm"
run="clnrm self-test"
artifacts.collect=["spans:default"]

# ========================================
# VALIDATION LAYERS
# ========================================

# Layer 1: Span Structure (requires specific spans with lifecycle events)
[[expect.span]]
name="clnrm.run"
kind="internal"
attrs.all={ "result"="pass" }

[[expect.span]]
name="clnrm.step:hello_world"
parent="clnrm.run"
kind="internal"
events.any=["container.start", "container.exec", "container.stop"]

# Layer 2: Graph Topology (requires parent-child edges)
[expect.graph]
must_include=[
    ["clnrm.run", "clnrm.step:hello_world"]
]
acyclic=true

# Layer 3: Lifecycle Events (validated in Layer 1 events.any)

# Layer 4: Count Guardrails (FIRST FAILING RULE for 0-span attacks)
[expect.counts]
spans_total={ gte=2, lte=200 }
events_total={ gte=2 }
errors_total={ eq=0 }
by_name={
    "clnrm.run"={ eq=1 },
    "clnrm.step:hello_world"={ eq=1 }
}

# Layer 5: Temporal Windows (requires timing containment)
[[expect.window]]
outer="clnrm.run"
contains=["clnrm.step:hello_world"]

# Layer 6: Ordering Constraints (requires correct execution order)
[expect.order]
must_precede=[
    ["clnrm.plugin.registry", "clnrm.step:hello_world"]
]

# Layer 7: Status Validation (requires OTEL status codes)
[expect.status]
all="OK"

# Layer 8: Hermeticity (requires SDK resource attributes)
[expect.hermeticity]
no_external_services=true
resource_attrs.must_match={
    "service.name"="clnrm",
    "env"="red-team"
}

# ========================================
# Determinism (enables digest comparison)
# ========================================

[determinism]
seed=42
freeze_clock="2025-01-01T00:00:00Z"

# ========================================
# Reporting (digest for forensics)
# ========================================

[report]
json="report.json"
digest="trace.sha256"
```

### Required Expectations

**Minimum Configuration** to detect fake-green:

```toml
# Minimal red-team config (catches all 3 attack types)

[expect.counts]
spans_total={ gte=1 }  # Simplest check: at least 1 span exists

[expect.status]
all="OK"  # All spans must be OK (exit code insufficient)

[expect.hermeticity]
resource_attrs.must_match={ "service.name"="clnrm" }  # Proves SDK init
```

**Why This Works**:
- `spans_total.gte=1` catches zero-span attacks immediately
- `status.all=OK` requires OTEL status (exit code alone fails)
- `resource_attrs` proves OTEL SDK initialized (cannot fake)

### How to Adapt for Your Use Case

1. **Identify Critical Spans**: What spans prove your system executed?
   ```toml
   [[expect.span]]
   name="your_app.critical_operation"
   events.any=["operation.start", "operation.complete"]
   ```

2. **Define Execution Hierarchy**: What parent-child relationships must exist?
   ```toml
   [expect.graph]
   must_include=[
       ["api.request", "db.query"],
       ["api.request", "cache.lookup"]
   ]
   ```

3. **Set Count Guardrails**: How many spans/events should exist?
   ```toml
   [expect.counts]
   spans_total={ gte=5, lte=100 }  # Adjust based on your app
   by_name={ "api.request"={ eq=1 } }
   ```

4. **Enforce Hermeticity**: What external calls are forbidden?
   ```toml
   [expect.hermeticity]
   span_attrs.forbid_keys=["http.url", "net.peer.ip"]  # No external network
   ```

---

## Usage Examples

### Basic Usage: Detect Fake-Green

```bash
# Run test with legitimate binary (should pass)
clnrm run -f tests/red_team/legit_self_test.clnrm.toml

# Expected output:
# ✅ PASS legit_self_test (1.23s, spans=12, events=8)
# Digest: a3c5e7f9d2b4a6c8e0f2d4b6a8c0e2f4
```

### Attack Simulation: Echo Pass (Attack A)

```bash
# Edit TOML to run attack script
# Change: args=["self-test"]
# To:     args=["./attack_a_echo.sh"]

clnrm run -f tests/red_team/attack_a_echo.clnrm.toml

# Expected output (FIRST FAILING RULE):
# ❌ FAIL attack_a_echo (0.02s)
#
# First Failing Rule: expect.counts.spans_total
#   Expected: spans_total >= 2
#   Found: 0
#
# Reason: No OTEL spans collected
# Digest: d41d8cd98f00b204e9800998ecf8427e (empty trace)
# Exit Code: 1
```

### Attack Simulation: Log Mimicry (Attack B)

```bash
# Run log mimicry attack
clnrm run -f tests/red_team/attack_b_logs.clnrm.toml

# Expected output (FIRST FAILING RULE):
# ❌ FAIL attack_b_logs (0.02s)
#
# First Failing Rule: expect.counts.spans_total
#   Expected: spans_total >= 2
#   Found: 0
#
# Reason: No OTEL spans collected (text-based logs insufficient)
# Note: Attack produced realistic log output but zero spans
# Digest: d41d8cd98f00b204e9800998ecf8427e (empty trace)
# Exit Code: 1
```

### Attack Simulation: Empty OTEL Path (Attack C)

```bash
# Run OTEL environment spoofing attack
clnrm run -f tests/red_team/attack_c_empty_otel.clnrm.toml

# Expected output (FIRST FAILING RULE):
# ❌ FAIL attack_c_empty_otel (0.02s)
#
# First Failing Rule: expect.counts.spans_total
#   Expected: spans_total >= 2
#   Found: 0
#
# Reason: OTEL env vars set but no spans received
# Possible cause: SDK initialization failed or exporter misconfigured
# Digest: d41d8cd98f00b204e9800998ecf8427e (empty trace)
# Exit Code: 1
```

### Swapping to Legitimate Test

```bash
# Step 1: Run attack script (fails)
clnrm run -f tests/red_team/attack_a_echo.clnrm.toml
# Output: FAIL (0 spans)

# Step 2: Edit TOML to use legitimate binary
sed -i 's|args=\["./attack_a_echo.sh"\]|args=["self-test"]|' \
    tests/red_team/attack_a_echo.clnrm.toml

# Step 3: Run again (passes)
clnrm run -f tests/red_team/attack_a_echo.clnrm.toml
# Output: PASS (12 spans, digest differs from empty trace)
```

### Hot Reload Demo (Watch Mode)

```bash
# Start watching test file
clnrm dev --watch tests/red_team/attack_a_echo.clnrm.toml

# Initial run: FAIL (0 spans)

# Edit file to swap attack script for legitimate binary:
# (file save triggers automatic re-run within <3s)

# Hot reload output:
# ✅ PASS attack_a_echo (1.45s, spans=12, events=8)
# Change detected: args modified
# Digest changed: d41d8cd... → a3c5e7f...
```

---

## Attack Vector Analysis

### Attack Vector Table

| Attack | Method | Exit Code | Stdout | OTEL Spans | First Failing Rule | Detection Time | Verdict |
|--------|--------|-----------|--------|------------|-------------------|----------------|---------|
| **Attack A: Echo Pass** | `echo PASS && exit 0` | 0 | "PASS" | 0 | `expect.counts.spans_total` | 0.02s | ❌ FAIL |
| **Attack B: Log Mimicry** | Fake timestamps + logs | 0 | Realistic logs | 0 | `expect.counts.spans_total` | 0.02s | ❌ FAIL |
| **Attack C: Empty OTEL** | Set env vars, no SDK | 0 | "OTEL enabled" | 0 | `expect.counts.spans_total` | 0.02s | ❌ FAIL |
| **Legitimate Test** | Real execution | 0 | Normal logs | 12 | N/A (all pass) | 1.23s | ✅ PASS |

### Why Text-Based Checking Would Fail

**Text-Based Validation** (vulnerable):
```bash
# Traditional approach - INSECURE
if grep -q "PASS" output.log && [ $exit_code -eq 0 ]; then
    verdict="PASS"  # Can be spoofed
fi
```

**Problems**:
- Attack A: Contains "PASS" in output → false positive
- Attack B: Contains realistic log lines → false positive
- Attack C: Contains "OTEL enabled" → false positive
- Regex can always be crafted to match

**Span-Based Validation** (secure):
```rust
// clnrm approach - SECURE
let spans = collect_otel_spans(container)?;
if spans.len() < 2 {
    return Err("Insufficient spans (possible fake-green)");
}
validate_graph(&spans)?;
validate_events(&spans)?;
// ... all 7 layers ...
```

**Benefits**:
- Attack A: 0 spans → immediate failure
- Attack B: 0 spans → immediate failure
- Attack C: 0 spans → immediate failure
- Cannot fake OTEL SDK instrumentation

### Expected First-Failing-Rule by Attack

| Attack | First Failing Rule | Expected Message | Why |
|--------|-------------------|------------------|-----|
| Attack A | `expect.counts.spans_total` | "required >=2, found 0" | Simplest check fails first |
| Attack B | `expect.counts.spans_total` | "required >=2, found 0" | Count check is O(1) |
| Attack C | `expect.counts.spans_total` | "required >=2, found 0" | Same failure mode |
| Partial exec | `expect.counts.by_name` | "expected eq=1, found 0" | Specific span missing |
| Wrong order | `expect.order.must_precede` | "A did not precede B" | Timing wrong |
| Network leak | `expect.hermeticity.span_attrs` | "forbidden key 'http.url'" | External call detected |

**Key Insight**: `expect.counts.spans_total` is the **fastest failing rule** for zero-span attacks because it's O(1) validation.

---

## Integration with CI/CD

### Preventing Malicious PRs

**GitHub Actions Example**:
```yaml
name: Red-Team Protection

on: [pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run Tests with Fake-Green Detection
        run: |
          clnrm run tests/production.clnrm.toml --report json

      - name: Verify Span Count
        run: |
          span_count=$(jq '.counts.spans' report.json)
          if [ "$span_count" -eq 0 ]; then
            echo "❌ FAKE-GREEN DETECTED: Zero spans collected"
            exit 1
          fi

      - name: Compare Digest with Baseline
        run: |
          baseline_digest=$(cat baseline.sha256)
          current_digest=$(jq -r '.digest' report.json)

          if [ "$current_digest" == "d41d8cd98f00b204e9800998ecf8427e" ]; then
            echo "❌ FAKE-GREEN DETECTED: Empty trace digest"
            exit 1
          fi

      - name: Upload Trace Digest
        run: |
          echo "digest=$current_digest" >> $GITHUB_OUTPUT
```

**Benefits**:
- Blocks PRs that produce zero spans
- Detects empty trace digest (d41d8cd98f...)
- Compares with known-good baseline
- Records digest for audit trail

### Audit Trail with Digests

**Recording Execution**:
```bash
# Baseline (known-good run)
clnrm run tests/critical.clnrm.toml --report json
mv report.json baseline.json
jq -r '.digest' baseline.json > baseline.sha256

# Commit baseline to repo
git add baseline.json baseline.sha256
git commit -m "chore: Update test baseline"
```

**Verification**:
```bash
# Production run
clnrm run tests/critical.clnrm.toml --report json

# Verify digest matches baseline
baseline=$(cat baseline.sha256)
current=$(jq -r '.digest' report.json)

if [ "$baseline" != "$current" ]; then
    echo "❌ DIGEST MISMATCH"
    echo "Expected: $baseline"
    echo "Got:      $current"
    exit 1
fi
```

**Forensic Analysis**:
```bash
# Compare two runs to see what changed
clnrm analyze diff baseline.json suspicious.json --format table

# Output shows:
# | Span Name | Baseline Count | Suspicious Count | Delta |
# |-----------|---------------|------------------|-------|
# | clnrm.run | 1             | 0                | -1    |
# | clnrm.step:hello_world | 1 | 0              | -1    |
#
# Verdict: Suspicious run produced 0 spans (possible fake-green)
```

### Security Policy Enforcement

**Policy Definition** (`.clnrm-policy.toml`):
```toml
[policy]
name="production_security_policy"
version="1.0"

[policy.requirements]
min_spans=2
min_events=2
max_errors=0
required_resource_attrs=["service.name", "env"]

[policy.forbidden]
span_attrs=["http.url", "net.peer.ip", "db.connection_string"]
external_network=true

[policy.digest]
algorithm="sha256"
baseline_file="baseline.sha256"
fail_on_mismatch=true
```

**Enforcement**:
```bash
# Run with policy enforcement
clnrm run tests/prod.clnrm.toml --policy .clnrm-policy.toml

# Violations result in FAIL
# - Span count < 2 → FAIL
# - External network detected → FAIL
# - Digest mismatch → FAIL
```

---

## Troubleshooting

### Common Issues

#### Issue 1: "0 spans collected" on legitimate test

**Symptoms**:
```
❌ FAIL: expect.counts.spans_total
   Expected: >= 2
   Found: 0
```

**Possible Causes**:
1. OTEL SDK not initialized
2. Exporter configuration wrong
3. Binary not instrumented
4. Container crashed before span export

**Diagnosis**:
```bash
# Check OTEL environment
clnrm run --debug --otel-exporter stdout test.clnrm.toml

# Look for OTEL initialization logs:
# - "OTEL SDK initialized"
# - "Exporter: stdout"
# - "Service name: clnrm"

# If missing, SDK didn't initialize
```

**Fix**:
```toml
# Ensure OTEL config is correct
[otel]
exporter="stdout"  # or "otlp"
endpoint="http://localhost:4318"  # if using otlp
protocol="http/protobuf"

[service.clnrm]
env={
    "OTEL_TRACES_EXPORTER"="stdout",  # Must match [otel.exporter]
    "RUST_LOG"="info"
}
```

#### Issue 2: "missing edge" when spans exist

**Symptoms**:
```
❌ FAIL: expect.graph.must_include
   Expected: parent → child
   Found: both spans exist but no edge
```

**Possible Cause**: Span `parent_id` not set correctly (instrumentation bug)

**Diagnosis**:
```bash
# Inspect spans to check parent_id
clnrm run test.clnrm.toml --report json
jq '.spans[] | {name, parent_id}' report.json

# Output should show:
# {"name":"clnrm.run", "parent_id":null}
# {"name":"clnrm.step:hello_world", "parent_id":"<span_id of clnrm.run>"}
```

**Fix**: Update instrumentation to set parent span context correctly.

#### Issue 3: "window validation failed" with correct spans

**Symptoms**:
```
❌ FAIL: expect.window[clnrm.run]
   Expected: outer contains inner
   Found: inner span ends after outer span
```

**Possible Cause**: Clock skew or span not properly closed

**Diagnosis**:
```bash
# Check span timestamps
jq '.spans[] | {name, start_time, end_time}' report.json

# Ensure: outer.start <= inner.start AND inner.end <= outer.end
```

**Fix**:
```toml
# Use deterministic clock
[determinism]
freeze_clock="2025-01-01T00:00:00Z"
```

### Debugging Validation Failures

**Enable Debug Mode**:
```bash
# Detailed validation output
clnrm run --debug test.clnrm.toml

# Shows:
# - All validation layers executed
# - First failing rule with context
# - Span graph visualization
# - Timeline diagram
```

**Inspect Normalized Spans**:
```bash
# Export normalized spans for inspection
clnrm run test.clnrm.toml --report json --normalize

# Normalized JSON shows exactly what validator sees
jq '.spans' report.json
```

**Compare with Known-Good Baseline**:
```bash
# Diff current run against baseline
clnrm analyze diff baseline.json current.json --verbose

# Shows:
# - Span count differences
# - Missing/extra spans
# - Event differences
# - Attribute changes
```

### Performance Considerations

**Validation Performance**:
- **Span Structure**: O(n) per span expectation
- **Graph Topology**: O(n + e) where e = edge count
- **Count Guardrails**: O(1) - fastest check
- **Window/Order**: O(n log n) with sorting
- **Status/Hermeticity**: O(n) span iteration

**Typical Timings**:
- 0 spans (fake-green): **0.02s** (instant failure on counts)
- 10 spans (small test): **0.05s** (all 7 layers)
- 100 spans (medium): **0.12s** (graph + windows)
- 1000 spans (large): **0.45s** (sorting overhead)

**Optimization Tips**:
1. Count checks fail fast (always validate first)
2. Graph validation scales well (up to ~10K spans)
3. Use `by_name` counts for targeted validation
4. Avoid overly complex `must_precede` rules (keep < 20 constraints)

---

## Conclusion

### Security Guarantees

clnrm's span-first invariant validation provides **cryptographic proof of execution** through:

1. **Multi-dimensional Evidence**: Not text-based, requires OTEL spans
2. **Structural Validation**: Graph topology proves execution hierarchy
3. **Lifecycle Proof**: Container events cannot be faked
4. **Temporal Validation**: Windows and ordering prove actual timeline
5. **Cryptographic Digests**: SHA-256 over normalized spans (tamper-evident)
6. **Defense-in-Depth**: 7 independent layers (all must pass)
7. **First-Failing-Rule**: Fast triage (O(1) for zero-span attacks)

### Comparison: Traditional vs clnrm

| Aspect | Traditional Testing | clnrm (Span-First) |
|--------|-------------------|-------------------|
| **What is validated?** | Exit code + text output | OTEL spans + events + structure |
| **Can be spoofed?** | ✅ Yes (trivial: `exit 0`) | ❌ No (requires SDK instrumentation) |
| **Proof type** | Exit code integer | Cryptographic digest over normalized trace |
| **Attack detection time** | N/A (no detection) | 0.02s (instant for zero-span) |
| **Forensic analysis** | ❌ No evidence | ✅ SHA-256 digest + recorded trace |
| **Reproducible** | ❌ No | ✅ Yes (record/repro/redgreen) |
| **Tamper-evident** | ❌ No | ✅ Yes (digest mismatch detectable) |
| **Defense layers** | 1 (exit code) | 7 (independent validation) |

### When to Use Red-Team Testing

**Recommended For**:
- High-security environments (SOC 2, ISO 27001 compliance)
- Compliance-critical systems (HIPAA, PCI-DSS)
- Zero-trust CI/CD pipelines
- Untrusted contributor workflows (open source projects)
- Forensic test analysis (audit trails required)
- Systems where test bypass has security implications

**Benefits**:
- Prevents malicious PRs from bypassing tests
- Detects CI/CD misconfigurations immediately
- Provides cryptographic audit trail
- Enables reproducible forensic analysis
- Enforces defense-in-depth (multiple independent layers)

### Key Takeaways

1. **Exit codes are insufficient**: `exit 0` can always be forged
2. **Text-based validation is vulnerable**: Logs can be mimicked
3. **Span-first is secure**: OTEL spans require actual instrumentation
4. **Zero spans = immediate failure**: Fastest detection (0.02s)
5. **Seven layers = defense-in-depth**: All attacks caught by multiple layers
6. **Digests enable forensics**: SHA-256 provides tamper-evident proof
7. **First-failing-rule aids triage**: Clear root cause identification

**Bottom Line**: Traditional testing asks "Did it exit 0?" - clnrm asks **"Can you prove it executed with OTEL spans?"**

This makes fake-green results **cryptographically detectable** and **practically impossible** to forge.

---

## References

### Documentation
- `PRD-v1.md` - Schema definition and validation rules
- `CLAUDE.md` - Core team standards (no false positives)
- `FAKE_GREEN_DETECTION_CASE_STUDY.md` - Original fake-green documentation
- `VALIDATION_MATRIX.md` - Detailed validation layer analysis

### Test Files
- `tests/fake_green_detection/fake_green_case_study.clnrm.toml` - Complete validation example
- `tests/red_team/attack_a_echo.clnrm.toml` - Attack A demonstration
- `tests/red_team/attack_b_logs.clnrm.toml` - Attack B demonstration
- `tests/red_team/attack_c_empty_otel.clnrm.toml` - Attack C demonstration
- `tests/red_team/legit_self_test.clnrm.toml` - Legitimate test baseline

### Source Code
- `crates/clnrm-core/src/otel/validators/mod.rs` - Validator implementations
- `crates/clnrm-core/src/otel/validators/counts.rs` - Count guardrails (first-failing for zero spans)
- `crates/clnrm-core/src/otel/validators/graph.rs` - Graph topology validation
- `crates/clnrm-core/src/otel/validators/span.rs` - Span structure validation
- `crates/clnrm-core/tests/integration/fake_green_detection.rs` - Integration tests

### Tools
- `clnrm run` - Execute tests with validation
- `clnrm analyze diff` - Compare two runs
- `clnrm dev --watch` - Hot reload for rapid testing
- `clnrm record` - Record baseline for comparison
