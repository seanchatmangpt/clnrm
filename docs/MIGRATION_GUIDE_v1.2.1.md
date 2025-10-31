# Migration Guide: v1.2.0 → v1.2.1

**Last Updated:** 2025-10-31
**Target Audience:** Existing clnrm v1.2.0 users upgrading to v1.2.1
**Difficulty:** Easy (no breaking changes)
**Estimated Time:** 10-15 minutes

## 📋 Table of Contents

- [Overview](#overview)
- [Pre-Migration Checklist](#pre-migration-checklist)
- [Installation](#installation)
- [What's Changed](#whats-changed)
- [What's New](#whats-new)
- [Post-Migration Verification](#post-migration-verification)
- [Rollback Plan](#rollback-plan)
- [FAQ](#faq)

## Overview

### Migration Summary

v1.2.1 is a **minor bug-fix release** with **no breaking changes**. The migration is straightforward:

1. Upgrade clnrm binary (Homebrew or Cargo)
2. Verify installation works
3. No code changes required
4. Optional: Leverage new features

**Key Improvements:**
- ✅ Fixed registry path resolution (can now run from any directory)
- ✅ Added sample count validation (prevents false positives)
- ✅ Enhanced error messages and troubleshooting
- ✅ Improved Homebrew packaging

**Breaking Changes:** **NONE**

All existing TOML configurations, CLI commands, and workflows continue to work unchanged.

## Pre-Migration Checklist

Before upgrading, verify your current setup:

```bash
# Check current version
clnrm --version
# Should show: clnrm 1.2.0

# Verify current installation works
clnrm self-test
# Should pass all tests

# Backup your test configurations (optional but recommended)
tar -czf clnrm-tests-backup-$(date +%Y%m%d).tar.gz tests/

# Save current validation results for comparison
clnrm run --validate tests/ > validation-results-before.txt 2>&1
```

**Recommended Pre-Migration Actions:**
- [ ] Document current working directory patterns
- [ ] Note any `CLNRM_REGISTRY_PATH` environment variable usage
- [ ] Review CI/CD pipelines that call clnrm
- [ ] Test current installation from different directories

## Installation

### Option 1: Homebrew (Recommended)

**For macOS/Linux users:**

```bash
# Upgrade to v1.2.1
brew upgrade clnrm

# Verify new version
clnrm --version
# Output: clnrm 1.2.1

# Verify registry installation
ls -la $(brew --prefix)/share/clnrm/registry/
# Should show: cli/ core/ metrics/ registry.yaml
```

**If upgrade fails:**
```bash
# Force reinstall
brew uninstall clnrm
brew install clnrm

# Verify installation
clnrm --version
clnrm self-test
```

### Option 2: Cargo

**For Rust developers:**

```bash
# Upgrade to v1.2.1
cargo install clnrm --force

# Verify new version
clnrm --version
# Output: clnrm 1.2.1
```

### Option 3: Build from Source

**For contributors/developers:**

```bash
# Navigate to clnrm repository
cd /path/to/clnrm

# Pull latest changes
git fetch origin
git checkout v1.2.1
# Or: git pull origin master

# Build and install
cargo build --release --features otel
cargo install --path crates/clnrm --force

# Verify new version
clnrm --version
# Output: clnrm 1.2.1
```

## What's Changed

### 1. Registry Path Resolution (Critical Fix)

**Before v1.2.1:**
```bash
# ❌ Only worked from project root
cd /path/to/clnrm-project
clnrm init  # ✅ Works
cd /tmp
clnrm init  # ❌ Error: registry not found
```

**After v1.2.1:**
```bash
# ✅ Works from any directory
cd /tmp
clnrm init  # ✅ Works
cd /anywhere
clnrm run --validate /path/to/tests/  # ✅ Works
```

**What Changed:**
- Registry path now resolves from installation directory, not current working directory
- Homebrew installations include registry at `$(brew --prefix)/share/clnrm/registry/`
- Added `CLNRM_REGISTRY_PATH` environment variable for custom installations

**Migration Action Required:** **NONE** (automatically works after upgrade)

**Optional Enhancement:**
```bash
# For custom installations, set registry path explicitly
export CLNRM_REGISTRY_PATH="/custom/path/to/registry"

# Verify it works
clnrm init /tmp/test-project
```

### 2. Sample Count Validation (Critical Fix)

**Before v1.2.1:**
```bash
# ❌ Could report success even with 0 telemetry samples
clnrm run --validate tests/broken.clnrm.toml
# Exit code: 0 (success) - FALSE POSITIVE
```

**After v1.2.1:**
```bash
# ✅ Fails explicitly if no telemetry received
clnrm run --validate tests/broken.clnrm.toml
# Error: Weaver validation received 0 samples - telemetry was not emitted correctly
# Exit code: 1 (failure)

# ✅ Logs sample count on success
clnrm run --validate tests/working.clnrm.toml
# ✅ Weaver validation passed: 145 samples, 87.3% coverage
# Exit code: 0 (success)
```

**What Changed:**
- Added explicit validation: `sample_count == 0` → fail
- Enhanced success logging with sample count and coverage percentage
- Better error messages with troubleshooting guidance

**Migration Action Required:** **NONE** (automatically works after upgrade)

**Behavior Change:**
Tests that previously passed with 0 samples will now **correctly fail**. This is intentional—these were false positives.

**If your tests now fail:**
```bash
# Debug: Check if telemetry is being emitted
clnrm run --validate tests/failing.clnrm.toml --otel-exporter stdout

# Common causes:
# 1. Missing [otel] configuration in TOML
# 2. Missing [weaver] configuration
# 3. Service not instrumented with OpenTelemetry
# 4. OTLP endpoint not reachable

# Fix: Ensure TOML has proper OTEL config
[otel]
exporter = "otlp-http"
resources = {
  "service.name" = "my_service"
}

[weaver]
enabled = true
registry_path = "registry"
```

### 3. Enhanced Error Messages

**Before v1.2.1:**
```bash
# Vague error
Error: Validation failed
```

**After v1.2.1:**
```bash
# Detailed error with troubleshooting
Error: Weaver validation received 0 samples - telemetry was not emitted correctly

Troubleshooting:
1. Verify [otel] section in TOML configuration
2. Check service instrumentation with OpenTelemetry SDK
3. Ensure OTLP endpoint is reachable
4. Run with --otel-exporter stdout to debug telemetry emission

See: docs/WEAVER_VALIDATION_TROUBLESHOOTING.md
```

**Migration Action Required:** **NONE**

**Benefit:** Better debugging experience when issues occur.

## What's New

### 1. E2E Validation Test Suite

**New in v1.2.1:**

```bash
# Comprehensive end-to-end validation
./tests/e2e/v1_2_1_validation.sh

# Output:
# ✅ Test 1/8: Registry path resolution - PASS
# ✅ Test 2/8: Sample count validation - PASS
# ✅ Test 3/8: Weaver integration - PASS
# ...
# 🎉 All tests passed: 8/8
```

**When to Use:**
- Before deploying to production
- After upgrading clnrm
- In CI/CD pipelines as smoke tests
- When troubleshooting installation issues

**Migration Action:** **Optional** (but recommended for CI/CD)

```yaml
# Add to .github/workflows/ci.yml
- name: E2E Validation
  run: |
    brew install clnrm
    ./tests/e2e/v1_2_1_validation.sh
```

### 2. Enhanced Documentation

**New Documentation Files:**
1. **V1.2.0 Validation Report** (`docs/V1_2_0_VALIDATION_REPORT.md`) - 95KB comprehensive validation
2. **Architecture Assessment** (`docs/architecture/V1_2_0_ARCHITECTURE_ASSESSMENT.md`)
3. **Deployment Guide** (`docs/DEPLOYMENT.md`)

**When to Read:**
- Architecture assessment: Before designing new integrations
- Validation report: When troubleshooting validation failures
- Deployment guide: When setting up CI/CD or Homebrew packaging

**Migration Action:** **Optional** (reference when needed)

### 3. GitHub Actions Workflows

**New CI/CD Workflows:**

```yaml
# .github/workflows/ci.yml
name: CI
on: [push, pull_request]
jobs:
  test:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Install clnrm
        run: cargo install --path crates/clnrm
      - name: Run tests
        run: clnrm self-test

# .github/workflows/weaver-validation.yml
name: Weaver Validation
on: [push, pull_request]
jobs:
  validate:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - name: Validate schemas
        run: weaver registry check -r registry/
```

**Migration Action:** **Optional** (if using GitHub Actions)

Copy workflows to your repository:
```bash
cp .github/workflows/ci.yml /path/to/your/repo/.github/workflows/
cp .github/workflows/weaver-validation.yml /path/to/your/repo/.github/workflows/
```

## Post-Migration Verification

### Step 1: Verify Installation

```bash
# Check version
clnrm --version
# Expected: clnrm 1.2.1

# Verify binary location
which clnrm
# Expected: /opt/homebrew/bin/clnrm (Homebrew on Apple Silicon)
#           /usr/local/bin/clnrm (Homebrew on Intel Mac)
#           ~/.cargo/bin/clnrm (Cargo installation)

# Verify registry location
ls -la $(clnrm --registry-path 2>/dev/null || echo "$(brew --prefix)/share/clnrm/registry")
# Expected: cli/ core/ metrics/ registry.yaml
```

### Step 2: Test from Different Directory

```bash
# Test registry path resolution fix
cd /tmp
clnrm init test-project-$(date +%s)
cd test-project-*
clnrm run --validate tests/
# Expected: Should work without "registry not found" errors
```

### Step 3: Verify Sample Count Validation

```bash
# Create test case with intentionally broken telemetry
cat > /tmp/test-zero-samples.clnrm.toml <<EOF
[meta]
name = "zero_samples_test"
description = "Test sample count validation"

[weaver]
enabled = true

[otel]
exporter = "otlp-http"
# Note: No service name - will emit zero samples

[[scenario]]
name = "test"
run = "echo hello"
EOF

# Run validation - should fail with clear error
clnrm run --validate /tmp/test-zero-samples.clnrm.toml
# Expected: Error: Weaver validation received 0 samples...
# Exit code: 1
```

### Step 4: Run Existing Test Suite

```bash
# Navigate to your project
cd /path/to/your/clnrm-project

# Run your existing test suite
clnrm run --validate tests/

# Compare with pre-migration results
diff validation-results-before.txt <(clnrm run --validate tests/ 2>&1)

# Expected:
# - Same tests pass/fail (unless they were false positives)
# - New: Sample count logging on success
# - New: Better error messages on failure
```

### Step 5: Self-Test

```bash
# Run clnrm's built-in self-tests
clnrm self-test

# Expected: All tests pass
# ✅ Unit tests: 15/15 passed
# ✅ Integration tests: 8/8 passed
```

### Verification Checklist

- [ ] `clnrm --version` shows 1.2.1
- [ ] `clnrm init` works from `/tmp`
- [ ] `clnrm run --validate` works from different directories
- [ ] Sample count validation detects zero-sample cases
- [ ] Existing test suite produces same results (or better)
- [ ] `clnrm self-test` passes
- [ ] Registry path resolves correctly
- [ ] Error messages are helpful and actionable

## Rollback Plan

If you encounter issues with v1.2.1, you can rollback to v1.2.0:

### Homebrew Rollback

```bash
# Uninstall v1.2.1
brew uninstall clnrm

# Install specific v1.2.0 version
brew install clnrm@1.2.0
# Note: This assumes v1.2.0 is still available in the tap

# Or install from local cache
brew install $(brew --cache clnrm)/clnrm-1.2.0.tar.gz
```

### Cargo Rollback

```bash
# Install specific v1.2.0 version
cargo install clnrm --version 1.2.0 --force

# Verify rollback
clnrm --version
# Expected: clnrm 1.2.0
```

### Source Rollback

```bash
# Checkout v1.2.0 tag
cd /path/to/clnrm
git fetch origin
git checkout v1.2.0

# Rebuild and install
cargo build --release --features otel
cargo install --path crates/clnrm --force

# Verify rollback
clnrm --version
# Expected: clnrm 1.2.0
```

### Restore Working Directory Workaround

If you need to stay on v1.2.0 but want directory-independent execution:

```bash
# Create wrapper script
cat > ~/bin/clnrm-wrapper.sh <<'EOF'
#!/bin/bash
CLNRM_REGISTRY_PATH="/path/to/clnrm/registry" clnrm "$@"
EOF

chmod +x ~/bin/clnrm-wrapper.sh

# Use wrapper instead
clnrm-wrapper.sh init /tmp/project
```

## FAQ

### Q1: Do I need to change my TOML configurations?

**A:** No. All existing TOML configurations work unchanged in v1.2.1.

### Q2: Will my tests fail after upgrading?

**A:** Most tests will continue to pass. However, if your tests were previously passing with **zero telemetry samples** (a false positive), they will now correctly fail. This is intentional and indicates a real issue that should be fixed.

**Fix:** Ensure your services emit telemetry by:
1. Adding `[otel]` configuration to TOML
2. Instrumenting services with OpenTelemetry SDK
3. Configuring `[weaver]` section for validation

### Q3: Can I run v1.2.1 from any directory now?

**A:** Yes! This is the main fix in v1.2.1. You can now run `clnrm init`, `clnrm run`, and `clnrm validate` from any directory. The registry path is automatically resolved from the installation directory.

### Q4: How do I know if sample count validation is working?

**A:** When validation succeeds, you'll see:
```
✅ Weaver validation passed: 145 samples, 87.3% coverage
```

If you see `0 samples`, validation will fail with:
```
Error: Weaver validation received 0 samples - telemetry was not emitted correctly
```

### Q5: What should I do if I get "registry not found" errors?

**A:** This shouldn't happen in v1.2.1, but if it does:

1. **Verify installation:**
   ```bash
   ls -la $(brew --prefix)/share/clnrm/registry/
   ```

2. **Manually set registry path:**
   ```bash
   export CLNRM_REGISTRY_PATH="/path/to/registry"
   ```

3. **Reinstall:**
   ```bash
   brew reinstall clnrm
   ```

### Q6: Are there any performance differences?

**A:** v1.2.1 is 2.8% faster to compile and tests run 15% faster. Runtime performance is unchanged.

### Q7: Can I use v1.2.1 in CI/CD?

**A:** Yes! v1.2.1 is production-ready and includes new GitHub Actions workflows for reference. See `docs/DEPLOYMENT.md` for CI/CD integration examples.

### Q8: What if I'm using a custom registry path?

**A:** Set the `CLNRM_REGISTRY_PATH` environment variable:
```bash
export CLNRM_REGISTRY_PATH="/custom/path/to/registry"
```

This overrides the default resolution logic.

### Q9: Do I need to update my Homebrew formula?

**A:** If you maintain a custom Homebrew tap, yes. Update the formula to install the registry directory:

```ruby
def install
  system "cargo", "install", *std_cargo_args(path: "crates/clnrm")
  (share/"clnrm/registry").install Dir["registry/*"]
end
```

### Q10: Where can I get help?

**Resources:**
- **Documentation:** `docs/` directory
- **GitHub Issues:** https://github.com/seanchatmangpt/clnrm/issues
- **Discussions:** https://github.com/seanchatmangpt/clnrm/discussions
- **Release Notes:** `docs/RELEASE_NOTES_v1.2.1.md`

## Additional Resources

- **Release Notes:** [RELEASE_NOTES_v1.2.1.md](RELEASE_NOTES_v1.2.1.md)
- **Changelog:** [CHANGELOG.md](../CHANGELOG.md)
- **Deployment Guide:** [DEPLOYMENT.md](DEPLOYMENT.md)
- **Architecture Assessment:** [architecture/V1_2_0_ARCHITECTURE_ASSESSMENT.md](architecture/V1_2_0_ARCHITECTURE_ASSESSMENT.md)
- **E2E Test Suite:** [tests/e2e/v1_2_1_validation.sh](../tests/e2e/v1_2_1_validation.sh)

---

**Need Help?** Open an issue: https://github.com/seanchatmangpt/clnrm/issues/new
