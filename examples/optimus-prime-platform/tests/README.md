# Optimus Prime Platform - Service Management Tests

This directory contains comprehensive service management tests for the Optimus Prime Platform using the Cleanroom testing framework.

## Quick Start

### Run All Tests

```bash
cd /Users/sac/clnrm/examples/optimus-prime-platform
./service-management-test.sh
```

### Run Individual Tests

```bash
cd /Users/sac/clnrm

# Validate configuration
./target/release/clnrm validate examples/optimus-prime-platform/tests/services-test.clnrm.toml

# Check service status
./target/release/clnrm services status

# AI load prediction
./target/release/clnrm services ai-manage --predict-load --horizon-minutes 10

# AI auto-scaling
./target/release/clnrm services ai-manage --auto-scale --predict-load

# AI resource optimization
./target/release/clnrm services ai-manage --optimize-resources

# Full AI suite
./target/release/clnrm services ai-manage --auto-scale --predict-load --optimize-resources

# System health check
./target/release/clnrm health
./target/release/clnrm health --verbose

# Run integration tests (requires Docker)
./target/release/clnrm run examples/optimus-prime-platform/tests/services-test.clnrm.toml
```

## Test Files

### services-test.clnrm.toml

Comprehensive multi-service integration test configuration with 4 services and 15 test steps.

## Test Results

See **[docs/SERVICE_MANAGEMENT_RESULTS.md](../docs/SERVICE_MANAGEMENT_RESULTS.md)** for complete results.

**Status**: ✅ SUCCESS (89% pass rate - 8/9 tests)
**System Health**: 93% (EXCELLENT)

## AI Service Management Features

- Load Prediction (configurable horizon)
- Auto-Scaling (load-based triggers)
- Resource Optimization (CPU/memory)
- Anomaly Detection
- Proactive Healing

## Requirements

- Cleanroom v0.4.0+
- Docker (optional, for integration tests)
- SurrealDB + Ollama (optional, for full AI features)

---

**Last Updated**: 2025-10-16
