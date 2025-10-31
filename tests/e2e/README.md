# End-to-End Test Suite

Comprehensive E2E validation tests for clnrm releases.

## Test Scripts

### v1_2_1_validation.sh

**Purpose:** Validates v1.2.1 registry path resolution and sample validation fixes.

**Test Coverage:**
1. **Registry Path Resolution (Project Root)** - Verifies auto-detection from project directory
2. **Registry Path Resolution (Non-Project)** - Tests behavior from arbitrary directories
3. **Explicit --registry-path Flag** - Validates manual registry path specification
4. **Sample Count Validation** - Checks telemetry sample reporting
5. **Weaver Live-Check Integration** - Validates Weaver schema conformance
6. **OTLP Export Verification** - Tests OpenTelemetry Protocol export
7. **Error Handling** - Validates proper errors for invalid paths
8. **Project Integration** - Tests with actual clnrm project setup

**Usage:**

```bash
# Run from project root
./tests/e2e/v1_2_1_validation.sh

# Run from any directory
bash /path/to/clnrm/tests/e2e/v1_2_1_validation.sh
```

**Prerequisites:**
- `clnrm` installed (via Homebrew or cargo install)
- `docker` (optional, for OTLP tests)
- `jq` (optional, for JSON validation)
- `weaver` (optional, for Weaver integration tests)

**Output:**

The script provides:
- ✅ Real-time test results with colored output
- 📊 Test summary with pass/fail counts
- 🔍 Detailed error messages for failures
- ⚠️  Warnings for skipped tests (missing prerequisites)

**Exit Codes:**
- `0` - All tests passed
- `1` - One or more tests failed

## Running Tests

### Quick Validation

```bash
# Just run the v1.2.1 validation
./tests/e2e/v1_2_1_validation.sh
```

### Full Prerequisites Check

```bash
# Check if all prerequisites are met
which clnrm docker jq weaver

# Install missing tools
cargo install --git https://github.com/open-telemetry/weaver weaver
brew install jq docker
```

### CI/CD Integration

```bash
# In CI pipeline
set -e
./tests/e2e/v1_2_1_validation.sh
echo "E2E validation passed"
```

### Debug Mode

```bash
# Run with bash debug output
bash -x ./tests/e2e/v1_2_1_validation.sh

# Check specific test
grep -A 20 "Test 2:" ./tests/e2e/v1_2_1_validation.sh
```

## Test Architecture

### Test Structure

Each E2E test follows this pattern:

```bash
# Banner and setup
log_info "Test X: Description"

# Execute test
if [test condition]; then
    log_success "Test passed"
else
    log_error "Test failed"
fi

# Cleanup (automatic via trap)
```

### Helper Functions

- `log_info(msg)` - Blue informational messages
- `log_success(msg)` - Green success messages (increments pass count)
- `log_error(msg)` - Red error messages (increments fail count)
- `log_warn(msg)` - Yellow warning messages
- `cleanup()` - Automatic cleanup on exit

### Isolated Test Environment

Tests run in isolated `/tmp` directories:
- Creates: `/tmp/clnrm-v1.2.1-e2e-test-$$`
- Auto-cleanup: Via `trap cleanup EXIT`
- No side effects: Original directory restored

## Test Scenarios Explained

### Scenario 1: Project Root Resolution

**What it tests:** Auto-detection of `registry/` from project root

**Expected behavior:**
```bash
cd /path/to/clnrm
clnrm run --validate  # Should find registry/ automatically
```

**Why it matters:** Most common use case for developers

---

### Scenario 2: Non-Project Directory

**What it tests:** Behavior when running from outside project

**Expected behavior:**
```bash
cd /tmp/random-dir
clnrm init  # Should work
# Registry lookup should handle gracefully
```

**Why it matters:** Users may run clnrm from any directory

---

### Scenario 3: Explicit Registry Path

**What it tests:** Manual registry specification works

**Expected behavior:**
```bash
clnrm run --validate --registry-path /custom/path/registry
```

**Why it matters:** Advanced users may have custom registry locations

---

### Scenario 4: Sample Validation

**What it tests:** Telemetry sample counting and reporting

**Expected behavior:**
```bash
clnrm run tests/ 2>&1 | grep "samples"
# Output: "Weaver received 42 samples"
```

**Why it matters:** Users need feedback on telemetry generation

---

### Scenario 5: Weaver Integration

**What it tests:** Schema validation via Weaver live-check

**Expected behavior:**
```bash
clnrm run --validate tests/
# Generates: validation_output/report.json
# Contains: sample_count, validation results
```

**Why it matters:** Core clnrm principle - Weaver as source of truth

---

### Scenario 6: OTLP Export

**What it tests:** OpenTelemetry Protocol export to collector

**Expected behavior:**
```bash
# With collector running
clnrm run tests/
# Telemetry exported to OTLP endpoint
```

**Why it matters:** Production observability integration

---

### Scenario 7: Error Handling

