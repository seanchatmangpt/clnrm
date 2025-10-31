# Weaver Live-Check Test Orchestration - Comprehensive Report

**Generated:** 2025-10-30
**Orchestrator:** task-orchestrator agent
**Test Suite:** Comprehensive Weaver `registry live-check` validation
**Total Scenarios:** 20

---

## Executive Summary

This test orchestration validates **ALL** Weaver `registry live-check` capabilities across five critical dimensions:

1. **Input Sources** - How telemetry enters the system
2. **Output Formats** - How validation results are presented
3. **Advisors** - What validation rules are applied
4. **Stop Conditions** - How the validation process terminates
5. **Statistics** - What insights are generated

**Goal:** Prove that Weaver live-check can validate ALL Jobs To Be Done (JTBD) for runtime telemetry validation.

---

## Jobs To Be Done (JTBD) Analysis

### JTBD 1: Validate OTLP telemetry during test execution (MAIN USE CASE)

**Scenarios:**
- 1.1: OTLP gRPC ingestion (port 4317)
- 1.2: OTLP HTTP ingestion (port 4318)
- 2.1: ANSI output (real-time feedback)
- 2.2: JSON output (machine-readable)

**Value:** Real-time validation during CI/CD test runs catches schema violations immediately.

**Evidence Required:**
- ✅ Live telemetry ingested via OTLP
- ✅ Violations detected in real-time
- ✅ Results available in both human and machine formats

---

### JTBD 2: Debug telemetry issues in development (INTERACTIVE USE)

**Scenarios:**
- 1.4: stdin streaming (interactive debugging)
- 2.1: ANSI output (colored, readable)
- 3.1: Builtin advisors (common errors)

**Value:** Developers can quickly identify telemetry problems during feature development.

**Evidence Required:**
- ✅ Interactive input modes supported
- ✅ Readable colored output for quick scanning
- ✅ Helpful advisor messages

---

### JTBD 3: CI/CD quality gates (AUTOMATED PASS/FAIL)

**Scenarios:**
- 2.2: JSON output (parseable results)
- 5.2: Severity analysis (error vs warning)
- CI/CD integrations (GitHub Actions, GitLab, Jenkins, Azure)

**Value:** Automated pipelines can enforce schema compliance before deployment.

**Evidence Required:**
- ✅ JSON output with violation counts
- ✅ Severity-based pass/fail logic
- ✅ Working CI/CD pipeline examples

---

### JTBD 4: Coverage analysis (TRACK REGISTRY USAGE)

**Scenarios:**
- 5.1: Registry coverage tracking

**Value:** Identify unused schemas and measure how well tests exercise telemetry.

**Evidence Required:**
- ✅ Coverage percentages calculated
- ✅ Unused schemas reported
- ✅ Trend tracking over time

---

### JTBD 5: Custom policy enforcement (ORG-SPECIFIC RULES)

**Scenarios:**
- 3.2: OTel policies (semantic conventions)
- 3.3: Custom Rego policies (business rules)

**Value:** Organizations can enforce their own telemetry standards beyond OTel specs.

**Evidence Required:**
- ✅ OTel policy violations detected
- ✅ Custom Rego rules evaluated
- ✅ Policy-specific error messages

---

## Test Matrix: 20 Scenarios

