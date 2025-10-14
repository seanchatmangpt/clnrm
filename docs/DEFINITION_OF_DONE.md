# Definition of Done for Cleanroom Features

This document maps each feature from the README.md to the core team's quality standards, providing specific acceptance criteria for production readiness.

## Core Quality Standards (Apply to ALL Features)

### Universal Requirements
- [ ] Code compiles without errors or warnings (`cargo build --release`)
- [ ] All tests pass (`cargo test`)
- [ ] No linting errors (`cargo clippy`)
- [ ] Zero usage of `unwrap()` or `expect()` in production code
- [ ] All traits remain `dyn` compatible (no async trait methods)
- [ ] No breaking changes without migration plan
- [ ] All functions use `Result<T, E>` types with meaningful errors
- [ ] Proper async/await patterns (async for I/O, sync for computation)
- [ ] No `println!` or `print!` in production code (use `tracing` instead)
- [ ] Proper module organization with specific imports (no wildcard imports)
- [ ] Documentation with examples
- [ ] Framework self-testing (feature tests itself)

## Feature-Specific Definition of Done

### 1. Hermetic Isolation

**README Claims:**
- Complete isolation from host system and other tests

**Definition of Done:**
- [ ] Tests run in separate containers with no shared state
- [ ] File system isolation verified (no cross-test file access)
- [ ] Network isolation tested (no cross-test network access)
- [ ] Environment variable isolation validated
- [ ] Process isolation confirmed (container cleanup between tests)
- [ ] Integration test: `tests/integration/hermetic_isolation_test.rs`
  - Test that two concurrent tests cannot access each other's resources
  - Test that environment variables don't leak between tests
  - Test that file system changes are isolated
- [ ] TOML test: `tests/framework/hermetic_isolation.toml`
- [ ] Self-testing: Framework validates its own isolation guarantees

**Core Team Standards:**
- [ ] Error handling for isolation failures uses `CleanroomError::isolation_error()`
- [ ] Async isolation checks use proper `tokio::test`
- [ ] AAA pattern in all isolation tests

### 2. Plugin-Based Architecture

**README Claims:**
- Extensible service system for any technology
- `ServicePlugin` trait with `type()`, `start()`, `stop()`, `health_check()`

**Definition of Done:**
- [ ] `ServicePlugin` trait is `dyn` compatible (no async methods)
- [ ] All trait methods return `Result<T, CleanroomError>`
- [ ] Plugin registry supports dynamic loading
- [ ] Plugin discovery mechanism implemented
- [ ] Plugin lifecycle management (start, stop, health check)
- [ ] Multiple plugins can coexist without conflicts
- [ ] Integration test: `tests/framework/plugin_system.toml` passes
- [ ] Unit tests for plugin registration, loading, execution
- [ ] Example plugin implementation in `examples/`
- [ ] Documentation: Plugin development guide

**Core Team Standards:**
- [ ] No `unwrap()` in plugin loading/execution
- [ ] Plugin trait methods are sync (use `tokio::task::block_in_place` internally if needed)
- [ ] Plugin errors use `CleanroomError::plugin_error()` with context
- [ ] Test descriptive names: `test_plugin_loads_successfully_with_valid_config()`

### 3. Container Reuse (Singleton Pattern)

**README Claims:**
- 10-50x performance improvement through singleton containers
- Container caching for subsequent runs

**Definition of Done:**
- [ ] Container registry with singleton pattern implemented
- [ ] `get_or_create_container()` method with idempotency
- [ ] First run creates container (measured 30-60s)
- [ ] Subsequent runs reuse container (measured 2-5ms)
- [ ] Container lifecycle managed (cleanup on environment drop)
- [ ] Performance benchmarks demonstrate 10-50x improvement
- [ ] Benchmark: `benches/container_lifecycle.rs` validates performance claims
- [ ] Integration test: `tests/containers_test.rs` validates reuse behavior
- [ ] TOML test: `tests/framework/container_lifecycle.toml`
- [ ] Self-testing: Framework measures its own container reuse performance

**Core Team Standards:**
- [ ] Container handle stored in `Arc<Mutex<HashMap>>` or similar
- [ ] No `unwrap()` when accessing container registry
- [ ] Async container operations use proper `async/await`
- [ ] Error handling for container failures with context

### 4. Built-in Observability

**README Claims:**
- Automatic tracing and metrics collection
- Zero configuration required
- Performance monitoring

