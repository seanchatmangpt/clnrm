# Writing Telemetry Schemas for clnrm

## Overview

Telemetry schemas define the contract between your code and Weaver validation. A well-written schema proves that features work by declaring exactly what telemetry they must produce.

## Principles

### 1. Required attributes prove behavior

If an attribute is critical to proving the feature works, make it **required**.

```yaml
# ✅ GOOD - container.id proves container actually ran
attributes:
  - id: container.id
    type: string
    requirement_level: required
    brief: ID of container that ran test

# ❌ BAD - optional doesn't prove anything
attributes:
  - id: container.id
    type: string
    requirement_level: optional  # Test could pass without container!
```

**Rule:** If feature cannot work without this attribute, it must be **required**.

### 2. Enums prevent invalid values

Don't use arbitrary strings when you know all possible values.

```yaml
# ✅ GOOD - constrained enum
attributes:
  - id: test.result
    type: enum
    requirement_level: required
    members:
      - id: pass
        value: "pass"
      - id: fail
        value: "fail"
      - id: error
        value: "error"
    brief: Test execution result

# ❌ BAD - arbitrary string
attributes:
  - id: test.result
    type: string  # Could be anything: "success", "ok", "passed", etc.
```

**Rule:** If values are known and finite, use enum.

### 3. Types enforce correctness

Use the correct type. Don't use string for everything.

```yaml
# ✅ GOOD - correct types
attributes:
  - id: test.isolated
    type: boolean  # true/false
  - id: test.duration_ms
    type: double   # 123.45
  - id: test.retry_count
    type: int      # 3

# ❌ BAD - everything is string
attributes:
  - id: test.isolated
    type: string   # "true" or "false"? Ambiguous!
  - id: test.duration_ms
    type: string   # "123.45" - can't do math on strings
```

**Rule:** Use the most specific type that represents the data.

### 4. Brief is documentation

Every attribute needs clear documentation.

```yaml
# ✅ GOOD - clear documentation
attributes:
  - id: container.id
    type: string
    requirement_level: required
    brief: Unique identifier of the Docker container that executed the test
    note: |
      This attribute proves that the test actually ran in an isolated container.
      If this attribute is missing, the test may have run on the host system
      instead of in a container, violating hermetic isolation.

# ❌ BAD - no context
attributes:
  - id: container.id
    type: string
    brief: Container ID
```

**Rule:** Explain WHY the attribute matters, not just WHAT it is.

## Example: Test Execution Schema

Here's a complete example that proves a test ran in an isolated container:

```yaml
groups:
  - id: span.clnrm.test_execution
    type: span
    brief: Represents a single test execution in an isolated container
    note: |
      This span captures the complete lifecycle of a test execution.
      Required attributes prove hermetic isolation and proper execution.

    attributes:
      # REQUIRED: Proves container actually ran
      - id: container.id
        type: string
        requirement_level: required
        brief: Unique identifier of the Docker container
        note: |
          Presence of this attribute proves the test ran in a container.
          If missing, test may have run on host (not isolated).
        examples:
          - "a3f5b8c9d2e1"
          - "test-container-12345"

      # REQUIRED: Proves isolation worked
      - id: test.isolated
        type: boolean
        requirement_level: required
        brief: Whether test ran in isolated environment
        note: |
          Must be true for hermetic testing. False indicates
          test shared state with other tests or host system.

      # REQUIRED: Proves test executed
      - id: test.result
        type: enum
        requirement_level: required
        members:
          - id: pass
            value: "pass"
            brief: Test passed all assertions
          - id: fail
            value: "fail"
            brief: Test failed at least one assertion
          - id: error
            value: "error"
            brief: Test encountered runtime error
        brief: Test execution result
        note: |
          Captures the final outcome of test execution.
          Distinguished from test.status which tracks span status.

      # REQUIRED: Proves which test ran
      - id: test.name
        type: string
        requirement_level: required
        brief: Name of the test that executed
        examples:
          - "test_container_isolation"
          - "test_plugin_lifecycle"

      # OPTIONAL: Nice to have but not critical
      - id: test.duration_ms
        type: double
        requirement_level: optional
        brief: Test execution duration in milliseconds
        examples:
          - 123.45
          - 5432.1

      # OPTIONAL: Additional context
      - id: test.retry_count
        type: int
        requirement_level: optional
        brief: Number of retries before test succeeded or failed
        examples:
          - 0
          - 3

      # OPTIONAL: Debugging info
      - id: test.config_file
        type: string
        requirement_level: optional
        brief: Path to TOML configuration file
        examples:
          - "tests/basic_test.clnrm.toml"
          - "/home/user/my_test.toml"
```

## Common Patterns

### Pattern 1: Lifecycle Spans

For operations with start/end, track critical phases:

```yaml
groups:
  - id: span.clnrm.container_lifecycle
    type: span
    brief: Container creation, execution, and cleanup

    attributes:
      - id: container.id
        type: string
        requirement_level: required
        brief: Container identifier

      - id: container.state
        type: enum
        requirement_level: required
        members:
          - id: creating
          - id: running
          - id: stopping
          - id: stopped
        brief: Current container state

      - id: container.image
        type: string
        requirement_level: required
        brief: Docker image used
        examples:
          - "alpine:latest"
          - "postgres:14"
```

