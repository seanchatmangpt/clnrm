# Weaver Schema Reference (v1.2.1)

This reference documents the schema structure in `registry/` and how to write new schemas for clnrm v1.2.1.

## Schema Registry Structure

```
registry/
├── registry_manifest.yaml         # Registry metadata
├── core/
│   ├── test_execution.yaml        # Test execution spans
│   ├── container_lifecycle.yaml   # Container lifecycle spans
│   └── plugin_system.yaml         # Plugin execution spans
└── metrics/
    └── test_metrics.yaml          # Performance metrics
```

## Registry Manifest

**File:** `registry/registry_manifest.yaml`

```yaml
registry_id: clnrm
registry_version: 1.2.1
schemas:
  - core/test_execution.yaml
  - core/container_lifecycle.yaml
  - core/plugin_system.yaml
  - metrics/test_metrics.yaml
```

## Core Schemas

### test_execution.yaml

Defines telemetry for test execution spans.

**Critical Attributes (Cannot Be Faked):**

```yaml
groups:
  - id: span.clnrm.test_execution
    type: span
    brief: "Test execution span with hermetic isolation"
    attributes:
      - id: container.id
        type: string
        requirement_level: required
        brief: "Unique container ID proving container ran"
        examples: ["abc-123-def", "container-uuid-here"]

      - id: test.isolated
        type: boolean
        requirement_level: required
        brief: "Proves hermetic isolation (separate container per test)"
        examples: [true]

      - id: test.duration_ms
        type: int
        requirement_level: required
        brief: "Actual execution duration in milliseconds (must be >0)"
        examples: [5234, 1200, 345]

      - id: test.result
        type: string
        requirement_level: required
        brief: "Test result: pass or fail"
        examples: ["pass", "fail"]

      - id: test.name
        type: string
        requirement_level: required
        brief: "Test name"
        examples: ["test_container_execution", "integration_test"]
```

**Optional Attributes:**

```yaml
      - id: test.suite
        type: string
        requirement_level: recommended
        brief: "Test suite name"
        examples: ["integration", "e2e", "performance"]

      - id: test.assertion_count
        type: int
        requirement_level: optional
        brief: "Number of assertions in test"

      - id: test.assertions_passed
        type: int
        requirement_level: optional
        brief: "Number of assertions that passed"

      - id: test.assertions_failed
        type: int
        requirement_level: optional
        brief: "Number of assertions that failed"
```

### container_lifecycle.yaml

Defines telemetry for container lifecycle operations.

**Critical Attributes (Proves Cleanup):**

```yaml
groups:
  - id: span.clnrm.container_lifecycle
    type: span
    brief: "Container lifecycle span tracking creation to cleanup"
    attributes:
      - id: container.id
        type: string
        requirement_level: required
        brief: "Unique container ID"

      - id: container.created_at
        type: int
        requirement_level: required
        brief: "Unix timestamp when container created"
        examples: [1698765432]

      - id: container.destroyed_at
        type: int
        requirement_level: required
        brief: "Unix timestamp when container cleaned up (proves cleanup)"
        examples: [1698765467]

      - id: cleanup.success
        type: boolean
        requirement_level: required
        brief: "Whether cleanup succeeded"
        examples: [true, false]
```

**Container Details:**

```yaml
      - id: container.image.name
        type: string
        requirement_level: required
        brief: "Container image name"
        examples: ["alpine", "postgres", "nginx"]

      - id: container.image.tag
        type: string
        requirement_level: recommended
        brief: "Container image tag"
        examples: ["latest", "15-alpine", "1.23"]

      - id: container.backend
        type: string
        requirement_level: recommended
        brief: "Container backend (docker/podman)"
        examples: ["docker", "podman"]

      - id: container.state
        type: string
        requirement_level: optional
        brief: "Container state transitions"
        examples: ["created", "running", "stopped", "removed"]
```

### plugin_system.yaml

Defines telemetry for plugin execution.

```yaml
groups:
  - id: span.clnrm.plugin_execution
    type: span
    brief: "Plugin execution span"
    attributes:
      - id: plugin.name
        type: string
        requirement_level: required
        brief: "Plugin name"
        examples: ["generic_container", "surrealdb", "postgres"]

      - id: plugin.type
        type: string
        requirement_level: required
        brief: "Plugin type"
        examples: ["service", "chaos", "validation"]

      - id: plugin.state
        type: string
        requirement_level: required
        brief: "Plugin state"
        examples: ["starting", "running", "stopped"]

      - id: plugin.startup_duration_ms
        type: int
        requirement_level: recommended
        brief: "Plugin startup duration"

      - id: plugin.health_check.performed
        type: boolean
        requirement_level: optional
        brief: "Whether health check was performed"

      - id: plugin.health_check.passed
        type: boolean
        requirement_level: optional
        brief: "Whether health check passed"
```

## Metrics Schemas

### test_metrics.yaml

Defines metrics for performance tracking.

```yaml
groups:
  - id: metric.clnrm.test_duration
    type: metric
    metric_name: clnrm.test.duration
    brief: "Test execution duration histogram"
    instrument: histogram
    unit: s

  - id: metric.clnrm.test_count
    type: metric
    metric_name: clnrm.test.count
    brief: "Test execution counter"
    instrument: counter
    unit: "{test}"

  - id: metric.clnrm.container_operations
    type: metric
    metric_name: clnrm.container.operations
    brief: "Container operation counter"
    instrument: counter
    unit: "{operation}"

  - id: metric.clnrm.container_lifetime
    type: metric
    metric_name: clnrm.container.lifetime
    brief: "Container lifetime histogram"
    instrument: histogram
    unit: s
```

