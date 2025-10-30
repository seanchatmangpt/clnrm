# CLI Reference

This chapter provides comprehensive documentation for all clnrm CLI commands, options, and usage patterns.

## Overview

The clnrm CLI provides these main command categories:
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
| `--env <ENV>` | Set environment name | `clnrm run --env production` |

## Core Commands

### `run` - Execute Tests

Execute tests from TOML configuration files.

```bash
clnrm run [OPTIONS] <PATHS...>
```

**Options:**

| Option | Description | Example |
|--------|-------------|---------|
| `--parallel` | Run tests in parallel | `clnrm run --parallel tests/` |
| `--workers <N>` | Number of parallel workers | `clnrm run --workers 4 tests/` |
| `--timeout <MINUTES>` | Test timeout in minutes | `clnrm run --timeout 30 tests/` |
| `--dry-run` | Validate without executing | `clnrm run --dry-run tests/` |
| `--baseline <NAME>` | Use performance baseline | `clnrm run --baseline production tests/` |
| `--check-regressions` | Check for performance regressions | `clnrm run --check-regressions tests/` |

**Examples:**

```bash
# Run all tests in directory
clnrm run tests/

# Run specific test file
clnrm run test.clnrm.toml

# Run with parallel execution
clnrm run --parallel --workers 4 tests/

# Run with performance baseline
clnrm run --baseline production tests/performance/

# Run with regression checking
clnrm run --check-regressions tests/performance/
```

### `validate` - Validate Configuration

Validate TOML configuration files without execution.

```bash
clnrm validate [OPTIONS] <PATHS...>
```

**Options:**

| Option | Description | Example |
|--------|-------------|---------|
| `--strict` | Enable strict validation | `clnrm validate --strict test.toml` |
| `--schema` | Validate against schema | `clnrm validate --schema test.toml` |
| `--format <FORMAT>` | Output format (text, json) | `clnrm validate --format json test.toml` |

**Examples:**

```bash
# Validate single file
clnrm validate test.clnrm.toml

# Validate directory recursively
clnrm validate tests/

# Validate with strict mode
clnrm validate --strict test.clnrm.toml

# Validate with JSON output
clnrm validate --format json tests/
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
- `performance` - Performance testing
- `chaos` - Chaos engineering

**Examples:**

```bash
# Initialize with default template
clnrm init

# Initialize with API template
clnrm init --template api

# Initialize in specific directory
clnrm init ./my-project

# Force overwrite existing files
clnrm init --force --template multi-service
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

### `services` - Service Management

Manage running services.

```bash
clnrm services <COMMAND> [OPTIONS]
```

**Subcommands:**

- `status` - Show service status
- `logs` - Show service logs
- `restart` - Restart services
- `stop` - Stop services

**Examples:**

```bash
# Show service status
clnrm services status

# Show service logs
clnrm services logs --tail 100

# Restart all services
clnrm services restart

# Stop specific service
clnrm services stop database
```

## Utility Commands

### `pull` - Pre-pull Images

Pre-pull Docker images for faster execution.

```bash
clnrm pull [OPTIONS] <PATHS...>
```

**Options:**

| Option | Description | Example |
|--------|-------------|---------|
| `--parallel` | Pull images in parallel | `clnrm pull --parallel tests/` |
| `--jobs <N>` | Number of parallel jobs | `clnrm pull --jobs 4 tests/` |
| `--dry-run` | Show what would be pulled | `clnrm pull --dry-run tests/` |

**Examples:**

```bash
# Pull images for all tests
clnrm pull tests/

# Pull with parallel downloads
clnrm pull --parallel --jobs 4 tests/

# Show what would be pulled
clnrm pull --dry-run tests/
```

### `fmt` - Format TOML Files

Format TOML files for consistency.

```bash
clnrm fmt [OPTIONS] <PATHS...>
```

**Options:**

| Option | Description | Example |
|--------|-------------|---------|
| `--check` | Check if files are formatted | `clnrm fmt --check tests/` |
| `--diff` | Show diff of changes | `clnrm fmt --diff test.toml` |

**Examples:**

```bash
# Format all TOML files
clnrm fmt tests/

# Check if files are formatted
clnrm fmt --check tests/

# Show formatting diff
clnrm fmt --diff test.toml
```

### `lint` - Lint Configuration

Lint TOML files for issues.

```bash
clnrm lint [OPTIONS] <PATHS...>
```

**Options:**

| Option | Description | Example |
|--------|-------------|---------|
| `--rules <RULES>` | Enable specific rules | `clnrm lint --rules security,performance` |
| `--fix` | Auto-fix issues | `clnrm lint --fix tests/` |
| `--format <FORMAT>` | Output format (text, json) | `clnrm lint --format json tests/` |

**Examples:**

```bash
# Lint all test files
clnrm lint tests/

# Lint with specific rules
clnrm lint --rules security,performance tests/

# Auto-fix issues
clnrm lint --fix tests/

# Lint with JSON output
clnrm lint --format json tests/
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
- `list` - List available templates

**Examples:**

```bash
# Render template
clnrm template render test.clnrm.toml.tera > test.clnrm.toml

# Validate template
clnrm template validate test.clnrm.toml.tera