### Pattern 2: Error Tracking

Always capture error details:

```yaml
attributes:
  - id: error.type
    type: string
    requirement_level:
      conditionally_required: "if span status is ERROR"
    brief: Type of error that occurred
    examples:
      - "ContainerStartupError"
      - "TimeoutError"

  - id: error.message
    type: string
    requirement_level:
      conditionally_required: "if span status is ERROR"
    brief: Human-readable error message

  - id: error.stack
    type: string
    requirement_level: optional
    brief: Stack trace if available
```

### Pattern 3: Resource Attributes

For services and plugins:

```yaml
attributes:
  - id: service.name
    type: string
    requirement_level: required
    brief: Name of the service plugin
    examples:
      - "surrealdb"
      - "ollama"

  - id: service.version
    type: string
    requirement_level: optional
    brief: Version of the service

  - id: service.port
    type: int
    requirement_level: required
    brief: Port the service listens on
    examples:
      - 8000
      - 11434
```

## Validation

Always validate your schema before using:

```bash
# Check schema syntax and semantics
weaver registry check -r registry/

# Expected output if valid:
# ✅ Schema validation passed
# Registry contains 5 groups, 23 attributes

# If invalid:
# ❌ Schema validation failed
# Error: Duplicate attribute ID 'container.id' in group 'span.clnrm.test_execution'
```

## Common Schema Mistakes

### Mistake 1: Optional Critical Attributes

```yaml
# ❌ WRONG - container.id optional
attributes:
  - id: container.id
    requirement_level: optional
# Test could pass without container running!

# ✅ CORRECT - required
attributes:
  - id: container.id
    requirement_level: required
```

### Mistake 2: String for Everything

```yaml
# ❌ WRONG - using strings for structured data
attributes:
  - id: test.isolated
    type: string  # "true" or "false"?
  - id: test.duration_ms
    type: string  # "123" - loses type safety

# ✅ CORRECT - proper types
attributes:
  - id: test.isolated
    type: boolean
  - id: test.duration_ms
    type: double
```

### Mistake 3: Missing Documentation

```yaml
# ❌ WRONG - no context
attributes:
  - id: foo
    type: string

# ✅ CORRECT - clear documentation
attributes:
  - id: container.id
    type: string
    requirement_level: required
    brief: Unique identifier of container that executed test
    note: Proves test ran in isolated container environment
```

### Mistake 4: Ambiguous Enums

```yaml
# ❌ WRONG - overlapping meanings
attributes:
  - id: status
    type: enum
    members:
      - id: ok        # Is this pass or running?
      - id: success   # Same as ok?
      - id: good      # Same as success?

# ✅ CORRECT - distinct meanings
attributes:
  - id: test.result
    type: enum
    members:
      - id: pass      # Test assertions passed
      - id: fail      # Test assertions failed
      - id: error     # Runtime error occurred
```

## Schema Evolution

### Adding Optional Attributes

Safe - backward compatible:

```yaml
# v1.0.0
attributes:
  - id: container.id
    requirement_level: required

# v1.1.0 - add optional attribute
attributes:
  - id: container.id
    requirement_level: required
  - id: container.runtime  # NEW - optional
    requirement_level: optional
    type: string
```

### Adding Required Attributes

**BREAKING CHANGE** - old telemetry will fail validation:

```yaml
# v1.0.0
attributes:
  - id: container.id
    requirement_level: required

# v2.0.0 - BREAKING - new required attribute
attributes:
  - id: container.id
    requirement_level: required
  - id: test.isolated  # NEW - REQUIRED
    requirement_level: required  # ← BREAKS v1.0.0 telemetry
    type: boolean
```

**Migration path:**
1. Add as optional in v1.1.0
2. Update all code to set it
3. Make required in v2.0.0

### Changing Types

**BREAKING CHANGE** - always requires major version bump:

```yaml
# v1.0.0
attributes:
  - id: test.duration_ms
    type: int

# v2.0.0 - BREAKING - type change
attributes:
  - id: test.duration_ms
    type: double  # ← BREAKS existing telemetry
```

## Testing Schemas

### Unit Test: Schema Validity

```bash
weaver registry check -r registry/ || exit 1
```

### Integration Test: Live Validation

```bash
# Run tests and validate telemetry matches schema
clnrm run tests/ --validate || exit 1
```

### Regression Test: No Breaking Changes

```bash
# Compare schema versions
weaver registry diff -r registry/ --base v1.0.0 --head v1.1.0

# Expect:
# No breaking changes detected
# Added 3 optional attributes
# No removed attributes
```

## Best Practices Summary

1. **Start with required attributes** - What MUST be true for feature to work?
2. **Use specific types** - Boolean for flags, enum for known values, int/double for numbers
3. **Document the why** - Explain why attribute matters, not just what it is
4. **Validate early** - Run `weaver registry check` before committing
5. **Version carefully** - Required attributes and type changes are breaking changes
6. **Test with real telemetry** - Run live-check to verify schema matches reality

## References

- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [Weaver Schema Format](https://github.com/open-telemetry/weaver)
- [clnrm Telemetry Registry](../registry/) - See existing schemas for examples
