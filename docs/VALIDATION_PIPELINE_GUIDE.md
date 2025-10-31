# Validation Pipeline Guide

**clnrm v1.2.0** - Docker + OTLP + Weaver Validation

## Overview

This guide documents the automated validation pipeline for clnrm, which integrates:

1. **Docker Daemon Management** - Cross-platform Docker startup and health checking
2. **OTLP Configuration** - OpenTelemetry Protocol setup for telemetry export
3. **Weaver Integration** - OpenTelemetry schema validation with live-check
4. **Test Execution** - Automated test runs with telemetry capture
5. **Report Validation** - Schema compliance verification

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                   Validation Pipeline                        │
├─────────────────────────────────────────────────────────────┤
│                                                               │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────┐     │
│  │   Docker    │───▶│    OTLP     │───▶│   Weaver    │     │
│  │   Startup   │    │   Config    │    │   Startup   │     │
│  └─────────────┘    └─────────────┘    └─────────────┘     │
│         │                   │                   │            │
│         └───────────────────┴───────────────────┘            │
│                             │                                │
│                             ▼                                │
│                     ┌─────────────┐                          │
│                     │    Tests    │                          │
│                     │   Execute   │                          │
│                     └─────────────┘                          │
│                             │                                │
│                             ▼                                │
│                     ┌─────────────┐                          │
│                     │   Report    │                          │
│                     │  Validate   │                          │
│                     └─────────────┘                          │
│                                                               │
└─────────────────────────────────────────────────────────────┘
```

## Quick Start

### One-Command Validation

```bash
# Run complete validation pipeline
./scripts/validation_pipeline.sh
```

This single command:
- ✅ Starts Docker if needed
- ✅ Configures OTLP environment
- ✅ Starts Weaver live-check
- ✅ Runs integration tests
- ✅ Generates validation report
- ✅ Validates schema compliance

### Docker Already Running

```bash
# Skip Docker startup phase
./scripts/validation_pipeline.sh --skip-docker
```

### Debug Mode (No Cleanup)

```bash
# Keep Weaver running for inspection
./scripts/validation_pipeline.sh --no-cleanup
```

## Script Reference

### 1. docker_startup.sh

**Purpose:** Cross-platform Docker daemon startup and detection

**Features:**
- Auto-detects Docker Desktop, Colima, or native Docker
- Handles macOS, Linux, Windows
- Verifies Docker functionality
- Provides troubleshooting guidance

**Usage:**

```bash
# Start Docker daemon
./scripts/docker_startup.sh

# Detect OS: macos, linux, windows
# Attempt startup using available method:
#   - Docker Desktop (macOS/Windows)
#   - Colima (macOS/Linux)
#   - systemd service (Linux)
# Wait for daemon ready (max 120s)
# Verify with hello-world container
```

**Environment Variables:**
- `MAX_WAIT` - Max wait time in seconds (default: 120)
- `CHECK_INTERVAL` - Check interval in seconds (default: 3)

**Exit Codes:**
- `0` - Docker started successfully
- `1` - Failed to start Docker

### 2. docker_health_check.sh

**Purpose:** Comprehensive Docker daemon health verification

**Features:**
- 10-point health check system
- Resource verification (CPU, memory, storage)
- Network and storage driver checks
- Detailed diagnostics

**Usage:**

```bash
# Comprehensive health check
./scripts/docker_health_check.sh check

# Wait for Docker and check
./scripts/docker_health_check.sh wait

# Quick check (responsive only)
./scripts/docker_health_check.sh quick

# Show Docker info
./scripts/docker_health_check.sh info
```

**Health Checks:**
1. ✅ Docker CLI installed
2. ✅ Daemon responsive
3. ✅ Version detectable
4. ✅ Can list containers
5. ⚠️ Can pull images (optional)
6. ⚠️ Can run containers (optional)
7. ⚠️ Sufficient resources (optional)
8. ✅ Networking functional
9. ✅ Storage functional
10. ⚠️ Cleanup works (optional)

### 3. otlp_config.sh

**Purpose:** OpenTelemetry Protocol environment configuration

**Features:**
- Configures OTEL environment variables
- Validates configuration
- Tests endpoint connectivity
- Generates export scripts

**Usage:**

```bash
# Export variables to current shell
source ./scripts/otlp_config.sh

# Validate config without exporting
./scripts/otlp_config.sh validate

# Test endpoint connectivity
./scripts/otlp_config.sh test

