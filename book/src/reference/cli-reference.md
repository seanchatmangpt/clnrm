# CLI Reference

This chapter provides comprehensive documentation for all clnrm v2.0.0 CLI commands, options, and usage patterns.

## Overview

The clnrm v2.0.0 CLI provides these main command categories:
- **Core Commands** - Basic test execution and validation
- **Plugin Commands** - Plugin management and discovery
- **Utility Commands** - Development and debugging tools
- **Advanced Commands** - Complex operations and administration

## Command Structure

```bash
clnrm [OPTIONS] [COMMAND] [ARGS...]
```

### Global Options

| Option | Description | Example |
|--------|-------------|---------|
| `--help` | Show help information | `clnrm --help` |
| `--version` | Show version information | `clnrm --version` |
| `--verbose` | Enable verbose logging | `clnrm run --verbose` |
| `--config <FILE>` | Specify configuration file | `clnrm run --config test.toml` |
| `--format <FORMAT>` | Output format (auto, text, json) | `clnrm run --format json` |

## Core Commands

### `run` - Execute Tests

Execute tests from TOML configuration files.

```bash
clnrm run [OPTIONS] [PATHS...]
```

**Options:**

| Option | Description | Example |
|--------|-------------|---------|
| `--parallel` | Run tests in parallel | `clnrm run --parallel tests/` |
| `--jobs <N>` | Number of parallel workers | `clnrm run --jobs 4 tests/` |
| `--fail-fast` | Stop on first failure | `clnrm run --fail-fast tests/` |
| `--watch` | Watch mode (rerun on changes) | `clnrm run --watch tests/` |
| `--force` | Force run all tests | `clnrm run --force tests/` |
| `--digest` | Generate reproducibility digest | `clnrm run --digest tests/` |
| `--validate` | Enable Weaver live-check validation | `clnrm run --validate tests/` |
| `--otel-exporter` | OTEL exporter type | `clnrm run --otel-exporter otlp-http tests/` |
| `--otel-endpoint` | OTEL endpoint | `clnrm run --otel-endpoint http://localhost:4318 tests/` |

**Examples:**

```bash
# Run all tests in directory
clnrm run tests/

# Run specific test file
clnrm run test.clnrm.toml

# Run with parallel execution
clnrm run --parallel --jobs 4 tests/

# Run with Weaver validation
clnrm run --validate --otel-exporter otlp-http tests/

# Run in watch mode
clnrm run --watch tests/
```

### `validate` - Validate Configuration

Validate TOML configuration files without execution.

```bash
clnrm validate [OPTIONS] [PATHS...]
```

**Options:**

| Option | Description | Example |
|--------|-------------|---------|
| `--strict` | Enable strict validation | `clnrm validate --strict test.toml` |

**Examples:**

```bash
# Validate single file
clnrm validate test.clnrm.toml

# Validate directory recursively
clnrm validate tests/

# Validate with strict mode
clnrm validate --strict test.clnrm.toml
```

### `init` - Initialize Project

Initialize a new project with sample configuration.

```bash
clnrm init [OPTIONS] [PATH]
```

**Options:**

| Option | Description | Example |
|--------|-------------|---------|
| `--template <TEMPLATE>` | Template to use | `clnrm init --template api` |
| `--force` | Overwrite existing files | `clnrm init --force` |

**Available Templates:**

- `default` - Basic test configuration
- `api` - API service testing
- `database` - Database testing
- `multi-service` - Multi-service orchestration

**Examples:**

```bash
# Initialize with default template
clnrm init

# Initialize with API template
clnrm init --template api

# Initialize in specific directory
clnrm init ./my-project
```

## Plugin Commands

### `plugins` - List Plugins

List all registered plugins.

```bash
clnrm plugins [OPTIONS]
```

**Options:**

| Option | Description | Example |
|--------|-------------|---------|
| `--format <FORMAT>` | Output format (table, json) | `clnrm plugins --format json` |
| `--details` | Show detailed plugin information | `clnrm plugins --details` |

**Examples:**

