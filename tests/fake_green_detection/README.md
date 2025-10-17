# Fake-Green Detection Case Study

## Overview

This case study demonstrates how clnrm's multi-layered validation system prevents **fake-green** test results—where a wrapper script claims success without actually executing tests.

## Failure Mode

**System Under Test**: `clnrm` executing its own self-tests in a sealed container

**Attack Vector**: A malicious or buggy wrapper script that:
```bash
echo "✅ Tests passed: 100%"
echo "PASS"
exit 0
```

Without proper validation, this would appear as a successful test run in CI/CD systems.

## Defense Layers

clnrm prevents fake-green results through **8 independent validation layers**:

### Layer 1: Span Structure Validation
```toml
[[expect.span]]
name="clnrm.step:hello_world"
parent="clnrm.run"
events.any=["container.start","container.exec","container.stop"]
```

**Detection**: Requires specific lifecycle events that only real container execution produces.

### Layer 2: Graph Topology Validation
```toml
[expect.graph]
must_include=[["clnrm.run","clnrm.step:hello_world"]]
acyclic=true
```

**Detection**: Validates parent-child edges that prove execution hierarchy. A fake script produces no spans, so no edges exist.

### Layer 3: Lifecycle Events Validation
```toml
events.any=["container.start","container.exec","container.stop"]
```

**Detection**: Real container execution emits these events. Fake scripts cannot forge OTEL events without the instrumentation.

### Layer 4: Count Guardrails
```toml
[expect.counts]
spans_total={ gte=2, lte=200 }
events_total={ gte=2 }
by_name={ "clnrm.step:hello_world"={ eq=1 } }
```

**Detection**: Enforces minimum span and event counts. Zero spans = instant failure.

### Layer 5: Temporal Windows
```toml
[[expect.window]]
outer="clnrm.run"
contains=["clnrm.step:hello_world"]
```

**Detection**: Child spans must be temporally contained within parent. Validates actual execution timeline.

### Layer 6: Ordering Constraints
```toml
[expect.order]
must_precede=[["clnrm.plugin.registry","clnrm.step:hello_world"]]
```

**Detection**: Enforces execution order. Fake scripts have no ordering to validate.

### Layer 7: Status Validation
```toml
[expect.status]
all="OK"
by_name={ "clnrm.*"="OK" }
```

**Detection**: Validates OTEL span status codes, not just exit codes. Fake scripts produce no span status.

### Layer 8: Hermeticity Assertions
```toml
[expect.hermeticity]
no_external_services=true
resource_attrs.must_match={ "service.name"="clnrm", "env"="ci" }
span_attrs.forbid_keys=["net.peer.name"]
```

**Detection**: Validates resource attributes from OTEL SDK. Fake scripts cannot forge these without actual instrumentation.

## Expected Outcome

When `fake_wrapper.sh` runs instead of real tests:

### ❌ Analyzer Failures:

1. **Missing Edges**:
   ```
   FAIL expect.graph.must_include [clnrm.run → clnrm.step:hello_world]
   ├─ expected edge not found
   └─ reason: zero spans collected
   ```

2. **Missing Lifecycle Events**:
   ```
   FAIL expect.span[clnrm.step:hello_world].events.any
   ├─ required events: [container.start, container.exec, container.stop]
   └─ found events: []
   ```

3. **Count Violations**:
   ```
   FAIL expect.counts.spans_total
   ├─ required: gte=2
   └─ actual: 0
   ```

4. **Status Violations**:
   ```
   FAIL expect.status.all=OK
   ├─ reason: no spans to validate
   └─ cannot determine status from exit code alone
   ```

5. **Hermeticity Violations**:
   ```
   FAIL expect.hermeticity.resource_attrs.must_match
   ├─ required: service.name=clnrm, env=ci
   └─ found: no resource attributes (no OTEL context)
   ```

### ✅ Digest Recorded:

Even on failure, clnrm records the (empty) trace digest:
```
digest: d41d8cd98f00b204e9800998ecf8427e  # empty trace SHA-256
verdict: FAIL
first_failure: { rule: "expect.counts.spans_total", expected: ">=2", actual: 0 }
```

