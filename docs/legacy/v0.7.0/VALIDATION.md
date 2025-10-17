# Configuration Validation - v0.7.0

**Version**: 0.7.0
**Module**: `clnrm-core::validation`
**Feature**: Fast static validation without container startup

## Overview

The validation system provides fast, static configuration validation without spinning up containers. It performs comprehensive shape checking, relationship validation, and error reporting with actionable suggestions, typically completing in <1s for complex configurations.

## Architecture

```
┌─────────────┐
│  Find Files │  (*.toml.tera)
└──────┬──────┘
       │ File list
       ↓
┌─────────────┐
│   Render    │  (Tera → TOML)
└──────┬──────┘
       │ Rendered TOML
       ↓
┌─────────────┐
│    Parse    │  (TOML → Config)
└──────┬──────┘
       │ Parsed config
       ↓
┌─────────────┐
│   Validate  │  (Shape + rules)
└──────┬──────┘
       │ Validation results
       ↓
┌─────────────┐
│   Report    │  (✅/❌ per file)
└─────────────┘
```

## Quick Start

### Validate Configuration Files

```bash
# Validate single file
$ clnrm validate tests/api.toml

✓ tests/api.toml is valid

# Validate directory
$ clnrm validate tests/

Validating 15 files...
✓ tests/api.toml
✓ tests/db.toml
❌ tests/auth.toml
  - [meta] section missing required 'name' field
  - Service 'db' referenced but not defined

✗ 1 file failed validation
```

### Dry Run Mode

```bash
# Fast validation without containers
$ clnrm run tests/ --dry-run

Validating configuration...
✓ All 15 files valid
Ready to run (would execute 45 scenarios)
```

### Continuous Validation

```bash
# Watch mode with validation
$ clnrm dev --watch --validate-only

👀 Watching for changes...
📝 Change detected: tests/api.toml.tera
✓ Validation passed (0.2s)
```

## Validation Categories

### 1. Required Blocks

Ensures essential configuration sections exist:

```bash
❌ Configuration must have either [meta] or [test.metadata] section
❌ Configuration must have at least one [[scenario]] or [[steps]]
```

**Example**:
```toml
# ❌ WRONG - missing meta and scenario
[otel]
exporter = "jaeger"

# ✅ CORRECT
[meta]
name = "my-test"
version = "0.7.0"

[[scenario]]
name = "test_scenario"
# ...
```

### 2. Invalid Structure

Validates field presence and format:

```bash
❌ [meta] section missing required 'name' field
❌ [meta] section missing required 'version' field
❌ Scenario 'test' must have at least one step
```

**Example**:
```toml
# ❌ WRONG - missing required fields
[meta]
description = "Test"

# ✅ CORRECT
[meta]
name = "api-test"
version = "0.7.0"
description = "API integration test"
```

### 3. Orphan References

Detects undefined service references:

```bash
❌ Step 'api_call' references undefined service 'api_server'
❌ Scenario 'load_test' step 'query' references undefined service 'database'
```

**Example**:
```toml
# ❌ WRONG - service not defined
[[scenario]]
name = "test"
[[scenario.steps]]
name = "call_api"
service = "api_server"  # Not defined!

# ✅ CORRECT
[services.api_server]
type = "generic_container"
image = "nginx:latest"

[[scenario]]
name = "test"
[[scenario.steps]]
name = "call_api"
service = "api_server"  # Defined ✓
```

### 4. Invalid Durations

Checks duration constraint validity:

```bash
❌ Span 'api.request' has invalid duration: min (500) > max (100)
```

**Example**:
```toml
# ❌ WRONG - min > max
[[expect.span]]
name = "api.request"
min_duration_ms = 500
max_duration_ms = 100

# ✅ CORRECT
[[expect.span]]
name = "api.request"
min_duration_ms = 100
max_duration_ms = 500
```

### 5. Circular Ordering

Detects cycles in temporal constraints:

```bash
❌ Circular temporal ordering detected involving span 'api.request'
```

**Example**:
```toml
# ❌ WRONG - circular dependency
[expect.order]
must_precede = [
  ["A", "B"],
  ["B", "C"],
  ["C", "A"]  # Creates cycle: A→B→C→A
]

# ✅ CORRECT - acyclic
[expect.order]
must_precede = [
  ["A", "B"],
  ["B", "C"]  # Linear: A→B→C
]
```

