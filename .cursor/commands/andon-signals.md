# Andon Signals (Visual Problem Indicators) - Multi-Step Workflow

## Purpose

This command guides agents to treat compiler errors, test failures, and warnings as Andon signals - visual indicators that something is wrong and work should stop. Andon means "lantern" or "sign" - a visual signal that alerts to problems. Experts stop and fix problems immediately when signals appear.

## Workflow Overview

```
Step 1: Monitor Andon Signals → Step 2: Stop When Signal Appears → Step 3: Investigate Root Cause → Step 4: Fix Root Cause → Step 5: Verify Signal Cleared
```

## Step-by-Step Instructions

### Step 1: Monitor Andon Signals

**Action**: Watch for visual signals that indicate problems.

**Andon signal types**:

1. **Compiler errors** - Red signals, must stop
   - Pattern: `error[E...]: <description>`
   - Severity: **CRITICAL** - Cannot proceed

2. **Compiler warnings** - Yellow signals, should stop
   - Pattern: `warning: <description>`
   - Severity: **HIGH** - Should fix before proceeding

3. **Test failures** - Red signals, must stop
   - Pattern: `test ... FAILED`
   - Severity: **CRITICAL** - Cannot proceed

4. **Linting errors** - Yellow/red signals, should stop
   - Pattern: Clippy warnings/errors
   - Severity: **HIGH** - Should fix before proceeding

5. **Performance regressions** - Yellow signals, investigate
   - Pattern: Tests taking longer than expected
   - Severity: **MEDIUM** - Investigate if significant

6. **Container failures** - Red signals, must stop
   - Pattern: Container startup failures, test execution failures
   - Severity: **CRITICAL** - Cannot proceed

7. **OpenTelemetry validation failures** - Red signals, must stop
   - Pattern: Weaver validation errors, schema mismatches
   - Severity: **CRITICAL** - Cannot proceed

**Action**: Set up signal monitoring

```bash
# Monitor compilation signals
cargo make check
# Look for: error[...] or warning: patterns

# Monitor test signals
cargo make test
# Look for: test ... FAILED patterns

# Monitor linting signals
cargo make lint
# Look for: warning: or error: patterns

# Monitor container signals
cargo make test-integration
# Look for: container startup failures, test execution failures

# Monitor OpenTelemetry signals
cargo make validate-otel
# Look for: validation errors, schema mismatches
```

**Principle**: "Andon signals are visual management" - Make problems immediately visible, don't hide them.

---

### Step 2: Stop When Signal Appears

**Action**: Immediately stop work when an Andon signal appears.

#### 2.1: Recognize Signal Severity

**Action**: Determine signal severity and response.

**Signal severity levels**:

- **CRITICAL (Red)** - Must stop immediately
  - Compiler errors
  - Test failures
  - Container failures
  - OpenTelemetry validation failures
  - **Response**: Stop all work, fix immediately

- **HIGH (Yellow)** - Should stop
  - Compiler warnings
  - Linting errors
  - **Response**: Stop current work, fix before proceeding

- **MEDIUM (Yellow)** - Investigate
  - Performance warnings
  - Code quality warnings
  - **Response**: Investigate, fix if significant

**Action**: Classify signal

```markdown
## Andon Signal Classification

### Critical Signals (Stop Immediately)
- [ ] Compiler error: `error[E0425]: cannot find function`
- [ ] Test failure: `test test_name ... FAILED`
- [ ] Container failure: `Failed to start container`
- [ ] OpenTelemetry validation failure: `Schema mismatch`

### High Signals (Stop and Fix)
- [ ] Compiler warning: `warning: unused variable`
- [ ] Linting error: `clippy::unwrap_used`

### Medium Signals (Investigate)
- [ ] Performance warning: Test taking longer than expected
- [ ] Code quality warning: Complexity too high
```

#### 2.2: Stop the Line

**Action**: Stop current work when signal appears.

**Stop the line principles**:
- **Don't ignore** - Never ignore Andon signals
- **Don't proceed** - Don't continue work with signals present
- **Don't hide** - Don't suppress warnings/errors
- **Fix immediately** - Address signal before continuing

**Example response**:
```bash
# Signal appeared: Compiler error
cargo make check
# Output: error[E0425]: cannot find function `test_function`

# STOP: Do not proceed with other work
# ACTION: Fix compiler error immediately
```

---

### Step 3: Investigate Root Cause

**Action**: Understand why the signal appeared.

#### 3.1: Read Signal Message

**Action**: Carefully read the signal message.

**What to look for**:
- **Error message** - What went wrong?
- **Location** - Where did it occur?
- **Context** - What was happening when it occurred?

**Example**:
```
error[E0425]: cannot find function `test_function` in this scope
  --> crates/clnrm-core/src/test.rs:10:5
   |
10 |     test_function();
   |     ^^^^^^^^^^^^ not found in this scope
```

**Analysis**:
- **What**: Function `test_function` not found
- **Where**: `crates/clnrm-core/src/test.rs:10:5`
- **Why**: Function doesn't exist or not imported

#### 3.2: Trace Root Cause

**Action**: Use root cause analysis to find why signal appeared.

**Questions to ask**:
- Why did this signal appear?
- What changed that caused it?
- Is this a symptom of a deeper problem?

**Example root cause analysis**:
```markdown
## Root Cause Analysis

**Signal**: Compiler error - function not found
**Why #1**: Function `test_function` doesn't exist
**Why #2**: Function was removed during refactoring
**Why #3**: Tests weren't updated after refactoring
**Root Cause**: Missing test update after refactoring
```

**Reference**: See [Root Cause Analysis](./root-cause-analysis.md) for detailed 5 Whys process

