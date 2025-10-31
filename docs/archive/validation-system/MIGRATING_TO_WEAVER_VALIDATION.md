# Migrating to Weaver Validation

## Overview

This guide helps you migrate existing clnrm features to use Weaver schema validation as the source of truth.

**Target:** clnrm v1.2.0 with 100% Weaver-validated features

## Why Migrate?

### Before (Traditional Testing)
```rust
#[test]
fn test_container_runs() {
    let container = create_container("alpine:latest");
    assert!(container.is_some());  // ✓ Test passes
    // But did container actually run?
    // Test just checks Option is Some!
}
```

**Problem:** Test can pass even if container didn't run.

### After (Weaver Validation)
```yaml
# Schema declares required telemetry
attributes:
  - id: container.id
    requirement_level: required
    brief: Proves container actually ran
```

```rust
// Code must emit telemetry matching schema
let container = create_container("alpine:latest").await?;
let span = trace_span!(
    "container_creation",
    container.id = %container.id()  // Required by schema
);
```

**Result:** Weaver validates container.id exists in telemetry. Cannot pass without actual container.

## Migration Process

### For Existing Features

#### Step 1: Define Schema

Create schema for your feature's telemetry:

```bash
# Create schema file
mkdir -p registry/groups
cat > registry/groups/container.yaml <<EOF
groups:
  - id: span.clnrm.container_creation
    type: span
    brief: Container creation and initialization

    attributes:
      - id: container.id
        type: string
        requirement_level: required
        brief: Unique container identifier

      - id: container.image
        type: string
        requirement_level: required
        brief: Docker image used

      - id: container.started
        type: boolean
        requirement_level: required
        brief: Whether container successfully started
EOF
```

**Key question:** What telemetry proves this feature works?

#### Step 2: Validate Schema

```bash
weaver registry check -r registry/

# Expected:
# ✅ Schema validation passed
```

#### Step 3: Generate Builders

```bash
weaver registry generate rust \
  --registry registry/ \
  --output crates/clnrm-core/src/telemetry/generated/
```

**Output:** Type-safe Rust builders for creating spans

#### Step 4: Update Code

Replace manual span creation with generated builders:

```rust
// ❌ OLD - manual span creation
let span = trace_span!(
    "container_creation",
    container.id = %container_id
);
// Easy to forget required attributes!

// ✅ NEW - generated builder
use crate::telemetry::generated::ContainerCreationSpan;

let span = ContainerCreationSpan::builder()
    .container_id(container_id)         // Required - won't compile without
    .container_image("alpine:latest")   // Required - won't compile without
    .container_started(true)            // Required - won't compile without
    .build();
```

