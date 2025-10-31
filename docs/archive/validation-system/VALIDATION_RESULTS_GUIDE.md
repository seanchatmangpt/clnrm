# Weaver Validation Results - Quick Reference Guide

## How to Read Validation Output

### Success Output
```bash
✅ WEAVER VALIDATION PASSED

All telemetry validated against schemas
Safe to proceed with release

Summary:
- Zero violations detected
- Coverage: 92.3%
- All critical behaviors validated
```

**Exit Code:** `0`
**Meaning:** Release APPROVED
**Action:** Proceed with deployment

---

### Failure Output
```bash
❌ VALIDATION FAILED - VIOLATIONS DETECTED

Details:
  - [span] clnrm.test_execution: Missing required attribute: container.id
  - [span] clnrm.container_lifecycle: Missing required attribute: container.destroyed_at

🚫 RELEASE BLOCKED

Action required:
1. Fix all violations
2. Improve coverage to 85%+
3. Re-run validation
```

**Exit Code:** `1`
**Meaning:** Release BLOCKED
**Action:** Fix violations before proceeding

---

## Understanding the Validation Report

### Report Location
```bash
/Users/sac/clnrm/validation_output/validation_report.json
```

### Report Structure

```json
{
  "advice_level_counts": {
    "violation": 0,       // MUST be 0 for release
    "improvement": 5,     // Suggestions, not blocking
    "information": 12     // FYI only
  },
  "registry_coverage": 0.92,  // MUST be >= 0.85
  "all_advice": [
    {
      "advice_level": "violation",
      "advice_type": "missing_required_attribute",
      "message": "Missing required attribute: container.id",
      "signal_name": "clnrm.test_execution",
      "signal_type": "span"
    }
  ],
  "seen_registry_attributes": {
    "container.id": 10,         // Seen 10 times
    "test.isolated": 8,         // Seen 8 times
    "test.result": 8,
    "container.destroyed_at": 10
  },
  "seen_non_registry_attributes": {
    "custom.attribute": 5       // Not in schema
  }
}
```

---

## Interpreting Metrics

### Advice Level Counts

| Level | Meaning | Blocks Release? | Action Required |
|-------|---------|-----------------|-----------------|
| `violation` | Schema requirement not met | ✅ YES | MUST fix before release |
| `improvement` | Suggestion for better telemetry | ❌ NO | Should fix in future |
| `information` | FYI message | ❌ NO | No action needed |

### Registry Coverage

| Coverage | Status | Meaning | Action |
|----------|--------|---------|--------|
| >= 90% | ✅ Excellent | All spans well-validated | Proceed with release |
| 85-89% | ⚠️ Good | Minimum met | Proceed, aim higher next time |
| < 85% | ❌ Insufficient | Too many spans missing | Block release, add telemetry |

### Critical Attributes

These MUST all be present in `seen_registry_attributes`:

| Attribute | Proves | Missing Means |
|-----------|--------|---------------|
| `container.id` | Container actually created | Tests using mocks, not real containers |
| `test.isolated` | Hermetic isolation working | Tests may not be isolated |
| `test.result` | Test executed to completion | Tests may not be finishing |
| `container.destroyed_at` | Cleanup happened | Resource leaks possible |

---

## Common Violations and Fixes

### 1. Missing Required Attribute

**Violation:**
```
Missing required attribute: container.id
Signal: clnrm.test_execution
```

**Cause:** Span created without setting required attribute

**Fix:**
```rust
let span = span!(
    Level::INFO,
    "clnrm.test",
    test.name = "my_test",
    container.id = %container_id,  // ← Add this
);
```

### 2. Attribute Type Mismatch

**Violation:**
```
Attribute 'test.duration_ms' expected type 'double', got 'int'
```

**Fix:**
```rust
// Wrong
span.record("test.duration_ms", 125);

// Right
span.record("test.duration_ms", 125.0);  // ← Use f64
```

### 3. Low Coverage

**Violation:**
```
Coverage: 60.0%
Target: 85%+
```

**Cause:** Many spans defined in schemas but not emitted by code

**Fix:**
1. Review schemas: `ls registry/core/`
2. Find missing span types
3. Add span creation to code:
   ```rust
   let _span = span!(Level::INFO, "clnrm.container.start", ...);
   ```

### 4. No Telemetry Received

**Violation:**
```
No validation report generated
Weaver may not have received any telemetry
```

**Causes:**
- OTLP export not configured
- Tests not creating spans
- Weaver listener not running

**Fix:**
```bash
# Check OTLP configuration
echo $OTEL_EXPORTER_OTLP_ENDPOINT  # Should be: http://localhost:4317

# Verify Weaver is running
lsof -i :4317

# Check test output for spans
cargo test -- --nocapture 2>&1 | grep "clnrm\."
```

---

## Querying the Validation Report

### Get All Violations
```bash
cat validation_output/validation_report.json | jq '.all_advice[] | select(.advice_level == "violation")'
```

### Get Coverage Percentage
```bash
cat validation_output/validation_report.json | jq '.registry_coverage * 100'
```

### Get Seen Attributes
```bash
cat validation_output/validation_report.json | jq '.seen_registry_attributes'
```

