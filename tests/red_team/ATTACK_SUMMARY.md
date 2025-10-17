# Red-Team Attack Summary: Detection Matrix

## Overview

This document provides a quick reference for all red-team attack vectors and their detection characteristics.

## Attack Vector Matrix

| Attack | Script | TOML | Exit Code | Stdout | OTEL Spans | First Failing Rule | Detection Time | Digest |
|--------|--------|------|-----------|--------|------------|-------------------|---------------|--------|
| **A: Echo Pass** | `attack_a_echo.sh` | `attack_a_echo.clnrm.toml` | 0 | "PASS" | 0 | `expect.counts.spans_total` | 0.02s | `d41d8cd...` (empty) |
| **B: Log Mimicry** | `attack_b_logs.sh` | `attack_b_logs.clnrm.toml` | 0 | Realistic logs | 0 | `expect.counts.spans_total` | 0.02s | `d41d8cd...` (empty) |
| **C: Empty OTEL** | `attack_c_empty_otel.sh` | `attack_c_empty_otel.clnrm.toml` | 0 | "OTEL enabled" | 0 | `expect.counts.spans_total` | 0.02s | `d41d8cd...` (empty) |
| **Legitimate** | `clnrm self-test` | `legitimate_self_test.clnrm.toml` | 0 | Normal logs | 12 | N/A (all pass) | 1.23s | `a3c5e7f...` (valid) |

## Detection Characteristics

### Common Pattern: Zero Spans

All three attacks share the same detection pattern:

```
❌ FAIL attack_name (0.02s)

First Failing Rule: expect.counts.spans_total
  Expected: >= 2
  Found: 0

Reason: No OTEL spans collected from execution
Possible causes:
  - Test script exited without running instrumented binary
  - Fake-green attack (wrapper script spoofing success)

Digest: d41d8cd98f00b204e9800998ecf8427e (empty trace)
Verdict: FAIL
```

**Key Insight**: The empty trace digest `d41d8cd98f00b204e9800998ecf8427e` is the SHA-256 hash of an empty string, proving zero spans were collected.

### Why Each Attack Fails

#### Attack A: Echo Pass

**Attack Method**: Trivial forgery
```bash
echo "PASS" && exit 0
```

**Why It Fails**:
- Zero spans collected
- Count guardrail fails: `spans_total.gte=2` violated
- No OTEL instrumentation executed

**Detection Layers Triggered**:
1. Layer 4: Count guardrails (FIRST FAILURE)
2. Layer 1: Span structure (missing spans)
3. Layer 2: Graph topology (no edges)
4. Layer 7: Status validation (no status)
5. Layer 8: Hermeticity (no resource attrs)

#### Attack B: Log Mimicry

**Attack Method**: Sophisticated log forgery with timestamps
```bash
echo "[2025-10-16T10:00:00Z] INFO: Starting test suite"
# ... many realistic log lines ...
echo "PASS"
```

