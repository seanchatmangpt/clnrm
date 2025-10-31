# Weaver Testing Infrastructure for clnrm

**Status:** ✅ READY FOR EXECUTION
**Version:** 1.0.0
**Date:** 2025-10-30

---

## Overview

This directory contains comprehensive testing infrastructure for validating clnrm's integration with OpenTelemetry Weaver, the official OTel schema validation tool.

**Purpose:** Weaver `registry live-check` is the **ONLY** source of truth for validating that clnrm's telemetry matches its schema declarations. Tests can have false positives; Weaver validation cannot.

---

## Directory Structure

```
tests/weaver/
├── README.md                              # This file - Overview
├── LIVE_CHECK_TEST_SUITE_SUMMARY.md       # Delivery summary
├── docker-compose.yml                     # Docker environment (OTLP + Weaver)
├── otel-collector-config.yaml             # OTLP Collector configuration
│
└── live-check/                            # Comprehensive test suite (20 scenarios)
    ├── README.md                          # Quick start guide
    ├── TEST_MATRIX.md                     # Test matrix (20 scenarios)
    ├── ORCHESTRATION_REPORT.md            # Comprehensive orchestration report
    ├── run_all_scenarios.sh               # Master orchestrator (EXECUTE THIS)
    │
    ├── samples/                           # Test data files (6+)
    │   ├── valid_spans.json               # Conformant telemetry
    │   ├── invalid_spans.json             # Violating telemetry
    │   └── ... (additional samples)
    │
    ├── input-sources/                     # Phase 1: Input tests (4 scenarios)
    │   ├── test_otlp_grpc.sh
    │   ├── test_otlp_http.sh
    │   ├── test_file_input.sh
    │   └── test_stdin_stream.sh
    │
    ├── output-formats/                    # Phase 2: Output tests (2 scenarios)
    │   ├── test_ansi_output.sh
    │   └── test_json_output.sh
    │
    ├── advisors/                          # Phase 3: Advisor tests (3 scenarios)
    │   ├── test_builtin_advisors.sh
    │   ├── test_otel_policies.sh
    │   └── test_custom_rego.sh
    │
    ├── stop-conditions/                   # Phase 4: Stop tests (4 scenarios)
    │   ├── test_sigint.sh
    │   ├── test_sighup.sh
    │   ├── test_http_stop.sh
    │   └── test_inactivity_timeout.sh
    │
    ├── statistics/                        # Phase 5: Statistics tests (2 scenarios)
    │   ├── test_coverage_tracking.sh
    │   └── test_severity_analysis.sh
    │
    ├── ci-cd/                             # CI/CD integrations (4 platforms)
    │   ├── github-actions.yml             # GitHub Actions pipeline
    │   ├── gitlab-ci.yml                  # GitLab CI/CD pipeline
    │   ├── jenkins-pipeline.groovy        # Jenkins pipeline
    │   ├── azure-pipelines.yml            # Azure DevOps pipeline
    │   └── README.md                      # CI/CD integration guide
    │
    └── results/                           # Test results (generated)
        ├── summary.json                   # Master summary
        ├── execution_log.jsonl            # Detailed log
        └── scenario_*.json                # Per-scenario results
```

---

## Quick Start

### Execute Complete Test Suite

```bash
# Navigate to live-check directory
cd /Users/sac/clnrm/tests/weaver/live-check

# Run ALL 20 scenarios
bash run_all_scenarios.sh

# View results
cat results/summary.json
```

### Expected Output

```
========================================
Weaver Live-Check Comprehensive Testing
========================================

Phase 0: Docker Environment Setup
✅ PASS: Docker Setup - Environment started successfully

Phase 1: Input Sources (4 scenarios)
✅ PASS: 1.1 OTLP gRPC
✅ PASS: 1.2 OTLP HTTP
✅ PASS: 1.3 File Input
✅ PASS: 1.4 stdin Stream

... (phases 2-5)

Test Execution Summary
Total Scenarios: 20
Passed: 18
Failed: 0
Warnings: 2
Success Rate: 90.0%

✅ All scenarios completed successfully!
```

---

## What This Tests

### 5 Jobs To Be Done (JTBD)

1. **Validate OTLP telemetry** during test execution (main use case)
2. **Debug telemetry issues** in development (interactive use)
3. **CI/CD quality gates** (automated pass/fail)
4. **Coverage analysis** (track registry usage)
5. **Custom policy enforcement** (org-specific rules)

### 20 Test Scenarios

| Phase | Scenarios | Purpose |
|-------|-----------|---------|
| Input Sources | 4 | OTLP gRPC, OTLP HTTP, File, stdin |
| Output Formats | 2 | ANSI (human), JSON (CI/CD) |
| Advisors | 3 | Builtin, OTel policies, Custom Rego |
| Stop Conditions | 4 | SIGINT, SIGHUP, HTTP, Timeout |
| Statistics | 2 | Coverage tracking, Severity analysis |

**Total:** 20 scenarios, 100% JTBD coverage

---

## Key Documents

### For Quick Start
- **`live-check/README.md`** - Installation, execution, troubleshooting

### For Understanding
- **`live-check/TEST_MATRIX.md`** - All 20 scenarios detailed
- **`live-check/ORCHESTRATION_REPORT.md`** - Architecture and analysis
- **`LIVE_CHECK_TEST_SUITE_SUMMARY.md`** - Delivery summary

