# Fake-Green Detection: Validation Matrix

## Evidence Required vs. Fake Script Produces

| Validation Layer | Evidence Required | Fake Script Produces | Detection | Verdict |
|-----------------|-------------------|---------------------|-----------|---------|
| **1. Span Structure** | `[[expect.span]]` with `events.any=["container.start","container.exec","container.stop"]` | 0 spans, 0 events | `span not found` | ❌ FAIL |
| **2. Graph Topology** | `must_include=[["clnrm.run","clnrm.step:hello_world"]]` | No edges (no spans to connect) | `missing edge` | ❌ FAIL |
| **3. Lifecycle Events** | Container lifecycle events (`start`, `exec`, `stop`) | No container interaction | `events.any: found []` | ❌ FAIL |
| **4. Count Guardrails** | `spans_total={ gte=2 }`, `events_total={ gte=2 }` | `spans=0`, `events=0` | `required >=2, found 0` | ❌ FAIL |
| **5. Temporal Windows** | `[[expect.window]]` outer contains children | No spans to measure timing | `no spans to validate` | ❌ FAIL |
| **6. Ordering Constraints** | `must_precede=[["plugin.registry","hello_world"]]` | No spans to order | `no spans to validate` | ❌ FAIL |
| **7. Status Validation** | `all="OK"` (OTEL span status) | No span status (only exit code 0) | `no spans to validate` | ❌ FAIL |
| **8. Hermeticity** | `resource_attrs.must_match={ "service.name"="clnrm" }` | No OTEL SDK context | `no resource attributes` | ❌ FAIL |

## Fake Script Output Analysis

### What the Fake Script Does:
```bash
#!/bin/bash
echo "✅ Tests passed: 100%"
echo "✅ Coverage: 95%"
echo "✅ All assertions passed"
echo "PASS"
exit 0
```

### What Traditional CI Sees:
- **stdout**: Pretty formatted success messages ✅
- **exit code**: `0` (success) ✅
- **Decision**: PASS ✅ (false positive)

### What clnrm Validation Sees:

#### Layer 1: Span Structure
```toml
[[expect.span]]
name="clnrm.step:hello_world"
parent="clnrm.run"
events.any=["container.start","container.exec","container.stop"]
```
**Expected**: Span with lifecycle events
**Found**: No spans at all
**Result**: ❌ `span 'clnrm.step:hello_world' not found in trace`

#### Layer 2: Graph Topology
```toml
[expect.graph]
must_include=[["clnrm.run","clnrm.step:hello_world"]]
```
**Expected**: Parent-child edge proving execution hierarchy
**Found**: No edges (cannot connect 0 spans)
**Result**: ❌ `missing edge [clnrm.run → clnrm.step:hello_world]`