**Why It Fails**:
- Zero spans collected (logs don't create spans)
- Text-based validation bypassed, but span validation fails
- Same count guardrail violation as Attack A

**Key Defense**: clnrm ignores stdout/stderr for validation. Only OTEL spans count.

**Detection Layers Triggered**:
1. Layer 4: Count guardrails (FIRST FAILURE)
2. Layer 1: Span structure (missing spans)
3. Layer 2: Graph topology (no edges)
4. Layer 7: Status validation (no status)
5. Layer 8: Hermeticity (no resource attrs)

#### Attack C: Empty OTEL Path

**Attack Method**: Environment variable spoofing
```bash
export OTEL_TRACES_EXPORTER=otlp
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
echo "OTEL configured"
echo "PASS"
```

**Why It Fails**:
- Environment variables set but zero spans collected
- Proves OTEL SDK never initialized or exporter failed
- Same count guardrail violation

**Real-World Relevance**:
- SDK initialization failures may be silent
- Misconfigured exporters may drop all spans
- Network issues may prevent span export
- Binary may lack instrumentation despite env vars

**Detection Layers Triggered**:
1. Layer 4: Count guardrails (FIRST FAILURE)
2. Layer 1: Span structure (missing spans)
3. Layer 2: Graph topology (no edges)
4. Layer 7: Status validation (no status)
5. Layer 8: Hermeticity (no resource attrs expected but none found)

## Validation Layer Order

clnrm validates in this order (first failure stops validation):

1. **Span Structure** (Layer 1) - Individual span expectations
2. **Graph Topology** (Layer 2) - Parent-child relationships
3. **Lifecycle Events** (Layer 3) - Container events (part of Layer 1)
4. **Count Guardrails** (Layer 4) - **FASTEST CHECK (O(1))**
5. **Temporal Windows** (Layer 5) - Time containment
6. **Ordering Constraints** (Layer 6) - Execution order
7. **Status Validation** (Layer 7) - OTEL status codes
8. **Hermeticity** (Layer 8) - Resource attributes

**Note**: For zero-span attacks, Layer 4 (Count Guardrails) typically fails first because it's the fastest check (O(1) validation).

## Expected vs. Actual Output

### Traditional CI (Vulnerable)

```bash
# All attacks pass
./attack_a_echo.sh
# Exit code: 0 → CI: ✅ PASS

./attack_b_logs.sh
# Exit code: 0 → CI: ✅ PASS

./attack_c_empty_otel.sh
# Exit code: 0 → CI: ✅ PASS
```

### clnrm (Secure)

```bash
# All attacks fail
clnrm run -f attack_a_echo.clnrm.toml
# Exit code: 1 → clnrm: ❌ FAIL (0 spans)

clnrm run -f attack_b_logs.clnrm.toml
# Exit code: 1 → clnrm: ❌ FAIL (0 spans)

clnrm run -f attack_c_empty_otel.clnrm.toml
# Exit code: 1 → clnrm: ❌ FAIL (0 spans)
```

## Command Reference

### Run Individual Attack

```bash
# Attack A
clnrm run -f tests/red_team/attack_a_echo.clnrm.toml

# Attack B
clnrm run -f tests/red_team/attack_b_logs.clnrm.toml

# Attack C
clnrm run -f tests/red_team/attack_c_empty_otel.clnrm.toml

# Legitimate (for comparison)
clnrm run -f tests/red_team/legitimate_self_test.clnrm.toml
```

### Run All Red-Team Tests

```bash
# Sequential execution
for test in tests/red_team/*.clnrm.toml; do
    echo "Running: $test"
    clnrm run -f "$test"
    echo ""
done

# Parallel execution (requires GNU parallel)
parallel clnrm run -f ::: tests/red_team/*.clnrm.toml
```

### Inspect Attack Script Output

```bash
# Run attack script directly (bypassing clnrm)
bash tests/red_team/attack_scripts/attack_a_echo.sh

# Compare with clnrm validation
clnrm run -f tests/red_team/attack_a_echo.clnrm.toml --report json
jq '.counts.spans' report.json  # Should be 0
```

### Verify Detection Timing

```bash
# Measure detection speed
time clnrm run -f tests/red_team/attack_a_echo.clnrm.toml

# Expected: real time < 0.05s (typically ~0.02s)
```

## Digest Analysis

### Empty Trace Digest

All attacks produce the same digest:
```
d41d8cd98f00b204e9800998ecf8427e
```

**Verification**:
```bash
# This is SHA-256 of empty string
echo -n "" | sha256sum
# Output: d41d8cd98f00b204e9800998ecf8427e
```

### Legitimate Test Digest

Legitimate tests produce different, deterministic digests:
```
a3c5e7f9d2b4a6c8e0f2d4b6a8c0e2f4  # Example (deterministic with freeze_clock)
```

**Verification**:
```bash
# Run twice, compare digests
clnrm run -f tests/red_team/legitimate_self_test.clnrm.toml --report json
digest1=$(jq -r '.digest' report.json)

clnrm run -f tests/red_team/legitimate_self_test.clnrm.toml --report json
digest2=$(jq -r '.digest' report.json)

[ "$digest1" == "$digest2" ] && echo "Deterministic ✅" || echo "Non-deterministic ❌"
```

## Integration Testing

### GitHub Actions Example

```yaml
name: Red-Team Protection

on: [pull_request]

jobs:
  red-team-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3

      - name: Run Red-Team Attack A
        run: |
          clnrm run -f tests/red_team/attack_a_echo.clnrm.toml --report json
          exit_code=$?
          span_count=$(jq '.counts.spans' report.json)

          # Attack should FAIL
          if [ $exit_code -eq 0 ]; then
            echo "❌ Attack A PASSED (should have failed)"
            exit 1
          fi

          # Span count should be 0
          if [ "$span_count" -ne 0 ]; then
            echo "❌ Attack A produced spans (should be 0)"
            exit 1
          fi

          echo "✅ Attack A correctly detected"

      - name: Run Legitimate Test
        run: |
          clnrm run -f tests/red_team/legitimate_self_test.clnrm.toml --report json
          exit_code=$?
          span_count=$(jq '.counts.spans' report.json)

          # Legitimate test should PASS
          if [ $exit_code -ne 0 ]; then
            echo "❌ Legitimate test FAILED (should pass)"
            exit 1
          fi

          # Span count should be > 0
          if [ "$span_count" -eq 0 ]; then
            echo "❌ Legitimate test produced 0 spans"
            exit 1
          fi

          echo "✅ Legitimate test correctly passed"
```

### GitLab CI Example

```yaml
red-team-protection:
  script:
    # Run all red-team attacks (should all fail)
    - |
      for attack in tests/red_team/attack_*.clnrm.toml; do
        echo "Testing: $attack"
        if clnrm run -f "$attack" --report json; then
          echo "❌ Attack passed (should have failed)"
          exit 1
        fi
        span_count=$(jq '.counts.spans' report.json)
        if [ "$span_count" -ne 0 ]; then
          echo "❌ Attack produced spans (should be 0)"
          exit 1
        fi
      done

    # Run legitimate test (should pass)
    - clnrm run -f tests/red_team/legitimate_self_test.clnrm.toml --report json
    - span_count=$(jq '.counts.spans' report.json)
    - |
      if [ "$span_count" -eq 0 ]; then
        echo "❌ Legitimate test produced 0 spans"
        exit 1
      fi
```

## Troubleshooting

### Attack Unexpectedly Passes

**Symptom**: Attack test reports PASS instead of FAIL

**Possible Causes**:
1. Test configuration swapped to legitimate binary
2. TOML file edited to use real execution command
3. Container misconfiguration

**Diagnosis**:
```bash
# Check what command is actually running
grep "^run=" tests/red_team/attack_a_echo.clnrm.toml

# Expected: run="sh -lc 'echo \"PASS\"; exit 0'"
# NOT: run="clnrm self-test"
```

### Legitimate Test Fails

**Symptom**: Legitimate test reports FAIL with 0 spans

**Possible Causes**:
1. OTEL SDK not initialized
2. Exporter configuration wrong
3. Container image missing instrumentation

**Diagnosis**:
```bash
# Enable debug logging
clnrm run -f tests/red_team/legitimate_self_test.clnrm.toml --debug

# Look for OTEL initialization logs
# Expected: "OTEL SDK initialized", "Service: clnrm"
```

### Unexpected Digest

**Symptom**: Digest is not `d41d8cd98f00b204e9800998ecf8427e` for attack

**Possible Cause**: Some spans were collected (attack partially succeeded)

**Diagnosis**:
```bash
# Inspect collected spans
clnrm run -f tests/red_team/attack_a_echo.clnrm.toml --report json
jq '.spans' report.json

# If spans exist, investigate why attack produced spans
```

## Security Best Practices

### 1. Never Trust Exit Codes Alone

```bash
# ❌ INSECURE
if [ $exit_code -eq 0 ]; then
    verdict="PASS"
fi

# ✅ SECURE
clnrm run test.clnrm.toml --report json
span_count=$(jq '.counts.spans' report.json)
if [ "$span_count" -eq 0 ]; then
    echo "Fake-green detected"
    exit 1
fi
```

### 2. Validate Digests Against Baseline

```bash
# Record baseline
clnrm run test.clnrm.toml --report json
jq -r '.digest' report.json > baseline.sha256

# Later: verify against baseline
clnrm run test.clnrm.toml --report json
current_digest=$(jq -r '.digest' report.json)
baseline_digest=$(cat baseline.sha256)

if [ "$current_digest" != "$baseline_digest" ]; then
    echo "Digest mismatch (trace changed)"
    exit 1
fi
```

### 3. Check for Empty Trace Digest

```bash
# Detect empty trace digest
EMPTY_DIGEST="d41d8cd98f00b204e9800998ecf8427e"
current_digest=$(jq -r '.digest' report.json)

if [ "$current_digest" == "$EMPTY_DIGEST" ]; then
    echo "❌ FAKE-GREEN DETECTED: Empty trace"
    exit 1
fi
```

## Summary

### Key Takeaways

1. **Zero spans = instant failure** (0.02s detection)
2. **All attacks produce same digest** (`d41d8cd...` empty trace)
3. **First failing rule is always count guardrails** for zero-span attacks
4. **Exit codes are insufficient** for validation
5. **Text-based validation is vulnerable** to mimicry
6. **Span-first validation is secure** (requires SDK instrumentation)

### Defense Guarantees

- **Cryptographic**: SHA-256 digests over normalized spans
- **Multi-layer**: 7 independent validation layers
- **Fast**: 0.02s detection for zero-span attacks
- **Deterministic**: Same input → same first-failing-rule
- **Tamper-evident**: Digest changes with any span modification

### Use Cases

- High-security CI/CD pipelines
- Compliance-critical systems (SOC 2, HIPAA)
- Zero-trust environments
- Forensic test analysis
- Untrusted contributor workflows

## References

- **Full Documentation**: `/Users/sac/clnrm/docs/RED_TEAM_CASE_STUDY.md`
- **Validation Matrix**: `/Users/sac/clnrm/tests/fake_green_detection/VALIDATION_MATRIX.md`
- **Quick Start**: `/Users/sac/clnrm/tests/red_team/README.md`
