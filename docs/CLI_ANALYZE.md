# clnrm analyze - OTEL Trace Validation

The `clnrm analyze` command validates OpenTelemetry traces against TOML-defined expectations, catching **fake-green** tests where assertions pass but telemetry proves nothing happened.

## Quick Start

```bash
# Run validation
clnrm analyze test.clnrm.toml --traces traces.json

# Example output
📊 OTEL Validation Report
========================

Test: clnrm_otel_full_surface
Traces: 47 spans, 123 events

Validators:
  ✅ Span Expectations (3/3 passed)
  ❌ Graph Structure (FAIL: missing edge clnrm.run→clnrm.step:hello_world)
  ✅ Counts (spans_total: 47, expected 2-200)
  ✅ Window Containment (all spans within clnrm.run window)
  ✅ Ordering (all constraints satisfied)
  ✅ Status (all spans OK)
  ✅ Hermeticity (no external services detected)

Result: FAIL (1/7 validators failed)
Digest: sha256:abc123... (recorded for reproduction)
```

## What It Does

The `analyze` command runs **7 validators** on collected OTEL traces:

1. **Span Expectations** - Verifies expected spans exist with correct attributes
2. **Graph Structure** - Validates parent-child relationships (topology)
3. **Counts** - Checks span cardinality (exact, min, max counts)
4. **Window Containment** - Ensures spans are temporally contained
5. **Ordering** - Validates temporal ordering constraints
6. **Status** - Checks span status codes (OK/ERROR/UNSET)
7. **Hermeticity** - Proves test isolation (no external services)

## Command Syntax

```bash
clnrm analyze <TEST_FILE> --traces <TRACES_FILE>
```

### Arguments

- `<TEST_FILE>` - Path to `.clnrm.toml` file containing expectations
- `--traces <TRACES_FILE>` - Path to JSON file with OTEL traces

### Exit Codes

- `0` - All validators passed
- `1` - Any validator failed

## TOML Configuration

Define expectations in your test TOML file using the `[expect]` section:

### Span Expectations

Validate individual span presence and attributes:

```toml
[[expect.span]]
name = "clnrm.run"
kind = "internal"
attrs.all = { "result" = "pass" }
duration_ms = { min = 10, max = 600000 }

[[expect.span]]
name = "clnrm.step:hello_world"
parent = "clnrm.run"
attrs.any = ["step.name=hello_world", "status=ok"]
events.any = ["container.start", "container.exec", "container.stop"]
```

### Graph Structure

Validate parent-child relationships:

```toml
[expect.graph]
must_include = [
    ["clnrm.run", "clnrm.step:hello_world"],
    ["clnrm.run", "clnrm.plugin.registry"]
]
must_not_cross = [
    ["clnrm.step:hello_world", "clnrm.plugin.registry"]
]
acyclic = true
```

### Counts

Validate span cardinality:

```toml
[expect.counts]
spans_total = { gte = 2, lte = 200 }
events_total = { gte = 2 }
errors_total = { eq = 0 }

[expect.counts.by_name]
"clnrm.run" = { eq = 1 }
"clnrm.step:hello_world" = { eq = 1 }
```

### Window Containment

Ensure temporal containment:

```toml
[[expect.window]]
outer = "clnrm.run"
contains = ["clnrm.step:hello_world", "clnrm.plugin.registry"]
```

### Ordering

Validate temporal ordering:

```toml
[expect.order]
must_precede = [
    ["clnrm.plugin.registry", "clnrm.step:hello_world"]
]
must_follow = [
    ["clnrm.step:hello_world", "clnrm.run"]
]
```

### Status

Validate span status codes:

```toml
[expect.status]
all = "OK"

[expect.status.by_name]
"clnrm.*" = "OK"
"error_span" = "ERROR"
```

### Hermeticity

Prove test isolation:

```toml
[expect.hermeticity]
no_external_services = true

[expect.hermeticity.resource_attrs]
must_match = { "service.name" = "clnrm", "env" = "ci" }

[expect.hermeticity.span_attrs]
forbid_keys = ["net.peer.name", "db.connection_string", "http.url"]
```

## Detecting Fake-Green Tests

### The Problem

A test assertion passes, but telemetry proves the operation never happened:

```rust
#[test]
fn test_container_created() {
    // BUG: forgot to await
    create_container("alpine:latest"); // Returns Future, never runs
    assert!(true); // Always passes - fake green!
}
```

### The Solution

OTEL validation catches this:

```toml
[[expect.span]]
name = "container.create"
attrs.all = { "image" = "alpine:latest" }
```

```bash
$ clnrm analyze test.toml --traces traces.json
❌ Span Expectations (FAIL: Expected span 'container.create' not found)
Result: FAIL (1/7 validators failed)
```

## Workflow Integration

### 1. Collect Traces

```bash
# Run tests with OTEL enabled
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 \
cargo test --features otel

# Traces saved to traces.json by collector
```

### 2. Validate Traces

```bash
# Run analysis
clnrm analyze tests/my_test.clnrm.toml --traces traces.json

# Exit code 1 = validation failed
```