### 6. Invalid Glob Patterns

Validates glob pattern syntax:

```bash
❌ Invalid glob pattern in span expectation 'api.[': Unclosed bracket
```

**Example**:
```toml
# ❌ WRONG - invalid glob
[[expect.span]]
name = "api.[invalid"  # Unclosed bracket

# ✅ CORRECT
[[expect.span]]
name = "api.*"  # Valid glob
```

### 7. OTEL Configuration Errors

Validates OpenTelemetry exporter configuration:

```bash
❌ Invalid OTEL exporter 'invalid'. Valid options: jaeger, otlp, otlp-http, otlp-grpc, datadog, newrelic
❌ OTEL sample_ratio must be between 0.0 and 1.0, got 1.5
```

**Example**:
```toml
# ❌ WRONG - invalid exporter
[otel]
exporter = "invalid"
sample_ratio = 1.5

# ✅ CORRECT
[otel]
exporter = "jaeger"
sample_ratio = 0.5
```

### 8. Container Image Validation (v0.7.0 Enhanced)

Validates Docker image format and naming:

```bash
❌ Service 'api' has empty image
❌ Service 'db' has invalid image format 'alpine latest' (contains spaces)
❌ Service 'web' has invalid image format 'test!' (invalid characters)
```

**Example**:
```toml
# ❌ WRONG - various issues
[services.bad1]
image = ""

[services.bad2]
image = "alpine latest"  # Space in name

[services.bad3]
image = "test!"  # Invalid character

# ✅ CORRECT
[services.good1]
image = "alpine:latest"

[services.good2]
image = "ubuntu:20.04"

[services.good3]
image = "docker.io/library/postgres:14"
```

### 9. Port Binding Validation (v0.7.0 Enhanced)

Checks for port conflicts and reserved ports:

```bash
❌ Service 'api' uses reserved port 80. Use ports >= 1024
❌ Port conflict: port 8080 used by multiple services: api, web
```

**Example**:
```toml
# ❌ WRONG - reserved port
[services.api]
ports = [80]  # Reserved system port

# ❌ WRONG - port conflict
[services.api]
ports = [8080]

[services.web]
ports = [8080]  # Conflict!

# ✅ CORRECT
[services.api]
ports = [8080]

[services.web]
ports = [9000]  # Unique port
```

### 10. Volume Mount Validation (v0.7.0 Enhanced)

Validates volume path format and safety:

```bash
❌ Service 'api' volume 0: host path 'data' must be absolute
❌ Service 'db' volume 0: mounting to system path '/etc' is dangerous
```

**Example**:
```toml
# ❌ WRONG - relative path
[[services.api.volumes]]
host_path = "data"  # Not absolute
container_path = "/app/data"

# ❌ WRONG - dangerous system path
[[services.db.volumes]]
host_path = "/tmp/data"
container_path = "/etc"  # System directory!

# ✅ CORRECT
[[services.api.volumes]]
host_path = "/tmp/data"  # Absolute
container_path = "/app/data"  # Safe path
```

### 11. Environment Variable Validation (v0.7.0 Enhanced)

Validates env var names and detects potential secrets:

```bash
❌ Service 'api': environment variable name cannot be empty
❌ Service 'db': invalid environment variable name '123_VAR'
❌ Service 'auth': potential hardcoded sensitive value in 'API_KEY'
```

**Example**:
```toml
# ❌ WRONG - various issues
[services.bad]
env = {
  "" = "value",           # Empty name
  "123_VAR" = "value",    # Starts with number
  "API_KEY" = "secret123" # Hardcoded secret
}

# ✅ CORRECT
[services.good]
env = {
  "APP_ENV" = "production",
  "DATABASE_URL" = "${DB_URL}",  # Template variable
  "API_KEY" = "$API_KEY"         # Environment reference
}
```

### 12. Service Dependencies (v0.7.0 Enhanced)

Detects circular service dependencies:

```bash
❌ Circular service dependency detected involving 'api'
```

**Example**:
```toml
# ❌ WRONG - circular dependency via health checks
[services.api]
[services.api.health_check]
cmd = ["curl", "http://db:5432"]  # api depends on db

[services.db]
[services.db.health_check]
cmd = ["curl", "http://api:8080"]  # db depends on api (cycle!)

# ✅ CORRECT - one-way dependency
[services.api]
[services.api.health_check]
cmd = ["curl", "http://db:5432"]  # api depends on db

[services.db]
[services.db.health_check]
cmd = ["pg_isready"]  # db has no dependency on api
```

