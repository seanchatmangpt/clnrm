# Fake-Green Detection TOML Schema Reference

## Table of Contents

1. [Overview](#overview)
2. [`[expect]` Root Section](#expect-root-section)
3. [`[[expect.span]]` - Span Expectations](#expectspan---span-expectations)
4. [`[expect.graph]` - Graph Structure](#expectgraph---graph-structure)
5. [`[expect.counts]` - Cardinality Validation](#expectcounts---cardinality-validation)
6. [`[[expect.window]]` - Window Containment](#expectwindow---window-containment)
7. [`[expect.order]` - Temporal Ordering](#expectorder---temporal-ordering)
8. [`[expect.status]` - Status Validation](#expectstatus---status-validation)
9. [`[expect.hermeticity]` - Hermeticity Validation](#expecthermeticity---hermeticity-validation)
10. [Complete Example](#complete-example)

---

## Overview

The `[expect]` section defines validation rules for fake-green detection. All fields are optional, but using multiple validators provides stronger guarantees.

**Schema Version:** 1.0.0

**File Format:** TOML (`.clnrm.toml`)

---

## `[expect]` Root Section

The top-level section containing all validation expectations.

### Structure

```toml
[expect]
# Span expectations (array of tables)
[[expect.span]]
# ... span expectation fields

# Graph topology expectations (single table)
[expect.graph]
# ... graph fields

# Count expectations (single table)
[expect.counts]
# ... count fields

# Window containment expectations (array of tables)
[[expect.window]]
# ... window fields

# Ordering expectations (single table)
[expect.order]
# ... ordering fields

# Status expectations (single table)
[expect.status]
# ... status fields

# Hermeticity expectations (single table)
[expect.hermeticity]
# ... hermeticity fields
```

### Validation Rules

- **All sections are optional**: Only configured validators run
- **Validators are independent**: Each validator runs regardless of others' results
- **Order doesn't matter**: Validators run in deterministic order internally
- **Failures short-circuit**: First validation failure marks test as FAIL

---

## `[[expect.span]]` - Span Expectations

**Purpose:** Validate that specific spans exist with expected attributes, events, and metadata.

**Type:** Array of tables (can have multiple `[[expect.span]]` sections)

### Fields

#### `name` (string, required)

Span name to validate.

**Examples:**
```toml
name = "clnrm.run"
name = "container.exec"
name = "http.request"
```

#### `kind` (string, optional)

Expected span kind.

**Valid values:** `"client"`, `"server"`, `"producer"`, `"consumer"`, `"internal"`

**Example:**
```toml
kind = "internal"
```

#### `parent` (string, optional)

Expected parent span name.

**Example:**
```toml
name = "child_span"
parent = "parent_span"
```

#### `attrs.all` (table, optional)

All specified attributes must exist with matching values (substring match).

**Example:**
```toml
[attrs.all]
"result" = "pass"
"test.name" = "integration"
"http.status_code" = "200"
```

#### `attrs.any` (table, optional)

At least one of the specified attributes must exist.

**Example:**
```toml
[attrs.any]
"error" = "true"
"failure" = "true"
```

#### `events.any` (array, optional)

Span must contain at least one of these events.

**Example:**
```toml
events.any = ["container.start", "container.exec", "container.stop"]
```

#### `events.all` (array, optional)

Span must contain all of these events.

**Example:**
```toml
events.all = ["request.start", "request.end"]
```

#### `duration_ms` (table, optional)

Expected span duration range in milliseconds.

**Fields:**
- `min` (integer): Minimum duration
- `max` (integer): Maximum duration

**Example:**
```toml
[duration_ms]
min = 10
max = 5000
```

### Complete Example

```toml
[[expect.span]]
name = "clnrm.run"
kind = "internal"
attrs.all = { "result" = "pass", "test.name" = "my_test" }
duration_ms = { min = 100, max = 30000 }

[[expect.span]]
name = "container.exec"
kind = "internal"
parent = "clnrm.run"
events.any = ["container.start", "container.exec", "container.stop"]

[[expect.span]]
name = "http.request"
kind = "server"
attrs.all = { "http.status_code" = "200" }
duration_ms = { min = 10, max = 1000 }
```

### Validation Rules

- Span with matching `name` must exist
- If `kind` specified, span kind must match exactly
- If `parent` specified, parent span must exist with correct relationship
- All `attrs.all` entries must exist and match (substring)
- At least one `attrs.any` entry must exist
- All `events.all` must be present
- At least one `events.any` must be present
- Duration must fall within `duration_ms` range

---

## `[expect.graph]` - Graph Structure

**Purpose:** Validate parent-child relationships between spans.

**Type:** Single table

### Fields

#### `must_include` (array of arrays, optional)

Required edges in the span graph. Each inner array has 2 elements: `[parent_name, child_name]`.

**Example:**
```toml
must_include = [
    ["clnrm.run", "clnrm.step"],
    ["clnrm.step", "container.exec"]
]
```

**Validation:** At least one parent span with `parent_name` must have a child span with `child_name`.

#### `must_not_cross` (array of arrays, optional)

Forbidden edges in the span graph.

**Example:**
```toml
must_not_cross = [
    ["child_a", "child_b"]  # These spans must not have parent-child relationship
]
```

#### `acyclic` (boolean, optional)

If `true`, validates that span graph has no cycles.

**Example:**
```toml
acyclic = true
```

### Complete Example

```toml
[expect.graph]
must_include = [
    ["clnrm.run", "clnrm.step:hello"],
    ["clnrm.run", "clnrm.step:world"]
]
must_not_cross = [
    ["clnrm.step:hello", "clnrm.step:world"]
]
acyclic = true
```

### Validation Rules

- All `must_include` edges must exist
- None of `must_not_cross` edges can exist
- If `acyclic = true`, graph must be acyclic (no cycles)

---

## `[expect.counts]` - Cardinality Validation

**Purpose:** Validate span counts (total, by name, errors, events).

**Type:** Single table

### Fields

#### `spans_total` (table, optional)

Expected total span count bounds.

**Sub-fields:**
- `gte` (integer): Greater than or equal to
- `lte` (integer): Less than or equal to
- `eq` (integer): Exactly equal to

**Examples:**
```toml
# At least 2 spans
[spans_total]
gte = 2

# Between 5 and 10 spans
[spans_total]
gte = 5
lte = 10

# Exactly 7 spans
[spans_total]
eq = 7
```

#### `events_total` (table, optional)

Expected total event count across all spans.

**Example:**
```toml
[events_total]
gte = 1
```

#### `errors_total` (table, optional)

Expected error span count (spans with error status).

**Example:**
```toml
[errors_total]
eq = 0  # No errors expected
```

#### `by_name` (table, optional)

Per-span-name count constraints.

**Example:**
```toml
[by_name]
"clnrm.run" = { eq = 1 }
"clnrm.step" = { gte = 1, lte = 10 }
"container.exec" = { gte = 1 }
```

### Complete Example

```toml
[expect.counts]

[spans_total]
gte = 2
lte = 100

[events_total]
gte = 5

[errors_total]
eq = 0

[by_name]
"clnrm.run" = { eq = 1 }
"clnrm.step" = { gte = 1 }
"container.exec" = { gte = 1 }
```

### Validation Rules

- `spans_total`: Actual span count must satisfy bounds
- `events_total`: Actual event count must satisfy bounds
- `errors_total`: Actual error count must satisfy bounds
- `by_name`: Each span name count must satisfy its bounds
- If `eq` is specified, `gte` and `lte` are ignored
- If only `gte` specified, no upper bound
- If only `lte` specified, no lower bound

---

## `[[expect.window]]` - Window Containment

**Purpose:** Validate that child spans are temporally contained within parent spans.

**Type:** Array of tables (can have multiple `[[expect.window]]` sections)

### Fields

#### `outer` (string, required)

Name of the outer (parent) span.

**Example:**
```toml
outer = "clnrm.run"
```

#### `contains` (array, required)

Names of child spans that must be temporally contained.

**Example:**
```toml
contains = ["clnrm.step", "container.exec"]
```

### Complete Example

```toml
[[expect.window]]
outer = "clnrm.run"
contains = ["plugin.registry", "clnrm.step"]

[[expect.window]]
outer = "clnrm.step"
contains = ["container.start", "container.exec", "container.stop"]
```

### Validation Rules

- Outer span must exist
- All child spans in `contains` must exist
- For each child:
  - `outer.start_time <= child.start_time`
  - `child.end_time <= outer.end_time`
- Timestamps must be present on all spans

---

## `[expect.order]` - Temporal Ordering

**Purpose:** Validate temporal ordering constraints between spans.

**Type:** Single table

### Fields

#### `must_precede` (array of arrays, optional)

First span must complete before second span starts. Each inner array: `[first, second]`.

**Example:**
```toml
must_precede = [
    ["plugin.registry", "container.start"],
    ["container.start", "test.execute"]
]
```

**Validation:** `first.end_time <= second.start_time`

#### `must_follow` (array of arrays, optional)

First span must start after second span completes. Each inner array: `[first, second]`.

**Example:**
```toml
must_follow = [
    ["container.stop", "test.execute"]
]
```

**Validation:** `first.start_time >= second.end_time`

### Complete Example

```toml
[expect.order]
must_precede = [
    ["plugin.registry", "clnrm.step"],
    ["container.start", "container.exec"]
]
must_follow = [
    ["container.stop", "container.exec"]
]
```

### Validation Rules

- Both spans in each pair must exist
- For `must_precede`: First span must complete before second starts
- For `must_follow`: First span must start after second completes
- All spans must have timestamps

---

## `[expect.status]` - Status Validation

**Purpose:** Validate OTEL span status codes (OK, ERROR, UNSET).

**Type:** Single table

### Fields

#### `all` (string, optional)

Expected status for all spans.

**Valid values:** `"OK"`, `"ERROR"`, `"UNSET"` (case-insensitive)

**Example:**
```toml
all = "OK"
```

#### `by_name` (table, optional)

Expected status by span name pattern (supports glob patterns).

**Example:**
```toml
[by_name]
"clnrm.*" = "OK"
"http.request" = "OK"
"error_handler" = "ERROR"
```

### Complete Example

```toml
[expect.status]
all = "OK"

[by_name]
"clnrm.*" = "OK"
"http.*" = "OK"
"test_*" = "OK"
```

### Validation Rules

- If `all` specified, every span must have that status
- For each `by_name` entry:
  - Pattern is glob (supports `*`, `?`, `[]`)
  - All matching spans must have specified status
  - At least one span must match pattern
- Status is case-insensitive
- Default status is `UNSET` if not specified in span

---

## `[expect.hermeticity]` - Hermeticity Validation

**Purpose:** Validate test isolation and hermetic execution.

**Type:** Single table

### Fields

#### `no_external_services` (boolean, optional)

If `true`, validates no external network services were accessed.

**Example:**
```toml
no_external_services = true
```

**Checks for attributes:**
- `net.peer.name`
- `net.peer.ip`
- `net.peer.port`
- `http.host`
- `http.url`
- `db.connection_string`
- `rpc.service`
- `messaging.destination`
- `messaging.url`

#### `resource_attrs.must_match` (table, optional)

Required resource attributes that must match exactly.

**Example:**
```toml
[resource_attrs.must_match]
"service.name" = "clnrm"
"deployment.environment" = "test"
"env" = "ci"
```

#### `span_attrs.forbid_keys` (array, optional)

Attribute keys that must NOT appear in any span.

**Example:**
```toml
[span_attrs.forbid_keys]
forbid_keys = ["net.peer.name", "http.url", "db.connection_string"]
```

### Complete Example

```toml
[expect.hermeticity]
no_external_services = true

[resource_attrs.must_match]
"service.name" = "clnrm"
"env" = "test"

[span_attrs.forbid_keys]
forbid_keys = ["net.peer.name", "http.url"]
```

### Validation Rules

- If `no_external_services = true`:
  - No span can contain external network attributes
- For `resource_attrs.must_match`:
  - All specified attributes must exist in resource attributes
  - Values must match exactly
- For `span_attrs.forbid_keys`:
  - None of the specified keys can appear in any span

---

## Complete Example

A comprehensive test configuration with all 7 validators:

```toml
# Complete fake-green detection configuration
[meta]
name = "comprehensive_validation_test"
version = "1.0.0"
description = "Demonstrates all 7 fake-green detection layers"

[otel]
exporter = "otlp"
endpoint = "http://localhost:4318"
protocol = "http/protobuf"
sample_ratio = 1.0
resources = { "service.name" = "clnrm", "env" = "test" }

[service.test_app]
plugin = "generic_container"
image = "alpine:latest"
env.OTEL_EXPORTER_OTLP_ENDPOINT = "http://localhost:4318"

[[scenario]]
name = "integration_test"
service = "test_app"
run = "echo 'test passed'"

# ============================================================================
# FAKE-GREEN DETECTION: 7 VALIDATION LAYERS
# ============================================================================

[expect]

# Layer 1: Lifecycle Events
[[expect.span]]
name = "container.exec"
kind = "internal"
events.any = ["container.start", "container.exec", "container.stop"]
attrs.all = { "test.name" = "integration_test" }

[[expect.span]]
name = "clnrm.run"
kind = "internal"
attrs.all = { "result" = "pass" }
duration_ms = { min = 100, max = 30000 }

# Layer 2: Graph Structure
[expect.graph]
must_include = [
    ["clnrm.run", "clnrm.step"],
    ["clnrm.step", "container.exec"]
]
acyclic = true

# Layer 3: Span Counts
[expect.counts]

[spans_total]
gte = 3

[errors_total]
eq = 0

[by_name]
"clnrm.run" = { eq = 1 }
"clnrm.step" = { gte = 1 }
"container.exec" = { gte = 1 }

# Layer 4: Temporal Ordering
[expect.order]
must_precede = [
    ["plugin.registry", "clnrm.step"],
    ["container.start", "container.exec"]
]

# Layer 5: Window Containment
[[expect.window]]
outer = "clnrm.run"
contains = ["clnrm.step"]

[[expect.window]]
outer = "clnrm.step"
contains = ["container.exec"]

# Layer 6: Status Validation
[expect.status]
all = "OK"

[by_name]
"clnrm.*" = "OK"
"container.*" = "OK"

# Layer 7: Hermeticity Validation
[expect.hermeticity]
no_external_services = true

[resource_attrs.must_match]
"service.name" = "clnrm"
"env" = "test"

[span_attrs.forbid_keys]
forbid_keys = ["net.peer.name", "http.url"]

# ============================================================================
# DETERMINISM & REPORTING
# ============================================================================

[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"

[report]
json = "report.json"
digest = "trace.sha256"
```

---

## Schema Validation

### Required Fields by Section

| Section | Required Fields | Optional Fields |
|---------|----------------|-----------------|
| `[[expect.span]]` | `name` | `kind`, `parent`, `attrs.*`, `events.*`, `duration_ms` |
| `[expect.graph]` | none | `must_include`, `must_not_cross`, `acyclic` |
| `[expect.counts]` | none | `spans_total`, `events_total`, `errors_total`, `by_name` |
| `[[expect.window]]` | `outer`, `contains` | none |
| `[expect.order]` | none | `must_precede`, `must_follow` |
| `[expect.status]` | none | `all`, `by_name` |
| `[expect.hermeticity]` | none | `no_external_services`, `resource_attrs`, `span_attrs` |

### Type Reference

| Type | TOML Example | Description |
|------|-------------|-------------|
| `string` | `"value"` | Text value |
| `integer` | `42` | Whole number |
| `boolean` | `true` or `false` | Boolean value |
| `array` | `["a", "b"]` | List of values |
| `table` | `{ key = "value" }` | Key-value pairs |
| `array of arrays` | `[["a", "b"], ["c", "d"]]` | Nested arrays |

---

## Migration Guide

### From v0.6.0 to v1.0.0

**Changes:**
1. `[test.metadata]` → `[meta]`
2. `[services.*]` → `[service.*]`
3. `[[steps]]` → `[[scenario]]`
4. Added `[expect]` section
5. All validators now optional

**Before (v0.6.0):**
```toml
[test.metadata]
name = "test"

[services.app]
type = "generic_container"

[[steps]]
name = "run"
command = ["echo", "hi"]
```

**After (v1.0.0):**
```toml
[meta]
name = "test"

[service.app]
plugin = "generic_container"

[[scenario]]
name = "run"
run = "echo hi"

# Add fake-green detection
[expect.status]
all = "OK"
```

---

## Best Practices

1. **Use all 7 validators** for maximum confidence
2. **Start with span and status validators** (easiest to configure)
3. **Add graph and counts next** (moderate complexity)
4. **Finish with order, window, hermeticity** (advanced)
5. **Test your expectations** on known-good and known-bad traces
6. **Document why each expectation exists** (comments in TOML)
7. **Use specific span names** (avoid wildcards where possible)
8. **Keep expectations maintainable** (don't over-specify)

---

## Troubleshooting

### Common Errors

**"Expected span not found"**
- Ensure OTEL exporter is configured
- Check span name matches exactly (case-sensitive)
- Verify instrumentation is enabled

**"Invalid status code"**
- Use only: `"OK"`, `"ERROR"`, `"UNSET"`
- Status is case-insensitive but must be quoted

**"Invalid glob pattern"**
- Use valid glob syntax: `*`, `?`, `[]`
- Escape special characters if needed

**"Resource attribute missing"**
- Check resource attributes in traces
- Verify OTEL SDK configuration
- Ensure resource attributes are set before span creation

---

## See Also

- [User Guide](FAKE_GREEN_DETECTION_USER_GUIDE.md) - How to use fake-green detection
- [Developer Guide](FAKE_GREEN_DETECTION_DEV_GUIDE.md) - How to add custom validators
- [CLI Reference](CLI_ANALYZE_REFERENCE.md) - Command-line usage

**Questions?** See [documentation](.) or file an issue.
