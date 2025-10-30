# OpenTelemetry Validation Test Suite

This directory contains comprehensive validation tests for clnrm's OpenTelemetry integration. These tests validate that OTEL instrumentation is working correctly and **not generating fake/simulated data**.

## Test Suite Overview

### 1. `test_span_generation.clnrm.toml`
**Focus**: Validates that OTEL spans are generated correctly during clnrm execution.

**Key Validations**:
- Spans are created for all major operations (run, test, step, container.exec)
- Span attributes match expected values
- Span count matches command execution count
- Performance overhead is acceptable (< 100ms)

**Fake-Green Detection**:
- Ensures span count matches executed commands
- Validates span timing is realistic
- Detects spans without corresponding execution

### 2. `test_trace_validation.clnrm.toml`
**Focus**: Validates trace structure and parent-child span relationships.

**Key Validations**:
- All spans share the same trace ID
- Parent-child relationships are correct
- Span hierarchy matches execution flow
- No orphaned spans exist

**Fake-Green Detection**:
- Validates parent span IDs are valid
- Ensures child spans execute within parent timeframe
- Detects invalid trace structures

### 3. `test_otlp_export.clnrm.toml`
**Focus**: Validates OTLP export functionality.

**Key Validations**:
- Spans export to stdout NDJSON correctly
- Export format is valid JSON
- All required fields are present (trace_id, span_id, timestamps, etc.)
- Resource attributes are correct

**Fake-Green Detection**:
- Ensures exported span count matches generated spans
- Validates no duplicate span IDs in exports
- Detects invalid trace/span IDs
- Ensures timestamps are monotonic

### 4. `test_fake_green_detection.clnrm.toml`
**Focus**: **Critical test** that specifically detects fake/simulated OTEL data.

**Key Validations**:
- Span count exactly matches command count (5 commands = 5 container.exec spans)
- Span timing is realistic (> 1μs, < 10s)
- All required attributes present and accurate
- No duplicate span IDs
- Timestamps are monotonic and properly ordered
- Child spans execute within parent span timeframe

**Fake-Green Detection Rules**:
1. **Exact Count Matching**: 5 commands MUST produce exactly 5 container.exec spans
2. **Realistic Timing**: All spans must have durations > 1μs and < 10s
3. **Valid Attributes**: All spans must have required attributes (command, container.id, etc.)
4. **No Duplicates**: Every span_id must be unique
5. **Monotonic Timestamps**: end_time > start_time for all spans
6. **Temporal Hierarchy**: Child spans must start after parent starts and end before parent ends
7. **Execution Matching**: Every command execution MUST have corresponding span
8. **No Fake Spans**: No spans without actual execution

**This test MUST fail if:**
- Spans are simulated/hardcoded
- Span count doesn't match execution
- Timing is unrealistic (e.g., 0ms duration with 100ms sleep)
- Attributes don't match actual commands

### 5. `test_span_timing.clnrm.toml`
**Focus**: Validates span timing accuracy with known sleep durations.

**Key Validations**:
- Instant commands complete in < 10ms
- 50ms sleep results in ~50ms span duration (±30%)
- 100ms sleep results in ~100ms span duration (±30%)
- 200ms sleep results in ~200ms span duration (±30%)
- No zero-duration spans
- No negative durations
- Spans ordered by start time

**Fake-Green Detection**:
- If span claims 0ms but sleep 200ms → **FAIL** (fake data)
- Timing variance beyond ±30% → **FAIL** (unrealistic)
- Zero/negative durations → **FAIL** (simulated)

### 6. `test_end_to_end.clnrm.toml`
**Focus**: Comprehensive end-to-end validation combining all checks.

**Key Validations**:
- Multi-service scenario (2 services, 8 steps)
- Full span generation validation
- Complete trace structure validation
- Export validation
- Performance validation
- Comprehensive fake-green detection

**This is the "master validation"** that proves clnrm's OTEL integration is production-ready.

## Running the Tests

### Run entire validation suite:
```bash
# Using Homebrew-installed clnrm (REQUIRED for validation)
clnrm run tests/otel_validation/

# Run with OTEL stdout export for inspection
clnrm run tests/otel_validation/ --otel-exporter stdout
```

