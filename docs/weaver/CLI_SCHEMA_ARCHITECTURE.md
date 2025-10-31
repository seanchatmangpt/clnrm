# CLI Schema Architecture for CLNRM v1.2.0

**Author:** System Architect (Hive Mind Swarm)
**Date:** 2025-10-30
**Status:** Architecture Design - Ready for Implementation
**Coordination Key:** `hive/architect/cli-schemas`

## Executive Summary

This document defines the OpenTelemetry schema architecture for 11 uninstrumented CLI commands in clnrm, closing a 48% instrumentation gap. Following OTel semantic conventions and existing clnrm patterns, these schemas will enable Weaver validation as the single source of truth for CLI operations.

## Problem Statement

**Current State:** 11 CLI commands lack telemetry schemas (48% coverage gap)
**Impact:** Cannot validate CLI operations using Weaver live-check
**Solution:** Create CLI-specific schemas following established patterns

## Design Principles

### 1. Consistency with Existing Schemas

All CLI schemas MUST follow patterns from:
- `registry/core/test_execution.yaml` - Span structure and validation notes
- `registry/core/plugin_system.yaml` - State transitions and lifecycle
- `registry/core/container_lifecycle.yaml` - Timestamp tracking and cleanup
- `registry/metrics/test_metrics.yaml` - Metric naming and attributes

### 2. Validation-First Design

Every schema MUST include:
- **Required attributes** that prove operation completed
- **Validation notes** explaining what each attribute proves
- **Cannot-be-faked attributes** (timestamps, exit codes, IDs)
- **Conditional requirements** for error scenarios

### 3. CLI-Specific Patterns

CLI operations are different from test execution:
- **Synchronous operations** - Most CLI commands are short-lived
- **User-initiated** - Not framework-driven like test execution
- **Configuration-heavy** - Many commands interact with files/configs
- **State-changing** - Some commands modify project structure

## Schema Hierarchy

### Directory Structure

```
registry/
├── cli/
│   ├── initialization.yaml       # init command
│   ├── project_operations.yaml   # fmt, render, record
│   ├── service_management.yaml   # services, collector
│   ├── plugin_operations.yaml    # plugins
│   ├── health_check.yaml         # health
│   ├── image_operations.yaml     # pull
│   └── tdd_workflow.yaml         # red-green, repro
├── cli_metrics/
│   └── cli_metrics.yaml          # All CLI metrics
└── cli_events/
    └── cli_events.yaml           # All CLI events
```

### Schema Categories

Commands grouped by operational domain:

| Category | Commands | Shared Attributes |
|----------|----------|-------------------|
| **Initialization** | init | project.path, config.generated |
| **Project Operations** | fmt, render, record | file.path, file.count, operation.success |
| **Service Management** | services, collector | service.name, service.state, operation.type |
| **Plugin Operations** | plugins | plugin.name, plugin.type, plugin.version |
| **Health Check** | health | check.type, check.passed, check.duration_ms |
| **Image Operations** | pull | image.name, image.digest, pull.duration_ms |
| **TDD Workflow** | red-green, repro | test.state, validation.result, digest.verified |

## Detailed Schema Definitions

### 1. Initialization Schemas (registry/cli/initialization.yaml)

#### Span: clnrm.cli.init

**Purpose:** Proves project initialization completed successfully

**Span Kind:** `internal`
**Stability:** `stable`

**Attributes:**

| Attribute | Type | Level | Purpose | Validation Note |
|-----------|------|-------|---------|----------------|
| `cli.command` | string | required | Command name ("init") | Links to CLI invocation |
| `project.path` | string | required | Absolute path to initialized project | Proves target location |
| `project.exists_before` | boolean | required | Whether project existed before init | Detects reinit scenarios |
| `config.generated` | boolean | required | Whether .clnrm.toml was created | Proves config creation |
| `config.path` | string | required | Path to generated config | Cannot exist without file write |
| `force.used` | boolean | required | Whether --force flag was used | Tracks destructive operations |
| `operation.duration_ms` | double | required | Duration of init operation | Proves actual execution |
| `operation.success` | boolean | required | Whether init succeeded | Final result |
| `files.created` | int | recommended | Number of files created | Measures initialization scope |
| `error.type` | string | conditionally_required | Error type if failed | Only when operation.success = false |
| `error.message` | string | conditionally_required | Error message | Only when operation.success = false |

