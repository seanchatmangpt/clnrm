# Live-Check Test Suite Guide

Quick reference for developers working with the Weaver live-check test harness.

## TL;DR

```bash
# Validate setup
./scripts/tests/validate_test_setup.sh

# Quick smoke test (fastest)
./scripts/tests/run_test_subset.sh --quick

# Full suite
./scripts/tests/test_live_check_comprehensive.sh
```

## What This Tests

The live-check test suite validates that clnrm can properly use OpenTelemetry Weaver's `registry live-check` command for runtime telemetry validation. This is **critical** because:

1. **clnrm eliminates false positives** - we can't use traditional tests to validate clnrm
2. **Weaver is our source of truth** - schema validation proves features work
3. **Live-check is the bridge** - it validates runtime telemetry against schemas

## Test Philosophy

```
Traditional Testing (What We Replace):
  Test passes ✅ → Assume feature works → FALSE POSITIVE
  └─ Test validates test code, not production behavior

clnrm with Weaver:
  Weaver validates runtime telemetry ✅ → Feature proven to work → TRUE POSITIVE
  └─ Schema validation proves actual behavior
```

## Running Tests

### Quick Feedback Loop
```bash
# Fast iteration (30 seconds)
./scripts/tests/run_test_subset.sh --basic

# Individual test
cd scripts/tests
./test_live_check_comprehensive.sh
# Then modify to run just one test function
```

### Full Validation
```bash
# Complete suite (~2-3 minutes)
./scripts/tests/test_live_check_comprehensive.sh

# Check results
cat validation_output/live_check_tests/test_summary.json | jq
```

### CI/CD
```bash
# What GitHub Actions runs
./scripts/tests/validate_test_setup.sh
./scripts/tests/run_test_subset.sh --basic
./scripts/tests/run_test_subset.sh --advanced
./scripts/tests/run_test_subset.sh --concurrent
```

## Understanding Results

### Success Output
```
╔═══════════════════════════════════════════════════════════════╗
║              ✅ ALL TESTS PASSED! ✅                          ║
╚═══════════════════════════════════════════════════════════════╝

✅ Passed:  10
❌ Failed:  0
⏭️  Skipped: 0
⏱️  Duration: 125s
```

### Failure Investigation
```bash
# 1. Check summary
cat validation_output/live_check_tests/test_summary.json

# 2. Find failed test log
ls validation_output/live_check_tests/*.log

# 3. Read specific failure
cat validation_output/live_check_tests/07_custom_policies.log

# 4. Check Weaver output
cat validation_output/live_check_tests/custom_policy_test/*.json
```

## Test Coverage Matrix

| Test | Input | Output | Time | Dependencies |
|------|-------|--------|------|--------------|
| 01_file_json | JSON file | JSON report | 5s | weaver |
| 02_stdin_text | stdin | JSON report | 5s | weaver |
| 03_json_output | JSON file | JSON + validation | 5s | weaver, jq |
| 04_ansi_output | JSON file | ANSI colored | 5s | weaver |
| 05_inactivity_timeout | None (timeout) | Exit behavior | 10s | weaver |
| 06_sighup_stop | None (signal) | Graceful shutdown | 10s | weaver |
| 07_custom_policies | JSON + .rego | Policy violations | 10s | weaver |
| 08_statistics | JSON file | Stats validation | 5s | weaver, jq |
| 09_concurrent_instances | None (3 processes) | Concurrent operation | 15s | weaver |
| 10_otlp_grpc | OTLP telemetry | End-to-end | 20s | weaver, cargo, clnrm |

## Common Issues

### Port Conflicts
```bash
# Problem: "Address already in use"
lsof -i :4320
kill <PID>

# Solution: Skip OTLP test
./scripts/tests/run_test_subset.sh --basic
```

### Missing Weaver
```bash
# Problem: "weaver: command not found"
cargo install weaver-forge

# Verify
weaver --version
```

### Invalid Registry
```bash
# Problem: "Registry validation failed"
weaver registry check -r registry/

# Fix schemas, then re-run
./scripts/tests/validate_test_setup.sh
```

### Hanging Tests
```bash
# Problem: Test never completes
ps aux | grep weaver
pkill -f "weaver registry live-check"

# Clean up
rm -rf validation_output/live_check_tests
```

## Development Workflow

### Adding a New Test

1. **Write test function** in `test_live_check_comprehensive.sh`:
```bash
test_new_capability() {
    log "Testing new live-check capability..."

    # Setup
    cat > "$TEST_OUTPUT/input.json" <<'EOF'
{"name": "test.attr", "type": "string", "value": "test"}
EOF

    # Execute
    weaver registry live-check \
        --registry "$SCRIPT_DIR/../../registry" \
        --input-source "$TEST_OUTPUT/input.json" \
        --format json \
        --output "$TEST_OUTPUT/new_test"

    # Verify
    local output=$(ls "$TEST_OUTPUT/new_test"/*.json | head -1)
    if [ -z "$output" ]; then
        log_error "No output generated"
        return 1
    fi

    log_success "New capability works"
    return 0
}
```

