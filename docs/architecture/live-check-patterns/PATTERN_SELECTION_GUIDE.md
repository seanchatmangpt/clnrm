# Live-Check Pattern Selection Guide

**Document Version:** 1.0.0
**Date:** 2025-10-30
**Status:** Architecture Guide

## Overview

This guide helps you select the appropriate Weaver `registry live-check` integration pattern based on your Job-To-Be-Done (JTBD) scenario.

---

## Pattern Comparison Matrix

| Pattern | Use Case | Response Time | Scope | Resource Cost | Automation |
|---------|----------|---------------|-------|---------------|------------|
| **Local Dev** | Feature development | Real-time | Changed files | Low | Manual |
| **Quick Validate** | Pre-push checks | 10-30s | Changed tests | Low | Manual |
| **Pre-Commit** | Git commit validation | 10-30s | Changed files | Low | Automatic |
| **CI/CD** | PR/Push validation | 2-5 min | All tests | Medium | Automatic |
| **Coverage** | Weekly/sprint tracking | 5-10 min | Full suite | Medium | Manual/Scheduled |
| **Production** | Live monitoring | 1-5 min | Samples | High | Continuous |

---

## Decision Tree

```
START: What do you need to validate?

├─ Are you actively coding?
│  ├─ YES → **Local Dev Pattern**
│  │  - Real-time feedback
│  │  - Interactive debugging
│  │  - Use: ./scripts/dev_live_check.sh
│  │
│  └─ NO → Continue...

├─ About to commit code?
│  ├─ YES → **Pre-Commit Pattern**
│  │  - Fast validation (<30s)
│  │  - Only changed files
│  │  - Use: Install pre-commit hook
│  │
│  └─ NO → Continue...

├─ Pushing to remote / Creating PR?
│  ├─ YES → **CI/CD Pattern**
│  │  - Full validation
│  │  - Quality gate
│  │  - Use: GitHub Actions workflow
│  │
│  └─ NO → Continue...

├─ Tracking progress over time?
│  ├─ YES → **Coverage Tracking Pattern**
│  │  - Historical trends
│  │  - Sprint metrics
│  │  - Use: ./scripts/track_coverage.sh
│  │
│  └─ NO → Continue...

└─ Monitoring production?
   └─ YES → **Production Monitoring Pattern**
      - Continuous validation
      - Alert on violations
      - Use: Kubernetes CronJob + alerts
```

---

## Pattern Details

### 1. Local Dev Pattern

**When to use:**
- Actively developing new features
- Debugging telemetry issues
- Iterating on instrumentation

**Characteristics:**
- **Response Time**: Real-time (streaming)
- **Scope**: Files you're working on
- **Resource Cost**: Low (single process)
- **Automation**: Manual start/stop

**Setup:**
```bash
# Terminal 1: Start live-check
./scripts/dev_live_check.sh

# Terminal 2: Run your code
export OTEL_EXPORTER_OTLP_ENDPOINT=http://localhost:4317
cargo test --features otel
```

**Pros:**
- Immediate feedback on violations
- No waiting for test completion
- Interactive debugging
- Works with any development workflow

**Cons:**
- Requires manual startup
- Terminal management overhead
- Not automated

**Best for:**
- Feature development
- Telemetry debugging
- Learning OTel schemas

---

### 2. Quick Validate Pattern

**When to use:**
- Before pushing to remote
- Testing specific changes
- Quick sanity checks

**Characteristics:**
- **Response Time**: 10-30 seconds
- **Scope**: Specified tests/files
- **Resource Cost**: Low
- **Automation**: Manual invocation

**Setup:**
```bash
# Validate specific test
./scripts/quick_validate.sh "cargo test test_name"

# Validate all tests
./scripts/quick_validate.sh "cargo test --all --features otel"
```

**Pros:**
- Fast feedback (<30s)
- Flexible test selection
- Simple script invocation
- No hook installation needed

**Cons:**
- Must remember to run
- Not automatic
- Manual process

**Best for:**
- Pre-push validation
- Testing specific changes
- Ad-hoc validation

---

### 3. Pre-Commit Pattern

**When to use:**
- Enforcing validation before commits
- Team-wide consistency
- Preventing broken telemetry from entering history

**Characteristics:**
- **Response Time**: 10-30 seconds
- **Scope**: Changed files in commit
- **Resource Cost**: Low
- **Automation**: Automatic on `git commit`

**Setup:**
```bash
# Install hook (one-time)
./scripts/install_hooks.sh

# Commits now run validation automatically
git commit -m "Add feature"

# Bypass if needed (not recommended)
git commit --no-verify -m "WIP"
```

**Pros:**
- Automatic enforcement
- No manual intervention
- Catches issues before commit
- Fast (only changed files)

**Cons:**
- Adds time to commit process
- Can be bypassed with --no-verify
- Requires hook installation per clone

**Best for:**
- Team enforcement
- Preventing broken commits
- Continuous validation

---

### 4. CI/CD Pattern

**When to use:**
- Pull request validation
- Pre-merge quality gates
- Branch protection rules

**Characteristics:**
- **Response Time**: 2-5 minutes
- **Scope**: All tests
- **Resource Cost**: Medium (CI resources)
- **Automation**: Automatic on push/PR

**Setup:**
```yaml
# .github/workflows/telemetry-validation.yml (already configured)
# Triggers on: push, pull_request
```

**Features:**
- Full test suite validation
- Quality gate enforcement
- PR comments with results
- Artifact upload for debugging
- Branch protection integration

**Pros:**
- Comprehensive validation
- Blocks bad PRs from merging
- Visible in PR status
- Historical artifact tracking
- No local setup required

**Cons:**
- Slower than local validation
- Uses CI minutes/resources
- Not immediate feedback
- Network latency