### For CI/CD Integration
- **`live-check/ci-cd/README.md`** - Complete integration guide
- **`live-check/ci-cd/*.yml`** - Production-ready pipelines

---

## CI/CD Integration

This test suite includes production-ready pipelines for:

1. **GitHub Actions** - `live-check/ci-cd/github-actions.yml`
2. **GitLab CI/CD** - `live-check/ci-cd/gitlab-ci.yml`
3. **Jenkins** - `live-check/ci-cd/jenkins-pipeline.groovy`
4. **Azure DevOps** - `live-check/ci-cd/azure-pipelines.yml`

### Quick Integration (GitHub Actions)

```bash
# Copy to .github/workflows
cp live-check/ci-cd/github-actions.yml ../../.github/workflows/weaver-validation.yml

# Commit and push
git add ../../.github/workflows/weaver-validation.yml
git commit -m "Add Weaver live-check validation to CI"
git push
```

See `live-check/ci-cd/README.md` for complete integration guide.

---

## Prerequisites

### Required Tools
- **Docker & Docker Compose** - Container orchestration
- **jq** - JSON parsing (`brew install jq`)
- **bash 4.0+** - Shell scripting

### Optional Tools
- **Weaver CLI** - For local testing without Docker
  ```bash
  curl -sSL https://github.com/open-telemetry/weaver/releases/latest/download/weaver-darwin-amd64 -o /usr/local/bin/weaver
  chmod +x /usr/local/bin/weaver
  ```

---

## Architecture

### Docker Environment

**File:** `docker-compose.yml`

**Services:**
- **otel-collector** - OTLP ingestion (ports 4317/4318)
- **weaver-validator** - Live-check validation

**Network:** `weaver-test-net` (bridge)

### Data Flow

```
Test Samples (JSON)
         │
         ▼
  OTLP Collector ◄── clnrm Application (Live)
         │
         ▼
  Weaver Live-Check
         │
         ├─► ANSI Output (human)
         │
         └─► JSON Output (CI/CD)
                  │
                  └─► Quality Gate (pass/fail)
```

---

## Success Criteria

### Overall Success
- [ ] ALL 20 scenarios execute without errors
- [ ] Success rate ≥ 90% (18+ scenarios pass)
- [ ] OTLP ingestion validated (gRPC + HTTP)
- [ ] Both output formats work (ANSI + JSON)
- [ ] All advisor types detect violations
- [ ] All stop conditions work
- [ ] Statistics accurately reported
- [ ] CI/CD examples work

### Per-Scenario Success
See `live-check/TEST_MATRIX.md` for detailed criteria.

---

## Troubleshooting

### Docker Issues

```bash
# Check Docker status
docker-compose ps

# View logs
docker-compose logs otel-collector
docker-compose logs weaver-validator

# Restart environment
docker-compose down && docker-compose up -d
```

### OTLP Collector Not Ready

```bash
# Health check
curl -f http://localhost:13133/

# Test OTLP endpoint
curl -X POST http://localhost:4318/v1/traces \
    -H "Content-Type: application/json" \
    -d @live-check/samples/valid_spans.json
```

### Weaver Not Found

```bash
# Install Weaver
curl -sSL https://github.com/open-telemetry/weaver/releases/latest/download/weaver-darwin-amd64 -o /usr/local/bin/weaver
chmod +x /usr/local/bin/weaver
weaver --version
```

See `live-check/README.md` for comprehensive troubleshooting.

---

## File Count Summary

- **Executable Scripts:** 16
- **Sample Data Files:** 6+
- **CI/CD Pipelines:** 4
- **Documentation Files:** 5
- **Total Files Created:** 26+

---

## Related Documentation

- **Weaver Integration Status:** `/Users/sac/clnrm/docs/WEAVER_V1_2_0_VALIDATION_SUMMARY.md`
- **Weaver Migration Plan:** `/Users/sac/clnrm/docs/WEAVER_REFACTOR_MIGRATION_PLAN.md`
- **clnrm README:** `/Users/sac/clnrm/README.md`
- **clnrm CLAUDE.md:** `/Users/sac/clnrm/CLAUDE.md`

---

## External Resources

- **Weaver Repository:** https://github.com/open-telemetry/weaver
- **OTel Semantic Conventions:** https://opentelemetry.io/docs/specs/semconv/
- **OTLP Specification:** https://opentelemetry.io/docs/specs/otlp/

---

## Next Steps

1. **Execute Tests:**
   ```bash
   cd live-check && bash run_all_scenarios.sh
   ```

2. **Review Results:**
   ```bash
   cat live-check/results/summary.json
   jq '.' live-check/results/execution_log.jsonl
   ```

3. **Integrate CI/CD:**
   - Choose platform (GitHub Actions recommended)
   - Copy pipeline configuration
   - Test in CI environment

4. **Iterate:**
   - Fix any failures
   - Update registry schemas if needed
   - Re-run until 100% pass

---

## License

Part of the clnrm project. See repository LICENSE.

---

**Test Infrastructure Version:** 1.0.0
**Status:** ✅ READY FOR EXECUTION
**Last Updated:** 2025-10-30

**Execute:** `cd live-check && bash run_all_scenarios.sh`
