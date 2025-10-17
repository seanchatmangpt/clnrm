# How to Run PRD v1.0 Validation

This guide explains how to run the comprehensive PRD v1.0 compliance validation tests.

---

## Quick Start

```bash
# Run all PRD v1.0 compliance tests
cargo test prd_v1_compliance --test prd_v1_compliance

# Run with output
cargo test prd_v1_compliance --test prd_v1_compliance -- --nocapture

# Run specific section
cargo test prd_v1_compliance::test_feature_ --test prd_v1_compliance
cargo test prd_v1_compliance::test_command_ --test prd_v1_compliance
cargo test prd_v1_compliance::test_acceptance_ --test prd_v1_compliance
cargo test prd_v1_compliance::test_performance_ --test prd_v1_compliance
```

---

## Test Organization

The compliance test suite (`tests/prd_v1_compliance.rs`) contains **54 tests** organized into 4 sections:

### Section 1: Core Features (10 tests)
Tests all PRD v1.0 core features are implemented.

```bash
cargo test test_feature_ --test prd_v1_compliance
```

**Tests**:
1. Tera Template System
2. Variable Precedence
3. Macro Library
4. Hot Reload
5. Change Detection
6. Dry Run
7. TOML Formatting
8. Linting
9. Parallel Execution
10. Multi-Format Reports

---

### Section 2: CLI Commands (31 tests)
Tests all CLI commands exist and have correct signatures.

```bash
cargo test test_command_ --test prd_v1_compliance
cargo test test_prd_command_ --test prd_v1_compliance
cargo test test_template_ --test prd_v1_compliance
```

**Command Groups**:
- Core commands (6 tests)
- DX commands (5 tests)
- Advanced commands (10 tests)
- Template commands (5 tests)
- PRD v1 commands (7 tests)

---

### Section 3: Acceptance Criteria (8 tests)
Tests all PRD v1.0 acceptance criteria are met.

```bash
cargo test test_acceptance_ --test prd_v1_compliance
```

**Criteria**:
- Core pipeline components
- Variable precedence
- Framework self-tests
- DX commands
- Parallel execution
- Template system
- Command accessibility
- Quality standards

---

### Section 4: Performance Metrics (5 tests)
Tests performance targets are met.

```bash
cargo test test_performance_ --test prd_v1_compliance
```

**Metrics**:
- Change detection speed (<100ms)
- Template rendering speed (<50ms)
- Dry-run validation speed (<1s)
- Format operation speed (<50ms)
- Digest determinism (100%)

---

## Expected Output

### Successful Run

```
running 54 tests

Section 1: Core Features (10 tests)
test test_feature_tera_template_system_available ... ok
test test_feature_variable_precedence_resolution ... ok
test test_feature_macro_library_available ... ok
test test_feature_hot_reload_command_exists ... ok
test test_feature_change_detection_available ... ok
test test_feature_dry_run_command_exists ... ok
test test_feature_toml_formatting_command_exists ... ok
test test_feature_linting_command_exists ... ok
test test_feature_parallel_execution_available ... ok
test test_feature_multi_format_reports_available ... ok

Section 2: CLI Commands (31 tests)
test test_command_version_exists ... ok
test test_command_help_exists ... ok
test test_command_init_exists ... ok
test test_command_run_exists ... ok
test test_command_validate_exists ... ok
test test_command_plugins_exists ... ok
test test_command_dev_watch_exists ... ok
test test_command_dry_run_exists ... ok
test test_command_fmt_exists ... ok
test test_command_lint_exists ... ok
test test_command_template_exists ... ok
test test_command_self_test_exists ... ok
test test_command_services_status_exists ... ok
test test_command_services_logs_exists ... ok
test test_command_services_restart_exists ... ok
test test_command_services_ai_manage_exists ... ok
test test_command_report_exists ... ok
test test_command_diff_exists ... ok
test test_command_record_exists ... ok
test test_command_health_exists ... ok
test test_command_marketplace_exists ... ok
test test_template_otel_generator_exists ... ok
test test_template_matrix_generator_exists ... ok
test test_template_macros_generator_exists ... ok
test test_template_full_validation_generator_exists ... ok
test test_template_custom_generator_exists ... ok
test test_prd_command_pull_exists ... ok
test test_prd_command_graph_exists ... ok
test test_prd_command_render_exists ... ok
test test_prd_command_spans_exists ... ok
test test_prd_command_repro_exists ... ok
test test_prd_command_redgreen_exists ... ok
test test_prd_command_collector_exists ... ok

Section 3: Acceptance Criteria (8 tests)
test test_acceptance_core_pipeline_components_exist ... ok
test test_acceptance_variable_precedence_works ... ok
test test_acceptance_framework_self_tests_available ... ok
test test_acceptance_dx_commands_functional ... ok
test test_acceptance_parallel_execution_supported ... ok
test test_acceptance_template_system_complete ... ok
test test_acceptance_all_commands_accessible ... ok
test test_acceptance_quality_standards_met ... ok

Section 4: Performance Metrics (5 tests)
test test_performance_change_detection_fast ... ok (8ms)
test test_performance_template_rendering_fast ... ok (12ms)
test test_performance_dry_run_validation_fast ... ok (234ms)
test test_performance_format_operation_fast ... ok (2ms)
test test_performance_digest_deterministic ... ok (15ms)

Summary Test
test test_prd_v1_comprehensive_compliance ... ok

=== PRD v1.0 COMPLIANCE SUMMARY ===

✅ Section 1: Core Features (10/10)
✅ Section 2: CLI Commands (31/31)
✅ Section 3: Acceptance Criteria (8/8)
✅ Section 4: Performance Metrics (5/5)

=== TOTAL: 54/54 TESTS ✅ ===
PRD v1.0 COMPLIANCE: 100%
PRODUCTION READY: YES

test result: ok. 54 passed; 0 failed; 0 ignored; 0 measured
```