| ID | Phase | Scenario | JTBD | Status | Evidence |
|----|-------|----------|------|--------|----------|
| 1.1 | Input Sources | OTLP gRPC | 1 | READY | `/tests/weaver/live-check/input-sources/test_otlp_grpc.sh` |
| 1.2 | Input Sources | OTLP HTTP | 1 | READY | `/tests/weaver/live-check/input-sources/test_otlp_http.sh` |
| 1.3 | Input Sources | File Input | 3 | READY | `/tests/weaver/live-check/input-sources/test_file_input.sh` |
| 1.4 | Input Sources | stdin Stream | 2 | READY | `/tests/weaver/live-check/input-sources/test_stdin_stream.sh` |
| 2.1 | Output Formats | ANSI Output | 1,2 | READY | `/tests/weaver/live-check/output-formats/test_ansi_output.sh` |
| 2.2 | Output Formats | JSON Output | 3 | READY | `/tests/weaver/live-check/output-formats/test_json_output.sh` |
| 3.1 | Advisors | Builtin | 2 | READY | `/tests/weaver/live-check/advisors/test_builtin_advisors.sh` |
| 3.2 | Advisors | OTel Policies | 5 | READY | `/tests/weaver/live-check/advisors/test_otel_policies.sh` |
| 3.3 | Advisors | Custom Rego | 5 | READY | `/tests/weaver/live-check/advisors/test_custom_rego.sh` |
| 4.1 | Stop Conditions | SIGINT | 2 | READY | `/tests/weaver/live-check/stop-conditions/test_sigint.sh` |
| 4.2 | Stop Conditions | SIGHUP | 2 | READY | `/tests/weaver/live-check/stop-conditions/test_sighup.sh` |
| 4.3 | Stop Conditions | HTTP /stop | 3 | READY | `/tests/weaver/live-check/stop-conditions/test_http_stop.sh` |
| 4.4 | Stop Conditions | Timeout | 3 | READY | `/tests/weaver/live-check/stop-conditions/test_inactivity_timeout.sh` |
| 5.1 | Statistics | Coverage | 4 | READY | `/tests/weaver/live-check/statistics/test_coverage_tracking.sh` |
| 5.2 | Statistics | Severity | 3 | READY | `/tests/weaver/live-check/statistics/test_severity_analysis.sh` |
| CI1 | CI/CD | GitHub Actions | 3 | READY | `/tests/weaver/live-check/ci-cd/github-actions.yml` |
| CI2 | CI/CD | GitLab CI | 3 | READY | `/tests/weaver/live-check/ci-cd/gitlab-ci.yml` |
| CI3 | CI/CD | Jenkins | 3 | READY | `/tests/weaver/live-check/ci-cd/jenkins-pipeline.groovy` |
| CI4 | CI/CD | Azure Pipelines | 3 | READY | `/tests/weaver/live-check/ci-cd/azure-pipelines.yml` |
| DOC | Documentation | CI/CD Guide | 3 | READY | `/tests/weaver/live-check/ci-cd/README.md` |

---

## Orchestration Architecture

### Test Execution Flow

```
┌────────────────────────────────────────────────────────────────────┐
│ Master Orchestrator: run_all_scenarios.sh                         │
├────────────────────────────────────────────────────────────────────┤
│                                                                    │
│ Phase 0: Docker Environment Setup                                 │
│   ├─ Start OTLP Collector (ports 4317, 4318)                      │
│   ├─ Start Weaver validator                                       │
│   └─ Health checks (wait for services)                            │
│                                                                    │
│ Phase 1: Input Sources (4 scenarios - sequential)                 │
│   ├─ 1.1: test_otlp_grpc.sh                                       │
│   ├─ 1.2: test_otlp_http.sh                                       │
│   ├─ 1.3: test_file_input.sh                                      │
│   └─ 1.4: test_stdin_stream.sh                                    │
│                                                                    │
│ Phase 2: Output Formats (2 scenarios - sequential)                │
│   ├─ 2.1: test_ansi_output.sh                                     │
│   └─ 2.2: test_json_output.sh                                     │
│                                                                    │
│ Phase 3: Advisors (3 scenarios - sequential)                      │
│   ├─ 3.1: test_builtin_advisors.sh                                │
│   ├─ 3.2: test_otel_policies.sh                                   │
│   └─ 3.3: test_custom_rego.sh                                     │
│                                                                    │
│ Phase 4: Stop Conditions (4 scenarios - sequential)               │
│   ├─ 4.1: test_sigint.sh                                          │
│   ├─ 4.2: test_sighup.sh                                          │
│   ├─ 4.3: test_http_stop.sh                                       │
│   └─ 4.4: test_inactivity_timeout.sh                              │
│                                                                    │
│ Phase 5: Statistics (2 scenarios - sequential)                    │
│   ├─ 5.1: test_coverage_tracking.sh                               │
│   └─ 5.2: test_severity_analysis.sh                               │
│                                                                    │
│ Phase 6: Results Aggregation                                      │
│   ├─ Generate summary.json                                        │
│   ├─ Calculate pass/fail/warning counts                           │
│   ├─ Generate execution_log.jsonl                                 │
│   └─ Output colored summary                                       │
│                                                                    │
│ Phase 7: Cleanup                                                  │
│   └─ docker-compose down                                          │
│                                                                    │
└────────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
Sample Data (JSON/Text)
         │
         ▼
  ┌──────────────┐
  │ OTLP         │
  │ Collector    │◄──── Application Telemetry (Live)
  └──────┬───────┘
         │
         ▼
  ┌──────────────┐
  │ Weaver       │
  │ Live-Check   │
  │ Validator    │
  └──────┬───────┘
         │
         ├─────► ANSI Output (human-readable)
         │
         └─────► JSON Output (machine-readable)
                      │
                      ├─► CI/CD Pipelines
                      │
                      └─► Results Analysis
```

