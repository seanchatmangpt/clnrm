# OTEL Validator Quick Reference

Quick reference guide for all 7 OTEL validators in clnrm v0.7.0.

## At-a-Glance Validator Matrix

| Validator | Purpose | Key Features | TOML Section |
|-----------|---------|--------------|--------------|
| **Span** | Individual span validation | name, kind, attrs, events, duration | `[[expect.span]]` |
| **Graph** | Topology validation | edges, cycles, hierarchy | `[expect.graph]` |
| **Count** | Cardinality validation | total, by-name, bounds | `[expect.counts]` |
| **Window** | Temporal containment | outer, contains | `[[expect.window]]` |
| **Order** | Temporal sequence | precede, follow | `[expect.order]` |
| **Status** | Status codes | all, by-name (glob) | `[expect.status]` |
| **Hermeticity** | Isolation validation | external services, attrs | `[expect.hermeticity]` |

---

## 1. SpanValidator (`[[expect.span]]`)

**Purpose**: Validate individual span properties

### Basic Usage

```toml
[[expect.span]]
name = "clnrm.run"
kind = "internal"
```

### All Features

```toml
[[expect.span]]
name = "clnrm.run"
parent = "root"                     # Parent span name
kind = "internal"                   # internal|server|client|producer|consumer

# All attributes must match (AND)
attrs.all = {
    "test.framework" = "clnrm",
    "version" = "0.7.0"
}

# Any attribute matches (OR)
attrs.any = [
    "env=test",
    "env=dev"
]

# Any event present
events.any = ["test.start", "test.end"]

# Duration bounds (milliseconds)
duration_ms.min = 100
duration_ms.max = 5000
```

---

## 2. GraphValidator (`[expect.graph]`)

**Purpose**: Validate span graph topology

### Basic Usage

```toml
[expect.graph]
must_include = [
    ["parent", "child"]
]
```

### All Features

```toml
[expect.graph]
# Required edges (parent → child)
must_include = [
    ["clnrm.run", "clnrm.test"],
    ["clnrm.test", "container.start"]
]

# Forbidden edges (isolation boundaries)
must_not_cross = [
    ["test_a", "test_b"],           # Tests must not call each other
    ["container_a", "container_b"]  # Containers isolated
]

# No cycles allowed
acyclic = true
```

---

## 3. CountValidator (`[expect.counts]`)

**Purpose**: Validate span counts

### Basic Usage

```toml
[expect.counts]
spans_total.eq = 10
```

### All Features

```toml
[expect.counts]
# Total spans
spans_total.gte = 5    # Greater than or equal
spans_total.lte = 50   # Less than or equal
spans_total.eq = 10    # Exactly (takes precedence)

# Events and errors
events_total.eq = 0
errors_total.eq = 0

# Per-name counts
[expect.counts.by_name]
"clnrm.run".eq = 1
"clnrm.test".gte = 3
"clnrm.test".lte = 10
"container.start".eq = 1
```

### CountBound Options

- `gte` - Greater than or equal (minimum)
- `lte` - Less than or equal (maximum)
- `eq` - Exactly equal (overrides gte/lte)
- Can combine: `gte = 5, lte = 10` for range

---

## 4. WindowValidator (`[[expect.window]]`)

**Purpose**: Validate temporal containment

### Basic Usage

```toml
[[expect.window]]
outer = "clnrm.run"
contains = ["clnrm.test"]
```

### Multiple Windows

```toml
[[expect.window]]
outer = "clnrm.run"
contains = ["clnrm.test", "container.start", "command.exec"]

[[expect.window]]
outer = "clnrm.test"
contains = ["setup", "execute", "teardown"]
```

### Validation Rule

```
outer.start_time ≤ child.start_time ≤ child.end_time ≤ outer.end_time
```

---

## 5. OrderValidator (`[expect.order]`)

**Purpose**: Validate temporal ordering

### Basic Usage

```toml
[expect.order]
must_precede = [
    ["first", "second"]
]
```

### All Features

```toml
[expect.order]
# First must complete before second starts
must_precede = [
    ["plugin.register", "plugin.start"],
    ["plugin.start", "test.execute"],
    ["test.execute", "cleanup"]
]

# First must start after second completes
must_follow = [
    ["cleanup", "test.execute"],
    ["report.generate", "cleanup"]
]
```

### Validation Rules

- `must_precede`: `first.end_time ≤ second.start_time`
- `must_follow`: `first.start_time ≥ second.end_time`

---

## 6. StatusValidator (`[expect.status]`)

**Purpose**: Validate span status codes

### Basic Usage

```toml
[expect.status]
all = "OK"
```

### All Features