## Writing New Schemas

### Step 1: Create Schema File

```bash
touch registry/custom/my_feature.yaml
```

### Step 2: Define Schema Structure

```yaml
# registry/custom/my_feature.yaml
groups:
  - id: span.clnrm.my_feature
    type: span
    brief: "My feature span description"
    attributes:
      - id: my_feature.attribute
        type: string
        requirement_level: required
        brief: "Description of what this attribute proves"
        examples: ["example1", "example2"]
```

### Step 3: Add to Registry Manifest

```yaml
# registry/registry_manifest.yaml
schemas:
  - core/test_execution.yaml
  - core/container_lifecycle.yaml
  - core/plugin_system.yaml
  - metrics/test_metrics.yaml
  - custom/my_feature.yaml  # New schema
```

### Step 4: Validate Schema

```bash
weaver registry check --registry registry/
# Expected: ✅ 0 violations, 0 warnings
```

### Step 5: Implement Code

```rust
// Emit telemetry matching schema
let tracer = global::tracer("clnrm");
let mut span = tracer
    .span_builder("clnrm.my_feature")
    .with_attributes(vec![
        KeyValue::new("my_feature.attribute", "value"),
    ])
    .start(&tracer);

// ... feature logic ...

span.end();
```

### Step 6: Validate with Weaver Live-Check

```bash
weaver registry live-check --registry registry/
# Check: my_feature.attribute appears in telemetry
```

## Attribute Requirements

### requirement_level

- **`required`**: MUST be present (Weaver fails if missing)
- **`recommended`**: SHOULD be present (warning if missing)
- **`optional`**: MAY be present (no warning)

### Choosing requirement_level

```yaml
# ✅ Use 'required' for attributes that prove features work
- id: container.id
  requirement_level: required  # Cannot fake, proves container ran

# ✅ Use 'recommended' for important operational data
- id: container.image.tag
  requirement_level: recommended  # Important but not critical

# ✅ Use 'optional' for nice-to-have metadata
- id: test.assertion_count
  requirement_level: optional  # Useful but not essential
```

## Attribute Types

### Supported Types

- **`string`**: Text values
- **`int`**: Integer values
- **`double`**: Floating point values
- **`boolean`**: true/false
- **`string[]`**: Array of strings
- **`int[]`**: Array of integers
- **`double[]`**: Array of doubles

### Type Examples

```yaml
attributes:
  # String
  - id: test.name
    type: string
    examples: ["my_test"]

  # Integer
  - id: test.duration_ms
    type: int
    examples: [5234]

  # Boolean
  - id: test.isolated
    type: boolean
    examples: [true]

  # Array of strings
  - id: container.ports
    type: string[]
    examples: [["8080", "8081"]]
```

## Best Practices

### 1. Start with Required Attributes

```yaml
# ✅ GOOD: Minimal required attributes
attributes:
  - id: container.id
    requirement_level: required
  - id: test.duration_ms
    requirement_level: required
```

### 2. Add Descriptive Briefs

```yaml
# ✅ GOOD: Clear, actionable brief
- id: container.destroyed_at
  brief: "Unix timestamp when container cleaned up (proves cleanup)"

# ❌ BAD: Vague brief
- id: container.destroyed_at
  brief: "Cleanup time"
```

### 3. Provide Examples

```yaml
# ✅ GOOD: Representative examples
- id: test.result
  examples: ["pass", "fail", "skip"]

# ❌ BAD: No examples
- id: test.result
  examples: []
```

### 4. Use Semantic Naming

```yaml
# ✅ GOOD: Follows OpenTelemetry conventions
- id: container.image.name
- id: container.image.tag
- id: http.method
- id: http.status_code

# ❌ BAD: Non-standard naming
- id: containerImageName
- id: img_tag
```

## Validating Schemas

### Check Schema Syntax

```bash
weaver registry check --registry registry/
```

**Expected output:**
```
✔ `clnrm` semconv registry `registry/` loaded (200 files)
✔ No `before_resolution` policy violation
✔ `clnrm` semconv registry resolved
✔ No `after_resolution` policy violation
```

### Common Validation Errors

**Error:** `Unknown attribute type`
```yaml
# ❌ WRONG
- id: my_attr
  type: float  # Not supported

# ✅ CORRECT
- id: my_attr
  type: double  # Use 'double' not 'float'
```

**Error:** `Missing required field`
```yaml
# ❌ WRONG
- id: my_attr
  type: string
  # Missing: requirement_level

# ✅ CORRECT
- id: my_attr
  type: string
  requirement_level: required
```

## Next Steps

1. **Understand Weaver validation**: See [Weaver Schema Validation](weaver-validation.md)
2. **Learn 80/20 strategy**: See [80/20 Validation Strategy](80-20-validation.md)
3. **Prevent false positives**: See [False Positive Detection](false-positive-detection.md)
4. **Set up CI/CD**: See [Production Deployment](../production-deployment/ci-cd-integration.md)

## Further Reading

- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [Weaver Schema Specification](https://github.com/open-telemetry/weaver/tree/main/docs)
- [clnrm Schema Registry](https://github.com/seanchatmangpt/clnrm/tree/master/registry)
