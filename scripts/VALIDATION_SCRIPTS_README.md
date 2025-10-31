# Validation Scripts - Quick Reference

**clnrm v1.2.0** - Automated validation pipeline scripts

## New Scripts (v1.2.0)

### 🚀 validation_pipeline.sh (12K)
**One-command end-to-end validation**

```bash
# Full pipeline: Docker → OTLP → Weaver → Tests → Report
./scripts/validation_pipeline.sh

# Skip Docker startup (if already running)
./scripts/validation_pipeline.sh --skip-docker

# Debug mode (no cleanup)
./scripts/validation_pipeline.sh --no-cleanup
```

### 🐳 docker_startup.sh (7.0K)
**Cross-platform Docker daemon startup**

```bash
# Auto-detect and start Docker (Desktop/Colima/systemd)
./scripts/docker_startup.sh

# Supports: macOS, Linux, Windows
# Waits max 120s for daemon ready
```

### ✅ docker_health_check.sh (9.3K)
**Comprehensive Docker health verification**

```bash
# Full health check (10 checks)
./scripts/docker_health_check.sh check

# Quick check
./scripts/docker_health_check.sh quick

# Wait for ready
./scripts/docker_health_check.sh wait
```

### 📡 otlp_config.sh (8.6K)
**OpenTelemetry environment configuration**

```bash
# Export to current shell
source ./scripts/otlp_config.sh

# Validate config
./scripts/otlp_config.sh validate

# Test connectivity
./scripts/otlp_config.sh test

# Generate export script
./scripts/otlp_config.sh generate
source /tmp/otlp_env.sh
```

### 🔧 weaver_startup.sh (12K)
**Weaver process lifecycle management**

```bash
# Start Weaver live-check
./scripts/weaver_startup.sh start

# Check status
./scripts/weaver_startup.sh status

# View logs
./scripts/weaver_startup.sh logs

# Stop gracefully (generates report)
./scripts/weaver_startup.sh stop
```

## Existing Scripts

### run_weaver_validation.sh (7.8K)
Original comprehensive Weaver validation with manual Docker checks

### test_otlp_chain.sh (3.5K)
Tests OTLP telemetry chain without Docker (library tests only)

### wait_for_docker.sh (748B)
Simple Docker wait script (superseded by docker_startup.sh)

### validate_docker_telemetry.sh (5.9K)
Docker-specific telemetry validation

### comprehensive_weaver_validation.sh (6.5K)
Original Weaver validation with pre-flight checks

## Quick Start

### First Time Setup

```bash
# 1. Ensure prerequisites
cargo install weaver  # Install Weaver
docker --version      # Verify Docker installed

# 2. Run validation
./scripts/validation_pipeline.sh
```

### Daily Development

```bash
# Start infrastructure once
./scripts/docker_startup.sh
./scripts/weaver_startup.sh start

# Configure environment
source ./scripts/otlp_config.sh

# Run tests iteratively
cargo test -p clnrm-core --test docker_integration --features otel

# Stop Weaver when done
./scripts/weaver_startup.sh stop
```

### CI/CD Integration

```bash
# Single command for CI
./scripts/validation_pipeline.sh

# Exit code 0 = success, 1 = failure
```

## Script Comparison

| Script | Purpose | When to Use |
|--------|---------|-------------|
| **validation_pipeline.sh** | Full automation | CI/CD, release validation |
| **docker_startup.sh** | Start Docker | Manual setup, debugging |
| **docker_health_check.sh** | Verify Docker | Troubleshooting, pre-flight |
| **otlp_config.sh** | Configure OTLP | Manual testing, debugging |
| **weaver_startup.sh** | Manage Weaver | Manual testing, long-running |
| run_weaver_validation.sh | Legacy validation | Existing workflows |
| test_otlp_chain.sh | Test OTLP only | No Docker available |

## Environment Variables

### Common Variables

```bash
# OTLP Configuration
export OTLP_PORT=4317           # OTLP gRPC port
export OTLP_PROTOCOL=grpc       # grpc or http
export SERVICE_NAME=clnrm       # Service identifier
export DEPLOYMENT_ENV=testing   # Environment name

# Weaver Configuration
export REGISTRY=registry/                # Schema registry path
export OUTPUT=validation_output/         # Output directory
export TIMEOUT=300                       # Inactivity timeout (seconds)

# Test Configuration
export TEST_PACKAGE=clnrm-core          # Cargo package
export TEST_SUITE=docker_integration    # Test suite name
export TEST_THREADS=1                   # Test parallelism

# Retry Configuration
export MAX_RETRIES=3                    # Retry attempts
export RETRY_DELAY=5                    # Retry delay (seconds)
```