```toml
[expect.status]
all = "OK"  # All spans must be OK

# Per-name patterns (glob support)
[expect.status.by_name]
"clnrm.*" = "OK"
"test_*" = "OK"
"http.request" = "ERROR"    # Expected to fail
"db.query?" = "OK"          # ? matches single char
"[abc]*.span" = "OK"        # [] for character class
```

### Status Codes

- `UNSET` - Not set (default)
- `OK` - Success
- `ERROR` - Failure

### Glob Patterns

- `*` - Match zero or more characters
- `?` - Match exactly one character
- `[abc]` - Match any character in set
- `[!abc]` - Match any character not in set

---

## 7. HermeticityValidator (`[expect.hermeticity]`)

**Purpose**: Validate test isolation

### Basic Usage

```toml
[expect.hermeticity]
no_external_services = true
```

### All Features

```toml
[expect.hermeticity]
# Detect external network access
no_external_services = true

# Required resource attributes
[expect.hermeticity.resource_attrs]
must_match = {
    "service.name" = "clnrm",
    "env" = "test"
}

# Forbidden span attributes
[expect.hermeticity.span_attrs]
forbid_keys = [
    "net.peer.name",
    "http.url",
    "db.connection_string"
]
```

### Detected Network Attributes

```rust
net.peer.name
net.peer.ip
net.peer.port
http.host
http.url
db.connection_string
rpc.service
messaging.destination
messaging.url
```

---

## Complete Example

```toml
[test.metadata]
name = "comprehensive_validation"
description = "All 7 validators in action"

# 1. SPAN VALIDATOR
[[expect.span]]
name = "clnrm.run"
kind = "internal"
attrs.all = { "test.framework" = "clnrm" }
duration_ms.min = 100

[[expect.span]]
name = "clnrm.test"
parent = "clnrm.run"
events.any = ["test.start", "test.end"]

# 2. GRAPH VALIDATOR
[expect.graph]
must_include = [
    ["clnrm.run", "clnrm.test"],
    ["clnrm.test", "container.start"]
]
must_not_cross = [["test_a", "test_b"]]
acyclic = true

# 3. COUNT VALIDATOR
[expect.counts]
spans_total.gte = 5
spans_total.lte = 50
errors_total.eq = 0

[expect.counts.by_name]
"clnrm.run".eq = 1
"clnrm.test".gte = 3

# 4. WINDOW VALIDATOR
[[expect.window]]
outer = "clnrm.run"
contains = ["clnrm.test", "container.start"]

# 5. ORDER VALIDATOR
[expect.order]
must_precede = [
    ["plugin.register", "test.execute"],
    ["test.execute", "cleanup"]
]

# 6. STATUS VALIDATOR
[expect.status]
all = "OK"

[expect.status.by_name]
"clnrm.*" = "OK"

# 7. HERMETICITY VALIDATOR
[expect.hermeticity]
no_external_services = true

[expect.hermeticity.resource_attrs]
must_match = { "service.name" = "clnrm", "env" = "test" }

[expect.hermeticity.span_attrs]
forbid_keys = ["net.peer.name", "http.url"]
```

---

## Common Patterns

### Pattern: Validate Full Test Lifecycle

```toml
[[expect.span]]
name = "test.setup"
events.any = ["setup.start", "setup.complete"]

[[expect.span]]
name = "test.execute"
events.any = ["execute.start", "execute.complete"]

[[expect.span]]
name = "test.teardown"
events.any = ["teardown.start", "teardown.complete"]

[expect.order]
must_precede = [
    ["test.setup", "test.execute"],
    ["test.execute", "test.teardown"]
]

[[expect.window]]
outer = "test.run"
contains = ["test.setup", "test.execute", "test.teardown"]
```

### Pattern: Validate Container Isolation

```toml
[expect.graph]
must_not_cross = [
    ["container_a", "container_b"],
    ["container_a", "container_c"],
    ["container_b", "container_c"]
]

[expect.hermeticity]
no_external_services = true

[expect.hermeticity.span_attrs]
forbid_keys = ["net.peer.name", "net.peer.ip"]
```

### Pattern: Validate Parallel Execution

```toml
[expect.counts.by_name]
"worker_1".eq = 1
"worker_2".eq = 1
"worker_3".eq = 1

[[expect.window]]
outer = "parallel_execution"
contains = ["worker_1", "worker_2", "worker_3"]

# Workers must NOT be ordered (can run in parallel)
# No must_precede constraints between workers
```

### Pattern: Detect Fake Green

```toml
# Ensure test actually ran
[[expect.span]]
name = "test.execute"
events.any = ["test.start", "test.end"]
duration_ms.min = 10  # Must take at least 10ms

# Ensure proper parent-child relationship
[expect.graph]
must_include = [["test.run", "test.execute"]]

# Count must match expected
[expect.counts.by_name]
"test.execute".eq = 1

# Status must be set (not UNSET)
[expect.status.by_name]
"test.execute" = "OK"
```

