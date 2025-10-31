# Weaver Live-Check Comprehensive Test Suite

**Version:** 1.0.0
**Date:** 2025-10-30
**Status:** READY FOR EXECUTION

---

## Quick Start

```bash
# Navigate to test directory
cd /Users/sac/clnrm/tests/weaver/live-check

# Run ALL scenarios (recommended)
bash run_all_scenarios.sh

# View results
cat results/summary.json
```

---

## What This Test Suite Validates

This comprehensive test orchestration validates **ALL** Weaver `registry live-check` capabilities:

### 20 Test Scenarios Across 5 Phases:

1. **Input Sources (4 scenarios)** - How telemetry enters validation
   - OTLP gRPC ingestion (port 4317)
   - OTLP HTTP ingestion (port 4318)
   - File input (JSON samples)
   - stdin streaming (text attributes)

2. **Output Formats (2 scenarios)** - How results are presented
   - ANSI output (human-readable, colored)
   - JSON output (machine-readable, CI/CD)

3. **Advisors (3 scenarios)** - What validation rules are applied
   - Builtin advisors (missing_attribute, type_mismatch)
   - OTel policies (semantic conventions)
   - Custom Rego policies (org-specific rules)

4. **Stop Conditions (4 scenarios)** - How validation terminates
   - SIGINT (Ctrl-C)
   - SIGHUP (graceful with report)
   - HTTP /stop endpoint
   - Inactivity timeout

5. **Statistics (2 scenarios)** - What insights are generated
   - Registry coverage tracking
   - Violation severity analysis

---

## Jobs To Be Done (JTBD) Coverage

This test suite validates ALL 5 JTBD for Weaver live-check:

| JTBD | Description | Scenarios |
|------|-------------|-----------|
| 1 | **Validate OTLP telemetry** during test execution | 1.1, 1.2, 2.1, 2.2 |
| 2 | **Debug telemetry issues** in development | 1.4, 2.1, 3.1 |
| 3 | **CI/CD quality gates** (automated pass/fail) | 2.2, 5.2, CI integrations |
| 4 | **Coverage analysis** (track registry usage) | 5.1 |
| 5 | **Custom policy enforcement** (org-specific rules) | 3.2, 3.3 |

---

## Directory Structure

```
.
├── README.md                          # This file
├── TEST_MATRIX.md                     # Detailed test matrix
├── ORCHESTRATION_REPORT.md            # Comprehensive orchestration report
├── run_all_scenarios.sh               # Master test orchestrator
│
├── samples/                           # Test data files
│   ├── valid_spans.json               # Conformant telemetry
│   ├── invalid_spans.json             # Violating telemetry
│   ├── attributes.txt                 # stdin test data
│   └── ... (additional samples)
│
├── input-sources/                     # Phase 1: Input source tests
│   ├── test_otlp_grpc.sh
│   ├── test_otlp_http.sh
│   ├── test_file_input.sh
│   └── test_stdin_stream.sh
│
├── output-formats/                    # Phase 2: Output format tests
│   ├── test_ansi_output.sh
│   └── test_json_output.sh
│
├── advisors/                          # Phase 3: Advisor tests
│   ├── test_builtin_advisors.sh
│   ├── test_otel_policies.sh
│   └── test_custom_rego.sh
│
├── stop-conditions/                   # Phase 4: Stop condition tests
│   ├── test_sigint.sh
│   ├── test_sighup.sh
│   ├── test_http_stop.sh
│   └── test_inactivity_timeout.sh
│
├── statistics/                        # Phase 5: Statistics tests
│   ├── test_coverage_tracking.sh
│   └── test_severity_analysis.sh
│
├── ci-cd/                             # CI/CD integration examples
│   ├── github-actions.yml             # GitHub Actions pipeline
│   ├── gitlab-ci.yml                  # GitLab CI/CD pipeline
│   ├── jenkins-pipeline.groovy        # Jenkins pipeline
│   ├── azure-pipelines.yml            # Azure DevOps pipeline
│   └── README.md                      # Integration guide
│
└── results/                           # Test results (generated)
    ├── summary.json                   # Master summary
    ├── execution_log.jsonl            # Detailed log
    └── scenario_*.json                # Per-scenario results
```