# Generate export script for later
./scripts/otlp_config.sh generate
source /tmp/otlp_env.sh
```

**Environment Variables:**

| Variable | Default | Description |
|----------|---------|-------------|
| `OTLP_PORT` | 4317 | OTLP gRPC port |
| `OTLP_PROTOCOL` | grpc | Protocol (grpc or http) |
| `SERVICE_NAME` | clnrm | Service name |
| `SERVICE_VERSION` | 1.2.0 | Service version |
| `DEPLOYMENT_ENV` | testing | Environment name |
| `RUST_LOG` | info | Rust log level |

**Exports:**
- `OTEL_EXPORTER_OTLP_ENDPOINT` - OTLP endpoint URL
- `OTEL_SERVICE_NAME` - Service identifier
- `OTEL_RESOURCE_ATTRIBUTES` - Resource metadata
- `OTEL_TRACES_SAMPLER` - Sampling strategy
- `OTEL_BSP_SCHEDULE_DELAY` - Batch delay (1000ms for testing)

### 4. weaver_startup.sh

**Purpose:** Weaver live-check process lifecycle management

**Features:**
- Process management (start/stop/restart/status)
- Port conflict resolution
- Graceful shutdown (SIGHUP) for report generation
- Log management

**Usage:**

```bash
# Start Weaver
./scripts/weaver_startup.sh start

# Check status
./scripts/weaver_startup.sh status

# View logs
./scripts/weaver_startup.sh logs

# Stop gracefully (generates report)
./scripts/weaver_startup.sh stop

# Force stop
./scripts/weaver_startup.sh force-stop

# Restart
./scripts/weaver_startup.sh restart
```

**Environment Variables:**

| Variable | Default | Description |
|----------|---------|-------------|
| `REGISTRY` | registry/ | Schema registry path |
| `OUTPUT` | validation_output/ | Output directory |
| `OTLP_PORT` | 4317 | OTLP gRPC port |
| `ADMIN_PORT` | 8080 | Admin API port |
| `TIMEOUT` | 300 | Inactivity timeout (seconds) |
| `PID_FILE` | /tmp/weaver.pid | PID file location |
| `LOG_FILE` | /tmp/weaver.log | Log file location |

**Process Management:**
- PID tracking in `/tmp/weaver.pid`
- Logs in `/tmp/weaver.log`
- Graceful shutdown with `SIGHUP` (generates report)
- Force shutdown with `SIGTERM`
- Automatic cleanup on exit

### 5. validation_pipeline.sh

**Purpose:** Unified end-to-end validation orchestrator

**Features:**
- 6-phase validation pipeline
- Error recovery with retries
- Automatic cleanup
- Detailed reporting

**Usage:**

```bash
# Full pipeline
./scripts/validation_pipeline.sh

# Skip Docker startup
./scripts/validation_pipeline.sh --skip-docker

# Skip test execution
./scripts/validation_pipeline.sh --skip-tests

# No cleanup (for debugging)
./scripts/validation_pipeline.sh --no-cleanup
```

**Environment Variables:**

| Variable | Default | Description |
|----------|---------|-------------|
| `TEST_PACKAGE` | clnrm-core | Cargo package to test |
| `TEST_SUITE` | docker_integration | Test suite name |
| `TEST_THREADS` | 1 | Test parallelism |
| `MAX_RETRIES` | 3 | Retry attempts |
| `RETRY_DELAY` | 5 | Retry delay (seconds) |

**Pipeline Phases:**

1. **Docker Startup** - Ensure Docker daemon running
2. **OTLP Config** - Configure telemetry environment
3. **Weaver Startup** - Start schema validator
4. **Test Execution** - Run tests with telemetry
5. **Report Generation** - Stop Weaver, generate report
6. **Report Validation** - Verify schema compliance

**Exit Codes:**
- `0` - All phases passed
- `1` - One or more phases failed

## Validation Criteria

### Success Criteria

The pipeline passes when:

1. ✅ **Docker Ready** - Daemon responsive and functional
2. ✅ **Weaver Started** - Listening on OTLP port
3. ✅ **Tests Passed** - All integration tests succeed
4. ✅ **Telemetry Received** - Samples > 0
5. ✅ **Zero Violations** - No schema compliance issues
6. ✅ **Coverage Target** - Registry coverage ≥ 70%

### Failure Modes

The pipeline can fail at:

| Phase | Failure Mode | Root Cause |
|-------|-------------|------------|
| Docker | Daemon not starting | Docker not installed / misconfigured |
| OTLP | Invalid config | Port in use / invalid parameters |
| Weaver | Won't start | Registry invalid / port conflict |
| Tests | Test failures | Code bugs / environment issues |
| Report | No telemetry | OTLP not configured / export failed |
| Validation | Violations found | Schema non-compliance / false positives |

## Troubleshooting

### Docker Issues

```bash
# Check Docker status
./scripts/docker_health_check.sh check

# View Docker info
./scripts/docker_health_check.sh info

# Start Docker manually
open -a Docker  # macOS
colima start    # Colima
sudo systemctl start docker  # Linux
```

### Weaver Issues

```bash
# Check Weaver status
./scripts/weaver_startup.sh status

# View logs
./scripts/weaver_startup.sh logs

# Check port conflicts
lsof -i :4317