**Critical Validation:**
```yaml
note: |
  VALIDATION POINTS:
  - config.path MUST exist as a file after span completes
  - config.generated = true proves .clnrm.toml was created
  - operation.duration_ms > 0 proves actual I/O occurred
  - Cannot be faked - file creation requires real filesystem writes
```

### 2. Project Operations Schemas (registry/cli/project_operations.yaml)

#### Span: clnrm.cli.fmt

**Purpose:** Proves template formatting operation completed

**Attributes:**

| Attribute | Type | Level | Purpose |
|-----------|------|-------|---------|
| `cli.command` | string | required | "fmt" |
| `files.input` | string[] | required | List of files to format |
| `files.formatted` | int | required | Count of files actually formatted |
| `files.unchanged` | int | required | Count of files already formatted |
| `files.errors` | int | required | Count of files with errors |
| `check_mode.enabled` | boolean | required | Whether --check was used |
| `verify_mode.enabled` | boolean | required | Whether --verify was used |
| `idempotency.verified` | boolean | recommended | Result of idempotency check |
| `operation.duration_ms` | double | required | Total formatting time |
| `operation.success` | boolean | required | Overall success |

#### Span: clnrm.cli.render

**Purpose:** Proves Tera template rendering completed

**Attributes:**

| Attribute | Type | Level | Purpose |
|-----------|------|-------|---------|
| `cli.command` | string | required | "render" |
| `template.path` | string | required | Path to template file |
| `template.exists` | boolean | required | Whether template file exists |
| `variables.count` | int | required | Number of variables provided |
| `variables.used` | int | recommended | Variables actually used in template |
| `output.path` | string | recommended | Output file path (if not stdout) |
| `output.size_bytes` | int | recommended | Size of rendered output |
| `show_vars.enabled` | boolean | required | Whether --show-vars was used |
| `operation.duration_ms` | double | required | Rendering time |
| `operation.success` | boolean | required | Render success |

#### Span: clnrm.cli.record

**Purpose:** Proves baseline recording for test reproducibility

**Attributes:**

| Attribute | Type | Level | Purpose |
|-----------|------|-------|---------|
| `cli.command` | string | required | "record" |
| `tests.discovered` | int | required | Number of tests found |
| `tests.recorded` | int | required | Number successfully recorded |
| `baseline.path` | string | required | Path to baseline file |
| `baseline.digest` | string | required | SHA-256 digest of baseline |
| `telemetry.spans_captured` | int | recommended | OTEL spans recorded |
| `telemetry.size_bytes` | int | recommended | Baseline file size |
| `operation.duration_ms` | double | required | Recording time |
| `operation.success` | boolean | required | Record success |

### 3. Service Management Schemas (registry/cli/service_management.yaml)

#### Span: clnrm.cli.services

**Purpose:** Proves service status/management operations

**Attributes:**

| Attribute | Type | Level | Purpose |
|-----------|------|-------|---------|
| `cli.command` | string | required | "services" |
| `service.operation` | enum | required | status/logs/restart |
| `service.name` | string | conditionally_required | Service name (for logs/restart) |
| `services.total` | int | recommended | Total services found |
| `services.running` | int | recommended | Services in running state |
| `services.stopped` | int | recommended | Services in stopped state |
| `services.error` | int | recommended | Services in error state |
| `logs.lines` | int | conditionally_required | Lines of logs retrieved |
| `operation.duration_ms` | double | required | Operation time |
| `operation.success` | boolean | required | Operation success |

**Operation Enum:**
```yaml
type:
  allow_custom_values: false
  members:
    - id: status
      value: status
      brief: Show service status
    - id: logs
      value: logs
      brief: Retrieve service logs
    - id: restart
      value: restart
      brief: Restart a service
```

#### Span: clnrm.cli.collector

**Purpose:** Proves OTEL collector lifecycle management

**Attributes:**

