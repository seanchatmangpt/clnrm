# Cleanroom v0.7.0+ CLI Guide

## 🚀 Overview

Cleanroom v0.7.0+ provides a streamlined, developer-experience-first CLI for hermetic integration testing with OpenTelemetry validation. The CLI emphasizes simplicity, speed, and reliability with no-prefix variables and Tera-first templating.

## 🎯 Core Commands

### `clnrm template otel`

Generate a new OTEL validation template.

```bash
# Generate a basic OTEL validation template
clnrm template otel > my-test.clnrm.toml

# Generate with custom variables
clnrm template otel --svc myapp --env prod > my-test.clnrm.toml
```

**Options:**
- `--svc <name>` - Service name (default: clnrm)
- `--env <env>` - Environment (default: ci)
- `--endpoint <url>` - OTEL endpoint (default: http://localhost:4318)
- `--exporter <type>` - OTEL exporter (default: otlp)
- `--image <image>` - Container image (default: registry/clnrm:1.0.0)

### `clnrm dev --watch`

Start development mode with hot reload.

```bash
# Watch for file changes and rerun tests
clnrm dev --watch

# Watch specific files only
clnrm dev --watch --only tests/my-test.clnrm.toml

# Watch with custom timebox
clnrm dev --watch --timebox 5000

# Watch with multiple workers
clnrm dev --watch --workers 4
```

**Options:**
- `--workers <n>` - Number of parallel workers (default: 1)
- `--only <file>::<scenario>` - Run only specific scenario
- `--timebox <ms>` - Maximum execution time per scenario
- `--no-clear` - Don't clear screen between runs

**Behavior:**
- Auto-detects changes to `.clnrm.toml` files
- Debounced event handling (200ms default)
- Shows first failing invariant immediately
- Hot loop remains stable across file changes

### `clnrm dry-run`

Validate configuration without executing containers.

```bash
# Validate all test files
clnrm dry-run

# Validate specific files
clnrm dry-run tests/my-test.clnrm.toml

# Validate with detailed output
clnrm dry-run --verbose
```

**Validates:**
- TOML syntax and structure
- Required blocks and keys
- Orphan service/scenario references
- Temporal ordering cycles
- Glob pattern validity

### `clnrm run`

Execute tests with container isolation.

```bash
# Run all tests (change-aware by default)
clnrm run

# Run with multiple workers
clnrm run --workers 4

# Force run all scenarios (ignore change detection)
clnrm run --force

# Run with JSON output
clnrm run --json > results.json
```

**Options:**
- `--workers <n>` - Parallel execution workers
- `--force` - Ignore change detection cache
- `--json` - JSON output format
- `--junit` - JUnit XML output

**Behavior:**
- Only changed scenarios run by default
- Dependent scenarios may be re-run if dependencies change
- Fresh container per scenario for hermeticity

### `clnrm pull`

Pull required container images.

```bash
# Pull all images referenced in tests
clnrm pull

# Pull specific images
clnrm pull myapp:latest otel/opentelemetry-collector:latest
```

### `clnrm diff`

Show differences between test runs.

```bash
# Show scenario changes
clnrm diff

# Show with JSON output
clnrm diff --json

# Show specific scenario diff
clnrm diff tests/my-test.clnrm.toml
```

### `clnrm graph`

Visualize test execution graph.

```bash
# ASCII graph visualization
clnrm graph --ascii

# Highlight missing edges
clnrm graph --ascii --highlight-missing

# Graph specific test file
clnrm graph tests/my-test.clnrm.toml
```

### `clnrm record`

Record test execution for later analysis.

```bash
# Record current test run
clnrm record

# Record with custom name
clnrm record --name "baseline-run"

# Record specific scenario
clnrm record tests/my-test.clnrm.toml
```

### `clnrm repro`

Reproduce previous test execution.

```bash
# Reproduce last run
clnrm repro

# Reproduce specific record
clnrm repro --id abc123

# Reproduce with same seed
clnrm repro --deterministic
```

### `clnrm redgreen`

Show pass/fail status with minimal output.

```bash
# Simple pass/fail for each scenario
clnrm redgreen

# With scenario names
clnrm redgreen --names

# Exit code indicates overall status
echo $?  # 0 = all passed, 1 = some failed
```

### `clnrm fmt`

Format TOML files deterministically.

```bash
# Format all test files
clnrm fmt

# Format specific files
clnrm fmt tests/*.clnrm.toml

# Check formatting (CI mode)
clnrm fmt --check

# Format and verify idempotency
clnrm fmt --verify
```

**Behavior:**
- Alphabetically sorted keys
- Consistent indentation
- `[vars]` table sorted by key name
- Idempotent operation

### `clnrm lint`

Lint test files for best practices.

```bash
# Lint all test files
clnrm lint

# Lint with specific rules
clnrm lint --rules missing-keys,orphan-services

# Lint in CI mode (non-zero exit on issues)
clnrm lint --strict
```

**Checks:**
- Missing required keys
- Orphan services/scenarios
- Invalid enum values
- Deprecated patterns

### `clnrm render --map`

Render templates and show variable mapping.

```bash
# Show how variables are resolved
clnrm render --map

# Show for specific file
clnrm render tests/my-test.clnrm.toml --map

# Show with environment context
clnrm render --map --env-file .env
```

### `clnrm spans`

Query and filter collected spans.

```bash
# Show all spans
clnrm spans

# Filter by span name
clnrm spans --grep "clnrm.run"

# Filter by attributes
clnrm spans --attr "service.name=myapp"

# Export to JSON
clnrm spans --json > spans.json
```

### `clnrm up collector`

Start local OTEL collector.

```bash
# Start collector with defaults
clnrm up collector

# Start with custom config
clnrm up collector --config collector.yaml

# Start in background
clnrm up collector --detach
```

### `clnrm down`

Stop local services.

```bash
# Stop collector and other services
clnrm down

# Stop specific service
clnrm down collector
```

## 🔧 Global Options

### `--help`

Show comprehensive help.

```bash
clnrm --help        # General help
clnrm run --help    # Command-specific help
```

### `--version`

Show version information.

```bash
clnrm --version
```

## 🚀 Development Workflow

### Typical Development Loop

```bash
# 1. Generate initial template
clnrm template otel > my-test.clnrm.toml

# 2. Start development mode
clnrm dev --watch

# 3. Edit my-test.clnrm.toml
# 4. Watch mode auto-detects changes and reruns
# 5. See results immediately (<3s latency)

# 6. Validate without containers
clnrm dry-run

# 7. Run full test suite
clnrm run

# 8. Check formatting
clnrm fmt --check
```

### CI/CD Integration

```bash
# Lint and format check
clnrm lint --strict
clnrm fmt --check

# Dry run validation
clnrm dry-run

# Execute tests
clnrm run --json --junit > results.xml

# Generate reports
clnrm diff --json
clnrm graph --ascii
```

## 📊 Performance Targets

- **Template cold run**: ≤5 seconds
- **Edit→rerun p50**: ≤1.5 seconds, p95 ≤3 seconds
- **Suite time improvement**: 30-50% vs v0.6.0 (change-aware + workers)
- **Dry-run validation**: <1 second for 10 files
- **First green**: <60 seconds for new users

## 🎯 Exit Codes

- `0` - Success
- `1` - Test failures or validation errors
- `2` - Configuration or environment errors
- `3` - Internal errors

## 🔍 Troubleshooting

### Common Issues

**"Template rendering failed"**
- Check TOML syntax with `clnrm dry-run`
- Verify variable references are correct
- Check for missing required variables

**"Container execution failed"**
- Ensure Docker/Podman is running
- Check image availability with `clnrm pull`
- Verify container resource limits

**"OTEL export failed"**
- Check collector status with `clnrm up collector`
- Verify endpoint configuration
- Check network connectivity

**"Watch mode not detecting changes"**
- Ensure file extensions are `.clnrm.toml`
- Check file permissions
- Verify inotify limits

---

*This CLI guide describes the v1.0 interface. For v0.7.0 compatibility information, see the migration guide.*