### 3. CI Integration

```yaml
# .github/workflows/ci.yml
- name: Run tests with OTEL
  run: cargo test --features otel
  env:
    OTEL_EXPORTER_OTLP_ENDPOINT: http://localhost:4318

- name: Validate OTEL traces
  run: clnrm analyze tests/*.clnrm.toml --traces traces.json
```

## Output Format

### Human-Readable (Default)

```
📊 OTEL Validation Report
========================

Test: clnrm_otel_full_surface
Traces: 47 spans, 123 events

Validators:
  ✅ Span Expectations (3/3 passed)
  ✅ Graph Structure (all 2 edges present)
  ✅ Counts (spans_total: 47)
  ✅ Window Containment (all 1 windows satisfied)
  ✅ Ordering (all constraints satisfied)
  ✅ Status (all spans OK)
  ✅ Hermeticity (no external services detected)

Result: PASS (7/7 validators passed)
Digest: sha256:abc123def456... (recorded for reproduction)
```

### Failed Validation

```
📊 OTEL Validation Report
========================

Test: my_failing_test
Traces: 10 spans, 5 events

Validators:
  ❌ Span Expectations (FAIL: Expected span 'critical_operation' not found)
  ✅ Graph Structure (all 1 edges present)
  ✅ Counts (spans_total: 10)
  ✅ Window Containment (all 1 windows satisfied)
  ✅ Ordering (all constraints satisfied)
  ✅ Status (all spans OK)
  ❌ Hermeticity (FAIL: Hermeticity validation failed: Span 'database_call' has forbidden attribute 'net.peer.name')

Result: FAIL (2/7 validators failed)
Digest: sha256:def789... (recorded for reproduction)
```

## Reproducibility

Every validation generates a SHA256 digest of the trace data:

```
Digest: sha256:abc123def456... (recorded for reproduction)
```

This enables:
- **Deterministic testing**: Same traces always produce same digest
- **Regression detection**: Compare digests across test runs
- **Baseline tracking**: Store digests as test baselines

## Common Use Cases

### Use Case 1: TDD with Telemetry

```bash
# 1. Write test with expectations (red)
clnrm analyze test.toml --traces empty.json
# ❌ FAIL: Expected span 'feature.implemented' not found

# 2. Implement feature

# 3. Run tests and validate (green)
clnrm analyze test.toml --traces traces.json
# ✅ PASS (7/7 validators passed)
```

### Use Case 2: Regression Detection

```bash
# Record baseline
clnrm analyze test.toml --traces baseline.json > baseline.txt

# After code changes
clnrm analyze test.toml --traces current.json > current.txt

# Compare
diff baseline.txt current.txt
```

### Use Case 3: Performance Validation

```toml
[[expect.span]]
name = "slow_operation"
duration_ms = { max = 5000 } # Fail if > 5 seconds
```

## Troubleshooting

### Error: "Failed to read test file"

Ensure test file exists and is valid TOML:

```bash
# Validate TOML syntax
clnrm lint test.toml
```

### Error: "Failed to parse test TOML"

Check for syntax errors in expectations:

```toml
# ❌ WRONG: edges must be arrays
[expect.graph]
must_include = ["parent", "child"]

# ✅ CORRECT: edges are nested arrays
[expect.graph]
must_include = [["parent", "child"]]
```

### Error: "Failed to read spans file"

Ensure traces.json contains valid OTEL span data:

```bash
# Verify JSON is valid
jq . traces.json
```

### No Spans Found

If validation finds 0 spans:

1. Check OTEL collector is running
2. Verify `OTEL_EXPORTER_OTLP_ENDPOINT` is set
3. Ensure test code emits spans
4. Check collector exports to JSON file

## Advanced Patterns

### Pattern 1: Multi-Service Validation

```toml
[[expect.span]]
name = "api.request"
attrs.all = { "service.name" = "api" }

[[expect.span]]
name = "db.query"
attrs.all = { "service.name" = "database" }

[expect.graph]
must_include = [["api.request", "db.query"]]
```

### Pattern 2: Error Scenario Testing

```toml
[expect.status]
by_name = {
    "happy_path.*" = "OK",
    "error_handler" = "ERROR"
}
```

### Pattern 3: Performance Budgets

```toml
[[expect.span]]
name = "critical_path"
duration_ms = { max = 100 }

[expect.counts]
spans_total = { lte = 50 } # Prevent span explosion
```

## See Also

- [OTEL PRD](/Users/sac/clnrm/OTEL-PRD.md) - Full OTEL validation specification
- [clnrm diff](/Users/sac/clnrm/docs/CLI_DIFF.md) - Compare traces across runs
- [clnrm record](/Users/sac/clnrm/docs/CLI_RECORD.md) - Record baselines
- [Fake Green Detection](/Users/sac/clnrm/docs/FAKE_GREEN_DETECTION_CASE_STUDY.md) - Case study

## Example: Full Test Configuration

See `/Users/sac/clnrm/tests/fake_green_detection/clnrm_otel_full_surface.clnrm.toml` for a complete example exercising all 7 validators.