# Restart Weaver
./scripts/weaver_startup.sh restart
```

### OTLP Issues

```bash
# Validate configuration
./scripts/otlp_config.sh validate

# Test connectivity
./scripts/otlp_config.sh test

# Check environment
env | grep OTEL_
```

### Report Issues

```bash
# Check report exists
ls -la validation_output/live_check.json

# View report summary
jq '.statistics' validation_output/live_check.json

# Check for violations
jq '.statistics.advice_level_counts' validation_output/live_check.json
```

## Integration with CI/CD

### GitHub Actions

```yaml
name: Weaver Validation

on: [push, pull_request]

jobs:
  validate:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rs/toolchain@v1
        with:
          toolchain: stable

      - name: Install Weaver
        run: cargo install weaver

      - name: Run Validation Pipeline
        run: ./scripts/validation_pipeline.sh

      - name: Upload Report
        uses: actions/upload-artifact@v3
        if: always()
        with:
          name: validation-report
          path: validation_output/
```

### GitLab CI

```yaml
weaver-validation:
  image: rust:latest
  services:
    - docker:dind
  before_script:
    - cargo install weaver
  script:
    - ./scripts/validation_pipeline.sh
  artifacts:
    when: always
    paths:
      - validation_output/
```

## Performance

### Timing Benchmarks

| Phase | Duration | Notes |
|-------|----------|-------|
| Docker Startup | 5-30s | Depends on cold start |
| OTLP Config | <1s | Instant |
| Weaver Startup | 3-5s | Registry validation |
| Test Execution | 10-60s | Varies by test suite |
| Report Generation | 1-2s | JSON export |
| Report Validation | <1s | JSON parsing |
| **Total** | **20-100s** | Full pipeline |

### Optimization Tips

1. **Keep Docker Running** - Use `--skip-docker` if already started
2. **Reduce Test Scope** - Use specific test filters
3. **Increase Test Threads** - `TEST_THREADS=4` (if tests allow)
4. **Cache Images** - Pre-pull test images
5. **SSD Storage** - Use fast disk for Docker

## Best Practices

### Development Workflow

```bash
# 1. Start infrastructure once
./scripts/docker_startup.sh
./scripts/weaver_startup.sh start

# 2. Configure environment
source ./scripts/otlp_config.sh

# 3. Iterate on tests
cargo test -p clnrm-core --test docker_integration --features otel

# 4. Generate final report
./scripts/weaver_startup.sh stop
```

### Pre-Commit Hook

```bash
# .git/hooks/pre-commit
#!/bin/bash
./scripts/validation_pipeline.sh --skip-docker
```

### Continuous Validation

```bash
# Watch mode (requires entr)
ls crates/**/*.rs | entr -r ./scripts/validation_pipeline.sh --skip-docker
```

## File Locations

### Scripts

- `/Users/sac/clnrm/scripts/docker_startup.sh`
- `/Users/sac/clnrm/scripts/docker_health_check.sh`
- `/Users/sac/clnrm/scripts/otlp_config.sh`
- `/Users/sac/clnrm/scripts/weaver_startup.sh`
- `/Users/sac/clnrm/scripts/validation_pipeline.sh`

### Generated Files

- `/tmp/weaver.pid` - Weaver process ID
- `/tmp/weaver.log` - Weaver logs
- `/tmp/otlp_env.sh` - OTLP environment export
- `/tmp/clnrm_test_output.log` - Test output
- `validation_output/live_check.json` - Validation report

### Registry

- `/Users/sac/clnrm/registry/` - OpenTelemetry schemas

## Advanced Usage

### Custom Port Configuration

```bash
# Use non-standard ports
OTLP_PORT=5317 ADMIN_PORT=9090 ./scripts/validation_pipeline.sh
```

### Different Test Suites

```bash
# Run specific test suite
TEST_PACKAGE=clnrm-core \
TEST_SUITE=integration_otel \
./scripts/validation_pipeline.sh
```

### Parallel Test Execution

```bash
# Run tests in parallel (if safe)
TEST_THREADS=4 ./scripts/validation_pipeline.sh
```

### Custom Registry

```bash
# Use different registry
REGISTRY=/path/to/custom/registry ./scripts/validation_pipeline.sh
```

### Extended Timeout

```bash
# Increase Weaver inactivity timeout
TIMEOUT=600 ./scripts/validation_pipeline.sh
```

## Support

For issues or questions:

1. Check logs: `./scripts/weaver_startup.sh logs`
2. Run health checks: `./scripts/docker_health_check.sh check`
3. Review report: `validation_output/live_check.json`
4. GitHub Issues: https://github.com/seanchatmangpt/clnrm/issues

## See Also

- [Weaver Documentation](https://github.com/open-telemetry/weaver)
- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/otel/)
- [clnrm Testing Guide](/Users/sac/clnrm/docs/TESTING.md)
- [Docker Integration Guide](/Users/sac/clnrm/docs/DOCKER_VALIDATION.md)