#### 3.3: Verify Root Cause

**Action**: Confirm root cause hypothesis.

**Verification**:
- Does fixing root cause clear the signal?
- Does data support root cause hypothesis?
- Are there other contributing factors?

---

### Step 4: Fix Root Cause

**Action**: Address the underlying cause, not just the symptom.

#### 4.1: Fix the Problem

**Action**: Implement fix that addresses root cause.

**Fix principles**:
- **Fix root cause** - Not just symptom
- **Fix completely** - Don't leave partial fixes
- **Fix safely** - Don't introduce new problems

**Example fix**:
```rust
// Root cause: Function removed, test not updated
// Fix: Add missing function or update test

// Option 1: Add missing function
fn test_function() -> Result<(), CleanroomError> {
    // Implementation
    Ok(())
}

// Option 2: Update test to use correct function
// test_function() -> actual_function()
```

#### 4.2: Verify Fix

**Action**: Ensure fix resolves the signal.

**Verification steps**:
1. Fix the problem
2. Re-run signal check
3. Verify signal cleared

**Example**:
```bash
# Fix applied
# Re-check signal
cargo make check
# Expected: No errors, signal cleared ✅
```

---

### Step 5: Verify Signal Cleared

**Action**: Confirm signal is resolved and won't return.

#### 5.1: Verify Signal Cleared

**Action**: Run checks to confirm signal gone.

**Verification**:
- ✅ Compiler errors cleared: `cargo make check`
- ✅ Test failures cleared: `cargo make test`
- ✅ Warnings cleared: `cargo make lint`
- ✅ Container failures cleared: `cargo make test-integration`
- ✅ OpenTelemetry validation cleared: `cargo make validate-otel`
- ✅ No new signals appeared

**Example**:
```bash
# Verify all signals cleared
cargo make check   # No errors ✅
cargo make test    # All tests pass ✅
cargo make lint    # No warnings ✅
cargo make test-integration  # Containers work ✅
cargo make validate-otel     # Validation passes ✅
```

#### 5.2: Prevent Signal Return

**Action**: Add controls to prevent signal from returning.

**Prevention methods**:
- **Tests** - Add tests to catch regression
- **Linting** - Enable linting rules in CI
- **Documentation** - Document why fix was needed
- **Code review** - Review to prevent similar issues

**Example**:
```rust
// Add test to prevent regression
#[tokio::test]
async fn test_container_startup_succeeds() -> Result<(), CleanroomError> {
    // Test that would fail if function removed again
    let container = Container::new("alpine:latest").await?;
    container.start().await?;
    assert!(container.is_running());
    Ok(())
}
```

#### 5.3: Monitor for New Signals

**Action**: Continue monitoring for new signals.

**Monitoring**:
- Run checks regularly
- Don't ignore warnings
- Fix signals immediately

---

## Complete Workflow Example

```bash
# Step 1: Monitor Andon Signals
cargo make check
# Signal appeared: error[E0425]: cannot find function

# Step 2: Stop When Signal Appears
# STOP: Do not proceed with other work
# ACTION: Fix compiler error immediately

# Step 3: Investigate Root Cause
# Root cause: Function removed during refactoring, test not updated

# Step 4: Fix Root Cause
# Fix: Add missing function or update test
# Applied fix

# Step 5: Verify Signal Cleared
cargo make check  # No errors ✅
cargo make test   # All tests pass ✅
# Signal cleared, work can proceed
```

## Andon Signal Response Matrix

| Signal Type | Severity | Response | Example |
|------------|----------|----------|---------|
| Compiler error | CRITICAL | Stop immediately, fix now | `error[E0425]` |
| Test failure | CRITICAL | Stop immediately, fix now | `test ... FAILED` |
| Container failure | CRITICAL | Stop immediately, fix now | `Failed to start container` |
| OpenTelemetry validation failure | CRITICAL | Stop immediately, fix now | `Schema mismatch` |
| Compiler warning | HIGH | Stop current work, fix before proceeding | `warning: unused` |
| Linting error | HIGH | Stop current work, fix before proceeding | `clippy::unwrap_used` |
| Performance warning | MEDIUM | Investigate, fix if significant | Test timeout |

## Integration with Other Commands

- **[Root Cause Analysis](./root-cause-analysis.md)** - Use 5 Whys in Step 3 to find root cause
- **[DMAIC Problem Solving](./dmaic-problem-solving.md)** - Use DMAIC to systematically fix signals
- **[Gemba Walk](./gemba-walk.md)** - Go to source in Step 3 to investigate
- **[Poka-Yoke Design](./poka-yoke-design.md)** - Use type system in Step 4 to prevent signals

## Expert Insights

**Why this matters**: Ignoring signals leads to accumulating problems. Experts treat every signal as important and fix them immediately.

**Key principle**: "Stop the line" - When an Andon signal appears, stop work and fix the problem immediately. Don't proceed with problems present.

**Remember**: Andon signals are visual management. They make problems immediately visible. Don't hide them, don't ignore them, fix them.

**Andon culture**: In Lean manufacturing, any worker can stop the production line if they see a problem. In coding, any developer should stop and fix problems when signals appear. This prevents defects from propagating.

**DfLSS alignment**: Andon signals help prevent both defects (quality) and waste (efficiency) - stopping problems early prevents rework (waste) and defects from propagating (quality). This aligns with DfLSS (Design for Lean Six Sigma) principles. Don't conflate DfLSS with DFSS (Design for Six Sigma) - DFSS only addresses quality, missing critical waste elimination. See [Root Cause Analysis - DfLSS vs DFSS](./root-cause-analysis.md#dflss-vs-dfss-critical-distinction) for why conflating DfLSS with DFSS is a huge error.