---

## Error Message Examples

### SpanValidator Error

```
Span assertion failed: span 'clnrm.test' does not exist
Found spans: [clnrm.run, clnrm.init, container.start]
```

### GraphValidator Error

```
Graph validation failed: required edge 'clnrm.run' -> 'clnrm.test' not found
Found spans: [clnrm.run, clnrm.init]
Expected parent 'clnrm.run' but no child 'clnrm.test' found
```

### CountValidator Error

```
Count for span name 'clnrm.test': expected exactly 3 items, found 1
Actual 'clnrm.test' spans: 1 (expected 3)
```

### WindowValidator Error

```
Window validation failed: child span 'clnrm.test' started before outer span 'clnrm.run'
child_start: 1700000000100000000
outer_start: 1700000000200000000
Difference: -100ms
```

### OrderValidator Error

```
Order validation failed: 'plugin.start' must precede 'test.execute' but no valid ordering found
plugin.start: end_time=1700000002000000000
test.execute: start_time=1700000001000000000
```

### StatusValidator Error

```
Status validation failed: span 'clnrm.test' matching pattern 'clnrm.*' has status ERROR but expected OK
Pattern 'clnrm.*' matched 3 spans:
- clnrm.run (status=OK) ✓
- clnrm.test (status=ERROR) ← violation
```

### HermeticityValidator Error

```
Hermeticity validation failed with 2 violation(s):

1. Span 'clnrm.test' contains external network attribute 'net.peer.name', indicating non-hermetic execution
   Span: clnrm.test
   Span ID: span_abc123
   Attribute: net.peer.name
   Actual: external.com

2. Resource attribute 'service.name' mismatch: expected 'clnrm', found 'wrong_service'
   Attribute: service.name
   Expected: clnrm
   Actual: wrong_service
```

---

## Testing Your Validators

### Run Validation Tests

```bash
# All validation tests
cargo test -p clnrm-core --lib validation

# Specific validator
cargo test -p clnrm-core --lib span_validator
cargo test -p clnrm-core --lib graph_validator
cargo test -p clnrm-core --lib count_validator
cargo test -p clnrm-core --lib window_validator
cargo test -p clnrm-core --lib order_validator
cargo test -p clnrm-core --lib status_validator
cargo test -p clnrm-core --lib hermeticity_validator
```

### Test with TOML Config

```bash
# Run with validators
clnrm run tests/fake_green_detection/fake_green_case_study.clnrm.toml

# Verbose output
clnrm run --verbose tests/integration/prd_otel_workflow.clnrm.toml
```

---

## Tips & Best Practices

### 1. Start Simple

```toml
# Start with basic validation
[expect.counts]
spans_total.gte = 1

[expect.status]
all = "OK"
```

### 2. Add Specificity Gradually

```toml
# Then add specific span checks
[[expect.span]]
name = "clnrm.run"

# Then add relationships
[expect.graph]
must_include = [["clnrm.run", "clnrm.test"]]
```

### 3. Use Multiple Windows for Nested Spans

```toml
# Outer window
[[expect.window]]
outer = "root"
contains = ["level1_a", "level1_b"]

# Inner window
[[expect.window]]
outer = "level1_a"
contains = ["level2_a", "level2_b"]
```

### 4. Combine Validators for Comprehensive Checks

```toml
# Count + Status + Graph = Strong validation
[expect.counts.by_name]
"clnrm.test".eq = 5

[expect.status.by_name]
"clnrm.test" = "OK"

[expect.graph]
must_include = [["clnrm.run", "clnrm.test"]]
```

### 5. Use Glob Patterns for Flexibility

```toml
# Match all test spans
[expect.status.by_name]
"test.*" = "OK"
"clnrm.test*" = "OK"
```

---

## Performance Notes

- **SpanValidator**: O(n) - Linear scan with hashmap lookup
- **GraphValidator**: O(n + e) - DFS traversal (n=nodes, e=edges)
- **CountValidator**: O(n) - Single pass counting
- **WindowValidator**: O(n × m) - n=spans, m=windows
- **OrderValidator**: O(n × c) - n=spans, c=constraints
- **StatusValidator**: O(n × p) - n=spans, p=patterns
- **HermeticityValidator**: O(n × k) - n=spans, k=forbidden keys

All validators are optimized for production use.

---

## Further Reading

- Full Documentation: `docs/VALIDATOR_COMPLETENESS_REPORT.md`
- PRD: `PRD-v1.md`
- DoD: `DoD-v1.md`
- TOML Reference: `docs/TOML_REFERENCE.md`

---

**Version**: v0.7.0
**Updated**: 2025-10-16
**Status**: Production Ready ✅