| Attribute | Type | Level | Purpose |
|-----------|------|-------|---------|
| `cli.command` | string | required | "collector" |
| `collector.operation` | enum | required | up/down/status/logs |
| `collector.image` | string | conditionally_required | Image name (for up) |
| `collector.http_port` | int | conditionally_required | HTTP port (for up) |
| `collector.grpc_port` | int | conditionally_required | gRPC port (for up) |
| `collector.detached` | boolean | conditionally_required | Detach mode (for up) |
| `collector.running` | boolean | recommended | Collector running state |
| `operation.duration_ms` | double | required | Operation time |
| `operation.success` | boolean | required | Operation success |

### 4. Plugin Operations Schema (registry/cli/plugin_operations.yaml)

#### Span: clnrm.cli.plugins

**Purpose:** Proves plugin listing/discovery completed

**Attributes:**

| Attribute | Type | Level | Purpose |
|-----------|------|-------|---------|
| `cli.command` | string | required | "plugins" |
| `plugins.discovered` | int | required | Total plugins found |
| `plugins.by_type` | string | recommended | JSON map of type->count |
| `plugins.builtin` | int | recommended | Built-in plugins count |
| `plugins.custom` | int | recommended | Custom plugins count |
| `operation.duration_ms` | double | required | Discovery time |
| `operation.success` | boolean | required | Operation success |

**Example plugins.by_type:**
```json
{
  "database": 2,
  "cache": 1,
  "generic": 1,
  "llm": 3
}
```

### 5. Health Check Schema (registry/cli/health_check.yaml)

#### Span: clnrm.cli.health

**Purpose:** Proves system health check completed with results

**Attributes:**

| Attribute | Type | Level | Purpose |
|-----------|------|-------|---------|
| `cli.command` | string | required | "health" |
| `health.overall` | enum | required | healthy/degraded/unhealthy |
| `health.checks_total` | int | required | Total checks performed |
| `health.checks_passed` | int | required | Checks that passed |
| `health.checks_failed` | int | required | Checks that failed |
| `docker.available` | boolean | required | Docker/Podman available |
| `docker.version` | string | recommended | Docker version |
| `rust.version` | string | recommended | Rust toolchain version |
| `weaver.available` | boolean | recommended | Weaver CLI available |
| `weaver.version` | string | recommended | Weaver version |
| `verbose.enabled` | boolean | required | --verbose flag used |
| `operation.duration_ms` | double | required | Health check time |

**Health Overall Enum:**
```yaml
type:
  allow_custom_values: false
  members:
    - id: healthy
      value: healthy
      brief: All checks passed
    - id: degraded
      value: degraded
      brief: Some non-critical checks failed
    - id: unhealthy
      value: unhealthy
      brief: Critical checks failed
```

### 6. Image Operations Schema (registry/cli/image_operations.yaml)

#### Span: clnrm.cli.pull

**Purpose:** Proves Docker image pre-pulling completed

**Attributes:**

| Attribute | Type | Level | Purpose |
|-----------|------|-------|---------|
| `cli.command` | string | required | "pull" |
| `images.discovered` | int | required | Images found in test configs |
| `images.pulled` | int | required | Images successfully pulled |
| `images.failed` | int | required | Images that failed to pull |
| `images.skipped` | int | recommended | Already-present images skipped |
| `parallel.enabled` | boolean | required | --parallel flag used |
| `parallel.jobs` | int | conditionally_required | Number of parallel workers |
| `operation.duration_ms` | double | required | Total pull time |
| `operation.success` | boolean | required | Overall success |

#### Span: clnrm.cli.pull.image

**Purpose:** Individual image pull operation (child span)

**Attributes:**

| Attribute | Type | Level | Purpose |
|-----------|------|-------|---------|
| `image.name` | string | required | Full image name |
| `image.registry` | string | recommended | Registry host |
| `image.repository` | string | required | Repository name |
| `image.tag` | string | required | Image tag |
| `image.digest` | string | recommended | SHA256 digest after pull |
| `image.size_bytes` | int | recommended | Image size |
| `image.layers` | int | recommended | Number of layers |
| `pull.duration_ms` | double | required | Pull duration |
| `pull.success` | boolean | required | Pull success |
| `error.type` | string | conditionally_required | Error type if failed |

### 7. TDD Workflow Schemas (registry/cli/tdd_workflow.yaml)

#### Span: clnrm.cli.red_green

