# Weaver CI/CD Workflow Guide

**Quick Reference for Using the Weaver Refactor Validation Workflow**

## Overview

The `weaver-refactor-validation.yml` workflow validates that clnrm's telemetry matches Weaver schema definitions. This is the **single source of truth** for proving features work (no false positives).

## When It Runs

### Automatic Triggers

```yaml
# Runs on push to main branches
git push origin master
git push origin develop

# Runs on pull requests
gh pr create --base master
```

### Manual Trigger

```bash
# GitHub UI: Actions → Weaver Refactor Validation → Run workflow

# Or using gh CLI:
gh workflow run weaver-refactor-validation.yml \
  -f test_suite=all \
  -f fail_on_violations=true
```

## Workflow Steps

### Step 1: Schema Validation (30s)

**What it does:**
- Validates `registry/` schema structure
- Uses `weaver registry check`

**Success criteria:**
- All schemas are valid
- No syntax errors

**On failure:**
- ❌ Schema validation FAILED
- Review schema files in `registry/`

### Step 2: Build with OTEL (2-3 min)

**What it does:**
- Builds clnrm with `--features otel`
- Verifies compilation

**Success criteria:**
- `cargo build --release --features otel` succeeds
- Binary created

**On failure:**
- ❌ Build failed
- Check Rust compilation errors

### Step 3: Live Telemetry Validation (2-5 min)

**What it does:**
- Starts OTLP collector (Docker service)
- Launches Weaver live-check listener
- Runs tests with OTLP export
- Validates runtime telemetry

**Success criteria:**
- ✅ `sample_count > 0` (telemetry captured)
- ✅ `violations == 0` (schema conformance)

**On failure:**

**Case 1: sample_count == 0**
```
❌ CRITICAL: No telemetry captured
```
**Causes:**
- OTEL features not enabled
- Tests don't exercise instrumented code
- OTLP export configuration incorrect

**Fix:**
1. Verify tests run with `--features otel`
2. Check `OTEL_EXPORTER_OTLP_ENDPOINT` is set
3. Add OTEL instrumentation to tested code

**Case 2: violations > 0**
```
❌ CRITICAL: Schema violations detected
```
**Causes:**
- Telemetry attributes don't match schema
- Missing required attributes
- Type mismatches

**Fix:**
1. Download `weaver-validation-results` artifact
2. Review `validation_output/live_check.json`
3. Check violation details
4. Update schema or telemetry code

### Step 4: PR Comment (< 1 min)

**What it does:**
- Posts/updates PR comment with results
- Shows sample count and violations
- Provides actionable feedback

**Example (Success):**
```markdown
## ✅ Weaver Refactor Validation
**Status:** PASSED

### Validation Results:
- Sample Count: 42 ✅
- Violations: 0 ✅

✅ Safe to Merge
```

**Example (Failure):**
```markdown
## ❌ Weaver Refactor Validation
**Status:** FAILED - Schema Violations

### Validation Results:
- Sample Count: 42 ✅
- Violations: 3 ❌

❌ Not Safe to Merge
Review validation report for details.
```

### Step 5: Deployment Gate (< 1 min)

**What it does:**
- Evaluates validation results
- Blocks merge if violations

**Gate Logic:**
```bash
PASS = (sample_count > 0) AND (violations == 0)
```

**On success:**
```
✅ DEPLOYMENT GATE: PASSED
Safe to merge and deploy.
```

**On failure:**
```
❌ DEPLOYMENT GATE: FAILED
Fix issues before merging.
```

## Interpreting Results

### ✅ All Green (Safe to Merge)

```
- ✅ Schema Validation: PASSED
- ✅ Build with OTEL: PASSED
- ✅ Live Validation: PASSED (42 samples, 0 violations)
- ✅ Deployment Gate: PASSED
```

**What this proves:**
- Schema is valid
- Code compiles with OTEL
- Tests emit telemetry (42 samples)
- Telemetry matches schema (0 violations)
- **No false positives** (runtime validation passed)

### ❌ Red: sample_count == 0

```
- ✅ Schema Validation: PASSED
- ✅ Build with OTEL: PASSED
- ❌ Live Validation: FAILED (0 samples, 0 violations)
- ❌ Deployment Gate: FAILED
```

**What this means:**
- Tests ran but didn't emit telemetry
- Cannot prove features work
- **High risk of false positives**

**Action:**
1. Check OTEL initialization in tests
2. Verify `--features otel` is enabled
3. Add instrumentation to tested code paths

### ❌ Red: violations > 0

```
- ✅ Schema Validation: PASSED
- ✅ Build with OTEL: PASSED
- ❌ Live Validation: FAILED (42 samples, 3 violations)
- ❌ Deployment Gate: FAILED
```

**What this means:**
- Tests emit telemetry (42 samples)
- Telemetry doesn't match schema (3 violations)
- Schema and runtime behavior diverged

**Action:**
1. Download validation report artifact
2. Review violation details
3. Fix schema or telemetry code

## Troubleshooting

### Issue: "OTLP collector not ready"

**Symptom:**
```
❌ OTLP collector not ready within 30s
```

**Cause:**
Docker service didn't start in time

**Fix:**
Re-run workflow (transient issue)

### Issue: "Weaver process died unexpectedly"