#### Layer 3: Lifecycle Events
```toml
events.any=["container.start","container.exec","container.stop"]
```
**Expected**: Real container lifecycle events
**Found**: No events (fake script doesn't launch containers)
**Result**: ❌ `required events not found`

#### Layer 4: Count Guardrails
```toml
[expect.counts]
spans_total={ gte=2, lte=200 }
events_total={ gte=2 }
by_name={ "clnrm.run"={ eq=1 }, "clnrm.step:hello_world"={ eq=1 } }
```
**Expected**: Minimum 2 spans, 2 events
**Found**: 0 spans, 0 events
**Result**: ❌ `spans_total: required >=2, found 0`

#### Layer 5: Temporal Windows
```toml
[[expect.window]]
outer="clnrm.run"
contains=["clnrm.step:hello_world","clnrm.plugin.registry"]
```
**Expected**: Child spans temporally contained in parent
**Found**: No spans to measure timing
**Result**: ❌ `no spans to validate window containment`

#### Layer 6: Ordering Constraints
```toml
[expect.order]
must_precede=[["clnrm.plugin.registry","clnrm.step:hello_world"]]
```
**Expected**: Correct execution order (registry before steps)
**Found**: No spans to order
**Result**: ❌ `no spans to validate ordering`

#### Layer 7: Status Validation
```toml
[expect.status]
all="OK"
by_name={ "clnrm.*"="OK" }
```
**Expected**: OTEL span status codes (not exit codes)
**Found**: No span status (exit code 0 is insufficient)
**Result**: ❌ `no spans to validate status`

#### Layer 8: Hermeticity
```toml
[expect.hermeticity]
no_external_services=true
resource_attrs.must_match={ "service.name"="clnrm", "env"="ci" }
```
**Expected**: SDK-generated resource attributes
**Found**: No OTEL SDK context (fake script has no instrumentation)
**Result**: ❌ `no resource attributes found`

## Digest Analysis

### Empty Trace Digest:
```
SHA-256: d41d8cd98f00b204e9800998ecf8427e
```

This is the SHA-256 hash of an **empty string**, proving:
- No spans were collected
- No events were recorded
- No actual execution occurred
- The fake script produced zero observability evidence

### Recorded for Forensics:
```json
{
  "digest": "d41d8cd98f00b204e9800998ecf8427e",
  "verdict": "FAIL",
  "first_failure": {
    "rule": "expect.counts.spans_total",
    "expected": ">=2",
    "actual": 0,
    "reason": "No spans collected from execution"
  },
  "counts": {
    "spans": 0,
    "events": 0,
    "errors": null
  }
}
```

## Attack Vector Analysis

### Attack: Fake Exit Code
- **Method**: `exit 0` without doing work
- **Traditional CI**: ✅ Accepts (false positive)
- **clnrm**: ❌ Rejects (no spans)

### Attack: Forge stdout
- **Method**: Echo "PASS" without executing
- **Traditional CI**: ✅ Accepts (false positive)
- **clnrm**: ❌ Rejects (no events)

### Attack: Mock Test Output
- **Method**: Replay old test output
- **Traditional CI**: ✅ Accepts (false positive)
- **clnrm**: ❌ Rejects (digest mismatch with freeze_clock)

### Attack: Network Replay
- **Method**: Replay old OTEL trace
- **Traditional CI**: N/A
- **clnrm**: ❌ Rejects (hermeticity check fails)

### Attack: Partial Execution
- **Method**: Run some tests, skip others, claim full pass
- **Traditional CI**: ✅ May accept
- **clnrm**: ❌ Rejects (exact count requirements)

### Attack: Time Manipulation
- **Method**: Alter timestamps to hide failures
- **Traditional CI**: ✅ May accept
- **clnrm**: ❌ Rejects (window validation + freeze_clock)

## Comparison: Traditional vs. clnrm

| Question | Traditional Testing | clnrm (Observability-First) |
|----------|-------------------|----------------------------|
| What is validated? | Exit code | OTEL spans + events + attributes |
| Can be faked? | ✅ Yes (trivial) | ❌ No (requires SDK instrumentation) |
| Proof type | Exit code integer | Cryptographic digest over normalized trace |
| Temporal validation | ❌ No | ✅ Yes (windows + ordering) |
| Lifecycle proof | ❌ No | ✅ Yes (container events required) |
| Reproducible? | ❌ No | ✅ Yes (record/repro/redgreen) |
| Tamper-evident? | ❌ No | ✅ Yes (SHA-256 digest) |
| Hermetic validation | ❌ No | ✅ Yes (resource attrs + forbid keys) |

## Security Guarantees

### Traditional Testing (Weak):
```
If exit_code == 0:
    verdict = PASS  # Can be spoofed
```

**Attack Complexity**: Trivial (`exit 0`)

### clnrm (Strong):
```
If all([
    spans_exist(),
    graph_valid(),
    events_present(),
    counts_match(),
    windows_valid(),
    ordering_correct(),
    status_ok(),
    hermetic()
]):
    digest = sha256(normalized_trace)
    verdict = PASS
```

**Attack Complexity**: Requires forging OTEL SDK instrumentation (nearly impossible)

## Conclusion

clnrm's 8-layer validation system provides **cryptographic proof of execution** through:

1. **Multi-dimensional evidence**: Not just exit codes
2. **SDK-enforced**: Cannot be faked without actual instrumentation
3. **Tamper-evident**: SHA-256 digest over normalized traces
4. **Reproducible**: record/repro/redgreen workflow
5. **Comprehensive**: 8 independent validation layers

This makes fake-green results **practically impossible** and **forensically detectable**.

## Files Reference

- `clnrm_otel_full_surface.clnrm.toml` - Complete v1.0 specification
- `fake_wrapper.sh` - Malicious wrapper demonstrating attack
- `demo_fake_green_detection.sh` - Interactive demonstration
- `README.md` - Technical documentation
- `VALIDATION_MATRIX.md` - This file
- `../../docs/FAKE_GREEN_DETECTION_CASE_STUDY.md` - Executive summary
