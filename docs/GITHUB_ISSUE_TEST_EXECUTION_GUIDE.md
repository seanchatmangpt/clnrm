# GitHub Issue Validation - Test Execution Guide

## Quick Start

This guide provides step-by-step instructions for running the comprehensive GitHub issue validation test suite.

---

## Prerequisites

### 1. Docker Must Be Running

```bash
# Verify Docker is running
docker ps

# Expected output: Container list or empty list (no errors)
```

### 2. Build the Project

```bash
# Clean build with OTEL features
cargo clean
cargo build --release --features otel

# Or build just for testing
cargo build --tests
```

---

## Running the Test Suite

### Option 1: Run All GitHub Issue Validation Tests

```bash
cd /Users/sac/clnrm
cargo test github_issue_validation
```

**Expected:** All 25+ tests should pass

### Option 2: Run Tests by Issue Number

```bash
# Issue #1: Container Isolation (5 tests)
cargo test issue_1

# Issue #2: Self-Test Command (4 tests)
cargo test issue_2

# Issue #6: Dev Watch Mode (5 tests)
cargo test issue_6
```

### Option 3: Run Specific Test with Verbose Output

```bash
# See detailed output for a single test
cargo test issue_1_containers_are_real_not_fake -- --nocapture --test-threads=1

# Or any specific test
cargo test issue_2_self_test_command_succeeds -- --nocapture
```

### Option 4: Run All Integration Tests

```bash
# Run all integration tests including GitHub issue validation
cargo test --test '*'
```

---

## Test Execution Workflow

### Step 1: Verify Environment

```bash
# Check Docker
docker ps

# Check Rust toolchain
rustc --version
cargo --version

# Check clnrm can be built
cargo check
```

### Step 2: Run Quick Smoke Test

```bash
# Run just the container isolation tests (fast)
cargo test issue_1_containers_are_real_not_fake
```

**Expected Result:**
```
test issue_1_containers_are_real_not_fake ... ok

test result: ok. 1 passed; 0 failed
```

### Step 3: Run Full Validation Suite

```bash
# Run all GitHub issue validation tests
cargo test github_issue_validation -- --test-threads=1 --nocapture
```

**Expected Result:**
```
running 25 tests
test issue_1_containers_are_real_not_fake ... ok
test issue_1_docker_container_count_increases ... ok
test issue_1_container_has_isolated_filesystem ... ok
test issue_1_alpine_package_manager_exists ... ok
test issue_1_multiple_containers_are_isolated ... ok
test issue_2_self_test_command_succeeds ... ok
test issue_2_self_test_with_suite_succeeds ... ok
test issue_2_self_test_with_invalid_suite_fails_gracefully ... ok
test issue_2_self_test_produces_real_results ... ok
test issue_6_dev_command_exists ... ok
test issue_6_dev_watch_config_creation_succeeds ... ok
test issue_6_dev_watch_supports_filter_patterns ... ok
test issue_6_dev_watch_supports_timeboxing ... ok
test issue_6_dev_watch_validates_paths_exist ... ok
[... additional tests ...]

test result: ok. 25 passed; 0 failed; 0 ignored
```

### Step 4: Verify No Regressions

```bash
# Run existing container isolation tests
cargo test container_isolation

# Run existing self-test tests
cargo test self_test
```

---

## Troubleshooting

### Problem: "Docker is not running"

**Solution:**
```bash
# macOS
open -a Docker

# Wait for Docker to start
docker ps
```

### Problem: "Compilation failed"

**Solution:**
```bash
# Clean and rebuild
cargo clean
cargo build --tests

# If still failing, check specific errors
cargo check
```

### Problem: "Test failed - container not created"

**Solution:**
```bash
# Verify Docker has images
docker images | grep alpine

# Pull Alpine if needed
docker pull alpine:latest

# Re-run test
cargo test issue_1_containers_are_real_not_fake
```

### Problem: "Test timeout"

**Solution:**
```bash
# Run with single thread to avoid resource contention
cargo test github_issue_validation -- --test-threads=1

# Or increase timeout for specific test
RUST_TEST_TIME_UNIT=60000 cargo test issue_1
```

### Problem: "Permission denied"

**Solution:**
```bash
# Check Docker permissions
docker ps

# If needed, add user to docker group (Linux)
sudo usermod -aG docker $USER

# macOS: Restart Docker Desktop
```

---

## Test Validation Checklist

Use this checklist to validate all GitHub issues are properly fixed:

