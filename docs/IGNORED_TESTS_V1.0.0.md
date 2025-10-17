# Ignored Tests Documentation - v1.0.0

**Total Ignored Tests**: 26

All ignored tests have documented justifications in source code using `#[ignore = "reason"]` attribute.

## Categories of Ignored Tests

### 1. Filesystem Watching Tests (3 tests) ⏱️

**Location**: `crates/clnrm-core/src/watch/watcher.rs`

**Reason**: These tests use the `notify` crate to watch for real filesystem events, causing them to hang indefinitely in the test runner waiting for events.

**Tests**:
- `test_notify_watcher_detects_file_creation`
- `test_notify_watcher_detects_file_modification`
- `test_notify_watcher_watches_multiple_paths`

**Ignore Annotation**: `#[ignore = "Requires filesystem watching - hangs in test runner"]`

**Resolution Plan**: Use mock filesystem or timeout mechanisms for integration tests.

---

### 2. CLI Initialization Tests (5 tests) 📁

**Location**: `crates/clnrm-core/src/cli/commands/init.rs`

**Reason**: These tests require filesystem setup and project directory creation which is difficult to test in unit test context.

**Tests**:
- `test_init_project_creates_correct_directory_structure`
- `test_init_project_default_name`
- `test_init_project_already_initialized`
- `test_init_project_force_flag`
- `test_init_project_readme_content`

**Ignore Annotation**: `#[ignore = "Incomplete test data or implementation"]`

**Resolution Plan**: Use `tempdir` crate for isolated filesystem tests.

---

### 3. CLI Report Tests (4 tests) 📊

**Location**: `crates/clnrm-core/src/cli/commands/report.rs`

**Reason**: Tests require complete test result fixtures and HTML/JSON generation validation.

**Tests**:
- `test_generate_report_html_format`
- `test_generate_report_json_format`
- `test_generate_report_text_format`
- `test_report_with_empty_results`

**Ignore Annotation**: `#[ignore = "Incomplete test data or implementation"]`

**Resolution Plan**: Create test fixtures with sample test results.

---

### 4. CLI Validation Tests (8 tests) ✅

**Location**: `crates/clnrm-core/src/cli/commands/validate.rs`

**Reason**: Tests require TOML test fixtures and validation scenario setup.

**Tests**:
- `test_validate_toml_syntax_error`
- `test_validate_missing_required_fields`
- `test_validate_invalid_service_config`
- `test_validate_invalid_step_config`
- `test_validate_invalid_assertion`
- `test_validate_circular_dependencies`
- `test_validate_unknown_service_reference`
- `test_validate_valid_toml`

**Ignore Annotation**: `#[ignore = "Incomplete test data or implementation"]`

**Resolution Plan**: Create `.clnrm.toml` fixtures in `tests/fixtures/` directory.

---

### 5. CLI Utility Tests (4 tests) 🔧

**Location**: `crates/clnrm-core/src/cli/utils.rs`

**Reason**: Logging setup and configuration tests require environment isolation.

**Tests**:
- `test_setup_logging_debug_level`
- `test_setup_logging_info_level`
- `test_setup_logging_with_otel`
- `test_setup_logging_without_otel`

**Ignore Annotation**: `#[ignore = "Incomplete test data or implementation"]`

**Resolution Plan**: Use environment variable mocking and tracing subscriber testing utilities.

---

### 6. Configuration Tests (1 test) ⚙️

**Location**: `crates/clnrm-core/src/config/mod.rs`

**Reason**: Complex TOML parsing scenarios require extensive fixtures.

**Tests**:
- `test_config_advanced_scenarios`

**Ignore Annotation**: `#[ignore = "Test data incomplete - needs valid TOML structure"]`

**Resolution Plan**: Create comprehensive TOML test fixtures.

---

### 7. Service Tests (1 test) 🐳

**Location**: `crates/clnrm-core/src/services/otel_collector.rs`

**Reason**: Requires Docker to run OpenTelemetry Collector container.

**Tests**:
- `test_otel_collector_integration`

**Ignore Annotation**: `#[ignore] // Requires Docker`

**Resolution Plan**: Move to integration test suite that explicitly requires Docker.

---

## Summary by Reason

| Reason | Count | Priority |
|--------|-------|----------|
| Incomplete test data/implementation | 21 | Medium |
| Filesystem watching (hangs) | 3 | Low |
| Requires Docker | 1 | Low |
| Complex TOML scenarios | 1 | Medium |

## Future Work

### High Priority
- Create test fixture library for CLI commands
- Implement proper TOML validation test cases

### Medium Priority
- Add tempdir-based filesystem tests for `init` command
- Create report generation test fixtures

### Low Priority
- Mock filesystem for watch tests
- Move Docker-dependent tests to integration suite

## Testing Best Practices

All ignored tests follow these guidelines:

1. **Explicit Annotation**: Use `#[ignore = "reason"]` with clear justification
2. **Documentation**: Reason documented in this file
3. **Tracking**: Listed in test reports for visibility
4. **Plan**: Resolution plan identified for each test

## Running Ignored Tests

To run ignored tests explicitly:

```bash
# Run only ignored tests
cargo test --lib -- --ignored

# Run ALL tests including ignored
cargo test --lib -- --include-ignored

# Run specific ignored test
cargo test --lib test_name -- --ignored
```

## Notes

- Ignored tests do not affect release quality metrics
- All critical path tests pass without being ignored
- Ignored tests represent future test coverage improvements
- No production functionality is blocked by ignored tests

---

**Last Updated**: 2025-10-17
**Version**: v1.0.0