This digest enables:
- Reproducible failure analysis
- Comparison with known-good runs
- Forensic investigation of the fake-green attempt

## Demonstration

### 1. Run the Fake Wrapper:
```bash
cd tests/fake_green_detection
./fake_wrapper.sh
```

**Output**:
```
🎭 FAKE WRAPPER: Attempting to spoof test results...

✅ Tests passed: 100%
✅ Coverage: 95%
✅ All assertions passed

PASS
```

**Exit Code**: `0` ✅ (appears successful)

### 2. Run clnrm Validation:
```bash
clnrm run fake_green_case_study.clnrm.toml
```

**Output**:
```
FAIL fake_green_case_study (spans=0, events=0, errors=N/A)

Validation Failures:
  ❌ expect.counts.spans_total: required >=2, found 0
  ❌ expect.graph.must_include: missing edge [clnrm.run → clnrm.step:hello_world]
  ❌ expect.span[clnrm.step:hello_world]: span not found
  ❌ expect.status.all: no spans to validate

Digest: d41d8cd98f00b204e9800998ecf8427e
Verdict: FAIL
First Failure: expect.counts.spans_total (no spans collected)
```

**Exit Code**: `1` ❌ (correctly fails)

## Key Insights

### 1. Exit Code ≠ Success
Traditional test frameworks rely on exit codes. clnrm requires **observability evidence**.

### 2. Layered Defense
Each validation layer is independent. Even if one layer fails, others catch the fake-green.

### 3. Cryptographic Proof
The SHA-256 digest provides tamper-evident proof of execution (or lack thereof).

### 4. Reproducible Analysis
The `record/repro/redgreen` flow enables forensic analysis:
```bash
clnrm record baseline.json       # Record known-good run
clnrm repro baseline.json        # Reproduce later
clnrm redgreen baseline.json     # Compare digests
```

## Anti-Spoofing Guarantees

| Attack Vector | Detection Method | Why It Works |
|--------------|------------------|--------------|
| Echo "PASS" + exit 0 | Span count = 0 | No OTEL instrumentation |
| Forge exit code | Graph validation | No parent-child edges |
| Mock stdout | Lifecycle events | No container.start events |
| Replay old trace | Digest mismatch | Deterministic freeze_clock |
| Fake OTEL spans | Resource attributes | SDK-generated attributes required |
| Network replay | Hermeticity check | Forbids external network spans |
| Time manipulation | Window validation | Temporal containment enforced |
| Partial execution | Count guardrails | Requires exact span counts |

## Production Use Cases

### CI/CD Pipeline:
```yaml
- name: Run Tests with Fake-Green Protection
  run: |
    clnrm run tests/production.clnrm.toml
    clnrm diff baseline.json --fail-on-mismatch
```

### Security Audit:
```bash
# Verify test results with cryptographic proof
clnrm run test.clnrm.toml --report json
jq '.digest' report.json  # Extract SHA-256 digest
echo "$DIGEST" | sha256sum --check  # Verify tamper-evidence
```

### Forensic Analysis:
```bash
# Compare suspicious run with known-good baseline
clnrm redgreen baseline.json suspicious.json
# Output shows exactly which spans/events differ
```

## Conclusion

clnrm's multi-layered validation system makes fake-green results **cryptographically detectable** through:
- Observable evidence (OTEL spans/events)
- Structural validation (graph topology)
- Temporal validation (windows, ordering)
- Resource validation (hermetic attributes)
- Deterministic digests (SHA-256 over normalized traces)

This case study demonstrates that **observability-first testing** provides stronger guarantees than traditional exit-code-based testing.

## Files

- `fake_green_case_study.clnrm.toml` - Full validation configuration
- `fake_wrapper.sh` - Malicious wrapper that attempts spoofing
- `README.md` - This documentation

## References

- PRD-v1.md: Variable precedence, schema definition
- .cursorrules: Core Team standards (no false positives)
- TOML_REFERENCE.md: Complete schema documentation
