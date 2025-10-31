# Live-Check Test Suite

Comprehensive test harness for OpenTelemetry Weaver `registry live-check` integration in clnrm.

## Overview

This test suite validates ALL live-check capabilities to ensure the clnrm Weaver integration works correctly. Since clnrm eliminates false positives, we validate the framework using Weaver's schema validation as the source of truth.

## Test Categories

### 1. Basic Tests (Fast - ~30s)
- **01_file_json**: File input with JSON format
- **02_stdin_text**: stdin input with text format
- **03_json_output**: JSON output format validation
- **04_ansi_output**: ANSI output format validation

### 2. Advanced Tests (Medium - ~60s)
- **05_inactivity_timeout**: Validates timeout behavior
- **06_sighup_stop**: SIGHUP graceful shutdown
- **07_custom_policies**: Custom OPA policy validation
- **08_statistics**: Statistics generation verification

### 3. Concurrent Tests (Slow - ~90s)
- **09_concurrent_instances**: Multiple simultaneous live-check processes
- **10_otlp_grpc**: OTLP gRPC input with real telemetry

## Quick Start

### Run Full Suite
```bash
cd /Users/sac/clnrm
./scripts/tests/test_live_check_comprehensive.sh
```

### Run Test Subsets
```bash
# Fast feedback loop (basic tests only)
./scripts/tests/run_test_subset.sh --basic

# Quick smoke test (fastest)
./scripts/tests/run_test_subset.sh --quick

# Advanced scenarios
./scripts/tests/run_test_subset.sh --advanced

# Concurrent and OTLP
./scripts/tests/run_test_subset.sh --concurrent

# List all tests
./scripts/tests/run_test_subset.sh --list
```

## Prerequisites

### Required
- **OpenTelemetry Weaver** (`weaver` command)
  ```bash
  # Install from: https://github.com/open-telemetry/weaver
  cargo install weaver-forge
  ```

- **Registry directory**: `/Users/sac/clnrm/registry/`
  - Must contain valid OTel schema definitions
  - Validated by `weaver registry check`

### Optional (for OTLP test)
- **Rust & Cargo**: For running clnrm telemetry tests
- **clnrm-core**: Built with `--features otel`

## Output Structure

All test outputs are written to: `/Users/sac/clnrm/validation_output/live_check_tests/`

```
validation_output/live_check_tests/
├── test_run_20250130_143022.log       # Full test run log
├── 01_file_json.log                   # Individual test logs
├── 02_stdin_text.log
├── ...
├── file_json/                         # Weaver output directories
│   └── live_check_report_*.json
├── stdin_text/
├── ...
├── test_summary.json                  # Structured summary
└── sample.json                        # Test data files
```

## Test Details

### 01_file_json
**Purpose**: Validate file input with JSON format
**What it tests**:
- Reading telemetry attributes from JSON file
- Validating against registry schemas
- Generating JSON output report

**Success criteria**:
- Output file generated
- Valid JSON structure
- Statistics present

### 02_stdin_text
**Purpose**: Validate stdin input with text format
**What it tests**:
- Reading attribute names from stdin (newline-separated)
- Processing text format input
- Generating report from text input

**Success criteria**:
- Accepts stdin input
- Processes text format correctly
- Generates valid output

### 03_json_output
**Purpose**: Validate JSON output format structure
**What it tests**:
- Complete JSON report generation
- Required statistics fields
- Proper JSON structure

**Success criteria**:
- Valid JSON syntax
- `statistics` object present
- `total_entities` field exists
- `registry_coverage` field exists

### 04_ansi_output
**Purpose**: Validate ANSI formatted output
**What it tests**:
- ANSI escape code generation
- Human-readable colored output
- Terminal-friendly formatting

**Success criteria**:
- Output contains ANSI escape sequences
- Readable format generated

### 05_inactivity_timeout
**Purpose**: Validate inactivity timeout behavior
**What it tests**:
- Process exits after N seconds of no input
- Timeout configuration works
- Graceful shutdown on timeout

**Success criteria**:
- Process exits after ~5 seconds (±1s tolerance)
- No hanging processes

### 06_sighup_stop
**Purpose**: Validate SIGHUP graceful shutdown
**What it tests**:
- SIGHUP signal handling
- Report generation before exit
- Graceful cleanup

**Success criteria**:
- Process responds to SIGHUP
- Exits within 10 seconds
- Report file generated

### 07_custom_policies
**Purpose**: Validate custom OPA policy loading
**What it tests**:
- Loading custom .rego files
- Policy evaluation
- Violation detection and reporting

**Success criteria**:
- Custom policy loaded
- Violations detected correctly
- Statistics reflect violations

### 08_statistics
**Purpose**: Validate statistics generation
**What it tests**:
- Complete statistics calculation
- Required statistics fields
- Accurate data collection

