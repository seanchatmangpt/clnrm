# CLI Live-Check Usage Guide

This document provides comprehensive examples for using the new Weaver live-check CLI flags.

## Quick Start

```bash
# Basic live-check validation
clnrm run --live-check tests/

# With 80/20 validation mode
clnrm run --live-check --validation-mode 80_20 tests/

# With custom registry path
clnrm run --live-check --registry-path ./custom-registry tests/
```

## CLI Flags Reference

### `clnrm run` Flags

| Flag | Description | Default | Example |
|------|-------------|---------|---------|
| `--live-check` | Enable Weaver live-check validation | `false` | `--live-check` |
| `--validation-mode <MODE>` | Validation mode: strict, lenient, 80_20, minimal | `strict` | `--validation-mode 80_20` |
| `--registry-path <PATH>` | Path to Weaver registry | (auto-resolved) | `--registry-path ./registry` |
| `--otlp-port <PORT>` | OTLP port for Weaver (0 = auto-discover) | `0` | `--otlp-port 4317` |
| `--admin-port <PORT>` | Admin port for Weaver (0 = auto-discover) | `0` | `--admin-port 8080` |
| `--diagnostic-format <FORMAT>` | Diagnostic output format: ansi, json, github | `ansi` | `--diagnostic-format json` |
| `--stop-timeout <SECONDS>` | Stop condition timeout | `300` | `--stop-timeout 600` |

## Usage Examples

### Basic Validation

Enable live-check validation with default settings:

```bash
clnrm run --live-check tests/
```

This will:
- Auto-discover the Weaver registry at `/usr/local/share/clnrm/registry`
- Use strict validation mode
- Auto-discover available OTLP and admin ports
- Display results in ANSI color format

### Validation Modes

#### Strict Mode (Default)

All violations fail validation:

```bash
clnrm run --live-check --validation-mode strict tests/
```

**Use for:**
- Production releases
- Compliance requirements
- Final certification

#### Lenient Mode

Only critical violations fail:

```bash
clnrm run --live-check --validation-mode lenient tests/
```

**Use for:**
- Development
- Iterative improvement
- Learning phase

#### 80/20 Mode

Focus on 20% of schemas that provide 80% of value:

```bash
clnrm run --live-check --validation-mode 80_20 tests/
```

**Use for:**
- Fast validation
- CI pipelines
- Quick feedback loops

#### Minimal Mode

Minimal validation for quick checks:

```bash
clnrm run --live-check --validation-mode minimal tests/
```

**Use for:**
- Local development
- Quick sanity checks
- Pre-commit hooks

### Custom Registry Path

Override the default registry location:

```bash
clnrm run --live-check --registry-path ./custom-registry tests/
```

**Use cases:**
- Development with custom schemas
- Testing schema changes
- Multi-project setups

### Port Configuration

#### Auto-Discovery (Default)

Let Weaver automatically find available ports:

```bash
clnrm run --live-check tests/
```

#### Manual Port Configuration

Specify exact ports to use:

```bash
clnrm run --live-check --otlp-port 4317 --admin-port 8080 tests/
```

**Use when:**
- Ports are known and available
- Running in containerized environments
- Coordinating with other services

### Diagnostic Formats

#### ANSI (Default)

Human-readable colored output:

```bash
clnrm run --live-check --diagnostic-format ansi tests/
```

#### JSON Format

Machine-readable output for tooling:

```bash
clnrm run --live-check --diagnostic-format json tests/
```

**Use for:**
- IDE integration
- Parsing results in scripts
- Storing validation history

#### GitHub Actions Format

GitHub Actions annotations:

```bash
clnrm run --live-check --diagnostic-format github tests/
```

**Use for:**
- CI/CD pipelines
- Pull request checks
- Automated reporting

### Advanced Examples

#### Full CI/CD Configuration

```bash
clnrm run \
  --live-check \
  --validation-mode 80_20 \
  --diagnostic-format github \
  --stop-timeout 600 \
  --parallel \
  --jobs 8 \
  tests/
```

#### Development Workflow

```bash
clnrm run \
  --live-check \
  --validation-mode lenient \
  --registry-path ./dev-registry \
  --diagnostic-format ansi \
  tests/
```

#### Production Release

```bash
clnrm run \
  --live-check \
  --validation-mode strict \
  --diagnostic-format json \
  --report-junit results.xml \
  tests/
```

## `clnrm live-check` Subcommands

### Show Status

Display current live-check configuration:

```bash
clnrm live-check status
```

**Output includes:**
- Weaver installation status
- Registry location
- Schema count
- Configuration values
- Available validation modes

### Validate Registry

Validate registry schemas:

```bash
clnrm live-check validate-registry --registry ./registry
```

**Use to:**
- Verify schema syntax
- Check registry structure
- Validate manifest file

### Test Weaver Installation

Test Weaver installation and capabilities:

```bash
clnrm live-check test-weaver
```