## Programmatic Usage

### Basic Validation

```rust
use clnrm_core::validation::shape::ShapeValidator;
use std::path::Path;

fn validate_config(path: &Path) -> Result<()> {
    let mut validator = ShapeValidator::new();
    let result = validator.validate_file(path)?;

    if result.passed {
        println!("✓ Configuration valid");
    } else {
        println!("✗ Validation failed:");
        for error in result.errors {
            println!("  - {}", error.message);
        }
    }

    Ok(())
}
```

### Validation with Error Categorization

```rust
use clnrm_core::validation::shape::{ShapeValidator, ErrorCategory};

let mut validator = ShapeValidator::new();
let result = validator.validate_file(path)?;

// Group errors by category
for error in result.errors {
    match error.category {
        ErrorCategory::MissingRequired => {
            println!("Missing required field: {}", error.message);
        }
        ErrorCategory::OrphanReference => {
            println!("Undefined reference: {}", error.message);
        }
        ErrorCategory::InvalidDuration => {
            println!("Duration error: {}", error.message);
        }
        ErrorCategory::CircularOrdering => {
            println!("Circular dependency: {}", error.message);
        }
        _ => {
            println!("Error: {}", error.message);
        }
    }
}
```

### Batch Validation

```rust
use clnrm_core::validation::shape::ShapeValidator;
use walkdir::WalkDir;

fn validate_directory(dir: &Path) -> Result<Vec<ValidationError>> {
    let mut validator = ShapeValidator::new();
    let mut all_errors = Vec::new();

    for entry in WalkDir::new(dir)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension() == Some("toml".as_ref()))
    {
        let result = validator.validate_file(entry.path())?;
        if !result.passed {
            all_errors.extend(result.errors);
        }
    }

    Ok(all_errors)
}
```

### Custom Validation Rules

```rust
use clnrm_core::validation::shape::{ShapeValidator, ShapeValidationError, ErrorCategory};

struct CustomValidator {
    base: ShapeValidator,
}

impl CustomValidator {
    fn validate_custom_rules(&mut self, config: &TestConfig) -> Result<()> {
        // Call base validation
        self.base.validate_config(config)?;

        // Add custom rules
        if let Some(ref meta) = config.meta {
            if !meta.name.starts_with("test_") {
                self.base.errors.push(ShapeValidationError::new(
                    ErrorCategory::InvalidStructure,
                    "Test name must start with 'test_'"
                ));
            }
        }

        Ok(())
    }
}
```

## CLI Integration

### Validation Command

```bash
# Validate single file
$ clnrm validate tests/api.toml

# Validate directory
$ clnrm validate tests/

# Validate with verbose output
$ clnrm validate tests/ --verbose

# Validate specific pattern
$ clnrm validate 'tests/**/*.toml'

# JSON output for tooling
$ clnrm validate tests/ --format json
```

### CI/CD Integration

**GitHub Actions**:
```yaml
- name: Validate configuration
  run: clnrm validate tests/ --format json > validation.json

- name: Upload validation results
  uses: actions/upload-artifact@v3
  with:
    name: validation-results
    path: validation.json
```

**Pre-commit Hook**:
```bash
#!/bin/sh
# .git/hooks/pre-commit

clnrm validate tests/ || {
    echo "Configuration validation failed"
    exit 1
}
```

## Performance

### Benchmarks

```bash
$ time clnrm validate tests/
✓ 100 files validated

real    0m0.842s
user    0m0.621s
sys     0m0.201s

# vs. full run with containers
$ time clnrm run tests/ --dry-run
✓ 100 files validated

real    0m45.234s
user    0m2.145s
sys     0m1.892s
```

**Performance improvement**: ~54x faster (45s → 0.84s)

### Optimization Tips

1. **Validate before running**: Catch errors early
2. **Use --dry-run**: Skip container startup
3. **Validate in CI**: Prevent bad configs from merging
4. **Watch mode**: Continuous validation during development

## Best Practices

### 1. Validate Before Commit

```bash
# Git pre-commit hook
#!/bin/sh
git diff --cached --name-only | grep '\.toml$' | while read file; do
    clnrm validate "$file" || exit 1
done
```