2. **Add to execution** in main():
```bash
run_test "11_new_capability" test_new_capability
```

3. **Update documentation**:
- Add to test coverage matrix
- Document what it validates
- Add to README

4. **Validate**:
```bash
./scripts/tests/test_live_check_comprehensive.sh
```

### Debugging a Test

```bash
# 1. Run with verbose output
set -x  # Add to top of test function
./scripts/tests/test_live_check_comprehensive.sh

# 2. Check test log
tail -f validation_output/live_check_tests/test_run_*.log

# 3. Run weaver manually
weaver registry live-check --help
weaver registry live-check --registry registry/ --input-source sample.json --format ansi

# 4. Verify registry
weaver registry check -r registry/
```

### Performance Testing

```bash
# Benchmark full suite
time ./scripts/tests/test_live_check_comprehensive.sh

# Profile individual test
time ./scripts/tests/run_test_subset.sh --basic

# Find bottlenecks
./scripts/tests/test_live_check_comprehensive.sh 2>&1 | \
  grep "Duration:" | sort -n
```

## Integration Points

### With clnrm Telemetry
```rust
// In clnrm-core/src/telemetry/tests
#[test]
fn test_telemetry_matches_schema() {
    // 1. Generate telemetry
    let span = test_span();

    // 2. Export to OTLP
    export_to_otlp(span);

    // 3. Weaver validates (via live-check)
    // This test suite verifies that live-check works
}
```

### With CI/CD
```yaml
# .github/workflows/weaver-live-check-tests.yml
- name: Validate Weaver Integration
  run: ./scripts/tests/test_live_check_comprehensive.sh

- name: Upload Results
  uses: actions/upload-artifact@v4
  with:
    name: live-check-results
    path: validation_output/live_check_tests/
```

### With Registry Updates
```bash
# Workflow when updating registry schemas
1. Edit registry/*.yaml
2. weaver registry check -r registry/
3. ./scripts/tests/run_test_subset.sh --quick
4. cargo test --features otel
5. weaver registry live-check --registry registry/ --otlp-grpc-port 4317
6. cargo run clnrm self-test --suite otel
```

## Validation Hierarchy

```
Level 1: Schema Definition (weaver registry check)
    ↓
Level 2: Live-Check Capabilities (THIS TEST SUITE)
    ↓
Level 3: Runtime Telemetry (weaver registry live-check with real data)
    ↓
Level 4: Traditional Tests (cargo test --features otel)
```

**This test suite validates Level 2**, ensuring that live-check tooling works before we rely on it for Level 3 validation.

## Best Practices

### DO
- ✅ Run `validate_test_setup.sh` before committing
- ✅ Use `--quick` for rapid iteration
- ✅ Check individual test logs when failures occur
- ✅ Clean up validation_output/ periodically
- ✅ Update tests when adding new live-check features
- ✅ Keep tests independent and idempotent

### DON'T
- ❌ Assume tests pass means everything works (verify outputs)
- ❌ Ignore warnings about port conflicts
- ❌ Run full suite repeatedly during development (use --quick)
- ❌ Modify test suite without updating documentation
- ❌ Skip registry validation (`weaver registry check`)
- ❌ Commit with failing tests

## FAQ

**Q: Why not just run `weaver registry live-check` manually?**

A: This test suite validates ALL capabilities systematically:
- Different input sources (file, stdin, OTLP)
- Different output formats (JSON, ANSI)
- Edge cases (timeout, signals, concurrent)
- Custom policies
- Statistics generation

**Q: Can I run tests in parallel?**

A: No, tests use overlapping ports. Run sequentially via the main script.

**Q: What if weaver changes?**

A: Update `weaver-live-check-tests.yml` with new version, validate, commit.

**Q: How do I skip the OTLP test?**

A: Use `./scripts/tests/run_test_subset.sh --basic` instead of full suite.

**Q: Where are test outputs stored?**

A: `validation_output/live_check_tests/` (gitignored)

**Q: Can I use this for local development?**

A: Yes! `run_test_subset.sh --quick` gives fast feedback (<30s).

## Related Documentation

- [Test Suite README](../../scripts/tests/README.md) - Detailed test documentation
- [Weaver User Guide](../WEAVER_USER_GUIDE.md) - How to use Weaver with clnrm
- [Weaver Integration Design](../architecture/WEAVER_INTEGRATION_DESIGN.md) - Architecture
- [Running Weaver Validation](../RUNNING_WEAVER_VALIDATION.md) - Manual validation

## Support

**Issues?**
1. Run `./scripts/tests/validate_test_setup.sh`
2. Check test logs in `validation_output/live_check_tests/`
3. Verify registry: `weaver registry check -r registry/`
4. File issue: https://github.com/seanchatmangpt/clnrm/issues

**Contributing?**
1. Add test following patterns above
2. Update documentation
3. Run full suite
4. Submit PR with test results