**Symptom:**
```
❌ Weaver process died unexpectedly
[Weaver logs shown]
```

**Cause:**
Weaver crashed during startup

**Fix:**
1. Review Weaver logs in output
2. Check registry schema is valid
3. Verify ports not in use

### Issue: "Validation report not found"

**Symptom:**
```
❌ Validation report not found
```

**Cause:**
Weaver didn't write report file

**Fix:**
1. Check Weaver logs
2. Verify graceful shutdown (SIGHUP)
3. Check output directory permissions

### Issue: "No telemetry in tests"

**Symptom:**
```
sample_count: 0
violations: 0
```

**Cause:**
Tests don't exercise OTEL-instrumented code

**Fix:**
```rust
// Add OTEL initialization in test
#[tokio::test]
async fn test_with_telemetry() {
    let _guard = init_otel(OtelConfig::default())?;

    // Exercise instrumented code
    my_instrumented_function().await?;
}
```

### Issue: "Schema violations"

**Symptom:**
```
violations: 3
details: [
  "Attribute 'deployment.environment' missing",
  "Attribute 'http.status_code' has wrong type",
  ...
]
```

**Cause:**
Runtime telemetry doesn't match schema

**Fix:**
1. Update schema to match runtime:
   ```yaml
   # registry/clnrm.yaml
   attributes:
     - id: deployment.environment
       type: string
       requirement_level: required
   ```

2. Or update code to match schema:
   ```rust
   span.set_attribute(KeyValue::new(
       "deployment.environment",
       env::var("ENV").unwrap_or_else(|_| "dev".to_string())
   ));
   ```

## Workflow Inputs

### test_suite (choice)

**Options:**
- `all` (default): Run all tests
- `unit`: Run only unit tests
- `integration`: Run only integration tests
- `telemetry`: Run only telemetry tests

**Usage:**
```bash
gh workflow run weaver-refactor-validation.yml -f test_suite=telemetry
```

### fail_on_violations (boolean)

**Options:**
- `true` (default): FAIL CI if violations > 0
- `false`: Allow violations (warning only)

**Usage:**
```bash
# Allow violations (for debugging)
gh workflow run weaver-refactor-validation.yml -f fail_on_violations=false
```

## Artifacts

### schema-validation (7 days)

**Contents:**
- `registry/` directory
- Schema validation results

**Download:**
```bash
gh run download <run-id> -n schema-validation
```

### weaver-validation-results (30 days)

**Contents:**
- `validation_output/live_check.json` - Validation report
- `weaver.log` - Weaver process logs
- Telemetry samples

**Download:**
```bash
gh run download <run-id> -n weaver-validation-results
```

**Inspect report:**
```bash
cd weaver-validation-results
cat validation_output/live_check.json | jq .
```

## Integration with Other Workflows

### weaver-refactor-validation.yml (This Workflow)

**Purpose:** Fast PR feedback
**Duration:** 5-8 minutes
**Use case:** Development iteration

### weaver-validation-gate.yml (Existing)

**Purpose:** Production certification
**Duration:** 10-15 minutes
**Use case:** Final merge gate

**Recommendation:**
- Use `weaver-refactor-validation.yml` for rapid feedback
- Use `weaver-validation-gate.yml` for comprehensive validation

## Best Practices

### 1. Fix violations immediately

Don't let violations accumulate. Each violation is a divergence between schema and reality.

### 2. Run locally before pushing

```bash
# Local validation (requires Docker)
./scripts/run_weaver_validation.sh

# Check results
cat validation_output/live_check.json | jq '.sample_count, .violations'
```

### 3. Use telemetry test suite

```bash
# Test only OTEL code
cargo test --features otel -p clnrm-core telemetry
```

### 4. Monitor sample count trends

Track how many telemetry samples are captured over time. Decreasing samples may indicate reduced coverage.

### 5. Keep schemas in sync

When adding telemetry, update schema FIRST, then implement code. Schema is the specification.

## Quick Commands

```bash
# Trigger workflow manually
gh workflow run weaver-refactor-validation.yml

# Check latest run status
gh run list --workflow=weaver-refactor-validation.yml --limit 1

# View run logs
gh run view <run-id> --log

# Download validation report
gh run download <run-id> -n weaver-validation-results

# View PR comment (in PR)
gh pr view <pr-number>

# Check deployment gate status
gh run view <run-id> | grep "Deployment Gate"
```

## Success Checklist

Before merging a PR, ensure:

- [ ] ✅ Schema validation passed
- [ ] ✅ Build with OTEL succeeded
- [ ] ✅ `sample_count > 0` (telemetry captured)
- [ ] ✅ `violations == 0` (schema conformance)
- [ ] ✅ Deployment gate passed
- [ ] ✅ PR comment shows "Safe to Merge"

**If any checkbox is unchecked, DO NOT MERGE.**

## References

- **Workflow file:** `.github/workflows/weaver-refactor-validation.yml`
- **Weaver docs:** [OpenTelemetry Weaver](https://github.com/open-telemetry/weaver)
- **Schema registry:** `registry/`
- **Validation scripts:** `scripts/`
- **Runbooks:** `docs/runbooks/`

---

**Remember:** Weaver validation is the ONLY way to prove features work. Tests can lie, telemetry schemas cannot.