## Output Files

| File | Description | Size |
|------|-------------|------|
| `validation_output/live_check.json` | Weaver validation report | ~50KB |
| `/tmp/weaver.pid` | Weaver process ID | ~10B |
| `/tmp/weaver.log` | Weaver logs | Variable |
| `/tmp/otlp_env.sh` | OTLP environment export | ~1KB |
| `/tmp/clnrm_test_output.log` | Test execution logs | Variable |

## Success Criteria

Pipeline passes when:
- ✅ Docker daemon responsive
- ✅ Weaver listening on :4317
- ✅ All tests pass
- ✅ Telemetry samples > 0
- ✅ Zero schema violations
- ✅ Coverage ≥ 70%

## Troubleshooting

### Docker won't start

```bash
# Check Docker installation
which docker

# Check Docker Desktop (macOS)
open -a Docker

# Check Colima
colima status

# Check service (Linux)
sudo systemctl status docker
```

### Weaver won't start

```bash
# Check Weaver installed
which weaver

# Check port available
lsof -i :4317

# Check registry valid
weaver registry check -r registry/

# View logs
./scripts/weaver_startup.sh logs
```

### No telemetry received

```bash
# Check OTLP configured
env | grep OTEL_

# Check Weaver listening
lsof -i :4317

# Test connectivity
./scripts/otlp_config.sh test

# Check test output
cat /tmp/clnrm_test_output.log
```

### Schema violations

```bash
# View violations
jq '.statistics' validation_output/live_check.json

# Check specific violations
jq '.samples[] | select(.advice_level == "violation")' validation_output/live_check.json

# Review schemas
ls -la registry/
```

## Performance Benchmarks

| Operation | Duration | Notes |
|-----------|----------|-------|
| Docker startup | 5-30s | Cold start |
| Docker health check | 1-3s | All checks |
| OTLP config | <1s | Instant |
| Weaver startup | 3-5s | With validation |
| Test execution | 10-60s | Varies by suite |
| Report generation | 1-2s | JSON export |
| **Full pipeline** | **20-100s** | End-to-end |

## Integration Examples

### GitHub Actions

```yaml
- name: Validate with Weaver
  run: ./scripts/validation_pipeline.sh
```

### GitLab CI

```yaml
weaver-validation:
  script: ./scripts/validation_pipeline.sh
  artifacts:
    paths: [validation_output/]
```

### Pre-commit Hook

```bash
#!/bin/bash
./scripts/validation_pipeline.sh --skip-docker
```

## Migration Guide

### From run_weaver_validation.sh

```bash
# Old (manual Docker check)
./scripts/wait_for_docker.sh && ./scripts/run_weaver_validation.sh

# New (automated)
./scripts/validation_pipeline.sh
```

### From test_otlp_chain.sh

```bash
# Old (no Docker)
./scripts/test_otlp_chain.sh

# New (with Docker)
./scripts/validation_pipeline.sh
```

### From manual commands

```bash
# Old (manual steps)
weaver registry live-check ... &
export OTEL_EXPORTER_OTLP_ENDPOINT=...
cargo test ...
kill $WEAVER_PID

# New (automated)
./scripts/validation_pipeline.sh
```

## Documentation

- **Comprehensive Guide:** `/Users/sac/clnrm/docs/VALIDATION_PIPELINE_GUIDE.md`
- **Weaver Integration:** `/Users/sac/clnrm/docs/WEAVER_V1_2_0_VALIDATION_SUMMARY.md`
- **Docker Validation:** `/Users/sac/clnrm/docs/DOCKER_VALIDATION.md`

## Support

For issues:
1. Check logs: `./scripts/weaver_startup.sh logs`
2. Run health check: `./scripts/docker_health_check.sh check`
3. Validate config: `./scripts/otlp_config.sh validate`
4. Review report: `validation_output/live_check.json`

GitHub Issues: https://github.com/seanchatmangpt/clnrm/issues
