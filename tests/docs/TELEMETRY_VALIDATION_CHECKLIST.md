# Telemetry Export Validation Checklist

## Pre-Validation Setup

- [ ] Docker installed and running
- [ ] Weaver CLI installed (`cargo install weaver`)
- [ ] OTLP collector available
- [ ] Registry directory with semantic conventions

## Critical Export Validation

### 1. Exporter Initialization
- [ ] OTLP gRPC exporter initializes
- [ ] OTLP HTTP exporter initializes
- [ ] Configuration validated
- [ ] Invalid configs rejected gracefully

### 2. Span Export
- [ ] `test_execution` spans export
- [ ] `container_lifecycle` spans export
- [ ] `plugin_execution` spans export
- [ ] Spans reach OTLP collector
- [ ] Export latency < 100ms

### 3. Required Attributes
- [ ] `container.id` present in all container spans
- [ ] `test.isolated` present in test spans
- [ ] `test.result` present with correct value
- [ ] `plugin.name` present in plugin spans
- [ ] `error.message` present on error spans
- [ ] `error.type` present on error spans

### 4. Error Telemetry
- [ ] Error spans have status = Error
- [ ] Error attributes exported
- [ ] Stack traces captured (if available)
- [ ] Error spans don't block other exports

### 5. Metrics Export
- [ ] `clnrm.test.duration` exports
- [ ] `clnrm.test.counter` exports
- [ ] `clnrm.container.count` exports
- [ ] Metric values accurate
- [ ] Metric aggregation correct

### 6. Edge Cases
- [ ] Special characters handled (unicode, quotes)
- [ ] Long attribute values (10K+ chars)
- [ ] Null bytes rejected/sanitized
- [ ] Network interruption handled
- [ ] Collector unavailable handled
- [ ] Buffer overflow handled gracefully
- [ ] 100+ concurrent exports work
- [ ] 1000+ rapid spans handled
- [ ] No deadlocks under load
- [ ] Export after shutdown handled

### 7. Context Propagation
- [ ] Trace ID propagated correctly
- [ ] Parent-child relationships maintained
- [ ] Baggage propagated
- [ ] Span hierarchy preserved

### 8. Weaver Validation
- [ ] Weaver receives all telemetry
- [ ] Zero semantic convention violations
- [ ] All required attributes detected
- [ ] No missing span types

## Validation Commands

```bash
# Run OTLP export tests
cargo test --test otlp_export --features otel

# Run edge case tests
cargo test --test export_edge_cases --features otel

# Run Weaver integration tests
cargo test --test weaver_integration --features otel --ignored

# Full validation suite
./tests/scripts/validate_otlp_export.sh
```

## Expected Results

### Test Pass Rate
- **Target**: 100% pass rate
- **Critical**: All required attribute tests must pass
- **Acceptable**: Some edge case tests may be skipped if optional

### Weaver Validation
- **Target**: Zero violations
- **Critical**: No missing required attributes
- **Critical**: All semantic conventions followed

### Performance
- **Target**: Export latency < 100ms
- **Target**: No deadlocks under load
- **Target**: 1000+ spans/second throughput

## Failure Investigation

### If Tests Fail

1. **Check OTLP collector logs**:
   ```bash
   docker logs otel-collector-test
   ```

2. **Check Weaver validation report**:
   ```bash
   cat validation_report.json | jq '.live_check_result'
   ```

3. **Run with debug logging**:
   ```bash
   RUST_LOG=debug cargo test --test otlp_export
   ```

4. **Verify network connectivity**:
   ```bash
   curl http://localhost:4317
   ```

### Common Issues

**Issue**: Exporter fails to initialize
- **Cause**: Invalid endpoint or network issue
- **Fix**: Check OTEL_EXPORTER_OTLP_ENDPOINT env var

**Issue**: Spans not exported
- **Cause**: Export timeout or buffer full
- **Fix**: Increase timeout, check collector health

**Issue**: Attributes missing
- **Cause**: Not set in code or filtered
- **Fix**: Verify attribute setting in span creation

**Issue**: Weaver violations
- **Cause**: Semantic convention not followed
- **Fix**: Review convention requirements, update attributes

## Sign-Off

### Validation Complete

- [ ] All critical tests pass
- [ ] Weaver validation passes
- [ ] Performance targets met
- [ ] Edge cases handled
- [ ] Documentation updated

**Validated by**: _________________

**Date**: _________________

**Commit**: _________________

## Notes

Add any observations or issues encountered during validation:

---

---

---
