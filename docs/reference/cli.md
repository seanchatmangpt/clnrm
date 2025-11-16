# CLI Commands Reference

Complete reference for all clnrm commands and flags.

## Command Overview

```
clnrm [COMMAND] [OPTIONS] [ARGS]
```

## Commands

### init

Initialize a new clnrm project.

```bash
clnrm init [OPTIONS]

OPTIONS:
  -d, --directory <DIR>    Project directory (default: current dir)
  -f, --force              Overwrite existing configuration
  -h, --help               Show help
```

**Example**:
```bash
clnrm init
clnrm init -d ./my-tests
```

### run

Execute test files matching pattern.

```bash
clnrm run [OPTIONS] [PATTERN]

OPTIONS:
  -p, --parallel                  Run tests in parallel
  -j, --jobs <N>                  Concurrency (default: 4)
  -x, --fail-fast                 Stop on first failure
  -f, --filter <PATTERN>          Only run matching tests
  -e, --exclude <PATTERN>         Skip matching tests
  -t, --timeout <MS>              Test timeout in milliseconds
  -o, --output <FORMAT>           Output format: plain, junit, json, html
  --live-check                    Enable Weaver validation
  --registry <PATH>               Weaver schema registry path
  -v, --verbose                   Verbose output
  --watch                         Watch for file changes and re-run
  -h, --help                      Show help

ARGS:
  PATTERN                         Test file pattern (default: *.clnrm.toml)
```

**Examples**:
```bash
# Run all tests
clnrm run

# Run with parallelism
clnrm run --parallel --jobs 16

# Run specific test
clnrm run tests/api.clnrm.toml

# Run matching pattern
clnrm run --filter "database*"

# With Weaver validation
clnrm run --live-check --registry registry/

# Generate JUnit report
clnrm run --output junit > results.xml

# Watch mode
clnrm run --watch
```

### validate

Validate TOML configuration files.

```bash
clnrm validate [OPTIONS] [FILES]

OPTIONS:
  -v, --verbose           Verbose output
  -h, --help             Show help

FILES:
  FILES                   TOML files to validate (default: *.clnrm.toml)
```

**Examples**:
```bash
clnrm validate
clnrm validate tests/api.clnrm.toml
clnrm validate tests/*.clnrm.toml
```

### plugins

List available service plugins.

```bash
clnrm plugins [OPTIONS]

OPTIONS:
  --detailed              Show detailed plugin information
  -h, --help             Show help
```

**Example**:
```bash
clnrm plugins
clnrm plugins --detailed
```

### self-test

Run framework self-validation tests.

```bash
clnrm self-test [OPTIONS]

OPTIONS:
  -s, --suite <SUITE>     Test suite: all, basic, otel, stress
  --otel-exporter <TYPE>  OTEL exporter: stdout, otlp-http
  -v, --verbose           Verbose output
  -h, --help             Show help
```

**Examples**:
```bash
clnrm self-test
clnrm self-test --suite otel
clnrm self-test --otel-exporter stdout
```

### health

Check system health and dependencies.

```bash
clnrm health [OPTIONS]

OPTIONS:
  -v, --verbose           Verbose output
  -h, --help             Show help
```

**Example**:
```bash
clnrm health
```

## Global Options

Apply to all commands:

```bash
--config <FILE>         Config file path (default: .clnrm/config.toml)
--log-level <LEVEL>    Logging level: debug, info, warn, error (default: info)
--help                 Show help message
--version              Show version information
```

## Environment Variables

Control behavior via environment:

```bash
# Logging
RUST_LOG=debug clnrm run

# Enable pooling
CLNRM_ENABLE_POOLING=1 clnrm run

# Pool configuration
CLNRM_POOL_SIZE=10
CLNRM_POOL_IDLE_TIMEOUT_MS=60000

# OTEL configuration
OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4318
OTEL_SDK_DISABLED=false
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (all tests passed) |
| 1 | Test failure (at least one test failed) |
| 2 | Configuration error (invalid TOML, missing files) |
| 3 | System error (Docker not available, permission denied) |
| 4 | Validation error (OTEL/Weaver validation failed) |

## Output Formats

### Plain (Default)

```
Testing my_test...
Scenario: scenario_1
  ✅ Output validation passed

━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Results:
  Total: 1
  Passed: 1
  Failed: 0
```

### JUnit

```xml
<?xml version="1.0" encoding="UTF-8"?>
<testsuites>
  <testsuite name="my_test" tests="1" failures="0">
    <testcase name="scenario_1" classname="my_test" time="0.245"/>
  </testsuite>
</testsuites>
```

### JSON

```json
{
  "total": 1,
  "passed": 1,
  "failed": 0,
  "tests": [
    {
      "name": "my_test",
      "scenarios": [
        {
          "name": "scenario_1",
          "status": "passed",
          "duration_ms": 245
        }
      ]
    }
  ]
}
```

## Useful Combinations

### Fast Feedback (Development)

```bash
CLNRM_ENABLE_POOLING=1 clnrm run --watch -x
```

### CI Pipeline (Full Suite)

```bash
clnrm run --parallel --jobs 16 --output junit > results.xml
```

### Debugging

```bash
clnrm run --verbose --fail-fast --timeout 30000
```

### With Validation

```bash
CLNRM_ENABLE_POOLING=1 clnrm run --live-check --registry registry/ --parallel --jobs 8
```

## See Also

- [How-To Guides](../how-to/)
- [TOML Configuration Reference](./toml-schema.md)
- [Environment Variables Reference](./environment-variables.md)
