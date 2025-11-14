# Remaining GitHub Actions Workflows - FMEA Refactoring Queue

**Status:** 1 of 8 complete (weaver-refactor-validation.yml ✅)
**Next:** ci.yml (RPN 25)

---

## Completed

### ✅ 1. weaver-refactor-validation.yml
- **Original RPN:** 25
- **Refactored RPN:** 2
- **Status:** COMPLETE
- **Summary:** `/home/user/clnrm/.github/workflows/WEAVER_REFACTOR_VALIDATION_FMEA_SUMMARY.md`

---

## Queue (Ordered by RPN)

### 🔴 2. ci.yml (RPN 25)
**Critical Issues:**
- Wrong Weaver package name (weaver-forge)
- Missing port cleanup
- Inadequate health checks
- Missing process verification

**Priority:** HIGH (Critical CI workflow)

**Refactoring Pattern:** Same as weaver-refactor-validation.yml
1. Fix weaver-forge → weaver-cli
2. Add port 4317 cleanup
3. Implement 15×2s dual-port polling
4. Add process checks before operations
5. Use SIGTERM → SIGKILL shutdown
6. Replace lsof with netcat

---

### 🔴 3. security-audit.yml (RPN 20)
**Critical Issues:**
- Missing dependency version pinning
- No timeout on cargo audit
- No fallback if audit fails

**Priority:** HIGH (Security-critical)

**Refactoring Needed:**
1. Pin cargo-audit version
2. Add timeout (300s)
3. Add retry logic (3 attempts)
4. Add fallback to cached results
5. Add SARIF upload for GitHub Security

---

### 🟡 4. release.yml (RPN 15)
**Critical Issues:**
- No validation before release
- Missing artifact verification
- No rollback mechanism

**Priority:** MEDIUM (Release automation)

**Refactoring Needed:**
1. Add pre-release validation step
2. Verify checksums of artifacts
3. Add smoke tests before publish
4. Add rollback script
5. Add release notes generation

---

### 🟡 5. docs-deploy.yml (RPN 12)
**Critical Issues:**
- No build verification
- Missing link checking
- No preview deployment

**Priority:** MEDIUM (Documentation)

**Refactoring Needed:**
1. Add mdbook build verification
2. Add link checker (lychee)
3. Add preview deployment to Netlify
4. Add broken link reporting
5. Add search index validation

---

### 🟢 6. benchmark.yml (RPN 10)
**Critical Issues:**
- No baseline comparison
- Missing regression detection
- No artifact upload

**Priority:** LOW (Performance monitoring)

**Refactoring Needed:**
1. Add baseline storage in git
2. Add regression detection (>5% slower)
3. Upload results as artifacts
4. Add performance trend graphs
5. Add comment with results on PR

---

### 🟢 7. dependency-update.yml (RPN 8)
**Critical Issues:**
- No breaking change detection
- Missing test runs before PR
- No changelog generation

**Priority:** LOW (Automation)

**Refactoring Needed:**
1. Add breaking change detection
2. Run tests before creating PR
3. Generate changelog from commits
4. Add version compatibility check
5. Add security advisory check

---

### 🟢 8. label-sync.yml (RPN 5)
**Critical Issues:**
- No validation of label config
- Missing conflict detection

**Priority:** LOW (Maintenance)

**Refactoring Needed:**
1. Validate labels.yml schema
2. Detect conflicting labels
3. Add dry-run mode
4. Add before/after report
5. Add label usage statistics

---

## Refactoring Standards (Apply to All)

### 1. Package Installation
```yaml
# ✅ CORRECT
cargo install weaver-cli --version ${WEAVER_VERSION}

# ❌ WRONG
cargo install weaver-forge --version ${WEAVER_VERSION}
```

### 2. Port Cleanup
```yaml
- name: Clean up port XXXX
  run: |
    if nc -z localhost XXXX 2>/dev/null; then
      PID=$(lsof -ti:XXXX || true)
      [ -n "$PID" ] && kill -9 $PID
      sleep 2
    fi
```