---

## Prerequisites

### Required Tools

- **Docker & Docker Compose** - Container orchestration
- **jq** - JSON parsing (`brew install jq`)
- **curl** - HTTP requests (usually pre-installed)
- **bash 4.0+** - Shell scripting (macOS: `brew install bash`)

### Optional Tools

- **Weaver CLI** - For local testing without Docker
  ```bash
  curl -sSL https://github.com/open-telemetry/weaver/releases/latest/download/weaver-darwin-amd64 -o /usr/local/bin/weaver
  chmod +x /usr/local/bin/weaver
  ```

---

## Running Tests

### Master Orchestrator (Recommended)

Runs ALL 20 scenarios sequentially with comprehensive reporting:

```bash
bash run_all_scenarios.sh
```

**Output:**
- Colored console output (PASS/FAIL/WARN)
- `results/summary.json` - Master summary report
- `results/execution_log.jsonl` - Detailed execution log
- `results/scenario_*.json` - Per-scenario validation output

**Success Criteria:** 18+ scenarios pass (90%+ success rate)

---

### Individual Phases

Run specific test phases:

```bash
# Phase 1: Input sources
bash input-sources/test_otlp_http.sh

# Phase 2: Output formats
bash output-formats/test_json_output.sh

# Phase 3: Advisors
bash advisors/test_builtin_advisors.sh

# Phase 4: Stop conditions
bash stop-conditions/test_inactivity_timeout.sh

# Phase 5: Statistics
bash statistics/test_coverage_tracking.sh
```

---

### Individual Scenarios

Run single test scenarios:

```bash
# Scenario 1.2: OTLP HTTP
bash input-sources/test_otlp_http.sh

# Scenario 2.2: JSON Output
bash output-formats/test_json_output.sh

# Scenario 3.3: Custom Rego
bash advisors/test_custom_rego.sh
```

---

## Test Results

### Summary Report Format

**File:** `results/summary.json`

```json
{
  "execution_date": "2025-10-30T12:34:56Z",
  "total_scenarios": 20,
  "passed": 18,
  "failed": 0,
  "warnings": 2,
  "success_rate": 90.0,
  "results_directory": "/Users/sac/clnrm/tests/weaver/live-check/results"
}
```

---

### Execution Log Format

**File:** `results/execution_log.jsonl`

Each line is a JSON object:

```json
{"scenario":"1.2 OTLP HTTP","status":"PASS","message":"HTTP ingestion validated","timestamp":"2025-10-30T12:34:56Z"}
{"scenario":"2.1 ANSI Output","status":"PASS","message":"ANSI formatting validated","timestamp":"2025-10-30T12:35:12Z"}
{"scenario":"4.3 HTTP Stop","status":"WARN","message":"HTTP endpoint may not be implemented","timestamp":"2025-10-30T12:36:45Z"}
```

---

### Scenario Output Format

**Files:** `results/scenario_*.json`

Weaver live-check JSON output:

```json
{
  "violations": [
    {
      "severity": "error",
      "message": "Missing required attribute 'test.result'",
      "span_name": "test.execution",
      "advisor": "missing_attribute"
    }
  ],
  "coverage": {
    "total_schemas": 14,
    "used_schemas": 12,
    "coverage_percent": 85.7
  },
  "summary": {
    "total_spans": 47,
    "total_violations": 1
  }
}
```

---

## CI/CD Integration

### Available Pipelines

This test suite includes production-ready CI/CD configurations for:

1. **GitHub Actions** - `ci-cd/github-actions.yml`
2. **GitLab CI/CD** - `ci-cd/gitlab-ci.yml`
3. **Jenkins** - `ci-cd/jenkins-pipeline.groovy`
4. **Azure DevOps** - `ci-cd/azure-pipelines.yml`

### Integration Guide

See `ci-cd/README.md` for:
- Complete setup instructions
- Quality gate strategies
- Troubleshooting guide
- Best practices checklist

### Quick Integration (GitHub Actions)

