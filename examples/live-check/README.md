# Weaver Live-Check Examples

This directory contains working examples of Weaver live-check validation configurations for clnrm v1.3.0.

## Quick Start

```bash
# Run any example
clnrm run examples/live-check/basic.clnrm.toml

# Run with custom mode
clnrm run examples/live-check/80-20.clnrm.toml --validate-mode strict
```

## Examples Overview

### 1. Basic (`basic.clnrm.toml`)

**Best for**: Learning, getting started

Minimal configuration showing the essentials:
- Single service
- Simple scenario
- Basic span validation

```bash
clnrm run examples/live-check/basic.clnrm.toml
```

### 2. 80/20 Mode (`80-20.clnrm.toml`)

**Best for**: Daily development, CI/CD pipelines

Demonstrates 80/20 validation mode:
- 6x faster than strict mode
- Critical spans only
- Multiple services
- Count validation

```bash
clnrm run examples/live-check/80-20.clnrm.toml
```

**Performance**: ~0.4s validation time

### 3. Strict Mode (`strict.clnrm.toml`)

**Best for**: Production releases, final certification

Comprehensive validation:
- 100% telemetry validation
- Graph structure checks
- Temporal ordering
- Hermeticity validation

```bash
clnrm run examples/live-check/strict.clnrm.toml
```

**Performance**: ~2.3s validation time

### 4. CI/CD Pipeline (`ci-cd.clnrm.toml`)

**Best for**: Continuous integration, GitHub Actions

Production CI/CD setup:
- Environment variable substitution
- Multiple test scenarios
- Optimized for CI performance
- Real-time streaming output

```bash
# Local testing
clnrm run examples/live-check/ci-cd.clnrm.toml

# CI environment
CI_COMMIT_SHA=abc123 CI_ENVIRONMENT=ci clnrm run examples/live-check/ci-cd.clnrm.toml
```

## Comparison

| Example | Speed | Coverage | Use Case | Services |
|---------|-------|----------|----------|----------|
| basic | Fast | Minimal | Learning | 1 |
| 80-20 | Very Fast | 80% | Development | 2 |
| strict | Slow | 100% | Production | 4 |
| ci-cd | Fast | 80% | CI/CD | 2 |

## Example Patterns

### Pattern 1: Container Lifecycle

All examples validate container cleanup:

```toml
[[expect.span]]
name = "container.lifecycle"
attrs.all = {
  "container.destroyed_at" = "*",
  "cleanup.success" = "true"
}
```

### Pattern 2: Service Health

Check service availability:

```toml
[[expect.span]]
name = "service.health_check"
attrs.all = {
  "health.status" = "healthy"
}
```

### Pattern 3: Graph Structure

Validate service interactions:

```toml
[expect.graph]
must_include = [
    ["http.server.request", "db.query"]
]
```

## Running Examples

### Prerequisites

```bash
# Install clnrm
brew install clnrm

# Or via cargo
cargo install clnrm

# Verify installation
clnrm --version  # Should show v1.3.0+
```

### Run Examples

```bash
# Basic example
clnrm run examples/live-check/basic.clnrm.toml

# With verbose output
clnrm run examples/live-check/basic.clnrm.toml --verbose

# With streaming output
clnrm run examples/live-check/80-20.clnrm.toml --stream

# Save reports
clnrm run examples/live-check/strict.clnrm.toml --output-dir ./my-reports
```

### Modify Examples

Copy and customize:

```bash
# Copy example
cp examples/live-check/80-20.clnrm.toml my-test.clnrm.toml

# Edit for your needs
vim my-test.clnrm.toml

# Run modified version
clnrm run my-test.clnrm.toml
```

## Example Output

### Success

```
✅ Validation: PASS

Weaver Live-Check Results:
  - Mode: 80/20
  - Samples: 124
  - Violations: 0
  - Coverage: 87.5%
  - Duration: 0.42s
```

### Failure

```
❌ Validation: FAIL

Violations:
  1. Missing required attribute 'container.id'
     Span: test.execute
     Fix: Add container.id to span
```

## Customization Guide

### Change Validation Mode

```toml
[weaver.validation]
mode = "80_20"  # Options: strict, 80_20, lenient, minimal
```

### Add Services

```toml
[service.myservice]
plugin = "generic_container"
image = "myimage:latest"
env = {
  "KEY" = "value"
}
```

### Add Scenarios

```toml
[[scenario]]
name = "my_test"
service = "myservice"
run = "my-test-command"
```

### Add Span Expectations

```toml
[[expect.span]]
name = "my.span"
attrs.all = {
  "my.attribute" = "value"
}
```

## Troubleshooting

### Issue: "Weaver not found"

```bash
# Install Weaver
cargo install weaver-cli
weaver --version
```

### Issue: "Zero samples received"

```toml
# Check OTEL configuration
[otel]
exporter = "otlp-http"  # Must be configured

[weaver]
enabled = true  # Must be enabled
```

### Issue: "Port conflict"

```toml
[weaver]
otlp_port = 0    # Use auto-discovery
admin_port = 0
```

## Next Steps

- **[Live-Check Guide](../../docs/LIVE_CHECK_GUIDE.md)** - Complete documentation
- **[Migration Guide](../../docs/MIGRATING_TO_V1_3_0.md)** - Upgrade from v1.2.x
- **[Best Practices](../../docs/LIVE_CHECK_BEST_PRACTICES.md)** - Advanced patterns
- **[Troubleshooting](../../docs/LIVE_CHECK_TROUBLESHOOTING.md)** - Problem solving

## Contributing

Found a useful pattern? Submit a PR with a new example!

---

**Last Updated**: 2025-10-31
**Version**: v1.3.0