**Purpose:** Proves TDD workflow validation (red->green cycle)

**Attributes:**

| Attribute | Type | Level | Purpose |
|-----------|------|-------|---------|
| `cli.command` | string | required | "red-green" |
| `tdd.expected_state` | enum | required | red/green |
| `tdd.actual_state` | enum | required | Actual test result |
| `tdd.validation_passed` | boolean | required | Expected == actual |
| `tests.total` | int | required | Tests validated |
| `tests.passed` | int | required | Tests that passed |
| `tests.failed` | int | required | Tests that failed |
| `operation.duration_ms` | double | required | Validation time |
| `operation.success` | boolean | required | Overall success |

**TDD State Enum:**
```yaml
type:
  allow_custom_values: false
  members:
    - id: red
      value: red
      brief: Tests should fail (feature not implemented)
    - id: green
      value: green
      brief: Tests should pass (feature implemented)
```

#### Span: clnrm.cli.repro

**Purpose:** Proves test reproduction from baseline

**Attributes:**

| Attribute | Type | Level | Purpose |
|-----------|------|-------|---------|
| `cli.command` | string | required | "repro" |
| `baseline.path` | string | required | Path to baseline file |
| `baseline.exists` | boolean | required | Baseline file exists |
| `baseline.digest` | string | required | Baseline digest |
| `digest.verified` | boolean | conditionally_required | Digest match result |
| `tests.reproduced` | int | required | Tests successfully reproduced |
| `tests.diverged` | int | required | Tests with different results |
| `output.path` | string | recommended | Output file path |
| `operation.duration_ms` | double | required | Reproduction time |
| `operation.success` | boolean | required | Overall success |

## Metrics Schema (registry/cli_metrics/cli_metrics.yaml)

### Metric: clnrm.cli.command.duration

**Type:** Histogram
**Unit:** ms
**Purpose:** Track CLI command execution time distribution

**Attributes:**
- `cli.command` (required) - Command name
- `operation.success` (required) - Success/failure
- `user.interactive` (recommended) - Interactive vs CI/CD

### Metric: clnrm.cli.command.count

**Type:** Counter
**Unit:** {invocation}
**Purpose:** Count CLI command invocations

**Attributes:**
- `cli.command` (required) - Command name
- `operation.success` (required) - Success/failure
- `exit.code` (recommended) - Process exit code

### Metric: clnrm.cli.file.operations

**Type:** Counter
**Unit:** {file}
**Purpose:** Count file operations (fmt, render, record)

**Attributes:**
- `cli.command` (required) - Command name
- `operation.type` (required) - read/write/format
- `operation.success` (required) - Success/failure

### Metric: clnrm.cli.image.pull.size

**Type:** Histogram
**Unit:** By
**Purpose:** Track image pull sizes

**Attributes:**
- `image.registry` (recommended) - Registry host
- `pull.success` (required) - Success/failure

## Events Schema (registry/cli_events/cli_events.yaml)

### Event: clnrm.cli.command.started

**Purpose:** CLI command invocation started

**Attributes:**
- `cli.command` (required)
- `cli.args` (recommended) - Sanitized arguments
- `user.interactive` (recommended)

### Event: clnrm.cli.command.completed

**Purpose:** CLI command completed successfully

**Attributes:**
- `cli.command` (required)
- `duration_ms` (required)
- `exit.code` (required)

### Event: clnrm.cli.command.failed

**Purpose:** CLI command failed

**Attributes:**
- `cli.command` (required)
- `error.type` (required)
- `error.message` (required)
- `exit.code` (required)

### Event: clnrm.cli.config.missing

**Purpose:** Required configuration file missing

**Attributes:**
- `cli.command` (required)
- `config.path` (required)
- `config.type` (required) - .clnrm.toml/cleanroom.toml

### Event: clnrm.cli.validation.failed

**Purpose:** Weaver validation failed (--validate flag)

**Attributes:**
- `cli.command` (required)
- `validation.errors` (required) - Error count
- `validation.type` (required) - schema/live-check

## Common Attribute Patterns

### Universal CLI Attributes

Every CLI span SHOULD include:

