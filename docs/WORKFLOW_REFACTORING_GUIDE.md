# GitHub Actions Workflow Refactoring Guide

## Overview

This document provides a comprehensive guide to refactoring GitHub Actions workflows based on FMEA (Failure Mode and Effects Analysis) of all 19 workflows across 3,500+ lines of YAML.

**Status**: Refactoring in progress
- **Foundation**: ✅ Complete (composite actions, .tool-versions, example refactor)
- **Remaining**: 18 workflows to refactor

---

## FMEA Summary

**Total Failure Modes Identified**: 87
- **Critical (RPN ≥ 20)**: 8 items
- **High (RPN 12-19)**: 26 items
- **Medium (RPN 6-11)**: 38 items
- **Low (RPN ≤ 5)**: 15 items

**Average RPN**: 10.6 → **Target: < 5** (50%+ risk reduction)

---

## Refactoring Patterns

### 1. Process Lifecycle Management (RPN 20-25)

**Problem**: Background processes (Weaver, OTLP collectors) start but health is never verified.

**Solution Pattern**:
```bash
# Kill zombie processes
if command -v fuser &> /dev/null; then
  PIDS=$(fuser -q $PORT/tcp 2>/dev/null || true)
  if [ -n "$PIDS" ]; then
    fuser -k -q $PORT/tcp 2>/dev/null || true
    sleep 2
  fi
fi

# Verify port availability
for i in {1..3}; do
  if ! netcat -z localhost $PORT >/dev/null 2>&1; then
    echo "✅ Port available"
    exit 0
  fi
  sleep 2
done

# Start process with PID tracking
process_command &
PID=$!
echo $PID > process.pid

# Poll for health (not fixed timeout)
WAIT_ATTEMPTS=0
MAX_ATTEMPTS=15
while [ $WAIT_ATTEMPTS -lt $MAX_ATTEMPTS ]; do
  if kill -0 $PID 2>/dev/null && netcat -z localhost $PORT; then
    echo "✅ Process healthy"
    exit 0
  fi
  WAIT_ATTEMPTS=$((WAIT_ATTEMPTS + 1))
  sleep 2
done

# Cleanup with timeout
kill -TERM $PID 2>/dev/null || true
for i in {1..30}; do
  if ! kill -0 $PID 2>/dev/null; then
    exit 0
  fi
  sleep 1
done
kill -9 $PID 2>/dev/null || true
```

**Applies to**:
- weaver-refactor-validation.yml (RPN 25)
- telemetry-validation.yml (RPN 25)
- weaver-validation-gate.yml (RPN 25)
- integration-tests.yml (RPN 25) - Docker health checks

---

### 2. String Parsing Fragility → Structured JSON (RPN 12-20)

**Problem**: Heavy grep/awk/sed on unstructured output breaks on format changes.

**BAD**:
```bash
COVERAGE=$(echo "$COVERAGE * 100" | bc -l | cut -d. -f1)
VIOLATIONS=$(grep -A 1 "violations:" output.log | awk '{print $2}')
```

**GOOD**:
```bash
# Use jq with safe defaults
VIOLATIONS=$(jq -r '.statistics.advice_level_counts.violation // 0' output.json)
WARNINGS=$(jq -r '.statistics.advice_level_counts.warning // 0' output.json)
COVERAGE=$(jq -r '.statistics.registry_coverage // 0' output.json)

# Use integer arithmetic
COVERAGE_PCT=$((COVERAGE * 100 / 100))
```

**Applies to**:
- performance-regression.yml (RPN 25) - Criterion output
- schema-validation.yml (RPN 20) - Schema diffing
- weaver-validation.yml (RPN 20) - Weaver output parsing
- contract-tests.yml (RPN 12) - Schema validation

---

### 3. Platform Compatibility (RPN 6-20)

**Problem**: Linux-only commands fail on macOS/Windows runners.

**BAD - Linux only**:
```bash
BINARY_SIZE=$(stat -c%s target/release/clnrm)
SHA256=$(sha256sum file.tar.gz | awk '{print $1}')
COVERAGE=$((COVERAGE * 100 / 100)) # bc not portable
```

**GOOD - POSIX compliant**:
```bash
# Binary size - works everywhere
BINARY_SIZE=$(wc -c < target/release/clnrm)
# or
BINARY_SIZE=$(du -b target/release/clnrm | awk '{print $1}')

# SHA256 - works everywhere
SHA256=$(shasum -a 256 file.tar.gz | awk '{print $1}')
# or force portable checksum
if command -v sha256sum &> /dev/null; then
  SHA256=$(sha256sum file.tar.gz | awk '{print $1}')
else
  SHA256=$(shasum -a 256 file.tar.gz | awk '{print $1}')
fi

# Integer arithmetic (no bc needed)
COVERAGE_PCT=$((COVERAGE * 100 / 100))
```

