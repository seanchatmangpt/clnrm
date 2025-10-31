# CLNRM Telemetry Schema Registry

This directory contains the complete OpenTelemetry semantic convention schemas for the Cleanroom Testing Framework (clnrm).

## Purpose

**clnrm exists to eliminate false positives.** This schema registry enables us to validate clnrm itself without falling into the false positive trap by:

1. Defining telemetry that PROVES runtime behavior
2. Using Weaver to validate actual telemetry against schemas
3. Making it impossible to fake implementation with stubs

## Directory Structure

```
registry/
├── registry_manifest.yaml      # Registry metadata and configuration
├── core/                        # Core behavior schemas
│   ├── test_execution.yaml     # Test execution spans
│   ├── container_lifecycle.yaml # Container lifecycle spans
│   └── plugin_system.yaml      # Plugin system spans
├── metrics/                     # Metrics schemas
│   └── test_metrics.yaml       # Test and container metrics
├── events/                      # Event schemas
│   └── test_events.yaml        # Test lifecycle events
├── VALIDATION_STRATEGY.md      # Complete validation strategy
└── README.md                   # This file
```

## Core Schemas

### test_execution.yaml

Defines `span.clnrm.test_execution` - the PRIMARY proof that clnrm works.

**Critical Attributes (all required):**
- `container.id` - CANNOT exist without real container
- `test.isolated` - Must be true, proves hermetic isolation
- `test.result` - Must be set, proves execution completed
- `test.duration_ms` - Must be > 0, proves actual execution time

**What it proves:**
- Container actually ran
- Test executed in isolation
- Test ran to completion
- Real execution time captured

### container_lifecycle.yaml

Defines `span.clnrm.container_lifecycle` - proves container creation and cleanup.

**Critical Attributes:**
- `container.created_at` - Proves creation happened
- `container.destroyed_at` - Proves cleanup happened
- `cleanup.success` - Must be true

**What it proves:**
- Containers are created
- Containers are cleaned up
- No resource leaks
- Complete lifecycle management

### plugin_system.yaml

Defines `span.clnrm.plugin_execution` and `span.clnrm.service_command`.

**Critical Attributes:**
- `plugin.state` - State transitions prove lifecycle
- `plugin.health_check.performed` - Proves health checking works
- `command.exit_code` - Proves commands actually execute

**What it proves:**
- Plugin system works
- Services start and stop
- Health checks execute
- Commands run in containers

## Metrics

### Test Metrics

- `clnrm.test.duration` (histogram) - Test execution time distribution
- `clnrm.test.count` (counter) - Tests executed by result
- `clnrm.container.count` (counter) - Containers by state
- `clnrm.container.lifetime` (histogram) - Container lifetime distribution
- `clnrm.plugin.operations` (counter) - Plugin operations by result
- `clnrm.isolation.score` (gauge) - Isolation quality (must be 1.0)

**Validation Strategy:**
- `created` count MUST equal `destroyed` count (no leaks)
- `isolation.score` MUST be 1.0 (perfect isolation)
- Duration distributions prove consistent performance

## Events

### Test Lifecycle Events

- `clnrm.test.started` - Test begins
- `clnrm.test.completed` - Test completes successfully
- `clnrm.test.failed` - Test fails or errors

**Validation Strategy:**
- Every `started` event must have matching `completed` or `failed`
- Orphaned `started` events indicate crashes

### Critical Events (should NEVER occur)

- `clnrm.container.leaked` - Container not cleaned up
- `clnrm.isolation.violation` - Shared state between tests

**Validation Strategy:**
- Presence of these events = FAILURE
- Indicates fundamental problems with clnrm

## Validation Workflows

### 1. Schema Validation

Check schemas are valid:

```bash
weaver registry check -r registry/
```

Expected output: All checks pass (✔)

### 2. Unit Test Validation

Run tests and validate telemetry:

```bash
# Run tests with OTEL to stdout
OTEL_EXPORTER=stdout cargo test --features otel 2> telemetry.json

# Validate against schemas
weaver validate --schema registry/ --input telemetry.json

# Check critical attributes exist
jq '.spans[] | select(.name == "clnrm.test_execution") | .attributes.container.id' telemetry.json
```

### 3. Live Validation

Validate during test execution:

```bash
# Start OTEL collector
docker run -d -p 4318:4318 otel/opentelemetry-collector

# Run tests with collector
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 cargo test --features otel

# Live check (in separate terminal)
weaver live-check --schema registry/ --endpoint http://localhost:4318/v1/traces
```

### 4. CI/CD Integration

See `VALIDATION_STRATEGY.md` for complete GitHub Actions workflow.

## Critical Attributes by Behavior

### Container Creation
- `container.id` (span.clnrm.test_execution)
- `container.created_at` (span.clnrm.container_lifecycle)

**Proves:** Container actually created

### Hermetic Isolation
- `test.isolated = true` (span.clnrm.test_execution)
- `clnrm.isolation.score = 1.0` (metric)

**Proves:** Tests run in complete isolation

### Container Cleanup
- `container.destroyed_at` (span.clnrm.container_lifecycle)
- `cleanup.success = true` (span.clnrm.container_lifecycle)
- `created == destroyed` (metric.clnrm.container.count)