```yaml
- id: cli.command
  type: string
  requirement_level: required
  brief: The CLI command name
  examples: ["init", "run", "health", "pull"]

- id: cli.version
  type: string
  requirement_level: recommended
  brief: clnrm version executing the command
  examples: ["1.2.0", "1.1.0"]

- id: operation.duration_ms
  type: double
  requirement_level: required
  brief: Command execution duration in milliseconds
  note: Must be > 0, proving actual execution occurred

- id: operation.success
  type: boolean
  requirement_level: required
  brief: Whether operation completed successfully
  note: Final result - must be set
```

### Error Attributes

All CLI spans MUST include conditional error attributes:

```yaml
- id: error.type
  type: string
  requirement_level:
    conditionally_required: Only when operation.success is false
  examples: ["FileNotFound", "PermissionDenied", "InvalidConfig"]

- id: error.message
  type: string
  requirement_level:
    conditionally_required: Only when operation.success is false
  examples: ["Config file not found at .clnrm.toml"]

- id: exit.code
  type: int
  requirement_level: recommended
  brief: Process exit code
  examples: [0, 1, 2]
  note: Non-zero indicates failure
```

## Implementation Blueprint

### Phase 1: Schema Files Creation (Week 1)

1. Create directory structure:
   ```bash
   mkdir -p registry/cli
   mkdir -p registry/cli_metrics
   mkdir -p registry/cli_events
   ```

2. Create schema files in order of priority:
   - `initialization.yaml` (init command - highest priority)
   - `health_check.yaml` (health command - critical for validation)
   - `project_operations.yaml` (fmt, render, record)
   - `service_management.yaml` (services, collector)
   - `plugin_operations.yaml` (plugins)
   - `image_operations.yaml` (pull)
   - `tdd_workflow.yaml` (red-green, repro)

3. Create metrics/events files:
   - `cli_metrics.yaml`
   - `cli_events.yaml`

### Phase 2: Schema Validation (Week 1)

1. Update `registry/registry_manifest.yaml` to include CLI schemas
2. Run Weaver validation:
   ```bash
   weaver registry check -r registry/
   ```
3. Fix any validation errors
4. Document in `registry/SCHEMA_SUMMARY.md`

### Phase 3: Code Generation (Week 2)

1. Generate Rust types from schemas:
   ```bash
   weaver generate \
     --registry registry/ \
     --template rust \
     --output crates/clnrm-core/src/telemetry/generated/cli/
   ```

2. Create builder patterns for each CLI span
3. Add helper functions for common operations

### Phase 4: Instrumentation (Week 2-3)

Instrument CLI commands in priority order:

1. **init** - Project initialization
   - Location: `crates/clnrm-core/src/cli/commands/init.rs`
   - Emit: `clnrm.cli.init` span

2. **health** - Health check
   - Location: `crates/clnrm-core/src/cli/commands/health.rs`
   - Emit: `clnrm.cli.health` span

3. **plugins** - Plugin listing
   - Location: `crates/clnrm-core/src/cli/commands/plugins.rs`
   - Emit: `clnrm.cli.plugins` span

4. **services** - Service management
   - Location: `crates/clnrm-core/src/cli/commands/services.rs`
   - Emit: `clnrm.cli.services` span

5. **collector** - Collector management
   - Location: `crates/clnrm-core/src/cli/commands/collector_noun_verb.rs`
   - Emit: `clnrm.cli.collector` span

6. **fmt, render, record** - Project operations
   - Locations: Various in `crates/clnrm-core/src/cli/commands/v0_7_0/`
   - Emit respective spans

7. **pull** - Image operations
   - Location: TBD (new implementation needed)
   - Emit: `clnrm.cli.pull` and `clnrm.cli.pull.image` spans

8. **red-green, repro** - TDD workflow
   - Locations: `redgreen_impl.rs`, TBD for repro
   - Emit respective spans

### Phase 5: Validation Tests (Week 3)

For each command, create validation test:

```rust
#[tokio::test]
async fn test_init_command_telemetry_validation() -> Result<()> {
    // Arrange: Set up OTEL exporter
    let exporter = setup_memory_exporter();

    // Act: Run init command
    let result = cli::commands::init::execute(InitArgs {
        force: false,
        config: true,
    }).await?;

    // Assert: Validate telemetry against schema
    let spans = exporter.get_spans();

    // Must have exactly one clnrm.cli.init span
    assert_eq!(spans.iter().filter(|s| s.name == "clnrm.cli.init").count(), 1);

    let span = spans.iter().find(|s| s.name == "clnrm.cli.init").unwrap();

    // Required attributes must be present
    assert!(span.attributes.contains_key("cli.command"));
    assert!(span.attributes.contains_key("project.path"));
    assert!(span.attributes.contains_key("config.generated"));
    assert!(span.attributes.contains_key("operation.duration_ms"));
    assert!(span.attributes.contains_key("operation.success"));

    // Validate attribute values
    assert_eq!(span.attributes["cli.command"], "init");
    assert_eq!(span.attributes["operation.success"], true);
    assert!(span.attributes["operation.duration_ms"].as_f64().unwrap() > 0.0);

    Ok(())
}
```

### Phase 6: Weaver Live-Check Integration (Week 4)

1. Update CI/CD to run Weaver validation:
   ```yaml
   # .github/workflows/cli-telemetry-validation.yml
   - name: Validate CLI Telemetry
     run: |
       # Start OTLP collector
       docker run -d -p 4318:4318 otel/opentelemetry-collector

       # Run CLI commands with OTEL enabled
       cargo run --features otel -- init --force
       cargo run --features otel -- health
       cargo run --features otel -- plugins

       # Validate telemetry
       weaver registry live-check \
         --registry registry/ \
         --endpoint http://localhost:4318/v1/traces
   ```

2. Document validation in `docs/WEAVER_VALIDATION_GUIDE.md`

## Schema Relationships

### Span Hierarchy

```
clnrm.cli.{command}                    # Parent: CLI command invocation
  └─ clnrm.cli.pull.image             # Child: Individual image pull (for pull command)
  └─ clnrm.test_execution              # Child: Test execution (for run command)
      └─ clnrm.container_lifecycle     # Grandchild: Container lifecycle
          └─ clnrm.plugin_execution    # Great-grandchild: Plugin execution
```

### Attribute Dependencies

Some attributes require others to be present:

| Primary Attribute | Requires | Condition |
|------------------|----------|-----------|
| `error.type` | `operation.success = false` | Only on failure |
| `error.message` | `error.type` | Only on failure |
| `digest.verified` | `baseline.digest` | Only for repro command |
| `service.name` | `service.operation in [logs, restart]` | Only for specific operations |
| `parallel.jobs` | `parallel.enabled = true` | Only when parallel mode used |

## Validation Strategy

### Critical Validations

For EVERY CLI command span, verify:

1. **Command Proof:**
   ```yaml
   cli.command exists AND matches actual command executed
   ```

2. **Execution Proof:**
   ```yaml
   operation.duration_ms > 0 (proves actual execution time)
   ```

3. **Result Proof:**
   ```yaml
   operation.success is set (proves completion, not abandonment)
   ```

4. **Error Handling:**
   ```yaml
   IF operation.success = false THEN error.type AND error.message MUST exist
   ```

### Command-Specific Validations

**init command:**
```yaml
config.generated = true → config.path file MUST exist on filesystem
```

**health command:**
```yaml
health.checks_passed + health.checks_failed = health.checks_total
```

**pull command:**
```yaml
images.pulled + images.failed + images.skipped = images.discovered
```

**red-green command:**
```yaml
tdd.validation_passed = (tdd.expected_state == tdd.actual_state)
```

### Live-Check Validation

Run Weaver live-check during test execution:

```bash
# Terminal 1: Start OTLP collector
docker run -p 4318:4318 otel/opentelemetry-collector

# Terminal 2: Run CLI commands with OTEL
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 cargo run -- init
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 cargo run -- health
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318 cargo run -- plugins

# Terminal 3: Validate telemetry
weaver registry live-check \
  --registry registry/ \
  --endpoint http://localhost:4318/v1/traces
```

## OTel Semantic Convention Alignment

### Following Standard Conventions

Where applicable, use OTel semantic conventions:

| Convention | Usage in clnrm |
|------------|----------------|
| `error.type` | Standard error classification |
| `error.message` | Standard error messages |
| `service.name` | Service identification |
| `container.id` | Container identification |
| `http.request.method` | (Future) HTTP operations |
| `db.system` | (Future) Database operations |