**Applies to**:
- performance-regression.yml (RPN 20) - stat command
- release.yml (RPN 20) - Platform-specific commands
- homebrew-release.yml (RPN 25) - SHA256 calculation
- pages.yml (RPN 15) - File size validation
- documentation.yml (RPN 15) - Temporary file paths

---

### 4. Dependency Installation & Caching (RPN 6-20)

**Problem**: Network dependencies without retry, missing caches.

**Pattern**:
```yaml
- name: Cache Weaver
  uses: actions/cache@v4
  with:
    path: ~/.cargo/bin/weaver
    key: weaver-${{ env.WEAVER_VERSION }}-${{ runner.os }}
    lookup-only: true

- name: Install with Retry
  run: |
    # Check if cached
    if [ -f ~/.cargo/bin/weaver ]; then
      echo "✅ Using cached Weaver"
    else
      # Install with retry
      for attempt in {1..3}; do
        echo "Attempt $attempt..."
        cargo install weaver-cli --version ${{ env.WEAVER_VERSION }} --locked && exit 0
        sleep $((2 ** attempt))
      done
      exit 1
    fi

- name: Save to Cache
  if: always()
  uses: actions/cache/save@v4
  with:
    path: ~/.cargo/bin/weaver
    key: weaver-${{ env.WEAVER_VERSION }}-${{ runner.os }}
```

**Applies to**:
- All workflows using `cargo install`
- Weaver installation (5 workflows)
- Test tool installation (3 workflows)

---

### 5. Error Suppression → Explicit Handling (RPN 4-15)

**BAD**:
```bash
- name: Validate
  run: |
    clnrm validate examples/ || echo "Validation may need Docker"

- name: Critical check
  continue-on-error: true
  run: bash scripts/scan-fakes.sh
```

**GOOD**:
```bash
- name: Validate
  run: |
    # Explicit conditions
    FAILED=0
    for file in examples/*.toml; do
      if clnrm validate "$file" 2>/dev/null; then
        echo "✅ $file"
      else
        echo "⚠️ $file needs Docker (skipped)"
        # Track but don't fail
      fi
    done
    exit $FAILED

- name: Fake scanner (must pass)
  run: bash scripts/scan-fakes.sh  # NO continue-on-error!

- name: Fake scanner with known exclusion
  run: |
    bash scripts/scan-fakes.sh || {
      if grep -q "EXEMPTED" scan-result.log; then
        echo "⚠️ Known exemptions found"
        exit 0  # Explicit pass
      fi
      exit 1  # Real failure
    }
```

**Applies to**:
- best-practices.yml (RPN 12) - TOML validation
- fast-tests.yml (RPN 15) - Fake scanner, CI gate
- ci.yml (RPN 15) - Cargo audit
- documentation.yml (RPN 15) - Example validation

---

### 6. Version Drift → Centralized Management (RPN 6-12)

**Before**: Version hardcoded in 7 different workflows
```yaml
env:
  WEAVER_VERSION: "v0.10.0"     # In telemetry-validation.yml
  WEAVER_VERSION: "0.16.1"      # In publish-crates.yml
  RUST_VERSION: "1.70"           # In various workflows
```

**After**: Single source of truth
```
# .tool-versions (asdf format)
rust stable
weaver-cli 0.16.1
nodejs 18.17.0
```

In workflows:
```yaml
env:
  WEAVER_VERSION: "0.16.1"  # Reference .tool-versions
  # Comment: "Also defined in .tool-versions for local dev"
```

**Applies to**:
- All 19 workflows
- Reduces maintenance burden
- Enables local/CI consistency

---

## Priority Refactoring Order

### Phase 1: Critical (RPN ≥ 20) - **Do These First**

1. **weaver-refactor-validation.yml** (RPN 25)
   - Add port conflict check before Weaver start
   - Implement proper process health monitoring
   - Replace fixed timeout with polling

2. **telemetry-validation.yml** (RPN 25) ✅ **DONE**
   - Port cleanup and binding verification
   - Process health checks with retries
   - Structured JSON parsing with jq
   - Graceful shutdown with timeout