### Get Missing Critical Attributes
```bash
for attr in container.id test.isolated test.result container.destroyed_at; do
  count=$(cat validation_output/validation_report.json | jq ".seen_registry_attributes.\"$attr\" // 0")
  if [ "$count" -eq 0 ]; then
    echo "MISSING: $attr"
  fi
done
```

### Get Non-Registry Attributes
```bash
cat validation_output/validation_report.json | jq '.seen_non_registry_attributes'
```

---

## Decision Matrix

### Should Release Be Approved?

```
┌────────────────────────────────────────────────────────────┐
│                   RELEASE DECISION MATRIX                   │
├────────────────────────────────────────────────────────────┤
│                                                             │
│  Violations = 0?                                           │
│  ├─ NO  → ❌ BLOCK RELEASE                                 │
│  └─ YES → Continue                                         │
│                                                             │
│  Coverage >= 85%?                                          │
│  ├─ NO  → ❌ BLOCK RELEASE                                 │
│  └─ YES → Continue                                         │
│                                                             │
│  All critical attributes present?                          │
│  ├─ NO  → ❌ BLOCK RELEASE                                 │
│  └─ YES → ✅ APPROVE RELEASE                               │
│                                                             │
└────────────────────────────────────────────────────────────┘
```

### Critical Attributes Checklist

```bash
# Run this to verify critical attributes
cat validation_output/validation_report.json | jq -r '
  .seen_registry_attributes |
  {
    "container.id": (.["container.id"] // 0),
    "test.isolated": (.["test.isolated"] // 0),
    "test.result": (.["test.result"] // 0),
    "container.destroyed_at": (.["container.destroyed_at"] // 0)
  } |
  to_entries[] |
  if .value > 0 then
    "✅ \(.key): \(.value) occurrences"
  else
    "❌ \(.key): MISSING"
  end
'
```

---

## Validation Logs

### Where to Find Logs

```
validation_output/
├── validation_report.json    # Main report
├── unit_tests.log           # Unit test output
├── integration_tests.log    # Integration test output
└── self_tests.log           # Self-test output
```

### Useful Log Queries

**Find all span names:**
```bash
grep "clnrm\." validation_output/*.log | grep -o "clnrm\.[^ ]*" | sort -u
```

**Count spans per type:**
```bash
grep "clnrm\." validation_output/*.log | grep -o "clnrm\.[^ ]*" | sort | uniq -c
```

**Find spans with errors:**
```bash
grep -i "error" validation_output/*.log | grep "clnrm\."
```

---

## Troubleshooting

### Problem: Validation script fails to start

**Symptoms:**
```
❌ Weaver failed to start
```

**Solution:**
```bash
# Check if ports are in use
lsof -i :4317
lsof -i :8080

# Kill existing processes
lsof -ti :4317 | xargs kill -9
lsof -ti :8080 | xargs kill -9

# Re-run validation
./scripts/comprehensive_weaver_validation.sh
```

### Problem: No telemetry received

**Symptoms:**
```
No validation report generated
```

**Solution:**
```bash
# Verify OTLP export in tests
export OTEL_EXPORTER_OTLP_ENDPOINT="http://localhost:4317"
export OTEL_EXPORTER_OTLP_PROTOCOL="grpc"

# Check if tests create spans
cargo test -- --nocapture 2>&1 | grep -i "span"

# Verify Weaver is listening
curl http://localhost:8080/health
```

### Problem: Coverage too low

**Symptoms:**
```
Coverage: 45.0%
Target: 85%+
```

**Solution:**
```bash
# Find schemas that aren't matched
cat validation_output/validation_report.json | jq '.all_advice[] | select(.advice_type == "unused_schema")'

# Identify which spans to add
ls registry/core/*.yaml

# Add missing span creation to code
```

---

## Example: Full Validation Flow

### 1. Run Validation
```bash
$ ./scripts/comprehensive_weaver_validation.sh

🚀 Starting Comprehensive Weaver Validation
==========================================

✅ Schemas valid
✅ Weaver started (PID: 12345)
✅ Unit tests passed
✅ Integration tests passed
✅ Self-tests passed
```

### 2. Check Results
```bash
$ cat validation_output/validation_report.json | jq '.advice_level_counts'
{
  "violation": 0,
  "improvement": 3,
  "information": 8
}

$ cat validation_output/validation_report.json | jq '.registry_coverage'
0.923
```

### 3. Interpret
- Violations: 0 ✅
- Coverage: 92.3% ✅
- Above 85% threshold ✅

### 4. Decision
```
✅ WEAVER VALIDATION PASSED
Safe to proceed with release
```

---

## Quick Reference Card

### Release Criteria (All MUST be true)
- [ ] Violations = 0
- [ ] Coverage >= 85%
- [ ] `container.id` present
- [ ] `test.isolated` present
- [ ] `test.result` present
- [ ] `container.destroyed_at` present

### Commands
```bash
# Run validation
./scripts/comprehensive_weaver_validation.sh

# Check violations
jq '.advice_level_counts.violation' validation_output/validation_report.json

# Check coverage
jq '.registry_coverage * 100' validation_output/validation_report.json

# List violations
jq '.all_advice[] | select(.advice_level == "violation")' validation_output/validation_report.json
```

### Decision
```
violations == 0 && coverage >= 0.85 && critical_attributes_present
  → ✅ APPROVE
else
  → ❌ BLOCK
```

---

**Remember: The validation results are OBJECTIVE. There are no subjective overrides.**