# List available templates
clnrm template list
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
- `compare <NAME>` - Compare against baseline

**Examples:**

```bash
# Create new baseline
clnrm baseline create production

# Update existing baseline
clnrm baseline update production

# List all baselines
clnrm baseline list

# Compare against baseline
clnrm baseline compare production
```

### `report` - Report Generation

Generate test reports.

```bash
clnrm report [OPTIONS] <COMMAND>
```

**Options:**

| Option | Description | Example |
|--------|-------------|---------|
| `--format <FORMAT>` | Report format (html, json, junit) | `clnrm report --format html` |
| `--output <FILE>` | Output file | `clnrm report --output report.html` |

**Subcommands:**

- `generate` - Generate comprehensive report
- `summary` - Generate summary report
- `trends` - Generate performance trends

**Examples:**

```bash
# Generate HTML report
clnrm report --format html --output test-report.html

# Generate JSON report
clnrm report --format json --output test-results.json

# Generate JUnit XML
clnrm report --format junit --output junit.xml

# Generate performance trends
clnrm report trends --baseline production
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
- `profile` - Profile execution
- `trace` - Generate execution traces

**Examples:**

```bash
# Watch for changes
clnrm dev watch tests/

# Run with debug output
clnrm dev debug tests/

# Profile execution
clnrm dev profile tests/

# Generate traces
clnrm dev trace tests/
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
| `--verbose` | Verbose output | `clnrm self-test --verbose` |

**Available Suites:**

- `framework` - Core framework functionality
- `container` - Container execution
- `plugin` - Plugin system
- `cli` - CLI functionality
- `otel` - OTEL integration

**Examples:**

```bash
# Run all self-tests
clnrm self-test

# Run specific suite
clnrm self-test --suite container

# Run with verbose output
clnrm self-test --verbose --suite otel
```

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
| 5 | Plugin error |

### Error Output

Errors are displayed in a structured format:

```bash
$ clnrm run invalid-test.toml
❌ Error: Configuration validation failed

Details:
  - Invalid TOML syntax in test.toml:23
  - Missing required field: 'test.metadata.name'
  - Invalid service configuration: 'services.api'

Suggestions:
  - Check TOML syntax
  - Add required metadata fields
  - Verify service configuration

Run 'clnrm validate test.toml' for detailed validation.
```

## Best Practices

### 1. Use Descriptive Command Names

```bash
# ✅ Good: Descriptive and clear
clnrm run tests/integration/ --parallel --workers 4

# ❌ Bad: Unclear purpose
clnrm run tests/
```

### 2. Use Appropriate Options

```bash
# ✅ Good: Use appropriate options for the task
clnrm run tests/performance/ --baseline production --check-regressions

# ❌ Bad: Missing important options
clnrm run tests/
```

### 3. Validate Before Running

```bash
# ✅ Good: Validate before running
clnrm validate test.toml && clnrm run test.toml

# ❌ Bad: Run without validation
clnrm run test.toml
```

### 4. Use Parallel Execution for Large Test Suites

```bash
# ✅ Good: Parallel execution for large suites
clnrm run tests/ --parallel --workers $(nproc)

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

**Issue: Performance regression detected**
```bash
# Check baseline
clnrm baseline compare production

# Run performance tests
clnrm run tests/performance/ --baseline production

# Update baseline if expected
clnrm baseline update production
```

**Issue: Plugin not found**
```bash
# List available plugins
clnrm plugins

# Check plugin configuration
clnrm validate test.toml

# Check plugin registration
clnrm plugins --details
```

### Debug Mode

Enable debug mode for detailed information:

```bash
# Run with debug output
clnrm run --verbose --debug tests/

# Generate execution traces
clnrm dev trace tests/

# Profile execution
clnrm dev profile tests/
```

## Examples

### Complete Test Workflow

```bash
#!/bin/bash
# Complete test workflow

echo "🔍 Validating configuration..."
clnrm validate tests/

echo "📦 Pre-pulling images..."
clnrm pull tests/

echo "🧪 Running tests..."
clnrm run tests/ --parallel --workers 4

echo "📊 Generating reports..."
clnrm report --format html,json --output reports/

echo "✅ Test workflow complete"
```

### Performance Testing Workflow

```bash
#!/bin/bash
# Performance testing workflow

echo "🏃 Running performance tests..."
clnrm run tests/performance/ --baseline production

echo "📈 Checking for regressions..."
if clnrm run tests/performance/ --check-regressions; then
    echo "✅ No regressions detected"
else
    echo "❌ Performance regression detected"
    clnrm report trends --baseline production
    exit 1
fi

echo "📊 Updating performance baseline..."
clnrm baseline update production

echo "✅ Performance testing complete"
```

## Next Steps

Now that you understand the CLI:

1. **Try the examples**: Run the CLI examples in this chapter
2. **Learn TOML schema**: Move on to [TOML Schema](toml-schema.md)
3. **Understand error handling**: Learn about [Error Handling](error-handling.md)
4. **Master advanced usage**: Review the other chapters for advanced patterns

## Further Reading

- [Command Line Interface Guidelines](https://clig.dev/)
- [CLI Best Practices](https://www.destroyallsoftware.com/talks/a-whole-new-world)