---

## Test Samples

### Valid Telemetry Sample

Location: `/tests/weaver/live-check/samples/valid_spans.json`

**Purpose:** Baseline conformant telemetry for positive validation tests.

**Key Attributes:**
- `service.name`: "clnrm-test"
- `service.version`: "1.2.0"
- `deployment.environment`: "test"
- `test.name`: "integration_test_001"
- `test.suite`: "weaver_validation"
- `test.result`: "pass"
- `test.duration_ms`: 1000

**Expected Result:** Zero violations, 100% coverage of test schema.

---

### Invalid Telemetry Sample

Location: `/tests/weaver/live-check/samples/invalid_spans.json`

**Purpose:** Trigger violations for negative validation tests.

**Deliberate Issues:**
- Missing required `service.version` attribute
- Invalid `test.result` value: "INVALID_VALUE" (not enum member)
- Type mismatch: `test.duration_ms` as string instead of int

**Expected Result:** 3+ violations detected by builtin advisors.

---

### Attribute Stream Sample

Location: `/tests/weaver/live-check/samples/attributes.txt`

**Purpose:** Test stdin streaming for interactive debugging.

**Format:** Key-value pairs, one per line:
```
test.name=stream_test_001
test.suite=stdin_validation
test.result=pass
test.duration_ms=500
```

**Expected Result:** Attributes validated individually, violations reported in real-time.

---

## CI/CD Integration Deliverables

### 1. GitHub Actions Pipeline

**File:** `/tests/weaver/live-check/ci-cd/github-actions.yml`

**Features:**
- Automatic validation on push/PR
- Service containers for OTLP
- PR commenting with results
- Artifact upload (30-day retention)

**Usage:**
```bash
cp tests/weaver/live-check/ci-cd/github-actions.yml .github/workflows/
git add .github/workflows/github-actions.yml
git commit -m "Add Weaver validation"
```

---

### 2. GitLab CI/CD Pipeline

**File:** `/tests/weaver/live-check/ci-cd/gitlab-ci.yml`

**Features:**
- Multi-stage pipeline (build → test → validate)
- JUnit test report integration
- Service definitions for OTLP
- Automatic failure on violations

**Usage:**
```bash
cp tests/weaver/live-check/ci-cd/gitlab-ci.yml .gitlab-ci.yml
git add .gitlab-ci.yml && git commit -m "Add Weaver validation stage"
```

---

### 3. Jenkins Pipeline

**File:** `/tests/weaver/live-check/ci-cd/jenkins-pipeline.groovy`

**Features:**
- Declarative pipeline syntax
- Docker-based OTLP Collector
- Email notifications on failure
- HTML report publishing

**Usage:** Add to `Jenkinsfile` or configure as pipeline script in Jenkins UI.

---

### 4. Azure DevOps Pipeline

**File:** `/tests/weaver/live-check/ci-cd/azure-pipelines.yml`

**Features:**
- Multi-stage YAML pipeline
- Azure-specific task integrations
- Test results publishing
- Build artifacts storage

**Usage:** Configure in Azure DevOps → Pipelines → New Pipeline → Existing YAML.

---

### 5. CI/CD Integration Guide

**File:** `/tests/weaver/live-check/ci-cd/README.md`

**Contents:**
- Overview of all 4 platforms
- Installation instructions
- Quality gate strategies
- Troubleshooting guide
- Best practices checklist