**Benefits:**
- Compile-time enforcement of required attributes
- Type safety (can't pass string for boolean)
- Auto-complete in IDE
- Refactoring-friendly

#### Step 5: Run Tests with Validation

```bash
# Run tests and validate telemetry
clnrm run tests/ --validate

# If validation passes:
# ✅ Tests passed
# ✅ Telemetry validated
# → Feature proven to work

# If validation fails:
# ✓ Tests passed
# ✗ Telemetry validation FAILED
# → Fix violations before shipping
```

#### Step 6: Fix Violations

Check validation report:

```bash
cat validation_output/validation_report.json | jq '.violations'
```

Common fixes:

```rust
// Violation: Missing required attribute
// Fix: Ensure attribute always set
span.record("container.id", &container.id());

// Violation: Wrong type
// Fix: Use correct type
span.record("container.started", &true);  // boolean, not "true"

// Violation: Invalid enum value
// Fix: Use schema-defined values
span.record("container.state", &"running");  // not "active"
```

#### Step 7: Update Tests

Add Weaver validation to test assertions:

```rust
#[tokio::test]
async fn test_container_creation_validated() -> Result<()> {
    // Arrange
    let env = CleanroomEnvironment::new().await?;

    // Act
    let container = env.create_container("alpine:latest").await?;

    // Assert - traditional
    assert!(container.is_running());

    // Assert - Weaver validation
    let validation = env.validate_telemetry().await?;
    assert_eq!(validation.violations.len(), 0,
        "Weaver validation failed: {:?}", validation.violations);

    Ok(())
}
```

### For New Features

**ALWAYS start with schema, not code.**

#### Step 1: Define What Proves Feature Works

Before writing any code, ask:

> "What telemetry would prove this feature works?"

Example for plugin lifecycle:

```
Feature: Service plugin start/stop
Proves working when:
- Plugin assigned unique ID
- Plugin state transitions: created → starting → running → stopping → stopped
- Plugin health check passes
- No errors during lifecycle
```

#### Step 2: Write Schema First

```yaml
groups:
  - id: span.clnrm.plugin_lifecycle
    type: span
    brief: Service plugin lifecycle management

    attributes:
      - id: plugin.id
        type: string
        requirement_level: required
        brief: Unique plugin identifier

      - id: plugin.name
        type: string
        requirement_level: required
        brief: Plugin name

      - id: plugin.state
        type: enum
        requirement_level: required
        members:
          - id: created
          - id: starting
          - id: running
          - id: stopping
          - id: stopped
        brief: Current plugin state

      - id: plugin.health_ok
        type: boolean
        requirement_level: required
        brief: Whether health check passed
```

#### Step 3: Generate Builders

```bash
weaver registry generate rust --registry registry/ \
  --output crates/clnrm-core/src/telemetry/generated/
```

#### Step 4: Write Tests Using Builders

```rust
#[tokio::test]
async fn test_plugin_lifecycle() -> Result<()> {
    use crate::telemetry::generated::PluginLifecycleSpan;

    // Arrange
    let env = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test", "alpine:latest");

    // Act & instrument with generated builder
    let span = PluginLifecycleSpan::builder()
        .plugin_id("test-123")
        .plugin_name("test")
        .plugin_state("starting")
        .plugin_health_ok(false)  // Not started yet
        .build();

    let _guard = span.enter();
    let handle = env.start_plugin(plugin).await?;

    // Update state
    span.record("plugin.state", &"running");
    span.record("plugin.health_ok", &true);

    // Assert
    assert!(handle.is_running());

    // Weaver validation
    let validation = env.validate_telemetry().await?;
    assert_eq!(validation.violations.len(), 0);

    Ok(())
}
```

#### Step 5: Implement Using Builders

```rust
impl ServicePlugin for GenericContainerPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        use crate::telemetry::generated::PluginLifecycleSpan;

        // Create span with required attributes
        let span = PluginLifecycleSpan::builder()
            .plugin_id(&self.id)
            .plugin_name(&self.name)
            .plugin_state("starting")
            .plugin_health_ok(false)
            .build();

        let _guard = span.enter();

        // Start container
        let container = self.backend.create_container(&self.image)?;

        // Update state
        span.record("plugin.state", &"running");

        // Health check
        let health_ok = self.health_check().is_ok();
        span.record("plugin.health_ok", &health_ok);

        Ok(ServiceHandle { container, plugin_id: self.id.clone() })
    }
}
```

#### Step 6: Validate

```bash
clnrm run tests/ --validate
```

**Success criteria:**
- ✅ Tests pass
- ✅ Zero violations
- ✅ Feature proven to work

## Migration Checklist

### Per Feature

- [ ] Schema written and validated (`weaver registry check`)
- [ ] Builders generated (`weaver registry generate rust`)
- [ ] Code updated to use generated builders
- [ ] Tests run with `--validate` flag
- [ ] Zero violations in validation report
- [ ] All required attributes present in telemetry
- [ ] Documentation updated

### Per Release

- [ ] All features have schemas
- [ ] All tests use Weaver validation
- [ ] CI/CD runs `--validate` and blocks on failures
- [ ] Zero test-only validation (all via Weaver)
- [ ] Schema registry version tagged

## Common Migration Challenges

### Challenge 1: Existing Tests Break

**Problem:** Tests pass before Weaver, fail after.

**Root cause:** Tests had false positives.

**Solution:**
1. Check validation report for violations
2. Fix code to emit required telemetry
3. Or update schema if requirement too strict

### Challenge 2: Too Many Required Attributes

**Problem:** Schema requires 20 attributes, hard to always set all.

**Root cause:** Over-specified schema.

**Solution:**
1. Distinguish critical vs nice-to-have
2. Make only critical attributes required
3. Make nice-to-have optional

```yaml
# Critical - proves feature works
- id: container.id
  requirement_level: required

# Nice to have - debugging info
- id: container.runtime_version
  requirement_level: optional
```

### Challenge 3: Generated Builders Complex

**Problem:** Builder API has too many methods, hard to use.

**Root cause:** Schema has too many optional attributes.

**Solution:**
1. Group related attributes into separate spans
2. Use builder pattern with defaults
3. Provide convenience constructors

```rust
// ❌ Too complex
let span = TestExecutionSpan::builder()
    .container_id(id)
    .test_isolated(true)
    .test_result("pass")
    .test_duration_ms(123.0)
    .test_retry_count(0)
    .test_config_file("test.toml")
    .build();

// ✅ Convenience constructor
let span = TestExecutionSpan::basic(container_id, "pass");
// Sets required attributes with sensible defaults
```

### Challenge 4: Validation Too Slow

**Problem:** `--validate` adds 30% overhead to test runs.

**Root cause:** Validation processes all telemetry.

**Solution:**
1. Run validation in CI, not locally
2. Cache validation results for unchanged tests
3. Validate incrementally (only changed tests)

```bash
# Local: skip validation for speed
clnrm run tests/

# CI: always validate
clnrm run tests/ --validate
```

## Migration Examples

### Example 1: Container Execution

**Before:**
```rust
#[test]
fn test_container_execution() {
    let env = CleanroomEnvironment::new();
    let result = env.execute_command(&["echo", "hello"]);
    assert!(result.is_ok());  // Could pass even if command failed!
}
```

**After:**
```yaml
# Schema
attributes:
  - id: container.id
    requirement_level: required
  - id: command.exit_code
    requirement_level: required
  - id: command.output
    requirement_level: required
```

```rust
#[tokio::test]
async fn test_container_execution_validated() -> Result<()> {
    use crate::telemetry::generated::CommandExecutionSpan;

    let env = CleanroomEnvironment::new().await?;
    let container = env.create_container("alpine:latest").await?;

    let span = CommandExecutionSpan::builder()
        .container_id(container.id())
        .command_exit_code(0)
        .command_output("hello")
        .build();

    let _guard = span.enter();
    let result = env.execute_command(&container, &["echo", "hello"]).await?;

    assert_eq!(result.exit_code, 0);
    assert_eq!(result.stdout, "hello\n");

    // Weaver validates container.id, exit_code, output all present
    let validation = env.validate_telemetry().await?;
    assert_eq!(validation.violations.len(), 0);

    Ok(())
}
```

### Example 2: Plugin Registration

**Before:**
```rust
#[test]
fn test_plugin_registration() {
    let env = CleanroomEnvironment::new();
    env.register_plugin(Box::new(SurrealDBPlugin::new()));
    assert_eq!(env.plugin_count(), 1);  // Just checks count!
}
```

**After:**
```yaml
# Schema
attributes:
  - id: plugin.id
    requirement_level: required
  - id: plugin.type
    requirement_level: required
  - id: plugin.registered
    requirement_level: required
```

```rust
#[tokio::test]
async fn test_plugin_registration_validated() -> Result<()> {
    use crate::telemetry::generated::PluginRegistrationSpan;

    let env = CleanroomEnvironment::new().await?;
    let plugin = Box::new(SurrealDBPlugin::new("test-db"));

    let span = PluginRegistrationSpan::builder()
        .plugin_id("test-db")
        .plugin_type("surrealdb")
        .plugin_registered(true)
        .build();

    let _guard = span.enter();
    env.register_plugin(plugin).await?;

    // Weaver validates plugin actually registered with correct ID/type
    let validation = env.validate_telemetry().await?;
    assert_eq!(validation.violations.len(), 0);

    Ok(())
}
```

## Next Steps

1. **Read schemas**: See existing schemas in `registry/` for examples
2. **Try migration**: Pick one small feature to migrate first
3. **Run validation**: Experience Weaver validation firsthand
4. **Report issues**: File issues for migration challenges
5. **Help others**: Share your migration experience

## Resources

- [Weaver User Guide](WEAVER_USER_GUIDE.md) - Using Weaver validation
- [Schema Writing Guide](SCHEMA_WRITING_GUIDE.md) - Writing schemas
- [Weaver Integration Plan](WEAVER_INTEGRATION_PLAN.md) - Technical details
- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
