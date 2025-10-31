# Weaver Live-Check Test Matrix

## Overview

Comprehensive test matrix for validating all Weaver `registry live-check` capabilities across inputs, outputs, advisors, and stop conditions.

**Test Execution Date:** 2025-10-30
**Total Scenarios:** 20
**Test Environment:** Docker Compose (OTLP Collector + Weaver)

---

## Jobs To Be Done (JTBD)

1. **Validate OTLP telemetry** during test execution (main use case)
2. **Debug telemetry issues** in development (interactive use)
3. **CI/CD quality gates** (automated pass/fail)
4. **Coverage analysis** (track registry usage)
5. **Custom policy enforcement** (org-specific rules)

---

## Phase 1: Input Sources (4 scenarios)

### Scenario 1.1: OTLP gRPC Ingestion
- **Input:** OTLP gRPC on port 4317
- **Command:** `weaver registry live-check --registry registry/ --otlp-grpc localhost:4317`
- **Test:** Send spans via gRPC, verify real-time validation
- **Expected:** Live validation output, schema conformance check
- **Script:** `tests/weaver/live-check/input-sources/test_otlp_grpc.sh`

### Scenario 1.2: OTLP HTTP Ingestion
- **Input:** OTLP HTTP on port 4318
- **Command:** `weaver registry live-check --registry registry/ --otlp-http http://localhost:4318`
- **Test:** Send spans via HTTP, verify real-time validation
- **Expected:** Live validation output, schema conformance check
- **Script:** `tests/weaver/live-check/input-sources/test_otlp_http.sh`

### Scenario 1.3: File Input (JSON Samples)
- **Input:** Pre-recorded JSON telemetry file
- **Command:** `weaver registry live-check --registry registry/ --file samples/spans.json`
- **Test:** Validate against static sample data
- **Expected:** Batch validation results, coverage report
- **Script:** `tests/weaver/live-check/input-sources/test_file_input.sh`

### Scenario 1.4: stdin Streaming
- **Input:** Text attributes piped to stdin
- **Command:** `cat samples/attributes.txt | weaver registry live-check --registry registry/ --stdin`
- **Test:** Stream validation of attribute-only data
- **Expected:** Streaming validation output
- **Script:** `tests/weaver/live-check/input-sources/test_stdin_stream.sh`

---

## Phase 2: Output Formats (2 scenarios)

### Scenario 2.1: ANSI Output (Human-Readable)
- **Output:** ANSI-formatted terminal output with colors
- **Command:** `weaver registry live-check --registry registry/ --output ansi`
- **Test:** Verify colored output, real-time updates
- **Expected:** Color-coded violations, streaming updates
- **Script:** `tests/weaver/live-check/output-formats/test_ansi_output.sh`

### Scenario 2.2: JSON Output (Machine-Readable)
- **Output:** JSON format for CI/CD parsing
- **Command:** `weaver registry live-check --registry registry/ --output json > results.json`
- **Test:** Parse JSON output, extract violation counts
- **Expected:** Valid JSON with structured violation data
- **Script:** `tests/weaver/live-check/output-formats/test_json_output.sh`

---

## Phase 3: Advisors (3 scenarios)

### Scenario 3.1: Builtin Advisors
- **Advisors:** `missing_attribute`, `type_mismatch`
- **Command:** `weaver registry live-check --registry registry/ --advisors builtin`
- **Test:** Trigger missing attribute and type errors
- **Expected:** Violations detected by builtin advisors
- **Script:** `tests/weaver/live-check/advisors/test_builtin_advisors.sh`

### Scenario 3.2: OTel Policies
- **Policies:** Naming conventions, namespace rules
- **Command:** `weaver registry live-check --registry registry/ --policies otel`
- **Test:** Send spans violating OTel naming conventions
- **Expected:** Policy violations reported
- **Script:** `tests/weaver/live-check/advisors/test_otel_policies.sh`

### Scenario 3.3: Custom Rego Policies
- **Policies:** Organization-specific rules (Rego)
- **Command:** `weaver registry live-check --registry registry/ --rego-policy custom.rego`
- **Test:** Validate against custom business rules
- **Expected:** Custom policy violations detected
- **Script:** `tests/weaver/live-check/advisors/test_custom_rego.sh`

---

## Phase 4: Stop Conditions (4 scenarios)

### Scenario 4.1: SIGINT (Ctrl-C)
- **Stop:** User sends SIGINT signal
- **Command:** `weaver registry live-check --registry registry/` (then Ctrl-C)
- **Test:** Send SIGINT after 10 seconds, verify graceful shutdown
- **Expected:** Immediate termination, partial report if available
- **Script:** `tests/weaver/live-check/stop-conditions/test_sigint.sh`