**Proves:** No resource leaks

### Plugin System
- `plugin.state` transitions (span.clnrm.plugin_execution)
- `plugin.health_check.performed = true` (span.clnrm.plugin_execution)

**Proves:** Plugin lifecycle management works

### Command Execution
- `command.exit_code` (span.clnrm.service_command)
- `command.output` (span.clnrm.service_command)

**Proves:** Commands execute inside containers

## Detecting False Positives

### Scenario: Stub Implementation

**Code:**
```rust
async fn create_container(&self, image: &str) -> Result<ContainerId> {
    Ok(ContainerId::new("fake-id"))  // Stub!
}
```

**Test:** Passes ✅
**Weaver:** Fails ❌

```
ERROR: span.clnrm.container_lifecycle missing required attribute: container.created_at
ERROR: No container lifecycle spans emitted
VERDICT: Implementation is stubbed
```

### Scenario: Resource Leak

**Code:**
```rust
async fn cleanup(&self) -> Result<()> {
    Ok(())  // Forgot to destroy container
}
```

**Test:** Passes ✅
**Weaver:** Fails ❌

```
ERROR: Metric clnrm.container.count:
  - created: 10
  - destroyed: 7
  - leak_count: 3
VERDICT: Resource leak detected
```

### Scenario: Isolation Failure

**Code:**
```rust
static SHARED_CONTAINER: OnceCell<Container> = OnceCell::new();
```

**Test:** Passes ✅
**Weaver:** Fails ❌

```
ERROR: Multiple test_execution spans share same container.id
ERROR: Isolation score: 0.45 (expected 1.0)
ERROR: event.clnrm.isolation.violation emitted
VERDICT: Hermetic isolation violated
```

## Success Criteria

The schema registry successfully eliminates false positives when:

1. ✅ **Stub implementations fail validation**
   - Missing required attributes detected
   - Cannot fake container.id

2. ✅ **Resource leaks detected**
   - Metric counts don't balance
   - Missing destroyed_at timestamps

3. ✅ **Isolation violations caught**
   - Shared container.id detected
   - Isolation score < 1.0

4. ✅ **Real behavior proven**
   - All required attributes present
   - State transitions complete
   - Metrics balanced

## Adding New Schemas

When adding new features to clnrm:

1. **Identify provable behavior**
   - What attribute can ONLY exist if feature works?
   - What cannot be faked?

2. **Create schema**
   - Add to appropriate directory (core/metrics/events)
   - Mark critical attributes as `required`
   - Document validation strategy in `note` field

3. **Add stability fields**
   ```yaml
   - id: my.attribute
     type: string
     stability: stable  # Required
     requirement_level: required
   ```

4. **Validate schema**
   ```bash
   weaver registry check -r registry/
   ```

5. **Implement instrumentation**
   - Emit telemetry matching schema
   - Ensure all required attributes populated

6. **Validate in tests**
   - Run with OTEL enabled
   - Validate telemetry against schema
   - Verify required attributes exist

## Integration with clnrm

The schema registry integrates with clnrm at multiple levels:

### Build Time
- Schemas checked in CI/CD (`weaver registry check`)
- Code generation from schemas (future)

### Test Time
- Instrumentation emits telemetry matching schemas
- Test assertions validate telemetry structure

### Runtime
- Live validation during test execution
- Metrics exported to observability backend

### Validation Time
- `clnrm self-test --validate-telemetry`
- Automatic schema validation
- Required attribute checking

## Dependencies

- **Weaver** - Schema validation and code generation
  - Install: `cargo install weaver-cli`
  - Docs: https://github.com/open-telemetry/weaver

- **OpenTelemetry** - Telemetry emission
  - Rust SDK: `opentelemetry`, `opentelemetry-otlp`
  - Collector: `otel/opentelemetry-collector`

## Maintenance

### Schema Updates

Schemas follow semantic versioning:

- **Patch** (1.0.x): Documentation only, backward compatible
- **Minor** (1.x.0): New optional attributes, backward compatible
- **Major** (x.0.0): Required attributes changed, breaking

Update `semconv_version` in `registry_manifest.yaml` when making changes.

### Validation

Before committing schema changes:

```bash
# Validate schemas
weaver registry check -r registry/

# Test instrumentation
cargo test --features otel

# Validate telemetry
weaver validate --schema registry/ --input telemetry.json
```

## Documentation

- `VALIDATION_STRATEGY.md` - Complete validation approach
- Individual schema files - Detailed attribute documentation
- `registry_manifest.yaml` - Registry metadata

## Next Steps

1. **Instrumentation** - Implement telemetry emission matching schemas
2. **Integration** - Add schema validation to CI/CD
3. **Testing** - Create tests that validate telemetry
4. **Documentation** - Developer guide for using schemas

## Contact

For questions about the schema registry:
- GitHub Issues: https://github.com/seanchatmangpt/clnrm/issues
- Documentation: See `docs/` directory in main repo

---

**Remember:** A passing test means NOTHING. Valid telemetry means EVERYTHING.