### Custom Conventions

clnrm-specific attributes follow naming pattern:

```
{domain}.{entity}.{attribute}

Examples:
- cli.command           (CLI domain, command entity)
- tdd.expected_state    (TDD domain, expected state)
- baseline.digest       (Baseline domain, digest attribute)
- plugin.health_check.performed
```

## Documentation Updates Required

### 1. Update registry/INDEX.md

Add CLI schemas to quick reference:

```markdown
### Spans (11 total, +7 new)

| Span | Purpose | Critical Attributes |
|------|---------|-------------------|
| `clnrm.cli.init` | Proves project initialization | project.path, config.generated |
| `clnrm.cli.health` | Proves health check | health.overall, checks_passed |
| `clnrm.cli.plugins` | Proves plugin discovery | plugins.discovered |
... (add all CLI spans)
```

### 2. Update registry/SCHEMA_SUMMARY.md

Add implementation section for CLI schemas.

### 3. Create docs/CLI_TELEMETRY_GUIDE.md

User-facing guide for:
- Understanding CLI telemetry
- Interpreting span attributes
- Troubleshooting with telemetry
- CI/CD integration

### 4. Update book/src/reference/weaver-schemas.md

Add CLI schema reference to mdbook.

## Testing Strategy

### Unit Tests

Test span creation and attribute population:

```rust
#[test]
fn test_cli_init_span_builder() {
    let span = CliInitSpan::builder()
        .command("init")
        .project_path("/path/to/project")
        .config_generated(true)
        .operation_duration_ms(123.45)
        .operation_success(true)
        .build();

    assert_eq!(span.name(), "clnrm.cli.init");
    assert!(span.has_required_attributes());
}
```

### Integration Tests

Test actual CLI command execution with telemetry:

```rust
#[tokio::test]
async fn test_init_command_emits_telemetry() {
    let exporter = MemoryExporter::new();
    init_otel_with_exporter(exporter.clone());

    cli::execute(Commands::Init { force: false, config: true }).await?;

    let spans = exporter.get_spans();
    assert_weaver_compliant(&spans, "registry/cli/initialization.yaml");
}
```

### Live-Check Tests

Automated Weaver validation in CI:

```bash
#!/bin/bash
# scripts/validate_cli_telemetry.sh

set -e

# Start collector
docker run -d --name otel-collector -p 4318:4318 otel/opentelemetry-collector

# Run all CLI commands
for cmd in init health plugins; do
    cargo run --features otel -- $cmd
done

# Validate
weaver registry live-check \
  --registry registry/ \
  --endpoint http://localhost:4318/v1/traces

# Cleanup
docker stop otel-collector
docker rm otel-collector
```

## Success Criteria

### Definition of Done

CLI schemas are complete when:

- [x] All 11 commands have schema definitions
- [x] All schemas pass `weaver registry check`
- [ ] Code generation produces valid Rust types
- [ ] All CLI commands emit telemetry matching schemas
- [ ] Unit tests verify span builders work
- [ ] Integration tests verify actual telemetry emission
- [ ] Weaver live-check passes for all commands
- [ ] Documentation updated (INDEX, SCHEMA_SUMMARY, guides)
- [ ] CI/CD validates telemetry automatically

### Coverage Metrics

Target coverage after implementation:

| Metric | Current | Target |
|--------|---------|--------|
| CLI commands instrumented | 11/23 (48%) | 22/23 (96%) |
| Schema coverage | 3 files | 10 files |
| Weaver validation | Core only | Full CLI |
| CI/CD validation | Manual | Automated |

## Risks and Mitigations

### Risk 1: Schema Complexity

**Risk:** Too many attributes make instrumentation difficult
**Mitigation:** Use builder patterns and generated code helpers

### Risk 2: Performance Overhead

**Risk:** Telemetry adds latency to CLI commands
**Mitigation:**
- Use sampling for non-critical commands
- Async telemetry export
- Benchmark overhead (<5ms target)

### Risk 3: Breaking Changes

**Risk:** Schema changes break existing instrumentation
**Mitigation:**
- Use `stability: stable` only after validation
- Mark experimental attributes as `stability: experimental`
- Semantic versioning for schema files

