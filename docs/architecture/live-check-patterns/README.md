# Weaver Live-Check Integration Patterns

**Document Version:** 1.0.0
**Date:** 2025-10-30

## Overview

This directory contains comprehensive integration patterns for OpenTelemetry Weaver `registry live-check` across all Job-To-Be-Done (JTBD) scenarios in the clnrm project.

## Documents

### [JTBD_INTEGRATION_PATTERNS.md](./JTBD_INTEGRATION_PATTERNS.md)

Complete implementation patterns for all 5 JTBD scenarios:

1. **Local Development Debugging** - Real-time validation during feature development
2. **CI/CD Quality Gate** - Automated validation in GitHub Actions
3. **Pre-Commit Hook** - Fast validation before commits
4. **Coverage Tracking** - Historical metrics and trends
5. **Production Monitoring** - Continuous validation of live telemetry

Each pattern includes:
- Complete implementation scripts
- Configuration examples
- Workflow diagrams
- Operational guidance

### [PATTERN_SELECTION_GUIDE.md](./PATTERN_SELECTION_GUIDE.md)

Decision framework for selecting the right pattern:

- Pattern comparison matrix
- Decision tree
- Cost-benefit analysis
- Integration examples
- Best practices

## Quick Start

### 1. Install Pre-Commit Hooks (Recommended First Step)

```bash
# One-time installation
./scripts/install_hooks.sh

# Commits now validate telemetry automatically
git commit -m "Add feature"
```

### 2. Local Development

```bash
# Terminal 1: Start live-check
./scripts/dev_live_check.sh

# Terminal 2: Run your code
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
cargo test --features otel
```

### 3. Quick Validation

```bash
# Validate specific changes
./scripts/quick_validate.sh "cargo test test_name"

# Validate all tests
./scripts/quick_validate.sh "cargo test --all --features otel"
```

### 4. CI/CD (Automatic)

```bash
# GitHub Actions runs automatically on:
# - Push to master/main
# - Pull requests
# See: .github/workflows/telemetry-validation.yml
```

### 5. Coverage Tracking

```bash
# Run coverage analysis
./scripts/track_coverage.sh

# View historical trends
./scripts/coverage_dashboard.py
```

### 6. Production Monitoring

```bash
# One-time validation
./scripts/validate_production_telemetry.sh --source https://prod-api/telemetry

# Continuous monitoring (Kubernetes)
kubectl apply -f k8s/telemetry-validation-cronjob.yaml
```

## Makefile Shortcuts

All patterns are accessible via Makefile:

```bash
# Development
make -f Makefile.weaver validate-dev      # Interactive development validation
make -f Makefile.weaver validate-quick    # Fast validation

# Git integration
make -f Makefile.weaver install-hooks     # Install pre-commit hooks

# Coverage
make -f Makefile.weaver track-coverage    # Track coverage
make -f Makefile.weaver coverage-dashboard # Show trends

# Production
make -f Makefile.weaver validate-prod     # Validate production
make -f Makefile.weaver export-metrics    # Export to Prometheus

# Help
make -f Makefile.weaver help              # Show all targets
```

## Pattern Comparison

| Pattern | Use Case | Response Time | Automation | Best For |
|---------|----------|---------------|------------|----------|
| **Local Dev** | Feature development | Real-time | Manual | Active coding |
| **Quick Validate** | Pre-push checks | 10-30s | Manual | Quick checks |
| **Pre-Commit** | Git validation | 10-30s | Automatic | Team enforcement |
| **CI/CD** | PR validation | 2-5 min | Automatic | Quality gates |
| **Coverage** | Metrics tracking | 5-10 min | Manual/Scheduled | Progress tracking |
| **Production** | Live monitoring | 1-5 min | Continuous | SLA compliance |

## Implementation Files

### Scripts (All Executable)

Located in `/scripts/`:

- `dev_live_check.sh` - Interactive development validation
- `quick_validate.sh` - Fast validation for changes
- `pre-commit.sh` - Pre-commit hook implementation
- `install_hooks.sh` - Hook installer
- `track_coverage.sh` - Coverage tracking
- `coverage_dashboard.py` - Coverage visualization
- `validate_production_telemetry.sh` - Production validation
- `export_validation_metrics.py` - Prometheus exporter

### Workflows

Located in `/.github/workflows/`:

- `telemetry-validation.yml` - CI/CD validation workflow
- `schema-validation.yml` - Schema syntax validation

### Kubernetes

Located in `/k8s/` (referenced in patterns):

- `telemetry-validation-cronjob.yaml` - Production monitoring CronJob

## Architecture Principles

All patterns follow these principles:

1. **Schema-First Validation** - Runtime telemetry must match registry schemas
2. **Fast Feedback** - <5 second feedback cycles where possible
3. **Zero Configuration** - Sensible defaults, explicit config when needed
4. **Observable Integration** - All patterns emit telemetry about validation
5. **Fail-Fast, Fail-Clear** - Immediate visibility with actionable errors

## Pattern Selection Decision Tree

```
Need validation?
  ├─ Actively coding? → Local Dev
  ├─ About to commit? → Pre-Commit
  ├─ Creating PR? → CI/CD (automatic)
  ├─ Tracking progress? → Coverage
  └─ Monitoring production? → Production
```

See [PATTERN_SELECTION_GUIDE.md](./PATTERN_SELECTION_GUIDE.md) for detailed decision framework.

## Validation Hierarchy

Weaver live-check validation is the **source of truth**:

```
1. Weaver Schema Validation (HIGHEST AUTHORITY)
   ├─ Runtime telemetry must match schemas
   └─ Only way to prove features work

2. Compilation (SECOND AUTHORITY)
   ├─ Type-safe builders prevent invalid telemetry
   └─ Proves code is valid

3. Traditional Tests (LOWEST AUTHORITY)
   ├─ Can have false positives
   └─ Supporting evidence only
```

**Critical principle:** If Weaver validation fails, the feature does NOT work, regardless of test results.

## Getting Help

- **Pattern Selection**: Read [PATTERN_SELECTION_GUIDE.md](./PATTERN_SELECTION_GUIDE.md)
- **Implementation**: Read [JTBD_INTEGRATION_PATTERNS.md](./JTBD_INTEGRATION_PATTERNS.md)
- **Issues**: Open GitHub issue with `weaver-integration` label

## Related Documentation

- `/docs/WEAVER_V1_2_0_VALIDATION_SUMMARY.md` - Current validation status
- `/docs/WEAVER_USER_GUIDE.md` - Weaver usage guide
- `/docs/RUNNING_WEAVER_VALIDATION.md` - Manual validation steps
- `/registry/` - OpenTelemetry schema registry

---

**Version History:**

- v1.0.0 (2025-10-30) - Initial comprehensive pattern design