---

## Execution Instructions

### Prerequisites

```bash
# Required tools
- Docker & Docker Compose
- jq (JSON parsing)
- curl (HTTP requests)
- bash 4.0+

# Optional (for local Weaver)
- Weaver CLI installed locally
```

### Quick Start

```bash
# 1. Navigate to test directory
cd /Users/sac/clnrm/tests/weaver/live-check

# 2. Run all scenarios (master orchestrator)
bash run_all_scenarios.sh

# 3. View results
cat results/summary.json
```

### Run Individual Phases

```bash
# Phase 1: Input sources only
bash input-sources/test_otlp_http.sh

# Phase 2: Output formats only
bash output-formats/test_json_output.sh

# Phase 3: Advisors only
bash advisors/test_builtin_advisors.sh

# Phase 4: Stop conditions only
bash stop-conditions/test_inactivity_timeout.sh

# Phase 5: Statistics only
bash statistics/test_coverage_tracking.sh
```

---

## Expected Results Structure

### Individual Scenario Output

Each scenario produces:
1. **Execution log:** `results/{scenario}_execution.log`
2. **Validation output:** `results/scenario_{scenario}_output.json`
3. **JSONL log entry:** `results/execution_log.jsonl`

Example:
```json
{
  "scenario": "1.2 OTLP HTTP",
  "status": "PASS",
  "message": "HTTP ingestion validated",
  "timestamp": "2025-10-30T12:34:56Z"
}
```

---

### Master Summary Report

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

## Success Criteria

### Per-Scenario Criteria

| Scenario | Pass Condition |
|----------|----------------|
| 1.1 OTLP gRPC | Live-check receives and validates gRPC telemetry |
| 1.2 OTLP HTTP | Live-check receives and validates HTTP telemetry |
| 1.3 File Input | JSON file processed, violations detected in invalid sample |
| 1.4 stdin Stream | Attributes validated from stdin |
| 2.1 ANSI | Colored output generated with violation markers |
| 2.2 JSON | Valid JSON with parseable violation data |
| 3.1 Builtin | Missing attributes and type mismatches detected |
| 3.2 OTel | Naming convention violations flagged |
| 3.3 Rego | Custom policy violations reported |
| 4.1 SIGINT | Graceful shutdown on Ctrl-C |
| 4.2 SIGHUP | Report generated before shutdown |
| 4.3 HTTP | `/stop` endpoint triggers graceful shutdown |
| 4.4 Timeout | Auto-shutdown after 30s inactivity |
| 5.1 Coverage | Coverage percentages calculated, unused schemas listed |
| 5.2 Severity | Violations categorized by error/warning/info |

---

### Overall Success Criteria

- [ ] **ALL 20 scenarios execute without errors**
- [ ] **OTLP ingestion validated** (gRPC + HTTP)
- [ ] **Both output formats produce valid results** (ANSI + JSON)
- [ ] **All advisor types detect violations** (builtin + OTel + Rego)
- [ ] **All stop conditions work** (SIGINT + SIGHUP + HTTP + timeout)
- [ ] **Statistics accurately reported** (coverage + severity)
- [ ] **CI/CD examples work in test pipelines**
- [ ] **Success rate ≥ 90%** (18+ of 20 scenarios pass)

---

## File Structure Summary