```bash
# List all plugins
clnrm plugins

# List with detailed information
clnrm plugins --details

# List in JSON format
clnrm plugins --format json
```

## Utility Commands

### `pull` - Pre-pull Images

Pre-pull Docker images for faster execution.

```bash
clnrm pull [OPTIONS] [PATHS...]
```

**Options:**

| Option | Description | Example |
|--------|-------------|---------|
| `--parallel` | Pull images in parallel | `clnrm pull --parallel tests/` |
| `--jobs <N>` | Number of parallel jobs | `clnrm pull --jobs 4 tests/` |

**Examples:**

```bash
# Pull images for all tests
clnrm pull tests/

# Pull with parallel downloads
clnrm pull --parallel --jobs 4 tests/
```

### `fmt` - Format TOML Files

Format TOML files for consistency.

```bash
clnrm fmt [OPTIONS] [PATHS...]
```

**Options:**

| Option | Description | Example |
|--------|-------------|---------|
| `--check` | Check if files are formatted | `clnrm fmt --check tests/` |

**Examples:**

```bash
# Format all TOML files
clnrm fmt tests/

# Check if files are formatted
clnrm fmt --check tests/
```

### `lint` - Lint Configuration

Lint TOML files for issues.

```bash
clnrm lint [OPTIONS] [PATHS...]
```

**Options:**

| Option | Description | Example |
|--------|-------------|---------|
| `--fix` | Auto-fix issues | `clnrm lint --fix tests/` |

**Examples:**

```bash
# Lint all test files
clnrm lint tests/

# Auto-fix issues
clnrm lint --fix tests/
```

## Advanced Commands

### `template` - Template Operations

Manage Tera templates.

```bash
clnrm template <COMMAND> [OPTIONS]
```

**Subcommands:**

- `render <TEMPLATE>` - Render template to TOML
- `validate <TEMPLATE>` - Validate template syntax

**Examples:**

```bash
# Render template
clnrm template render test.clnrm.toml.tera > test.clnrm.toml

# Validate template
clnrm template validate test.clnrm.toml.tera
```

### `baseline` - Baseline Management

Manage performance baselines.

```bash
clnrm baseline <COMMAND> [OPTIONS]
```

**Subcommands:**

- `create <NAME>` - Create new baseline
- `update <NAME>` - Update existing baseline
- `list` - List baselines

**Examples:**

```bash
# Create new baseline
clnrm baseline create production

# Update existing baseline
clnrm baseline update production

# List all baselines
clnrm baseline list
```

### `report` - Report Generation

Generate test reports.

```bash
clnrm report [OPTIONS] [COMMAND]
```

**Options:**

| Option | Description | Example |
|--------|-------------|---------|
| `--format <FORMAT>` | Report format (html, json, junit) | `clnrm report --format html` |
| `--output <FILE>` | Output file | `clnrm report --output report.html` |

**Examples:**

```bash
# Generate HTML report
clnrm report --format html --output test-report.html

# Generate JSON report
clnrm report --format json --output test-results.json
```

## Development Commands

### `dev` - Development Tools

Development and debugging tools.

```bash
clnrm dev <COMMAND> [OPTIONS]
```

**Subcommands:**

- `watch` - Watch for changes and re-run
- `debug` - Run with debug output

**Examples:**

```bash
# Watch for changes
clnrm dev watch tests/

# Run with debug output
clnrm dev debug tests/
```

### `self-test` - Framework Self-Testing

Test the framework itself.

```bash
clnrm self-test [OPTIONS]
```

**Options:**

| Option | Description | Example |
|--------|-------------|---------|
| `--suite <SUITE>` | Test suite to run | `clnrm self-test --suite otel` |

**Examples:**

```bash
# Run all self-tests
clnrm self-test

# Run specific suite
clnrm self-test --suite container
```

## v2.0.0 Breaking Changes

### Configuration Changes
- `[services.X]` → `[containers.X]`
- `service = "X"` → `container = "X"`
- `[test.metadata]` → `[test]`
- Removed `type = "generic_container"` field