### 3. Health Checks (15 × 2s)
```yaml
MAX_ATTEMPTS=15
INTERVAL=2

for attempt in $(seq 1 $MAX_ATTEMPTS); do
  # Check process exists
  if ! ps -p $PID >/dev/null 2>&1; then
    exit 1
  fi

  # Check port
  if nc -z localhost PORT 2>/dev/null; then
    echo "✅ Ready"
    exit 0
  fi

  sleep $INTERVAL
done
```

### 4. Process Verification
```yaml
# Before critical operations
if ! ps -p $PID >/dev/null 2>&1; then
  echo "❌ Process died"
  exit 1
fi
```

### 5. Graceful Shutdown
```yaml
# SIGTERM → wait → SIGKILL
kill -TERM $PID || true
MAX_WAIT=15
for i in $(seq 1 $MAX_WAIT); do
  ! ps -p $PID >/dev/null 2>&1 && break
  [ $i -eq $MAX_WAIT ] && kill -9 $PID
  sleep 1
done
```

### 6. Dependencies
```yaml
# ✅ Use netcat for port checking
sudo apt-get install -y jq netcat-openbsd bc

# ❌ Don't use lsof alone
sudo apt-get install -y jq lsof bc
```

### 7. Logging
```yaml
# Use emoji indicators
echo "🚀 Starting..."
echo "✅ Success"
echo "❌ Failed"
echo "⚠️  Warning"
echo "🔍 Checking..."
echo "⏳ Waiting..."
```

---

## Workflow-Specific Patterns

### CI Workflows (ci.yml, weaver-refactor-validation.yml)
- Must check BOTH admin AND OTLP ports
- Need process verification before tests
- Require detailed failure logging
- Must upload artifacts on failure

### Security Workflows (security-audit.yml)
- Must have timeouts
- Need retry logic
- Require fallback mechanisms
- Must upload SARIF to GitHub Security

### Release Workflows (release.yml)
- Must validate before publishing
- Need checksum verification
- Require rollback scripts
- Must generate changelogs

### Benchmark Workflows (benchmark.yml)
- Must compare against baseline
- Need regression detection
- Require trend visualization
- Must comment on PRs

---

## Testing Checklist (For Each Refactored Workflow)

- [ ] YAML syntax validates
- [ ] All package names correct
- [ ] Port cleanup present (if applicable)
- [ ] Health checks use 15×2s pattern
- [ ] Process verification at critical points
- [ ] Graceful shutdown (SIGTERM → SIGKILL)
- [ ] Dependencies use netcat
- [ ] Logging uses emoji indicators
- [ ] Error cases show logs
- [ ] Artifacts uploaded on failure
- [ ] RPN reduction documented
- [ ] Summary document created

---

## Progress Tracking

| Workflow | RPN | Status | Date | RPN Reduction |
|----------|-----|--------|------|---------------|
| weaver-refactor-validation.yml | 25 | ✅ Complete | 2025-11-14 | 25 → 2 (92%) |
| ci.yml | 25 | ⏳ Pending | - | - |
| security-audit.yml | 20 | ⏳ Pending | - | - |
| release.yml | 15 | ⏳ Pending | - | - |
| docs-deploy.yml | 12 | ⏳ Pending | - | - |
| benchmark.yml | 10 | ⏳ Pending | - | - |
| dependency-update.yml | 8 | ⏳ Pending | - | - |
| label-sync.yml | 5 | ⏳ Pending | - | - |

**Total RPN Before:** 120
**Total RPN After (Projected):** 25 (79% reduction)

---

**Next Action:** Refactor `ci.yml` (RPN 25) using same pattern as weaver-refactor-validation.yml

**Generated:** 2025-11-14
**Status:** 1/8 complete (12.5%)
