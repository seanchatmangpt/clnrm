# Weaver Validation Checklist

This checklist must be completed successfully before the Weaver refactor can be considered complete.

## Pre-Validation

- [ ] All schemas defined in `/Users/sac/clnrm/registry/`
- [ ] `weaver registry check -r registry/` passes with zero errors
- [ ] Code generated from schemas (spans, metrics, events helpers)
- [ ] Mock tests pass (interface contracts validated)
- [ ] OTLP exporter configured correctly
- [ ] Weaver listener starts successfully on ports 4317 (gRPC) and 8080 (admin)

## Validation Execution

- [ ] Unit tests run with OTLP export to `localhost:4317`
- [ ] Integration tests run with OTLP export to `localhost:4317`
- [ ] Self-tests run with OTLP export: `clnrm self-test --otel-exporter otlp`
- [ ] Docker integration tests run with OTLP export
- [ ] Weaver receives all telemetry data
- [ ] Validation report generated at `validation_output/validation_report.json`

## Validation Results

- [ ] **Zero violations detected** (CRITICAL)
- [ ] **85%+ registry coverage** (CRITICAL)
- [ ] All required attributes present:
  - [ ] `container.id` (proves container actually ran)
  - [ ] `test.isolated` (proves hermetic isolation)
  - [ ] `test.result` (proves test executed to completion)
  - [ ] `container.destroyed_at` (proves cleanup happened)
- [ ] All span types validated:
  - [ ] `clnrm.container_lifecycle`
  - [ ] `clnrm.test_execution`
  - [ ] `clnrm.plugin.registry`
  - [ ] `clnrm.service.start`
- [ ] All metric types validated (if metrics implemented)
- [ ] All event types validated (if events implemented)
- [ ] Error cases validated (spans with error status)

## Release Criteria

### MUST BE TRUE

- [ ] **Weaver validation passes** (0 violations)
- [ ] **Coverage >= 85%**
- [ ] **No blocking issues**
- [ ] All tests pass
- [ ] Documentation updated

### CANNOT BE TRUE

- [ ] ❌ ANY violations detected
- [ ] ❌ Coverage < 85%
- [ ] ❌ Critical attributes missing
- [ ] ❌ Error telemetry missing
- [ ] ❌ Stub/mock implementations in production code

## Final Verdict

### ✅ RELEASE APPROVED

Conditions:
- All "MUST BE TRUE" items checked
- All "CANNOT BE TRUE" items unchecked
- Weaver validation script exits with code 0
- No manual overrides or exceptions

### ❌ RELEASE BLOCKED

Any of these conditions blocks release:
- Violations > 0
- Coverage < 85%
- Missing critical attributes
- Test failures
- Incomplete telemetry

## Validation Command

Run comprehensive validation:

```bash
./scripts/comprehensive_weaver_validation.sh
```

## Success Criteria

**Script Output:**
```
✅ WEAVER VALIDATION PASSED

All telemetry validated against schemas
Safe to proceed with release

Summary:
- Zero violations detected
- Coverage: 92.3%
- All critical behaviors validated
```

**Exit Code:** `0`

## Failure Response

If validation fails:

1. **Review violations:**
   ```bash
   cat validation_output/validation_report.json | jq '.all_advice[] | select(.advice_level == "violation")'
   ```

2. **Identify missing spans:**
   ```bash
   cat validation_output/validation_report.json | jq '.seen_registry_attributes'
   ```

3. **Fix issues** (common problems):
   - Missing span creation in code
   - Incorrect attribute names
   - Missing OTLP export configuration
   - Tests not exercising code paths

4. **Re-run validation:**
   ```bash
   ./scripts/comprehensive_weaver_validation.sh
   ```

## Validator Authority

The Weaver Validator agent has FINAL AUTHORITY on release readiness.

If the validator says:
- ❌ **NOT VALID** → Release is BLOCKED (no exceptions)
- ✅ **VALID** → Release is APPROVED

**NO MANUAL OVERRIDES ALLOWED**

The validator's decision is based on objective telemetry data, not subjective code review or test results.

## Critical Behaviors That Must Be Proven

These behaviors CANNOT be faked by stub implementations:

1. **Container Creation:**
   - Span: `clnrm.container.start`
   - Attribute: `container.id` with real UUID
   - Event: `container.start` with image and ID

2. **Test Execution:**
   - Span: `clnrm.test`
   - Attribute: `test.isolated = true`
   - Attribute: `test.result` = pass/fail/error
   - Duration > 0ms

3. **Container Cleanup:**
   - Event: `container.stop` with exit code
   - Attribute: `container.destroyed_at` with timestamp
   - Attribute: `cleanup.success = true`

4. **Plugin Lifecycle:**
   - Span: `clnrm.plugin.registry`
   - Span: `clnrm.service.start` with service type
   - Health check events

5. **Error Handling:**
   - Spans with error status
   - Error events with type and message
   - Cleanup even on failure

## Measurement

Success is measured by:

- **Violations:** MUST be 0
- **Coverage:** MUST be >= 85%
- **Critical Attributes:** MUST all be present
- **Span Completeness:** MUST have all expected spans
- **Error Cases:** MUST validate error scenarios

## Documentation

After successful validation, update:

- [ ] `docs/TELEMETRY.md` - Telemetry architecture
- [ ] `docs/VALIDATION.md` - Validation strategy
- [ ] `README.md` - Production telemetry capabilities
- [ ] `CHANGELOG.md` - Weaver integration release notes

## Sign-Off

**Weaver Validator Agent:** ________________

**Date:** ________________

**Verdict:** ✅ APPROVED / ❌ BLOCKED

**Coverage:** ________%

**Violations:** ________

**Notes:**
_________________________________________________________________
_________________________________________________________________
_________________________________________________________________