**Definition of Done:**
- [ ] `tracing` crate integrated for all operations
- [ ] Span creation for container lifecycle events
- [ ] Span creation for plugin operations
- [ ] Span creation for test execution
- [ ] Metrics collection (container count, execution time, etc.)
- [ ] Observability works with no user configuration
- [ ] Integration test: `tests/integration_otel.rs` passes
- [ ] TOML test validates observability captures metrics
- [ ] Example: `examples/observability_setup.rs` demonstrates usage
- [ ] Self-testing: Framework validates its own tracing/metrics

**Core Team Standards:**
- [ ] Use `tracing::info!`, `tracing::warn!`, `tracing::error!` (NO `println!`)
- [ ] Structured logging with context fields
- [ ] No logging in hot paths (performance consideration)
- [ ] Async tracing setup uses proper initialization

### 5. Professional CLI

**README Claims:**
- `clnrm init`, `clnrm run`, `clnrm report`, `clnrm services`, `clnrm validate`
- Parallel execution with `--parallel --jobs N`
- Watch mode with `--watch`
- Multiple output formats (human, junit, html)
- Interactive debugging with `--interactive`

**Definition of Done:**
- [ ] CLI binary at `src/bin/clnrm.rs` compiles
- [ ] All subcommands implemented and tested
- [ ] `clnrm --help` shows comprehensive help
- [ ] `clnrm run tests/` executes all TOML tests
- [ ] `clnrm run --parallel --jobs 4` runs tests concurrently
- [ ] `clnrm report --format junit` generates JUnit XML
- [ ] `clnrm report --format html` generates HTML report
- [ ] `clnrm services status` shows service status
- [ ] `clnrm validate` validates TOML configuration
- [ ] Integration test: `tests/framework/cli_functionality.toml`
- [ ] CLI test script: `examples/cli_bash.sh` demonstrates usage
- [ ] Documentation: `docs/CLI_GUIDE.md` complete

**Core Team Standards:**
- [ ] CLI argument parsing uses proper error handling (no `unwrap()`)
- [ ] All CLI commands return `Result<(), CleanroomError>`
- [ ] CLI uses `tracing` for output (not `println!` in logic)
- [ ] Async CLI operations properly handled

### 6. TOML Configuration System

**README Claims:**
- Declarative test definitions without code
- `[test.metadata]`, `[services]`, `[[steps]]`, `[assertions]` sections
- Configuration parsing and validation

**Definition of Done:**
- [ ] TOML parser handles all documented configuration sections
- [ ] Configuration validation with meaningful error messages
- [ ] `clnrm validate` command validates TOML files
- [ ] Schema documentation in `docs/TOML_REFERENCE.md`
- [ ] Integration test: `tests/framework/cli_functionality.toml` is valid
- [ ] Error handling for malformed TOML with user-friendly messages
- [ ] Example TOML files in `tests/` directory
- [ ] TOML deserialization uses `serde` with proper error types

**Core Team Standards:**
- [ ] TOML parsing errors use `CleanroomError::config_error()` with context
- [ ] No `unwrap()` when deserializing TOML
- [ ] Validation logic is sync (pure computation)
- [ ] Test: `test_toml_parsing_with_invalid_config_fails_gracefully()`

### 7. Regex Validation in Container Output

**README Claims:**
- Pattern matching with `expected_output_regex`
- Negative matching with `expected_output_regex_not`

**Definition of Done:**
- [ ] Regex engine integrated (`regex` crate)
- [ ] `expected_output_regex` field in TOML step configuration
- [ ] `expected_output_regex_not` field in TOML step configuration
- [ ] Regex validation executes after command completion
- [ ] Validation failures provide clear error messages
- [ ] Integration test with regex patterns in `tests/`
- [ ] TOML test: Multiple steps with regex validation
- [ ] Example: `tests/container_lifecycle.toml` uses regex validation

**Core Team Standards:**
- [ ] Regex compilation errors handled with `CleanroomError::validation_error()`
- [ ] No `unwrap()` when compiling or matching regex
- [ ] Regex matching is sync operation
- [ ] Test: `test_regex_validation_matches_expected_pattern()`

### 8. Rich Assertions Library

**README Claims:**
- Domain-specific validation helpers
- `container_should_have_executed_commands`
- `execution_should_be_hermetic`
- `plugin_should_be_loaded`
- `observability_should_capture_metrics`