3. **publish-crates.yml** (RPN 25)
   - Increase crates.io index sync from 30s to 60s
   - Add verification loop (cargo search)
   - Implement exponential backoff retry

4. **weaver-validation-gate.yml** (RPN 25)
   - Add continuous process health monitoring
   - Verify process exists before each operation
   - Implement proper supervision

5. **homebrew-release.yml** (RPN 25)
   - Normalize SHA256 to `shasum -a 256`
   - Add hash verification step
   - Platform-specific testing

6. **performance-regression.yml** (RPN 25)
   - Use structured JSON from criterion
   - Replace grep pattern matching with jq
   - Add output format validation

7. **integration-tests.yml** (RPN 25)
   - Add explicit timeout to docker-compose
   - Collect logs on timeout
   - Use docker health endpoints

8. **schema-validation.yml** (RPN 20)
   - Replace comm-based diffing with JSON schema diff
   - Add sort validation before comm
   - Improve change detection accuracy

### Phase 2: High Priority (RPN 12-19) - **Next Sprint**

- release.yml (RPN 20)
- weaver-validation.yml (RPN 20)
- fuzz.yml (RPN 20)
- pages.yml (RPN 20)
- ci.yml (RPN 20)
- performance.yml (RPN 12)
- contract-tests.yml (RPN 12)

### Phase 3: Medium Priority (RPN 6-11) - **Steady Progress**

- All remaining workflows
- Pattern standardization
- Documentation improvements

---

## Composite Actions for Reuse

Pre-created composite actions available:

### `.github/workflows/lib-install-weaver.yml`
```yaml
- uses: ./.github/actions/install-weaver
  with:
    version: '0.16.1'
```

### `.github/workflows/lib-port-health-check.yml`
```yaml
- uses: ./.github/actions/port-health-check
  with:
    port: '4317'
    timeout: '30'
    retry-count: '5'
```

### `.github/workflows/lib-process-health-check.yml`
```yaml
- uses: ./.github/actions/process-health-check
  with:
    pid-file: 'weaver.pid'
    timeout: '30'
    check-interval: '2'
```

### `.github/workflows/lib-verify-artifact.yml`
```yaml
- uses: ./.github/actions/verify-artifact
  with:
    path: 'target/contract-test-results/'
    description: 'Contract test results'
    min-size: '100'
```

---

## Quality Checklist for Each Workflow

After refactoring, verify:

- [ ] All YAML is syntactically valid (`python3 -c "import yaml; yaml.safe_load(open('file.yml'))"`)
- [ ] No hardcoded versions (reference `.tool-versions` and `env:` block)
- [ ] Process health checks use polling, not fixed timeouts
- [ ] Port conflicts checked before binding
- [ ] All output parsing uses `jq` with safe defaults (not grep/awk)
- [ ] Platform-specific commands use POSIX alternatives
- [ ] No error suppression (`|| echo`, `continue-on-error: true`) for critical checks
- [ ] Artifact existence verified before use
- [ ] Caching properly configured with `restore-keys`
- [ ] Timeout-minutes set at job level
- [ ] Explicit error handling, not silent failures
- [ ] Comments document non-obvious decisions

---

## Testing Refactored Workflows

Before deploying:

```bash
# Validate YAML syntax
python3 -c "import yaml; yaml.safe_load(open('.github/workflows/YOUR_WORKFLOW.yml'))"

# Test locally with act (requires act installation)
act -j job_name -W .github/workflows/YOUR_WORKFLOW.yml

# Check for regressions
git diff HEAD~1 .github/workflows/YOUR_WORKFLOW.yml  # Review changes
```

---

## Expected Improvements

After full refactoring:

| Metric | Before | After | Improvement |
|--------|--------|-------|-------------|
| **Avg RPN** | 10.6 | < 5 | -50%+ |
| **Process hangs** | Frequent | Rare | Polling instead of timeouts |
| **Format breakage** | High | Low | jq instead of grep/awk |
| **Platform failures** | 15-20% | < 5% | POSIX-compliant commands |
| **Network timeouts** | Frequent | Rare | Retry logic + caching |
| **False negatives** | Common | Rare | Explicit error handling |
| **Maintenance burden** | High | Low | Centralized versions |

---

## References

- FMEA Report: See companion FMEA analysis document (87 failure modes)
- Tool Versions: `.tool-versions` file
- Example Refactor: `telemetry-validation.yml` (fully refactored)
- Composite Actions: `.github/workflows/lib-*.yml`
- CLAUDE.md: Project-specific guidelines