**Best for:**
- Pull request validation
- Merge protection
- Team collaboration
- Release gates

---

### 5. Coverage Tracking Pattern

**When to use:**
- Sprint/weekly metrics
- Tracking validation progress
- Schema coverage goals

**Characteristics:**
- **Response Time**: 5-10 minutes
- **Scope**: Full test suite
- **Resource Cost**: Medium
- **Automation**: Manual or scheduled

**Setup:**
```bash
# Run coverage analysis
./scripts/track_coverage.sh

# View dashboard
./scripts/coverage_dashboard.py

# View last 7 days
./scripts/coverage_dashboard.py --days 7
```

**Features:**
- Historical trend tracking
- Coverage percentage over time
- Violation tracking
- Baseline enforcement (80% default)
- Visual ASCII charts

**Pros:**
- Historical metrics
- Progress tracking
- Baseline enforcement
- Visual trends
- Git commit correlation

**Cons:**
- Slower (full suite)
- Manual invocation (unless scheduled)
- Requires Python for dashboard

**Best for:**
- Sprint metrics
- Coverage goals
- Progress tracking
- Management reporting

---

### 6. Production Monitoring Pattern

**When to use:**
- Validating live production telemetry
- Detecting schema drift
- Continuous compliance monitoring

**Characteristics:**
- **Response Time**: 1-5 minutes
- **Scope**: Telemetry samples from production
- **Resource Cost**: High (continuous)
- **Automation**: Continuous (CronJob)

**Setup:**
```bash
# One-time validation
./scripts/validate_production_telemetry.sh --source https://prod-api/telemetry

# Kubernetes CronJob (see k8s/telemetry-validation-cronjob.yaml)
kubectl apply -f k8s/telemetry-validation-cronjob.yaml

# Prometheus metrics export
python3 scripts/export_validation_metrics.py --port 9090
```

**Features:**
- Continuous monitoring
- Slack/PagerDuty alerts
- Prometheus metrics
- Sample-based validation
- Historical log

**Pros:**
- Real production validation
- Continuous monitoring
- Alert integration
- Metrics export
- Detects drift

**Cons:**
- High resource cost
- Requires production access
- Network dependencies
- Complex setup

**Best for:**
- Production monitoring
- SLA compliance
- Schema drift detection
- 24/7 validation

---

## Pattern Combinations

### Recommended Workflow

For maximum effectiveness, use multiple patterns together:

**Development Phase:**
1. **Local Dev** - Real-time feedback while coding
2. **Quick Validate** - Before committing major changes

**Commit Phase:**
3. **Pre-Commit** - Automatic validation on commit
4. **Quick Validate** - Pre-push sanity check

**Collaboration Phase:**
5. **CI/CD** - PR validation and merge protection

**Metrics Phase:**
6. **Coverage Tracking** - Weekly sprint metrics

**Production Phase:**
7. **Production Monitoring** - Continuous live validation

---

## Pattern Selection Checklist

Use this checklist to select the right pattern:

- [ ] **Are you actively coding?**
  - YES → Local Dev Pattern

- [ ] **Need validation before commit?**
  - YES → Pre-Commit Pattern

- [ ] **Pushing code or creating PR?**
  - YES → CI/CD Pattern

- [ ] **Tracking metrics over time?**
  - YES → Coverage Tracking Pattern

- [ ] **Monitoring production?**
  - YES → Production Monitoring Pattern

- [ ] **Quick sanity check?**
  - YES → Quick Validate Pattern

---

## Cost-Benefit Analysis

| Pattern | Setup Cost | Run Cost | Value Delivered |
|---------|-----------|----------|-----------------|
| **Local Dev** | Low (1 min) | Low (manual) | High (immediate feedback) |
| **Quick Validate** | Low (1 min) | Low (per run) | Medium (fast validation) |
| **Pre-Commit** | Medium (5 min) | Low (automatic) | High (prevent bad commits) |
| **CI/CD** | Medium (15 min) | Medium (CI resources) | Very High (quality gate) |
| **Coverage** | Low (1 min) | Medium (full suite) | Medium (metrics) |
| **Production** | High (30 min) | High (continuous) | Very High (live validation) |

---

## Integration Examples

### Makefile Targets

All patterns are available via Makefile:

```bash
# Development
make -f Makefile.weaver validate-dev
make -f Makefile.weaver validate-quick

# Git integration
make -f Makefile.weaver install-hooks

# Coverage
make -f Makefile.weaver track-coverage
make -f Makefile.weaver coverage-dashboard

# Production
make -f Makefile.weaver validate-prod
make -f Makefile.weaver export-metrics
```

### VS Code Tasks

Configure VS Code to run patterns:

```json
// .vscode/tasks.json
{
  "version": "2.0.0",
  "tasks": [
    {
      "label": "Validate: Local Dev",
      "type": "shell",
      "command": "./scripts/dev_live_check.sh"
    },
    {
      "label": "Validate: Quick",
      "type": "shell",
      "command": "./scripts/quick_validate.sh \"cargo test --features otel\""
    }
  ]
}
```

---

## Summary

**Choose your pattern based on:**

1. **Speed needs** → Quick Validate or Pre-Commit
2. **Automation needs** → Pre-Commit or CI/CD
3. **Scope needs** → Local Dev (focused) or CI/CD (comprehensive)
4. **Cost constraints** → Local Dev (cheapest) or Production (most expensive)
5. **Metrics needs** → Coverage Tracking

**Default recommendation:**
- Install **Pre-Commit** hook for automatic validation
- Use **Local Dev** when developing features
- Let **CI/CD** handle PR validation automatically
- Run **Coverage Tracking** weekly for metrics

This combination provides comprehensive coverage with minimal overhead.