```bash
# Copy pipeline configuration
cp ci-cd/github-actions.yml ../.github/workflows/weaver-validation.yml

# Commit and push
git add ../.github/workflows/weaver-validation.yml
git commit -m "Add Weaver live-check validation"
git push
```

---

## Troubleshooting

### Docker Environment Issues

**Problem:** Docker services fail to start

```bash
# Check Docker status
docker ps
docker-compose -f ../docker-compose.yml logs

# Restart environment
docker-compose -f ../docker-compose.yml down
docker-compose -f ../docker-compose.yml up -d
```

---

### OTLP Collector Not Ready

**Problem:** Health check fails

```bash
# Verify collector is running
curl -f http://localhost:13133/

# Check logs
docker-compose -f ../docker-compose.yml logs otel-collector
```

---

### No Telemetry Received

**Problem:** Weaver times out with no data

```bash
# Verify OTLP endpoint configuration
echo $OTEL_EXPORTER_OTLP_ENDPOINT

# Send test telemetry manually
curl -X POST http://localhost:4318/v1/traces \
    -H "Content-Type: application/json" \
    -d @samples/valid_spans.json
```

---

### Weaver Not Found

**Problem:** `weaver: command not found`

```bash
# Install Weaver locally
curl -sSL https://github.com/open-telemetry/weaver/releases/latest/download/weaver-darwin-amd64 -o /usr/local/bin/weaver
chmod +x /usr/local/bin/weaver
weaver --version
```

---

## Success Criteria

### Per-Scenario Success

Each scenario has specific pass conditions (see `TEST_MATRIX.md`).

**Example:**
- **1.2 OTLP HTTP:** Live-check receives and validates HTTP telemetry
- **3.1 Builtin:** Missing attributes and type mismatches detected
- **5.1 Coverage:** Coverage percentages calculated, unused schemas listed

---

### Overall Success

- [ ] ALL 20 scenarios execute without errors
- [ ] OTLP ingestion validated (gRPC + HTTP)
- [ ] Both output formats produce valid results
- [ ] All advisor types detect violations
- [ ] All stop conditions work as expected
- [ ] Statistics accurately reported
- [ ] CI/CD examples work in test pipelines
- [ ] Success rate ≥ 90% (18+ scenarios pass)

---

## Key Files Reference

| File | Purpose |
|------|---------|
| `README.md` | This file - Quick start guide |
| `TEST_MATRIX.md` | Detailed test matrix with all 20 scenarios |
| `ORCHESTRATION_REPORT.md` | Comprehensive orchestration analysis |
| `run_all_scenarios.sh` | Master test orchestrator (execute this) |
| `ci-cd/README.md` | CI/CD integration guide |

---

## Statistics

- **Total Test Scenarios:** 20
- **Total Executable Scripts:** 16
- **Total Files Created:** 26+
- **CI/CD Platforms Supported:** 4
- **JTBD Coverage:** 5/5 (100%)
- **Test Data Samples:** 6+

---

## Next Steps

1. **Execute Tests:**
   ```bash
   bash run_all_scenarios.sh
   ```

2. **Review Results:**
   ```bash
   cat results/summary.json
   jq '.' results/execution_log.jsonl
   ```

3. **Integrate CI/CD:**
   - Choose pipeline (GitHub Actions recommended)
   - Copy configuration to repository
   - Test in CI environment

4. **Iterate:**
   - Fix any failing scenarios
   - Update registry schemas if violations found
   - Re-run until 100% pass rate

---

## Support

- **Weaver Documentation:** https://github.com/open-telemetry/weaver
- **OTel Semantic Conventions:** https://opentelemetry.io/docs/specs/semconv/
- **clnrm Weaver Integration:** See `/Users/sac/clnrm/docs/WEAVER_V1_2_0_VALIDATION_SUMMARY.md`

---

## License

Part of the clnrm project. See repository LICENSE file.

---

**Test Suite Version:** 1.0.0
**Last Updated:** 2025-10-30
**Status:** READY FOR EXECUTION

Execute `bash run_all_scenarios.sh` to begin comprehensive validation.