**Checks:**
- Weaver command availability
- `weaver registry` support
- `weaver registry live-check` support
- Version information

### Show Validation Modes

Display available validation modes with descriptions:

```bash
clnrm live-check modes
```

**Shows:**
- Mode names and descriptions
- Use cases for each mode
- Example commands
- TOML configuration examples

### Show Weaver Version

Display Weaver version:

```bash
clnrm live-check version
```

## CLI Precedence Rules

CLI flags take precedence over TOML configuration:

```toml
# test.clnrm.toml
[weaver]
enabled = true
validation_mode = "lenient"
```

```bash
# CLI overrides TOML
clnrm run --validation-mode strict test.clnrm.toml
# Result: Uses "strict" mode, not "lenient"
```

### Precedence Order

1. **CLI flags** (highest priority)
2. **TOML configuration**
3. **Default values** (lowest priority)

## Environment Variables

### `CLNRM_REGISTRY_PATH`

Override default registry path resolution:

```bash
export CLNRM_REGISTRY_PATH=/path/to/custom/registry
clnrm run --live-check tests/
```

**Use for:**
- Development environments
- Custom installations
- Multi-registry setups

## Integration with Existing Flags

### Combining with Other `clnrm run` Flags

```bash
# Parallel execution with live-check
clnrm run --live-check --parallel --jobs 8 tests/

# With JUnit report
clnrm run --live-check --report-junit results.xml tests/

# With OTEL export
clnrm run --live-check --otel-exporter otlp-grpc --otel-endpoint http://localhost:4317 tests/

# Watch mode with live-check
clnrm run --live-check --watch tests/

# Sharded execution
clnrm run --live-check --shard 1/4 tests/
```

## Troubleshooting

### Weaver Not Found

```bash
clnrm live-check test-weaver
# Output: ✗ Weaver not found

# Install Weaver
cargo install weaver-cli
```

### Registry Not Found

```bash
clnrm live-check status
# Output: ✗ Registry not found

# Set registry path
export CLNRM_REGISTRY_PATH=/path/to/registry
# OR
clnrm run --live-check --registry-path /path/to/registry tests/
```

### Port Conflicts

```bash
# Use auto-discovery (default)
clnrm run --live-check tests/

# OR specify different ports
clnrm run --live-check --otlp-port 4318 --admin-port 8081 tests/
```

## Best Practices

### Local Development

```bash
clnrm run \
  --live-check \
  --validation-mode lenient \
  --watch \
  tests/
```

### CI/CD Pipelines

```bash
clnrm run \
  --live-check \
  --validation-mode 80_20 \
  --diagnostic-format github \
  --parallel \
  --report-junit results.xml \
  tests/
```

### Production Validation

```bash
clnrm run \
  --live-check \
  --validation-mode strict \
  --diagnostic-format json \
  --stop-timeout 600 \
  tests/
```

### Pre-Commit Hook

```bash
#!/bin/bash
clnrm run \
  --live-check \
  --validation-mode minimal \
  --fail-fast \
  tests/
```

## Help Text

Get help on any command:

```bash
# Main help
clnrm --help

# Run command help
clnrm run --help

# Live-check subcommands help
clnrm live-check --help
clnrm live-check status --help
clnrm live-check validate-registry --help
```

## Examples by Use Case

### First-Time User

```bash
# Check if Weaver is installed
clnrm live-check test-weaver

# Show current configuration
clnrm live-check status

# Run first validation
clnrm run --live-check tests/
```

### CI/CD Engineer

```bash
# Fast validation for CI
clnrm run \
  --live-check \
  --validation-mode 80_20 \
  --diagnostic-format github \
  --parallel \
  --jobs 8 \
  --report-junit results.xml \
  tests/
```

### Schema Developer

```bash
# Validate custom registry
clnrm live-check validate-registry --registry ./my-registry

# Run tests with custom registry
clnrm run \
  --live-check \
  --registry-path ./my-registry \
  --validation-mode lenient \
  tests/
```

### QA Engineer

```bash
# Strict validation for release
clnrm run \
  --live-check \
  --validation-mode strict \
  --diagnostic-format json \
  --output validation-report.json \
  tests/
```

## Migration from TOML-Only Configuration

### Before (TOML only)

```toml
[weaver]
enabled = true
validation_mode = "80_20"
```

```bash
clnrm run tests/
```

### After (CLI flags)

```bash
# Equivalent CLI command
clnrm run --live-check --validation-mode 80_20 tests/

# Override TOML
clnrm run --validation-mode strict tests/  # Uses strict, not 80_20
```

## Summary

The new CLI flags provide:

- **Flexibility**: Configure validation without modifying TOML files
- **Precedence**: CLI flags override TOML configuration
- **Convenience**: Quick access to common validation scenarios
- **Integration**: Works seamlessly with existing `clnrm run` flags
- **Discovery**: Subcommands for status, testing, and exploration

For more information, see:
- `clnrm run --help`
- `clnrm live-check --help`
- Phase 1-2 implementation documentation