### Issue #1: Container Isolation
- [ ] `issue_1_containers_are_real_not_fake` passes
- [ ] `issue_1_docker_container_count_increases` passes
- [ ] `issue_1_container_has_isolated_filesystem` passes
- [ ] `issue_1_alpine_package_manager_exists` passes
- [ ] `issue_1_multiple_containers_are_isolated` passes

**Validation Command:**
```bash
cargo test issue_1 -- --nocapture
```

### Issue #2: Self-Test Command
- [ ] `issue_2_self_test_command_succeeds` passes
- [ ] `issue_2_self_test_with_suite_succeeds` passes
- [ ] `issue_2_self_test_with_invalid_suite_fails_gracefully` passes
- [ ] `issue_2_self_test_produces_real_results` passes

**Validation Command:**
```bash
cargo test issue_2 -- --nocapture
```

### Issue #6: Dev Watch Mode
- [ ] `issue_6_dev_command_exists` passes
- [ ] `issue_6_dev_watch_config_creation_succeeds` passes
- [ ] `issue_6_dev_watch_supports_filter_patterns` passes
- [ ] `issue_6_dev_watch_supports_timeboxing` passes
- [ ] `issue_6_dev_watch_validates_paths_exist` passes

**Validation Command:**
```bash
cargo test issue_6 -- --nocapture
```

### Additional Validation
- [ ] CleanroomEnvironment tests pass
- [ ] Backend trait tests pass
- [ ] Error handling tests pass
- [ ] Integration test passes

**Validation Command:**
```bash
cargo test cleanroom_environment
cargo test backend_trait
cargo test error_handling
cargo test integration_all_critical_features_work
```

---

## Continuous Integration

### GitHub Actions Workflow

```yaml
# Add to .github/workflows/test.yml

- name: Run GitHub Issue Validation Tests
  run: |
    cargo test github_issue_validation -- --test-threads=1 --nocapture
```

### Local Pre-Commit Hook

```bash
#!/bin/bash
# .git/hooks/pre-commit

echo "Running GitHub issue validation tests..."
cargo test github_issue_validation

if [ $? -ne 0 ]; then
    echo "❌ GitHub issue validation tests failed!"
    exit 1
fi

echo "✅ All GitHub issue validation tests passed!"
```

---

## Test Execution Times

Expected execution times for test suite:

| Test Category | Test Count | Expected Time |
|---------------|------------|---------------|
| Issue #1 (Container Isolation) | 5 | ~10-15 seconds |
| Issue #2 (Self-Test) | 4 | ~5-8 seconds |
| Issue #6 (Dev Watch) | 5 | ~2-3 seconds |
| Additional Validation | 11 | ~5-10 seconds |
| **TOTAL** | **25** | **~25-40 seconds** |

---

## Success Criteria

**All tests MUST pass** before declaring issues as fixed:

```bash
# Run complete validation
cargo test github_issue_validation

# Expected final output:
# test result: ok. 25 passed; 0 failed; 0 ignored; 0 measured
```

---

## Additional Commands

### Generate Test Coverage Report

```bash
# Install tarpaulin if needed
cargo install cargo-tarpaulin

# Generate coverage for GitHub issue tests
cargo tarpaulin --out Html --output-dir coverage -- github_issue_validation
```

### Run Tests with Profiling

```bash
# Install flamegraph if needed
cargo install flamegraph

# Profile test execution
cargo flamegraph --test github_issue_validation
```

### Run Tests in Watch Mode

```bash
# Install cargo-watch if needed
cargo install cargo-watch

# Auto-run tests on file changes
cargo watch -x "test github_issue_validation"
```

---

## Support

If tests fail unexpectedly:

1. Check Docker is running: `docker ps`
2. Check Rust version: `rustc --version` (need 1.70+)
3. Clean and rebuild: `cargo clean && cargo build --tests`
4. Run single test with verbose: `cargo test <test_name> -- --nocapture`
5. Check GitHub Issues: https://github.com/seanchatmangpt/clnrm/issues

---

## Summary

**Quick validation of all GitHub issue fixes:**

```bash
# One command to rule them all
cargo test github_issue_validation && echo "✅ ALL GITHUB ISSUES VALIDATED!"
```

**Expected result:** All 25+ tests pass, proving:
- ✅ Containers are real, not fake
- ✅ Self-test command works correctly
- ✅ Dev watch mode is fully functional
- ✅ Error handling follows best practices
- ✅ Integration workflow succeeds

**No fake tests. No fake validation. Only real proof.**