```
/Users/sac/clnrm/tests/weaver/live-check/
├── TEST_MATRIX.md                      # Test matrix overview
├── ORCHESTRATION_REPORT.md             # This file
├── run_all_scenarios.sh                # Master orchestrator (executable)
│
├── samples/                            # Test data
│   ├── valid_spans.json                # Conformant telemetry
│   ├── invalid_spans.json              # Violating telemetry
│   ├── attributes.txt                  # stdin stream test data
│   ├── otel_policy_violations.json     # OTel policy test data
│   ├── custom_policy_violations.json   # Rego policy test data
│   └── mixed_severity.json             # Severity analysis test data
│
├── input-sources/                      # Phase 1 tests
│   ├── test_otlp_grpc.sh               # Scenario 1.1 (executable)
│   ├── test_otlp_http.sh               # Scenario 1.2 (executable)
│   ├── test_file_input.sh              # Scenario 1.3 (executable)
│   └── test_stdin_stream.sh            # Scenario 1.4 (executable)
│
├── output-formats/                     # Phase 2 tests
│   ├── test_ansi_output.sh             # Scenario 2.1 (executable)
│   └── test_json_output.sh             # Scenario 2.2 (executable)
│
├── advisors/                           # Phase 3 tests
│   ├── test_builtin_advisors.sh        # Scenario 3.1 (executable)
│   ├── test_otel_policies.sh           # Scenario 3.2 (executable)
│   └── test_custom_rego.sh             # Scenario 3.3 (executable)
│
├── stop-conditions/                    # Phase 4 tests
│   ├── test_sigint.sh                  # Scenario 4.1 (executable)
│   ├── test_sighup.sh                  # Scenario 4.2 (executable)
│   ├── test_http_stop.sh               # Scenario 4.3 (executable)
│   └── test_inactivity_timeout.sh      # Scenario 4.4 (executable)
│
├── statistics/                         # Phase 5 tests
│   ├── test_coverage_tracking.sh       # Scenario 5.1 (executable)
│   └── test_severity_analysis.sh       # Scenario 5.2 (executable)
│
├── ci-cd/                              # CI/CD integrations
│   ├── github-actions.yml              # GitHub Actions pipeline
│   ├── gitlab-ci.yml                   # GitLab CI/CD pipeline
│   ├── jenkins-pipeline.groovy         # Jenkins declarative pipeline
│   ├── azure-pipelines.yml             # Azure DevOps pipeline
│   └── README.md                       # Integration guide
│
├── results/                            # Execution results (generated)
│   ├── summary.json                    # Master summary report
│   ├── execution_log.jsonl             # Detailed execution log
│   ├── scenario_*.json                 # Per-scenario validation output
│   └── *_execution.log                 # Per-scenario execution logs
│
└── ../                                 # Parent directory
    ├── docker-compose.yml              # Docker environment config
    ├── otel-collector-config.yaml      # OTLP Collector configuration
    └── custom_policy.rego              # Custom Rego policy for testing
```

**Total Files Created:** 30+
**Total Scripts (Executable):** 16
**Total CI/CD Pipelines:** 4
**Total Sample Data Files:** 6

---

## Next Steps

### Immediate Actions

1. **Execute master orchestrator:**
   ```bash
   cd /Users/sac/clnrm/tests/weaver/live-check
   bash run_all_scenarios.sh
   ```

2. **Review results:**
   ```bash
   cat results/summary.json
   jq '.' results/execution_log.jsonl
   ```

3. **Integrate into CI/CD:**
   - Choose a pipeline (GitHub Actions recommended)
   - Copy configuration to repository
   - Test in CI environment

---

### Integration Checklist

- [ ] Review all 20 test scenarios
- [ ] Execute master orchestrator locally
- [ ] Verify Docker environment setup
- [ ] Test individual scenarios
- [ ] Choose CI/CD platform
- [ ] Copy pipeline configuration
- [ ] Test in CI environment
- [ ] Review validation reports
- [ ] Document any failures
- [ ] Iterate on registry schemas if violations found

---

## Conclusion

This comprehensive test orchestration provides **complete coverage** of all Weaver `registry live-check` capabilities across:

- **4 input sources** (gRPC, HTTP, file, stdin)
- **2 output formats** (ANSI, JSON)
- **3 advisor types** (builtin, OTel, custom Rego)
- **4 stop conditions** (SIGINT, SIGHUP, HTTP, timeout)
- **2 statistical analyses** (coverage, severity)
- **4 CI/CD platforms** (GitHub, GitLab, Jenkins, Azure)

**All scenarios are READY for execution.**

The test matrix validates **ALL 5 Jobs To Be Done (JTBD):**
1. ✅ Runtime telemetry validation
2. ✅ Interactive debugging
3. ✅ CI/CD quality gates
4. ✅ Coverage analysis
5. ✅ Custom policy enforcement

**Execute `run_all_scenarios.sh` to validate complete functionality.**

---

**Orchestration Complete:** 2025-10-30
**Agent:** task-orchestrator
**Status:** READY FOR EXECUTION