**What it tests:** Proper errors for invalid configurations

**Expected behavior:**
```bash
clnrm run --registry-path /invalid/path
# Error: Registry not found at /invalid/path
```

**Why it matters:** User experience and debugging

---

### Scenario 8: Project Integration

**What it tests:** Real-world usage with actual project

**Expected behavior:**
```bash
cd /path/to/clnrm
clnrm self-test --verbose
# Uses project registry/, runs validation
```

**Why it matters:** Dogfooding - clnrm tests itself

## Interpreting Results

### Success Output

```
╔════════════════════════════════════════════════════════╗
║  ✅ ALL TESTS PASSED - v1.2.1 validation successful  ║
╚════════════════════════════════════════════════════════╝

Key validations:
  ✓ Registry path resolution working
  ✓ Sample validation output functional
  ✓ Weaver integration ready
  ✓ Error handling robust
```

**Meaning:** v1.2.1 release is ready for production

---

### Partial Success

```
Total tests:  8
Passed:       6
Failed:       0
Warnings:     2  ⚠️  Docker not available
                  ⚠️  Weaver not installed
```

**Meaning:** Core functionality works, optional features skipped

**Action:** Install missing tools for full validation

---

### Failure Output

```
❌ Test 2: Registry path resolution from non-project directory
   Error: Registry not found

╔════════════════════════════════════════════════════════╗
║  ❌ SOME TESTS FAILED - review output above           ║
╚════════════════════════════════════════════════════════╝
```

**Meaning:** Critical bug found, release not ready

**Action:** Fix issue before release

## Adding New Tests

### Template for New E2E Test

```bash
#!/bin/bash
set -e

# Copy helper functions from v1_2_1_validation.sh

echo "🧪 New Feature E2E Test"

# Test 1: Core functionality
log_info "Test 1: Core feature"
if [test condition]; then
    log_success "Test passed"
else
    log_error "Test failed"
fi

# Summary
echo "Tests passed: $TESTS_PASSED / $TOTAL_TESTS"
```

### Checklist for New Tests

- [ ] Isolated test environment (`/tmp/test-$$`)
- [ ] Auto-cleanup via `trap cleanup EXIT`
- [ ] Colored output (green/red/yellow/blue)
- [ ] Pass/fail tracking
- [ ] Prerequisites check
- [ ] Clear test descriptions
- [ ] Meaningful exit codes
- [ ] Integration with CI/CD

## Troubleshooting

### Test Hangs

**Symptom:** Test doesn't complete

**Solution:**
```bash
# Add timeout to problematic command
timeout 30s clnrm run tests/
```

---

### Permission Denied

**Symptom:** Cannot execute script

**Solution:**
```bash
chmod +x ./tests/e2e/*.sh
```

---

### Docker Not Available

**Symptom:** Docker tests skipped

**Solution:**
```bash
# Install Docker
brew install docker

# Or skip Docker tests
# Tests automatically skip if Docker unavailable
```

---

### Weaver Not Found

**Symptom:** Weaver tests skipped

**Solution:**
```bash
# Install Weaver
cargo install --git https://github.com/open-telemetry/weaver weaver

# Verify installation
weaver --version
```

---

### Tests Pass But Feature Broken

**Symptom:** Tests pass, but manual testing fails

**Solution:** This is exactly what clnrm prevents!
```bash
# Always validate with Weaver live-check
weaver registry live-check --registry registry/

# Weaver validation is the source of truth
# Tests can have false positives, Weaver cannot
```

## Best Practices

1. **Run Before Release** - Always run E2E suite before version bump
2. **Check Prerequisites** - Ensure Docker, Weaver, jq installed for full coverage
3. **Read Warnings** - Warnings indicate skipped tests, not failures
4. **Validate with Weaver** - E2E tests support Weaver validation, not replace it
5. **Add Tests for Bugs** - Each bug fix should get an E2E test
6. **Keep Tests Fast** - Target <30 seconds total execution
7. **Use in CI/CD** - Integrate into GitHub Actions workflow
8. **Document Changes** - Update this README when adding tests

## Related Documentation

- [Testing Guide](/docs/TESTING.md) - Overall testing strategy
- [Weaver Integration](/docs/WEAVER_INTEGRATION.md) - Schema validation details
- [Production Validation](/docs/PRODUCTION_VALIDATION_GUIDE.md) - Release checklist
- [CI/CD Integration](/docs/CICD_INTEGRATION.md) - Automated testing

## Version History

### v1.2.1 (Current)
- Initial E2E test suite
- Registry path validation
- Sample count verification
- Weaver integration tests
- OTLP export verification
- Error handling tests

### Future Enhancements
- [ ] Performance regression tests
- [ ] Multi-platform validation (Linux/macOS/Windows)
- [ ] Container orchestration tests (Docker Compose)
- [ ] Plugin system validation
- [ ] Concurrent test execution
- [ ] Memory leak detection
- [ ] Security vulnerability scanning