### Execution Model Changes
- Commands now execute via `docker exec` into running containers
- Environment variables persist across steps
- Container lifecycle is more predictable

## Error Handling

### Exit Codes

clnrm uses standard exit codes:

| Code | Description |
|------|-------------|
| 0 | Success |
| 1 | General error |
| 2 | Configuration error |
| 3 | Execution error |
| 4 | Validation error |

### Error Output

Errors are displayed in a structured format:

```bash
$ clnrm run invalid-test.toml
❌ Error: Configuration validation failed

Details:
  - Invalid TOML syntax in test.toml:23
  - Missing required field: 'test.name'

Suggestions:
  - Check TOML syntax
  - Add required test fields

Run 'clnrm validate test.toml' for detailed validation.
```

## Best Practices

### 1. Use Appropriate Options

```bash
# ✅ Good: Use appropriate options for the task
clnrm run tests/ --parallel --jobs 4 --validate

# ❌ Bad: Missing important options
clnrm run tests/
```

### 2. Validate Before Running

```bash
# ✅ Good: Validate before running
clnrm validate test.toml && clnrm run test.toml

# ❌ Bad: Run without validation
clnrm run test.toml
```

### 3. Use Parallel Execution for Large Test Suites

```bash
# ✅ Good: Parallel execution for large suites
clnrm run tests/ --parallel --jobs $(nproc)

# ❌ Bad: Sequential execution for large suites
clnrm run tests/
```

## Troubleshooting

### Common Issues

**Issue: Container execution fails**
```bash
# Check if Docker is running
docker info

# Check if images are available
clnrm pull tests/

# Run with verbose output
clnrm run --verbose tests/
```

**Issue: Environment variables not persisting**
```bash
# In v2.0.0, env vars persist across steps
# Make sure you're using [containers.X] not [services.X]
clnrm validate test.toml
```

**Issue: Plugin not found**
```bash
# List available plugins
clnrm plugins

# Check plugin configuration
clnrm validate test.toml
```

### Debug Mode

Enable debug mode for detailed information:

```bash
# Run with debug output
clnrm run --verbose tests/

# Generate execution traces
clnrm dev debug tests/
```

## Examples

### Complete Test Workflow

```bash
#!/bin/bash
# Complete test workflow for v2.0.0

echo "🔍 Validating configuration..."
clnrm validate tests/

echo "📦 Pre-pulling images..."
clnrm pull tests/

echo "🧪 Running tests..."
clnrm run tests/ --parallel --jobs 4 --validate

echo "📊 Generating reports..."
clnrm report --format html,json --output reports/

echo "✅ Test workflow complete"
```

### Migration Workflow (v1.x to v2.0.0)

```bash
#!/bin/bash
# Migration workflow

echo "🔄 Migrating from v1.x to v2.0.0..."

# Update configuration files
find . -name "*.clnrm.toml" -exec sed -i 's/\[services\./[containers./g' {} \;
find . -name "*.clnrm.toml" -exec sed -i 's/service = /container = /g' {} \;
find . -name "*.clnrm.toml" -exec sed -i 's/\[test\.metadata\]/[test]/g' {} \;

# Remove type fields
find . -name "*.clnrm.toml" -exec sed -i '/type = "generic_container"/d' {} \;

echo "✅ Migration complete. Run 'clnrm validate' to verify."
```

## Next Steps

Now that you understand the CLI:

1. **Migrate from v1.x**: See [Migration Guide](../docs/V2_0_0_MIGRATION_GUIDE.md)
2. **Learn v2.0.0 config**: Move on to [TOML Schema](toml-schema.md)
3. **Understand error handling**: Learn about [Error Handling](error-handling.md)
4. **Master advanced usage**: Review the other chapters for advanced patterns

## Further Reading

- [Command Line Interface Guidelines](https://clig.dev/)
- [v2.0.0 Migration Guide](../docs/V2_0_0_MIGRATION_GUIDE.md)
- [v2.0.0 Architecture](../docs/V2_0_0_ARCHITECTURE.md)