**Success criteria**:
- All required fields present:
  - `total_entities`
  - `registry_coverage`
  - `advice_level_counts`

### 09_concurrent_instances
**Purpose**: Validate multiple simultaneous instances
**What it tests**:
- No port conflicts
- Independent operation
- Proper timeout behavior for all instances

**Success criteria**:
- All 3 instances start successfully
- All run concurrently
- All stop after timeout

### 10_otlp_grpc
**Purpose**: Validate OTLP gRPC input (end-to-end)
**What it tests**:
- OTLP gRPC listener
- Real telemetry ingestion
- Integration with clnrm telemetry

**Success criteria**:
- gRPC listener starts
- Accepts OTLP telemetry
- Generates report from real data

**Note**: May be skipped if:
- Port 4320 in use
- Cargo not available
- clnrm-core not built with otel features

## Understanding Test Results

### Success (Exit 0)
All tests passed. Live-check capabilities validated.

### Failure (Exit 1)
One or more tests failed. Check individual test logs:
```bash
cat /Users/sac/clnrm/validation_output/live_check_tests/FAILED_TEST_NAME.log
```

### Test Summary JSON
Structured summary for CI/CD integration:
```json
{
  "timestamp": "2025-01-30T14:30:22Z",
  "duration_seconds": 125,
  "total_tests": 10,
  "passed": 9,
  "failed": 1,
  "skipped": 0,
  "passed_tests": ["01_file_json", ...],
  "failed_tests": ["10_otlp_grpc"],
  "skipped_tests": []
}
```

## Integration with clnrm Validation

This test suite is part of the clnrm v1.2.0 Weaver integration validation hierarchy:

### Level 1: Schema Definition (Highest Authority)
```bash
weaver registry check -r registry/
```

### Level 2: Live-Check Capabilities (This Test Suite)
```bash
./scripts/tests/test_live_check_comprehensive.sh
```

### Level 3: Runtime Telemetry Validation
```bash
weaver registry live-check --registry registry/ --otlp-grpc-port 4317
# Run clnrm tests that generate telemetry
```

### Level 4: Traditional Tests (Supporting Evidence)
```bash
cargo test --features otel
clnrm self-test --suite otel
```

## Continuous Integration

### GitHub Actions Example
```yaml
- name: Test Weaver Live-Check
  run: |
    # Install weaver
    cargo install weaver-forge

    # Run test suite
    ./scripts/tests/test_live_check_comprehensive.sh

    # Check results
    jq '.failed' validation_output/live_check_tests/test_summary.json | \
      grep -q '^0$' || exit 1
```

## Troubleshooting

### "weaver command not found"
Install OpenTelemetry Weaver:
```bash
cargo install weaver-forge
# Or follow: https://github.com/open-telemetry/weaver
```

### "Registry directory not found"
Ensure you're running from clnrm root:
```bash
cd /Users/sac/clnrm
ls registry/  # Should show schema files
```

### Port conflicts (OTLP test)
Stop other OTLP collectors:
```bash
lsof -i :4320
kill <PID>
```

Or skip OTLP test:
```bash
./scripts/tests/run_test_subset.sh --basic
```

### Timeout tests hanging
Check for background processes:
```bash
ps aux | grep weaver
pkill -f "weaver registry live-check"
```

## Development

### Adding New Tests

1. Add test function to `test_live_check_comprehensive.sh`:
```bash
test_new_feature() {
    log "Testing new feature..."
    # Test implementation
    return 0
}
```

2. Add to main execution:
```bash
run_test "11_new_feature" test_new_feature
```

3. Update this README with test documentation

### Test Best Practices

- **Independent**: Each test should be completely independent
- **Idempotent**: Tests should clean up after themselves
- **Descriptive**: Use clear logging with `log()`, `log_error()`, `log_success()`
- **Verifiable**: Check actual outputs, don't assume success
- **Fast**: Keep individual tests under 30 seconds when possible

## Related Documentation

- [Weaver Integration Design](/Users/sac/clnrm/docs/architecture/WEAVER_INTEGRATION_DESIGN.md)
- [Weaver V1.2.0 Validation Summary](/Users/sac/clnrm/docs/WEAVER_V1_2_0_VALIDATION_SUMMARY.md)
- [Running Weaver Validation](/Users/sac/clnrm/docs/RUNNING_WEAVER_VALIDATION.md)
- [Weaver User Guide](/Users/sac/clnrm/docs/WEAVER_USER_GUIDE.md)

## Support

For issues or questions:
1. Check individual test logs in `validation_output/live_check_tests/`
2. Review Weaver documentation: https://github.com/open-telemetry/weaver
3. File issue: https://github.com/seanchatmangpt/clnrm/issues