### 2. Use in CI Pipeline

```yaml
stages:
  - validate
  - test

validate-config:
  stage: validate
  script:
    - clnrm validate tests/
  only:
    - merge_requests
```

### 3. Validate in Watch Mode

```bash
# Continuous validation during development
$ clnrm dev --watch --validate-only
```

### 4. Use JSON Output for Tooling

```bash
# Generate machine-readable validation report
$ clnrm validate tests/ --format json | jq '.errors[] | select(.category == "OrphanReference")'
```

## Troubleshooting

### Validation Passes but Run Fails

**Problem**: Config validates but fails at runtime

**Cause**: Validation only checks static structure, not runtime behavior

**Solution**: Validation catches syntax errors, not runtime issues like:
- Container image doesn't exist
- Network connectivity problems
- Service startup failures

Use dry-run to catch more issues:

```bash
$ clnrm run tests/ --dry-run
```

### False Positives in Validation

**Problem**: Validator reports errors for valid config

**Cause**: Template variables not expanded

**Solution**: Ensure Tera rendering happens before validation:

```bash
# ✅ CORRECT - renders first
$ clnrm validate tests/api.toml.tera

# Internally:
# 1. Render Tera template
# 2. Validate rendered TOML
```

### Too Many Validation Errors

**Problem**: Overwhelming number of errors

**Solution**: Fix errors in order:

1. **Required blocks** (highest priority)
2. **Invalid structure**
3. **Orphan references**
4. **Other errors**

```bash
# Show only high-priority errors
$ clnrm validate tests/ --severity high
```

## Implementation Details

### Validation Algorithm

1. **Parse** TOML configuration
2. **Check** required blocks exist
3. **Validate** OTEL configuration
4. **Check** scenario structure
5. **Validate** service references
6. **Check** duration constraints
7. **Detect** circular ordering
8. **Validate** glob patterns
9. **Check** container images (v0.7.0)
10. **Validate** port bindings (v0.7.0)
11. **Check** volume mounts (v0.7.0)
12. **Validate** environment variables (v0.7.0)
13. **Detect** service dependencies (v0.7.0)

### Error Categorization

```rust
pub enum ErrorCategory {
    MissingRequired,     // Missing essential blocks
    InvalidStructure,    // Malformed configuration
    OrphanReference,     // Undefined references
    InvalidDuration,     // Invalid time constraints
    CircularOrdering,    // Circular dependencies
    InvalidGlob,         // Malformed patterns
    OtelError,          // OTEL configuration issues
}
```

### DFS Cycle Detection

Uses depth-first search for detecting circular dependencies:

```rust
fn has_cycle_dfs(
    node: &str,
    graph: &HashMap<String, Vec<String>>,
    visited: &mut HashSet<String>,
    rec_stack: &mut HashSet<String>,
) -> bool {
    visited.insert(node.to_string());
    rec_stack.insert(node.to_string());

    if let Some(neighbors) = graph.get(node) {
        for neighbor in neighbors {
            if !visited.contains(neighbor) {
                if has_cycle_dfs(neighbor, graph, visited, rec_stack) {
                    return true;
                }
            } else if rec_stack.contains(neighbor) {
                return true; // Cycle detected
            }
        }
    }

    rec_stack.remove(node);
    false
}
```

## API Reference

See [Rust documentation](https://docs.rs/clnrm-core/latest/clnrm_core/validation/) for complete API reference.

## Related Features

- [Formatting](FORMATTING.md) - Format before validation
- [Watch Mode](WATCH.md) - Continuous validation
- [Cache System](CACHE.md) - Skip validated files

## Migration from v0.6.0

v0.6.0 had basic TOML parsing validation only.

v0.7.0 adds comprehensive shape validation with:
- Enhanced error messages with suggestions
- Container image format validation
- Port conflict detection
- Volume mount safety checks
- Environment variable validation
- Service dependency cycle detection

**Upgrade path**: No breaking changes - enhanced validation is automatic!

```bash
# v0.6.0 validation
$ clnrm validate tests/api.toml
✓ Valid TOML syntax

# v0.7.0 validation (more comprehensive)
$ clnrm validate tests/api.toml
✓ Valid TOML syntax
✓ Required blocks present
✓ All service references valid
✓ No port conflicts
✓ Volume mounts safe
✓ Environment variables valid
✓ No circular dependencies
```
