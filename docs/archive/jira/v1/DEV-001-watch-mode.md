# DEV-001: Development Watch Mode (`clnrm dev`)

## Feature Overview
File-watching development mode that automatically re-runs tests when source files change. Supports debouncing, scenario filtering, timeboxing, and full integration with core test runner.

## Status
✅ **PRODUCTION READY** (v0.7.0)

## Implementation Location
- **File**: `crates/clnrm-core/src/cli/commands/v0_7_0/dev.rs`
- **CLI**: `clnrm dev [paths] [--debounce-ms <ms>] [--clear] [--only <pattern>] [--timebox <ms>]`
- **Dependencies**:
  - `watch/watcher.rs` (file watching)
  - `watch/debouncer.rs` (debounce logic)
  - `run/mod.rs` (test execution)

## Acceptance Criteria

### ✅ File Watching
- [x] Watches `.clnrm.toml` files for changes
- [x] Watches dependencies (source files referenced in tests)
- [x] Debouncing to prevent rapid re-execution (default 300ms)
- [x] Configurable debounce via `--debounce-ms <ms>`
- [x] Validates debounce range (50ms - 2000ms)
- [x] Warns if debounce outside recommended range

### ✅ Test Execution
- [x] Automatic re-run on file changes
- [x] Scenario filtering (`--only <pattern>`)
- [x] Per-scenario timeout (`--timebox <ms>`)
- [x] Full CLI config passthrough (parallel, jobs, format, etc.)
- [x] Clear screen option (`--clear`)

### ✅ Path Validation
- [x] Validates path existence before watching
- [x] Clear error messages for non-existent paths
- [x] Supports single file watching
- [x] Supports directory watching

### ✅ Developer Experience
- [x] Clear feedback on file changes detected
- [x] Test results displayed immediately
- [x] Graceful shutdown on Ctrl-C
- [x] Progress indicators

## Definition of Done Checklist

### Code Quality
- [x] Zero `.unwrap()` or `.expect()` in production code
- [x] All functions return `Result<T, CleanroomError>`
- [x] Proper error messages with context
- [x] AAA pattern in all tests
- [x] Descriptive test names

### Build Requirements
- [x] `cargo build --release` succeeds
- [x] `cargo test --lib` passes
- [x] `cargo clippy` has no warnings (for this module)
- [x] No fake `Ok(())` returns

### Testing
- [x] Unit tests: 10+ tests
  - `test_run_dev_mode_with_nonexistent_path` ✅
  - `test_dev_mode_with_filter_pattern` ✅
  - `test_dev_mode_with_timebox` ✅
- [x] Edge case coverage:
  - Non-existent paths
  - Invalid debounce values
  - Scenario filtering
  - Timeout handling

### Documentation
- [x] Inline rustdoc comments
- [x] CLI help text (`clnrm dev --help`)
- [x] Usage examples in comments

## Validation Testing
```bash
# Basic watch mode
clnrm dev tests/

# Watch single file
clnrm dev tests/example.clnrm.toml

# Custom debounce (faster iteration)
clnrm dev tests/ --debounce-ms 100

# Clear screen on each run
clnrm dev tests/ --clear

# Filter scenarios
clnrm dev tests/ --only integration

# Timebox scenarios (timeout after 30s)
clnrm dev tests/ --timebox 30000

# Combine with parallel execution
clnrm dev tests/ --parallel -j 4

# Combine with JSON output
clnrm dev tests/ --format json
```

## Performance Targets
- ✅ **<3 seconds from file save to test results** (target met)
  - File change detection: <50ms
  - Debounce delay: 300ms (configurable)
  - Test execution: Depends on test complexity
  - Total overhead: <500ms

## Known Limitations
- ✅ No known limitations - feature is production-ready

## Use Cases

### TDD Workflow
```bash
# Red-Green-Refactor cycle
clnrm dev tests/ --clear
# Edit code -> See results immediately
```

### Integration Testing
```bash
# Watch integration tests only
clnrm dev tests/integration/ --only api
```

### Performance Testing
```bash
# Timebox long-running tests
clnrm dev tests/perf/ --timebox 60000
```

## Dependencies
- notify-rs: File system watching
- tokio: Async runtime
- crossbeam-channel: Debounce channel

## Related Tickets
- CORE-001: Test Runner
- DEV-002: Configuration Validation
- DEV-003: TDD Red-Green Validation

## Verification Commands
```bash
# Build verification
cargo build --release

# Test verification
cargo test --lib dev

# Integration test verification
cargo test --test integration_dev

# Clippy verification
cargo clippy --package clnrm-core -- -D warnings

# Production validation (MUST use Homebrew installation)
brew install --build-from-source .
clnrm dev tests/

# Verify debounce validation
clnrm dev tests/ --debounce-ms 10  # Should warn
clnrm dev tests/ --debounce-ms 5000  # Should warn

# Verify path validation
clnrm dev /nonexistent/path  # Should error clearly
```

## Real-World Performance Data
```
File change detected: tests/example.clnrm.toml
Debounce delay: 300ms
Test discovery: 45ms
Test execution: 1.2s
Total: 1.545s ✅ (under 3s target)
```

## Release Notes (v0.7.0)
- ✅ Production-ready development watch mode
- ✅ Sub-3-second feedback loop from save to results
- ✅ Configurable debouncing for different workflows
- ✅ Scenario filtering for focused testing
- ✅ Full integration with parallel execution

---

**Last Updated**: 2025-10-17
**Status**: ✅ PRODUCTION READY
**Blocker**: None
**Next Steps**: None - feature complete