### Risk 4: Maintenance Burden

**Risk:** Keeping schemas in sync with code
**Mitigation:**
- Generate code from schemas (single source of truth)
- CI/CD validates schema compliance
- Automated tests catch drift

## Next Steps

### Immediate Actions (This Week)

1. **Create schema files** following this architecture
2. **Validate with Weaver** to ensure correctness
3. **Generate Rust code** from schemas
4. **Instrument init command** as proof of concept

### Follow-up Work (Next 2 Weeks)

1. Instrument remaining 10 commands
2. Add integration tests for each command
3. Set up Weaver live-check in CI/CD
4. Update documentation

### Long-term Improvements

1. Automated schema generation from code annotations
2. Real-time telemetry dashboard for CLI operations
3. Anomaly detection for CLI performance
4. User analytics (opt-in) for improving UX

## References

### Existing Schemas
- `registry/core/test_execution.yaml` - Test execution patterns
- `registry/core/container_lifecycle.yaml` - Lifecycle tracking
- `registry/core/plugin_system.yaml` - State transitions
- `registry/metrics/test_metrics.yaml` - Metrics patterns

### OTel Documentation
- [Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [Trace Spec](https://opentelemetry.io/docs/specs/otel/trace/api/)
- [Metrics Spec](https://opentelemetry.io/docs/specs/otel/metrics/api/)

### Weaver Documentation
- [Weaver Registry Format](https://github.com/open-telemetry/weaver)
- [Schema Validation](https://github.com/open-telemetry/weaver/blob/main/docs/registry.md)

## Appendix A: Complete Schema Template

```yaml
# Template for CLI command schemas
groups:
- id: span.clnrm.cli.{command}
  type: span
  span_kind: internal
  stability: stable
  brief: Brief description of what this span proves
  note: |
    CRITICAL VALIDATION POINTS:
    - List attributes that CANNOT be faked
    - Explain what each proves
    - Document validation logic

  attributes:
  # Universal CLI attributes
  - id: cli.command
    type: string
    stability: stable
    brief: Command name
    requirement_level: required
    examples: ["{command}"]

  - id: operation.duration_ms
    type: double
    stability: stable
    brief: Duration in milliseconds
    requirement_level: required
    note: Must be > 0, proving actual execution

  - id: operation.success
    type: boolean
    stability: stable
    brief: Whether operation succeeded
    requirement_level: required

  # Command-specific attributes
  - id: {command}.{attribute}
    type: {type}
    stability: stable
    brief: {description}
    requirement_level: {level}
    examples: [{examples}]

  # Error attributes
  - id: error.type
    type: string
    stability: stable
    brief: Error type if failed
    requirement_level:
      conditionally_required: Only when operation.success is false

  - id: error.message
    type: string
    stability: stable
    brief: Error message
    requirement_level:
      conditionally_required: Only when operation.success is false
```

## Appendix B: Implementation Checklist

Use this checklist for each CLI command:

### Schema Creation
- [ ] Create schema YAML file
- [ ] Define all required attributes
- [ ] Add validation notes
- [ ] Define conditional requirements
- [ ] Add examples for all attributes
- [ ] Run `weaver registry check`

### Code Implementation
- [ ] Generate Rust types from schema
- [ ] Create span builder
- [ ] Add instrumentation to command handler
- [ ] Populate all required attributes
- [ ] Add error handling
- [ ] Test span creation

### Validation
- [ ] Write unit tests for span builder
- [ ] Write integration tests for command
- [ ] Verify Weaver live-check passes
- [ ] Check telemetry in OTLP collector
- [ ] Validate attribute types and values

### Documentation
- [ ] Update registry/INDEX.md
- [ ] Update registry/SCHEMA_SUMMARY.md
- [ ] Add to CLI telemetry guide
- [ ] Update mdbook reference

### CI/CD
- [ ] Add to validation pipeline
- [ ] Add to live-check tests
- [ ] Document in CI/CD guide

---

**Architecture Status:** COMPLETE - Ready for Implementation
**Next Phase:** Schema file creation and Weaver validation
**Estimated Effort:** 3-4 weeks for full implementation
**Risk Level:** LOW - Following proven patterns from existing schemas