### Run individual tests:
```bash
# Span generation validation
clnrm run tests/otel_validation/test_span_generation.clnrm.toml

# Trace validation
clnrm run tests/otel_validation/test_trace_validation.clnrm.toml

# OTLP export validation
clnrm run tests/otel_validation/test_otlp_export.clnrm.toml

# Fake-green detection (CRITICAL)
clnrm run tests/otel_validation/test_fake_green_detection.clnrm.toml

# Span timing validation
clnrm run tests/otel_validation/test_span_timing.clnrm.toml

# End-to-end validation
clnrm run tests/otel_validation/test_end_to_end.clnrm.toml
```

### Run with detailed OTEL output:
```bash
# Export spans to stdout NDJSON for inspection
RUST_LOG=debug clnrm run tests/otel_validation/test_fake_green_detection.clnrm.toml --otel-exporter stdout-ndjson

# Export spans to OTLP collector (requires collector running)
clnrm run tests/otel_validation/ --otel-exporter http://localhost:4318
```

## Validation Criteria

### PASS Criteria:
✅ All expected spans present
✅ Span count matches command count
✅ Span timing is realistic
✅ Trace structure correct with valid parent-child relationships
✅ All required attributes present and accurate
✅ Exports valid NDJSON with required fields
✅ Performance overhead < configured limit
✅ No duplicate span IDs
✅ Timestamps monotonic and properly ordered

### FAIL Criteria (Fake-Green Detection):
❌ Span count doesn't match command count
❌ Zero-duration or negative-duration spans
❌ Unrealistic timing (e.g., 0ms for 200ms sleep)
❌ Missing required attributes
❌ Duplicate span IDs
❌ Non-monotonic timestamps
❌ Child spans outside parent timeframe
❌ Spans without corresponding execution
❌ Invalid trace/span IDs

## Integration with CI/CD

These tests are designed for CI/CD integration:

```yaml
# Example GitHub Actions workflow
- name: OTEL Validation Tests
  run: |
    clnrm run tests/otel_validation/ --format junit > otel-validation.xml

- name: Upload OTEL Validation Results
  uses: actions/upload-artifact@v3
  with:
    name: otel-validation-results
    path: otel-validation.xml
```

## 80/20 Approach

This validation suite follows the 80/20 principle:

**20% of validation effort covers 80% of OTEL issues**:
1. Span generation (test_span_generation.clnrm.toml)
2. Fake-green detection (test_fake_green_detection.clnrm.toml)
3. End-to-end validation (test_end_to_end.clnrm.toml)

**Full coverage** includes trace structure, export, and timing validation.

## Findings and Analysis

### Current Implementation Status (as of analysis):

**✅ Production-Ready Components**:
- `telemetry.rs`: Full OTEL initialization with traces, metrics, logs
- `spans` module: Comprehensive span creation helpers
- `events` module: Span event recording
- `metrics` module: Helper functions for metrics
- Export configuration (OTLP HTTP/gRPC, stdout, stdout NDJSON)

**⚠️ Validation Functions with Simulated Data**:
Located in `telemetry.rs::validation` module (lines 243-292):

1. `is_otel_initialized()`: Returns hardcoded `true`
2. `span_exists()`: Returns simulated `Ok(true)` without checking real spans
3. `capture_test_spans()`: Returns simulated `Ok(3)` without capturing real spans

**Impact**: These functions were placeholders and have been superseded by the real validation infrastructure in `validation/otel/`.

**✅ Real Validation Infrastructure** (Production-Ready):
- `validation/otel/validator.rs`: Full validator with real span data
- `validation/otel/span_processor.rs`: Real span collection
- Methods: `validate_span_real()`, `validate_trace_real()`, `validate_export_real()`

**Recommendation**:
1. The simulated validation functions in `telemetry.rs::validation` are legacy and should be removed or updated to use the real validation infrastructure.
2. All tests should use `OtelValidator` with `ValidationSpanProcessor` for real span validation.
3. The tests in this suite validate the **production-ready** validation infrastructure.

## Next Steps

1. Run validation suite against current clnrm installation
2. Identify any failing tests
3. Fix issues in OTEL implementation
4. Ensure 100% pass rate
5. Integrate into CI/CD pipeline