### Scenario 4.2: SIGHUP (Graceful Report)
- **Stop:** SIGHUP signal triggers report generation
- **Command:** `weaver registry live-check --registry registry/` (then SIGHUP)
- **Test:** Send SIGHUP, verify report before shutdown
- **Expected:** Full report generated, graceful termination
- **Script:** `tests/weaver/live-check/stop-conditions/test_sighup.sh`

### Scenario 4.3: HTTP /stop Endpoint
- **Stop:** HTTP request to `/stop` endpoint
- **Command:** `weaver registry live-check --registry registry/ --http-api :8080`
- **Test:** `curl -X POST http://localhost:8080/stop`
- **Expected:** Graceful shutdown via HTTP API
- **Script:** `tests/weaver/live-check/stop-conditions/test_http_stop.sh`

### Scenario 4.4: Inactivity Timeout
- **Stop:** No telemetry received for timeout period
- **Command:** `weaver registry live-check --registry registry/ --timeout 30s`
- **Test:** Wait 30s without sending data
- **Expected:** Auto-shutdown after timeout, final report
- **Script:** `tests/weaver/live-check/stop-conditions/test_inactivity_timeout.sh`

---

## Phase 5: Statistics & Coverage (2 scenarios)

### Scenario 5.1: Registry Coverage Tracking
- **Feature:** Track which schemas were validated
- **Command:** `weaver registry live-check --registry registry/ --coverage-report`
- **Test:** Send spans for subset of schemas, verify coverage metrics
- **Expected:** Coverage percentages, unused schema report
- **Script:** `tests/weaver/live-check/statistics/test_coverage_tracking.sh`

### Scenario 5.2: Violation Severity Analysis
- **Feature:** Categorize violations by severity (error, warning, info)
- **Command:** `weaver registry live-check --registry registry/ --severity-report`
- **Test:** Trigger mix of error/warning/info violations
- **Expected:** Severity breakdown in final report
- **Script:** `tests/weaver/live-check/statistics/test_severity_analysis.sh`

---

## CI/CD Integration Examples

### GitHub Actions Integration
```yaml
- name: Weaver Live Validation
  run: |
    weaver registry live-check \
      --registry registry/ \
      --otlp-http http://localhost:4318 \
      --output json \
      --timeout 60s > results.json

    # Fail if violations detected
    violations=$(jq '.violations | length' results.json)
    if [ "$violations" -gt 0 ]; then exit 1; fi
```

### GitLab CI Integration
```yaml
weaver_validation:
  script:
    - weaver registry live-check --registry registry/ --output json --timeout 30s > results.json
    - test "$(jq '.violations | length' results.json)" -eq 0
  artifacts:
    reports:
      junit: results.json
```

### Jenkins Integration
```groovy
stage('Weaver Validation') {
  steps {
    sh 'weaver registry live-check --registry registry/ --output json > results.json'
    sh 'test $(jq ".violations | length" results.json) -eq 0'
  }
}
```

---

## Test Execution Summary

| Phase | Scenario | Status | Evidence |
|-------|----------|--------|----------|
| 1.1 | OTLP gRPC | PENDING | - |
| 1.2 | OTLP HTTP | PENDING | - |
| 1.3 | File Input | PENDING | - |
| 1.4 | stdin Stream | PENDING | - |
| 2.1 | ANSI Output | PENDING | - |
| 2.2 | JSON Output | PENDING | - |
| 3.1 | Builtin Advisors | PENDING | - |
| 3.2 | OTel Policies | PENDING | - |
| 3.3 | Custom Rego | PENDING | - |
| 4.1 | SIGINT | PENDING | - |
| 4.2 | SIGHUP | PENDING | - |
| 4.3 | HTTP /stop | PENDING | - |
| 4.4 | Inactivity Timeout | PENDING | - |
| 5.1 | Coverage Tracking | PENDING | - |
| 5.2 | Severity Analysis | PENDING | - |

**Total: 0/20 scenarios executed**

---

## Execution Instructions

1. **Setup Docker Environment:**
   ```bash
   cd /Users/sac/clnrm
   docker-compose -f tests/weaver/docker-compose.yml up -d
   ```

2. **Run All Tests:**
   ```bash
   bash tests/weaver/live-check/run_all_scenarios.sh
   ```

3. **Run Individual Phase:**
   ```bash
   bash tests/weaver/live-check/input-sources/run_phase1.sh
   ```

4. **View Results:**
   ```bash
   cat tests/weaver/live-check/results/summary.json
   ```

---

## Success Criteria

- [ ] All 20 scenarios execute without errors
- [ ] OTLP ingestion validated (gRPC + HTTP)
- [ ] Both output formats produce valid results
- [ ] All advisor types detect violations
- [ ] All stop conditions work as expected
- [ ] Coverage and statistics accurately reported
- [ ] CI/CD examples work in test pipelines

---

**Next Steps:** Execute test scenarios and collect evidence.