**Definition of Done:**
- [ ] Assertion trait/struct implemented
- [ ] All documented assertions implemented
- [ ] Custom assertion support for plugins
- [ ] Assertions integrated with TOML `[assertions]` section
- [ ] Clear error messages when assertions fail
- [ ] Unit tests for each assertion type
- [ ] Integration test using all assertion types
- [ ] Documentation: Examples of custom assertions

**Core Team Standards:**
- [ ] Assertions return `Result<(), CleanroomError>`
- [ ] No `unwrap()` in assertion logic
- [ ] Async assertions use proper `async fn` when needed (I/O bound)
- [ ] Sync assertions for simple checks (computation bound)
- [ ] Test: `test_assertion_container_executed_commands_validates_count()`

## TOML Test Suite Requirements

All features MUST have corresponding TOML tests demonstrating self-testing:

- [ ] `tests/framework/cli_functionality.toml` - CLI feature tests
- [ ] `tests/framework/container_lifecycle.toml` - Container reuse tests
- [ ] `tests/framework/plugin_system.toml` - Plugin loading tests
- [ ] `tests/integration/end_to_end.toml` - Full workflow integration
- [ ] `tests/framework/hermetic_isolation.toml` - Isolation validation
- [ ] `tests/framework/observability.toml` - Tracing/metrics tests

## Performance Requirements

- [ ] Benchmark suite in `benches/` directory
- [ ] Container reuse benchmark shows 10-50x improvement
- [ ] Parallel execution benchmark demonstrates scaling
- [ ] Performance regression tests in CI/CD
- [ ] `cargo bench` runs without errors

## Documentation Requirements

- [ ] README.md accurately reflects implemented features
- [ ] API documentation (`cargo doc --open --no-deps`)
- [ ] CLI guide (`docs/CLI_GUIDE.md`)
- [ ] TOML reference (`docs/TOML_REFERENCE.md`)
- [ ] Examples for each major feature in `examples/`
- [ ] Contributing guide with testing requirements

## CI/CD Integration Requirements

- [ ] GitHub Actions / GitLab CI configuration examples work
- [ ] JUnit XML output format validated
- [ ] HTML report format validated
- [ ] Tests run in CI without manual intervention

## Validation Commands

Run these commands to validate Definition of Done:

```bash
# Compilation
cargo build --release

# All tests pass
cargo test

# No linting errors
cargo clippy -- -D warnings

# No unwrap/expect in production code (src/, excluding tests/)
! grep -r "\.unwrap()" src/ --include="*.rs" | grep -v "test"
! grep -r "\.expect(" src/ --include="*.rs" | grep -v "test"

# Benchmarks run
cargo bench

# Documentation builds
cargo doc --no-deps

# CLI works
cargo run -- --help
cargo run -- run tests/framework/

# Format check
cargo fmt -- --check
```

## Feature Acceptance Criteria Summary

A feature is considered **production-ready** when:

1. All universal requirements are met
2. All feature-specific requirements are met
3. Framework self-tests pass (framework tests itself)
4. Documentation is complete and accurate
5. TOML-based tests demonstrate the feature
6. Performance benchmarks validate claims
7. No breaking changes introduced
8. Migration plan exists if breaking changes are necessary

## Notes

- The core team standards emphasize that **no code is complete** until it passes ALL criteria
- Framework self-testing is mandatory - Cleanroom must test itself
- Error handling is paramount - no panics in production code
- Trait compatibility must be maintained - `dyn` compatibility required
- Testing philosophy: AAA pattern, descriptive names, proper async handling

## Implementation Status

### Completed Features
- ✅ Container Volume Mounting (src/backend/testcontainer.rs)
  - Volume mounts field updated with read-only flag
  - with_volume() method accepts read_only parameter
  - Proper volume mounting implementation using testcontainers API
  - Unit tests for volume builder functionality
  - Comprehensive validation tests for read-write, read-only, and multiple mounts
  - All tests pass and implementation follows core team standards

### In Progress Features
- 🔄 Framework self-testing implementation
- 🔄 Plugin-based architecture
- 🔄 Container reuse (singleton pattern)
- 🔄 Built-in observability
- 🔄 Professional CLI
- 🔄 TOML configuration system
- 🔄 Regex validation in container output
- 🔄 Rich assertions library

### Pending Features
- ⏳ Hermetic isolation validation
- ⏳ Performance benchmarking
- ⏳ CI/CD integration examples
- ⏳ Complete documentation suite