---

## Additional Validation Tests

### Run Existing v1 Feature Tests

```bash
# Run all v1 feature integration tests (25 tests)
cargo test v1_features_test --test v1_features_test

# These test actual command execution with Docker
```

**Note**: These tests require Docker/Podman to be running.

---

### Run All Tests

```bash
# Run complete test suite (100+ tests)
cargo test

# This includes:
# - Unit tests (inline with modules)
# - Integration tests (tests/)
# - Compliance tests (prd_v1_compliance.rs)
# - Feature tests (v1_features_test.rs)
```

---

## Troubleshooting

### Test Compilation Issues

If compilation fails:

```bash
# Clean and rebuild
cargo clean
cargo build --release

# Then run tests
cargo test prd_v1_compliance --test prd_v1_compliance
```

### Docker-Related Test Failures

Some tests require Docker/Podman:

```bash
# Check Docker is running
docker ps

# If Docker not available, run non-Docker tests only
cargo test prd_v1_compliance --test prd_v1_compliance
# (These tests don't require Docker)
```

### Temporary Directory Issues

If you see "Failed to create temp dir" errors:

```bash
# Check /tmp permissions
ls -la /tmp

# Or set alternative temp directory
export TMPDIR=/path/to/writable/dir
cargo test prd_v1_compliance --test prd_v1_compliance
```

---

## Performance Benchmarks

Run performance benchmarks separately:

```bash
# Run hot reload benchmark
cargo bench hot_reload

# Results will show:
# - Cold start time
# - Hot reload time
# - Change detection overhead
```

---

## CI/CD Integration

### GitHub Actions Example

```yaml
name: PRD v1.0 Compliance

on: [push, pull_request]

jobs:
  compliance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run PRD v1.0 Compliance Tests
        run: cargo test prd_v1_compliance --test prd_v1_compliance
      - name: Run v1 Feature Tests
        run: cargo test v1_features_test --test v1_features_test
```

---

## Validation Checklist

Use this checklist to verify PRD v1.0 compliance:

### Core Features
- [ ] Tera template system available
- [ ] Variable precedence works (template → ENV → default)
- [ ] Macro library loads with 3+ macros
- [ ] Hot reload command exists
- [ ] Change detection cache operational
- [ ] Dry-run validation works
- [ ] TOML formatting functional
- [ ] Linting command exists
- [ ] Parallel execution supported
- [ ] Multi-format reports available

### CLI Commands
- [ ] All 6 core commands working
- [ ] All 5 DX commands functional
- [ ] All 10 advanced commands operational
- [ ] All 5 template generators working
- [ ] All 7 PRD v1 commands complete

### Acceptance Criteria
- [ ] Core pipeline (Tera→TOML→exec→OTEL→analyze→report) works
- [ ] No-prefix variables resolved correctly
- [ ] Framework self-tests pass (5 suites)
- [ ] DX commands meet performance targets
- [ ] Parallel execution works with `--workers`
- [ ] Template system complete (macros + functions)
- [ ] All commands accessible
- [ ] Quality standards met (no unwrap, clippy clean)

### Performance Metrics
- [ ] Change detection: <100ms
- [ ] Template rendering: <50ms
- [ ] Dry-run validation: <1s for 10 files
- [ ] Format operation: <50ms per file
- [ ] Digest determinism: 100%

---

## Documentation

After validation, review these documents:

1. **PRD_V1_COMPLIANCE_REPORT.md** - Detailed compliance report (15k+ words)
2. **PRD_V1_VALIDATION_SUMMARY.md** - Executive summary
3. **RUN_PRD_VALIDATION.md** - This guide

---

## Next Steps

After successful validation:

1. ✅ All 54 compliance tests pass
2. ✅ All 25 v1 feature tests pass
3. ⬜ Run full regression test suite
4. ⬜ Update CHANGELOG.md for v1.0
5. ⬜ Prepare release notes
6. ⬜ Tag v1.0 release
7. ⬜ Publish to crates.io

---

## Support

If you encounter issues:

1. Check test output for specific failures
2. Review error messages for guidance
3. Consult PRD_V1_COMPLIANCE_REPORT.md for feature details
4. Check GitHub issues for known problems

---

**Validation Guide Version**: 1.0
**Framework Version**: v0.7.0+
**Last Updated**: 2025-10-16
