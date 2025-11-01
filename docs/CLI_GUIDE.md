# CLI Guide

Complete reference for the `clnrm` command-line interface.

## Table of Contents

- [Global Options](#global-options)
- [Commands](#commands)
  - [run](#run) - Execute tests
  - [init](#init) - Initialize project
  - [validate](#validate) - Validate configuration
  - [health](#health) - System health check
  - [self-test](#self-test) - Framework validation
  - [plugins](#plugins) - List plugins
  - [live-check](#live-check) - Weaver validation
- [Performance Flags (v1.4.0)](#performance-flags-v140)
- [Environment Variables](#environment-variables)
- [Examples](#examples)

## Global Options

```bash
clnrm [OPTIONS] <COMMAND>
```

### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-v, --verbose` | Increase verbosity (use multiple times: -v, -vv, -vvv) | Silent |
| `-c, --config <FILE>` | Configuration file path | Auto-detect |
| `-f, --format <FORMAT>` | Output format: auto, human, json, junit | `auto` |
| `--version` | Show version information | - |
| `--help` | Show help message | - |

### Output Formats

- **auto**: Human-readable in TTY, JSON in pipes
- **human**: Colored, formatted output for terminals
- **json**: Structured JSON output for automation
- **junit**: JUnit XML format for CI integration

## Commands

### run

Execute test files with optional parallelization and Weaver validation.

```bash
clnrm run [OPTIONS] [PATHS]...
```

#### Arguments

- `[PATHS]...` - Test files or directories (default: discover all *.clnrm.toml)

#### Options

**Execution Control**

| Flag | Description | Default |
|------|-------------|---------|
| `-p, --parallel` | Run tests in parallel | Sequential |
| `-j, --jobs <N>` | Maximum concurrent workers | `4` |
| `--fail-fast` | Stop on first failure | Continue all tests |
| `-w, --watch` | Watch mode (rerun on file changes) | Single run |
| `--force` | Bypass cache, force rerun all tests | Use cache |

**Performance (v1.4.1)**

Container pooling is enabled via environment variable `CLNRM_ENABLE_POOLING=1` (not a CLI flag).

**Sharding**

| Flag | Description | Default |
|------|-------------|---------|
| `--shard <i/m>` | Run shard i of m (e.g., 1/4) | All tests |

**Reproducibility**

| Flag | Description | Default |
|------|-------------|---------|
| `--digest` | Generate SHA-256 digest for reproducibility | No digest |

**Reporting**

| Flag | Description | Default |
|------|-------------|---------|
| `--report-junit <FILE>` | Generate JUnit XML report | No report |

**Weaver Validation**

| Flag | Description | Default |
|------|-------------|---------|
| `--validate` | Enable Weaver live-check validation | Disabled |
| `--live-check` | Alias for --validate | Disabled |
| `--validation-mode <MODE>` | Validation mode: strict, lenient, 80_20, minimal | `strict` |
| `--registry-path <PATH>` | Path to Weaver registry | Auto-detect |
| `--otlp-port <PORT>` | OTLP port for Weaver (0 = auto) | `0` |
| `--admin-port <PORT>` | Admin port for Weaver (0 = auto) | `0` |
| `--diagnostic-format <FMT>` | Diagnostic format: ansi, json, github | `ansi` |
| `--stop-timeout <SECS>` | Stop condition timeout | `300` |

**OpenTelemetry Export**

| Flag | Description | Default |
|------|-------------|---------|
| `--otel-exporter <TYPE>` | Exporter: none, stdout, otlp-http, otlp-grpc | `none` |
| `--otel-endpoint <URL>` | OTLP endpoint URL | - |

#### Examples

```bash
# Run all tests (auto-discovery)
clnrm run

# Run specific test
clnrm run tests/api_test.clnrm.toml

# Run all tests in directory
clnrm run tests/

# Parallel execution with container pooling
CLNRM_ENABLE_POOLING=1 clnrm run --parallel --jobs 16

# Run with Weaver validation
clnrm run --live-check --registry registry/

# Export telemetry to OTLP collector
clnrm run --otel-exporter otlp-http --otel-endpoint http://localhost:4318

# Sharded execution (CI parallelization)
clnrm run --shard 1/4  # Run shard 1 of 4

# Generate JUnit XML for CI
clnrm run --report-junit results.xml

# Watch mode for development
clnrm run --watch tests/

# Fail fast for quick feedback
clnrm run --fail-fast --parallel
```

### init

Initialize a new test project or configuration file.

```bash
clnrm init [OPTIONS]
```

#### Options

| Flag | Description |
|------|-------------|
| `--force` | Force reinitialize if already initialized |
| `--config` | Generate cleanroom.toml configuration file |

#### Examples

```bash
# Initialize with default structure
clnrm init

# Force reinitialize
clnrm init --force

# Generate configuration file
clnrm init --config
```

### validate

Validate test configuration files for syntax and semantic errors.

```bash
clnrm validate <FILES>...
```

#### Arguments

- `<FILES>...` - Test files to validate (required)

#### Examples

```bash
# Validate single file
clnrm validate tests/api_test.clnrm.toml

# Validate multiple files
clnrm validate tests/*.clnrm.toml

# Validate and show all errors
clnrm validate -vv tests/
```

### health

Check system health and readiness for running tests.

```bash
clnrm health [OPTIONS]
```

#### Options

| Flag | Description |
|------|-------------|
| `--verbose` | Show detailed health information |

#### Checks

- Docker daemon availability
- Container runtime version
- Available system resources (memory, CPU)
- Network connectivity
- Required dependencies

#### Examples

```bash
# Basic health check
clnrm health

# Detailed health information
clnrm health --verbose
```

### self-test

Run framework self-validation tests.

```bash
clnrm self-test [OPTIONS]
```

#### Options

| Flag | Description | Default |
|------|-------------|---------|
| `-s, --suite <NAME>` | Run specific test suite | All suites |
| `-r, --report` | Generate detailed report | Summary only |
| `--otel-exporter <TYPE>` | Export telemetry | `none` |
| `--otel-endpoint <URL>` | OTLP endpoint | - |

#### Test Suites

- **framework** - Core framework functionality
- **container** - Container lifecycle and isolation
- **plugin** - Plugin system and service integrations
- **cli** - Command-line interface
- **otel** - OpenTelemetry integration
- **weaver** - Weaver live-check validation

#### Examples

```bash
# Run all self-tests
clnrm self-test

# Run specific suite
clnrm self-test --suite otel

# Generate detailed report
clnrm self-test --report

# Export telemetry during self-test
clnrm self-test --suite otel --otel-exporter stdout
```

### plugins

List available service plugins.

```bash
clnrm plugins
```

Displays all registered service plugins with their capabilities and configuration options.

### live-check

Weaver live-check validation utilities.

```bash
clnrm live-check <COMMAND>
```

#### Subcommands

**status** - Show Weaver integration status
```bash
clnrm live-check status
```

**validate-registry** - Validate Weaver registry schemas
```bash
clnrm live-check validate-registry <PATH>
```

**test-weaver** - Test Weaver connectivity
```bash
clnrm live-check test-weaver
```

**modes** - Show available validation modes
```bash
clnrm live-check modes
```

**version** - Show Weaver version
```bash
clnrm live-check version
```

## Performance Configuration (v1.4.1)

### Container Pooling

Enable container pooling for 80% faster test startup via environment variable:

```bash
# Enable pooling (environment variable only, not a CLI flag)
CLNRM_ENABLE_POOLING=1 clnrm run
```

**When to use:**
- Large test suites (>100 tests)
- Parallel execution
- Repeated test runs (watch mode, CI)
- High-concurrency scenarios

**Configuration:**
```bash
# Pool size (default: 10)
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_MAX_SIZE=50 clnrm run

# Minimum idle containers (default: 5)
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_MIN_IDLE=10 clnrm run

# Combined example
CLNRM_ENABLE_POOLING=1 \
CLNRM_POOL_MAX_SIZE=100 \
CLNRM_POOL_MIN_IDLE=20 \
  clnrm run --parallel --jobs 16
```

See [Container Pooling](CONTAINER_POOLING.md) for detailed tuning guide.

### Concurrency Control

Control parallel execution:

```bash
# Default concurrency (4 workers)
clnrm run --parallel

# Custom worker count
clnrm run --parallel --jobs 16

# Maximum concurrency (CPU count)
clnrm run --parallel --jobs $(nproc)

# With pooling for best performance
CLNRM_ENABLE_POOLING=1 clnrm run --parallel --jobs 16
```

**Tuning guidelines:**
- **CPU-bound tests**: `jobs = CPU count`
- **I/O-bound tests**: `jobs = CPU count × 2-4`
- **Container-heavy**: `jobs = CPU count`, enable pooling
- **Memory-limited**: `jobs = available_GB / 2`

See [Performance Tuning](PERFORMANCE_TUNING.md) for optimization strategies.

## Environment Variables

### Core Configuration

| Variable | Description | Default |
|----------|-------------|---------|
| `CLNRM_CONFIG` | Configuration file path | Auto-detect |
| `CLNRM_LOG_LEVEL` | Log level: trace, debug, info, warn, error | `info` |
| `CLNRM_LOG_FORMAT` | Log format: text, json | `text` |

### Container Pooling (v1.4.0)

| Variable | Description | Default |
|----------|-------------|---------|
| `CLNRM_ENABLE_POOLING` | Enable container pooling | `false` |
| `CLNRM_POOL_MAX_SIZE` | Maximum pool size | `10` |
| `CLNRM_POOL_MIN_IDLE` | Minimum idle containers | `5` |
| `CLNRM_POOL_IDLE_TIMEOUT` | Idle timeout (seconds) | `300` |
| `CLNRM_POOL_HEALTH_CHECK_INTERVAL` | Health check interval (seconds) | `60` |

### OpenTelemetry

| Variable | Description | Default |
|----------|-------------|---------|
| `OTEL_EXPORTER_OTLP_ENDPOINT` | OTLP endpoint URL | - |
| `OTEL_EXPORTER_OTLP_PROTOCOL` | Protocol: http/protobuf, grpc | `http/protobuf` |
| `OTEL_SERVICE_NAME` | Service name | `clnrm` |
| `OTEL_RESOURCE_ATTRIBUTES` | Resource attributes (key=value pairs) | - |

### Weaver

| Variable | Description | Default |
|----------|-------------|---------|
| `WEAVER_REGISTRY_PATH` | Weaver registry path | `registry/` |
| `WEAVER_VALIDATION_MODE` | Validation mode | `strict` |
| `WEAVER_OTLP_PORT` | OTLP port (0 = auto) | `0` |
| `WEAVER_ADMIN_PORT` | Admin port (0 = auto) | `0` |

## Examples

### Basic Workflows

**Quick validation during development:**
```bash
# Watch mode with fast feedback
clnrm run --watch --fail-fast tests/
```

**CI/CD integration:**
```bash
# Parallel execution with JUnit XML
clnrm run --parallel --jobs $(nproc) --report-junit results.xml
```

**Production validation:**
```bash
# Full validation with Weaver
clnrm run --live-check --validation-mode strict --registry registry/
```

### Performance Optimization

**Maximum throughput:**
```bash
# Container pooling + parallel execution
CLNRM_ENABLE_POOLING=1 \
CLNRM_POOL_MAX_SIZE=100 \
  clnrm run --parallel --jobs 16
```

**Memory-constrained environment:**
```bash
# Smaller pool, limited concurrency
CLNRM_ENABLE_POOLING=1 \
CLNRM_POOL_MAX_SIZE=20 \
  clnrm run --parallel --jobs 4
```

### Debugging

**Verbose output:**
```bash
# Debug logging
clnrm -vvv run tests/failing_test.clnrm.toml

# Trace all operations
CLNRM_LOG_LEVEL=trace clnrm run --verbose
```

**Export telemetry for analysis:**
```bash
# Export to stdout
clnrm run --otel-exporter stdout tests/

# Export to OTLP collector
clnrm run \
  --otel-exporter otlp-http \
  --otel-endpoint http://localhost:4318 \
  tests/
```

### Advanced Use Cases

**Sharded CI execution:**
```bash
# In GitHub Actions matrix (4 parallel jobs)
# Job 1:
clnrm run --shard 1/4 --report-junit results-1.xml

# Job 2:
clnrm run --shard 2/4 --report-junit results-2.xml

# Job 3:
clnrm run --shard 3/4 --report-junit results-3.xml

# Job 4:
clnrm run --shard 4/4 --report-junit results-4.xml
```

**Reproducible test runs:**
```bash
# Generate digest
clnrm run --digest tests/ > test_digest.txt

# Verify reproducibility
clnrm run --digest tests/ | diff - test_digest.txt
```

**Multi-environment testing:**
```bash
# Test against different OTEL backends
for backend in jaeger datadog newrelic; do
  clnrm run \
    --otel-exporter otlp-http \
    --otel-endpoint "http://${backend}:4318" \
    tests/
done
```

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success (all tests passed) |
| 1 | Test failures or validation errors |
| 2 | Configuration error |
| 3 | System error (Docker unavailable, etc.) |
| 130 | Interrupted (Ctrl+C) |

## Performance Benchmarks

**v1.4.0 performance improvements:**

| Scenario | v1.3.0 | v1.4.0 | Improvement |
|----------|--------|--------|-------------|
| Startup time (pool hit) | 2-5s | 0.1-0.5ms | **80% faster** |
| Throughput (1000 tests) | ~50/s | ~500/s | **10x faster** |
| Memory overhead | 512MB | 768MB | +50% (acceptable) |
| Max concurrency | 50-100 | 500-1000 | **10x higher** |

## Troubleshooting

**Container pooling not working:**
```bash
# Verify environment
echo $CLNRM_ENABLE_POOLING  # Should be "1" or "true"

# Check pool stats (verbose mode)
CLNRM_ENABLE_POOLING=1 clnrm run -vv tests/

# Expected output:
# Pool stats: hits=95%, misses=5%, active=10, idle=15
```

**High memory usage:**
```bash
# Reduce pool size
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_MAX_SIZE=20 clnrm run

# Reduce idle containers
CLNRM_ENABLE_POOLING=1 CLNRM_POOL_MIN_IDLE=5 clnrm run
```

**Weaver validation failures:**
```bash
# Validate registry first
clnrm live-check validate-registry registry/

# Test Weaver connectivity
clnrm live-check test-weaver

# Use lenient mode for debugging
clnrm run --live-check --validation-mode lenient
```

## See Also

- [Container Pooling Guide](CONTAINER_POOLING.md) - Detailed pooling configuration
- [Performance Tuning Guide](PERFORMANCE_TUNING.md) - Optimization strategies
- [Weaver Configuration](WEAVER_TOML_CONFIGURATION.md) - Live-check setup
- [TOML Reference](../book/src/reference/toml-schema.md) - Configuration format
- [Troubleshooting](TROUBLESHOOTING.md) - Common issues and solutions

---

**Version**: 1.4.1
**Updated**: 2025-11-01
**Agent**: Documentation Corrector (Agent 9/16)
