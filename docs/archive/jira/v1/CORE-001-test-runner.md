# CORE-001: Test Runner (`clnrm run`)

## Feature Overview
Core test execution engine that discovers, executes, and reports on `.clnrm.toml` test files with support for parallel execution, caching, sharding, and multiple output formats.

## Status
✅ **PRODUCTION READY**

## Implementation Location
- **File**: `crates/clnrm-core/src/cli/commands/run/mod.rs`
- **CLI**: `clnrm run [paths] [flags]`
- **Dependencies**:
  - `executor.rs` (test execution logic)
  - `cache.rs` (incremental testing)
  - `src/cleanroom.rs` (CleanroomEnvironment)

## Acceptance Criteria

### ✅ Test Discovery
- [x] Auto-discovers `.clnrm.toml` files in specified paths
- [x] Supports single file execution
- [x] Supports directory recursion
- [x] Validates file existence before execution

### ✅ Execution Modes
- [x] Sequential execution (default)
- [x] Parallel execution with `--parallel` flag
- [x] Configurable parallelism via `-j <N>` (defaults to CPU count)
- [x] Fail-fast mode (`--fail-fast`) stops on first failure

### ✅ Incremental Testing
- [x] Cache-based test skipping (skips unchanged tests)
- [x] Force re-run with `--force` flag
- [x] SHA-256 digest tracking for cache invalidation

### ✅ CI/CD Integration
- [x] Test sharding support (`--shard i/m` format like `1/4`)
- [x] JUnit XML report generation (`--report-junit <file>`)
- [x] Multiple output formats: auto, human, json, junit, tap
- [x] Exit code 0 for success, non-zero for failure

### ✅ Watch Mode
- [x] File change detection (`--watch`)
- [x] Automatic re-execution on changes
- [x] Debounced execution

### ⚠️ Interactive Mode (Partial)
- [x] Flag exists (`--interactive`)
- [ ] Full TUI implementation (shows warning currently)

### ✅ Reproducibility
- [x] Digest generation (`--digest` flag)
- [x] Deterministic execution when seed configured

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
- [x] Unit tests: 15+ tests in inline `#[cfg(test)]` modules
- [x] Integration tests:
  - `tests/run_tests_sequential_with_results_empty_paths` ✅
  - `tests/run_tests_parallel_with_results_empty_paths` ✅
- [x] Edge case coverage:
  - Empty paths
  - Invalid paths
  - Non-existent files
  - Parallel execution
  - Cache behavior

### Documentation
- [x] Inline rustdoc comments
- [x] CLI help text (`clnrm run --help`)
- [x] Usage examples in code comments
- [x] User guide documentation

### Validation Testing
```bash
# Test discovery
clnrm run tests/

# Single file
clnrm run tests/example.clnrm.toml

# Parallel execution
clnrm run tests/ --parallel -j 4

# Fail-fast mode
clnrm run tests/ --fail-fast

# Test sharding for CI
clnrm run tests/ --shard 1/4
clnrm run tests/ --shard 2/4

# JUnit XML report
clnrm run tests/ --report-junit results.xml

# JSON output
clnrm run tests/ --format json

# Watch mode
clnrm run tests/ --watch

# Force re-run (bypass cache)
clnrm run tests/ --force

# Digest generation
clnrm run tests/ --digest
```

## Known Limitations
- ⚠️ Interactive mode (`--interactive`) not fully implemented - shows warning
- ✅ All other advertised features are production-ready

## Performance Targets
- ✅ Single test execution: <1s overhead
- ✅ Parallel execution: Near-linear scaling up to CPU count
- ✅ Cache hit: <50ms overhead

## Dependencies
- Testcontainers-rs: Container orchestration
- Tokio: Async runtime
- Rayon: Parallel execution
- SHA-256: Cache key generation

## Related Tickets
- CORE-002: Framework Self-Test
- CORE-003: Configuration Validation
- OTEL-001: OpenTelemetry Integration (blocked)

## Verification Commands
```bash
# Build verification
cargo build --release

# Test verification
cargo test --lib run

# Integration test verification
cargo test --test integration

# Clippy verification
cargo clippy --package clnrm-core -- -D warnings

# Production validation (MUST use Homebrew installation)
brew install --build-from-source .
clnrm run tests/
```

## Release Notes (v1.0.0)
- ✅ Production-ready test runner with parallel execution
- ✅ Incremental testing with cache support
- ✅ CI/CD integration with sharding and JUnit XML
- ✅ Watch mode for development workflows
- ⚠️ Interactive mode to be completed in v1.1.0

---

**Last Updated**: 2025-10-17
**Status**: ✅ PRODUCTION READY
**Blocker**: None
**Next Steps**: Complete interactive mode in v1.1.0
