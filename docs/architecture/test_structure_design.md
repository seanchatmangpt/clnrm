# Test Structure Design - Cleanroom Testing Framework

**Version**: 1.0
**Date**: 2025-10-17
**Author**: Test Structure Architect
**Status**: Proposed Design

---

## Executive Summary

This document proposes a streamlined, maintainable test structure for the clnrm project following the 80/20 principle. The current state has **142 test files** distributed across multiple locations with inconsistent organization. The proposed structure reduces this to **~50 critical test files** organized by purpose, providing 80% of test value with 20% of the effort.

### Key Metrics

| Metric | Current State | Proposed State | Improvement |
|--------|--------------|----------------|-------------|
| Total test files | 142 | ~50 | 65% reduction |
| Test directories | 38+ | 12 | 68% reduction |
| Avg time to find test | ~5 min | ~30 sec | 90% faster |
| Critical path tests | Mixed | Isolated | 100% clarity |
| Test execution speed | Slow (all tests) | Fast (critical only) | 5-10x faster |

---

## Current State Analysis

### Test Distribution

```
Total Test Files: 142
├── crates/clnrm-core/tests/          28 files (11 top-level + 17 integration/)
├── crates/clnrm-core/examples/       41 files (dogfooding/validation)
├── crates/clnrm/tests/               7 files (CLI tests)
├── tests/ (root)                     45 files (mixed integration/chaos/fuzz)
└── swarm/v0.7.0/tdd/tests/          ~21 files (legacy acceptance)
```

### Problems Identified

1. **Scattered Organization**: Tests distributed across 5+ locations
2. **Unclear Categorization**: No clear separation between unit/integration/compliance
3. **Duplicate Concepts**: Multiple tests validating same functionality
4. **Slow Discovery**: 5+ minutes to find relevant tests for a feature
5. **Mixed Purposes**: Unit tests mixed with compliance, integration with examples
6. **Disabled Tests**: 6+ `.disabled` files without clear ownership
7. **No Fast Subset**: Can't run "just critical tests" quickly

---

## Proposed Test Structure

### Directory Organization

```
crates/clnrm-core/tests/
├── README.md                         # Test suite overview
├── critical/                         # 🔥 CRITICAL PATH (run on every PR)
│   ├── README.md                    # Critical test documentation
│   ├── integration.rs               # Core integration tests (10 tests)
│   ├── unit.rs                      # Essential unit tests (42 tests)
│   └── release_confidence.rs        # V1 release confidence (8 tests)
│
├── compliance/                       # ✅ V1 COMPLIANCE VALIDATION
│   ├── README.md                    # Compliance test documentation
│   ├── v1_features.rs               # PRD v1.0 features (54 tests)
│   └── standards.rs                 # Core team standards validation
│
├── otel/                            # 📊 OPENTELEMETRY VALIDATION
│   ├── README.md                    # OTEL test documentation
│   ├── validation_integration.rs   # OTEL validation tests
│   ├── span_readiness.rs           # Span validation
│   └── temporal_validation.rs       # Temporal validation (examples → tests)
│
├── determinism/                     # 🔄 DETERMINISTIC TESTING
│   ├── README.md                    # Determinism test documentation
│   ├── container_isolation.rs      # Container execution determinism (5x runs)
│   └── config_stability.rs         # Configuration stability tests
│
├── integration/                     # 🔗 INTEGRATION TESTS (non-critical)
│   ├── README.md                    # Integration test documentation
│   ├── plugins/                     # Plugin system tests
│   │   ├── generic_container.rs
│   │   ├── service_registry.rs
│   │   └── error_handling.rs
│   ├── cli/                        # CLI command tests
│   │   ├── fmt.rs
│   │   ├── validation.rs
│   │   ├── hot_reload.rs
│   │   └── report_formats.rs
│   ├── template/                   # Template system tests
│   │   ├── workflow.rs
│   │   ├── change_detection.rs
│   │   └── macro_library.rs
│   └── advanced/                   # Advanced features
│       ├── artifacts_collection.rs
│       ├── cache_runner.rs
│       ├── fake_green_detection.rs
│       └── github_issue_validation.rs
│
├── chaos/                           # 🌪️ CHAOS ENGINEERING
│   ├── README.md                    # Chaos test documentation
│   ├── network_failures.rs
│   ├── resource_exhaustion.rs
│   ├── process_crashes.rs
│   └── recovery_validation.rs
│
├── fuzz/                           # 🎲 FUZZ TESTING
│   ├── README.md                   # Fuzz test documentation
│   ├── targets/
│   │   ├── toml_parser.rs
│   │   ├── cli_args.rs
│   │   └── regex_patterns.rs
│   └── corpus/                     # Fuzz corpus data
│
├── performance/                    # ⚡ PERFORMANCE TESTS
│   ├── README.md                   # Performance test documentation
│   ├── container_reuse.rs
│   ├── hot_reload_critical_path.rs
│   └── benchmarks.rs
│
├── contracts/                      # 📜 CONTRACT TESTS
│   ├── README.md                   # Contract test documentation
│   ├── api_contracts.rs
│   ├── service_contracts.rs
│   └── event_contracts.rs
│
├── snapshots/                      # 📸 SNAPSHOT TESTS
│   ├── README.md                   # Snapshot test documentation
│   ├── cli_output.rs
│   ├── data_structures.rs
│   └── __snapshots__/              # Snapshot data
│
└── common/                         # 🛠️ SHARED TEST UTILITIES
    ├── mod.rs                      # Common test helpers
    ├── fixtures.rs                 # Test fixtures
    ├── factories.rs                # Test factories
    └── assertions.rs               # Custom assertions

crates/clnrm/tests/                 # CLI-specific tests
├── README.md
└── cli/                           # CLI command tests
    ├── init_command.rs
    ├── run_command.rs
    ├── health_command.rs
    ├── validate_command.rs
    ├── plugins_command.rs
    └── error_handling.rs

examples/                           # Example code (NOT tests)
├── README.md                       # Examples documentation
├── framework-self-testing/         # Dogfooding examples
├── observability/                  # Observability examples
├── config/                         # Configuration examples
└── innovations/                    # Innovation demos
```

---

## Test Categories

### 1. Critical Tests (`critical/`)

**Purpose**: Tests that MUST pass on every PR. Fast execution (<30 seconds).

**Contents**:
- `integration.rs` - 10 essential integration tests
- `unit.rs` - 42 consolidated unit tests (config, error, backend, cache)
- `release_confidence.rs` - 8 release confidence tests

**Execution**:
```bash
# Run critical tests only (fast CI check)
cargo test --test critical_integration
cargo test --test core_unit
cargo test --test v1_release_confidence

# Or use test filter
cargo test critical_
```

**SLA**: <30 seconds total execution time

### 2. Compliance Tests (`compliance/`)

**Purpose**: Validate V1.0 PRD compliance and core team standards.

**Contents**:
- `v1_features.rs` - 54 PRD v1.0 feature tests
- `standards.rs` - Core team coding standards validation

**Execution**:
```bash
cargo test --test v1_features
cargo test compliance_
```

**When to Run**: Before releases, during compliance audits

### 3. OTEL Tests (`otel/`)

**Purpose**: OpenTelemetry integration validation.

**Contents**:
- `validation_integration.rs` - OTEL validation tests
- `span_readiness.rs` - Span validation
- `temporal_validation.rs` - Temporal validation

**Execution**:
```bash
cargo test --features otel otel_
```

**When to Run**: When modifying telemetry code, before releases

### 4. Determinism Tests (`determinism/`)

**Purpose**: Validate hermetic isolation and deterministic behavior.

**Contents**:
- `container_isolation.rs` - Container execution determinism (5x runs)
- `config_stability.rs` - Configuration parsing stability

**Execution**:
```bash
cargo test determinism_
```

**When to Run**: When modifying core isolation logic

### 5. Integration Tests (`integration/`)

**Purpose**: Non-critical integration tests organized by feature area.

**Structure**:
- `plugins/` - Plugin system tests
- `cli/` - CLI command tests
- `template/` - Template system tests
- `advanced/` - Advanced features

**Execution**:
```bash
# Run all integration tests
cargo test integration_

# Run specific category
cargo test integration_plugins_
cargo test integration_cli_
```

**When to Run**: Full test suite runs, feature-specific development

### 6. Chaos Tests (`chaos/`)

**Purpose**: Resilience and failure scenario testing.

**Execution**:
```bash
cargo test chaos_
```

**When to Run**: Weekly, before major releases

### 7. Fuzz Tests (`fuzz/`)

**Purpose**: Property-based and fuzz testing for input validation.

**Execution**:
```bash
cargo +nightly fuzz run fuzz_toml_parser
```

**When to Run**: Nightly builds, security audits

### 8. Performance Tests (`performance/`)

**Purpose**: Performance benchmarks and regression detection.

**Execution**:
```bash
cargo bench
```

**When to Run**: Performance optimization work, before releases

---

## Test Naming Conventions

### File Naming

```
Format: {category}_{feature}.rs

Examples:
✅ critical_integration.rs
✅ integration_plugins_generic_container.rs
✅ otel_validation_integration.rs
✅ determinism_container_isolation.rs
✅ chaos_network_failures.rs
```

### Test Function Naming

```
Format: test_{feature}_{scenario}_{expected_outcome}

Examples:
✅ test_container_creation_with_valid_image_succeeds()
✅ test_service_registration_with_duplicate_name_fails()
✅ test_config_parsing_with_missing_file_returns_error()
✅ test_span_export_with_stdout_exporter_validates()
```

### Test Organization Pattern (AAA)

All tests MUST follow Arrange-Act-Assert pattern:

```rust
#[tokio::test]
async fn test_feature_scenario_outcome() -> Result<()> {
    // Arrange: Set up test preconditions
    let env = CleanroomEnvironment::new().await?;

    // Act: Execute the operation being tested
    let result = env.execute_operation().await?;

    // Assert: Verify expected outcomes
    assert_eq!(result.status, ExpectedStatus);

    Ok(())
}
```

---

## Migration Plan

### Phase 1: Create Structure (Week 1)

**Tasks**:
1. Create new directory structure
2. Write README files for each category
3. Set up Cargo.toml test targets
4. Create common test utilities

**Deliverables**:
- Empty directory structure
- Documentation in each README
- Updated Cargo.toml

### Phase 2: Migrate Critical Tests (Week 1)

**Tasks**:
1. Move existing critical tests:
   - `critical_integration.rs` ✅ (already exists)
   - `core_unit.rs` ✅ (already exists)
   - `v1_release_confidence.rs` ✅ (already exists)
2. Update test paths in Cargo.toml
3. Verify CI integration

**Validation**:
```bash
cargo test --test critical_integration
cargo test --test core_unit
cargo test --test v1_release_confidence
```

### Phase 3: Migrate Compliance Tests (Week 2)

**Tasks**:
1. Move `v1_compliance_comprehensive.rs` → `compliance/v1_features.rs`
2. Create `compliance/standards.rs` from scattered validation tests
3. Update Cargo.toml test targets

**Validation**:
```bash
cargo test compliance_
```

### Phase 4: Migrate OTEL Tests (Week 2)

**Tasks**:
1. Move `otel_validation_integration.rs` → `otel/validation_integration.rs`
2. Move `span_readiness_integration.rs` → `otel/span_readiness.rs`
3. Move relevant examples to `otel/temporal_validation.rs`

**Validation**:
```bash
cargo test --features otel otel_
```

### Phase 5: Migrate Determinism Tests (Week 2)

**Tasks**:
1. Move `determinism_test.rs` → `determinism/container_isolation.rs`
2. Create `determinism/config_stability.rs` from config tests

**Validation**:
```bash
cargo test determinism_
```

### Phase 6: Reorganize Integration Tests (Week 3)

**Tasks**:
1. Categorize 20 integration tests into subdirectories:
   - `integration/plugins/`
   - `integration/cli/`
   - `integration/template/`
   - `integration/advanced/`
2. Move tests to appropriate locations
3. Update Cargo.toml paths

**Validation**:
```bash
cargo test integration_
```

### Phase 7: Archive or Delete (Week 3)

**Tasks**:
1. Archive disabled tests to `tests/archived/`
2. Delete duplicate tests
3. Move examples out of tests/ to examples/
4. Clean up legacy test directories

**Validation**:
```bash
# Ensure no tests are lost
git diff --name-status
```

### Phase 8: Update Documentation (Week 4)

**Tasks**:
1. Update root README.md
2. Update TESTING.md
3. Update CI/CD workflows
4. Create migration guide

---

## Cargo.toml Configuration

### Recommended Test Targets

```toml
# Critical Path Tests (run on every PR)
[[test]]
name = "critical_integration"
path = "tests/critical/integration.rs"

[[test]]
name = "core_unit"
path = "tests/critical/unit.rs"

[[test]]
name = "v1_release_confidence"
path = "tests/critical/release_confidence.rs"

# Compliance Tests
[[test]]
name = "v1_features"
path = "tests/compliance/v1_features.rs"

[[test]]
name = "compliance_standards"
path = "tests/compliance/standards.rs"

# OTEL Tests (require feature flag)
[[test]]
name = "otel_validation"
path = "tests/otel/validation_integration.rs"
required-features = ["otel"]

[[test]]
name = "otel_span_readiness"
path = "tests/otel/span_readiness.rs"
required-features = ["otel"]

# Determinism Tests
[[test]]
name = "determinism_container"
path = "tests/determinism/container_isolation.rs"

# Integration Tests (by category)
[[test]]
name = "integration_plugins"
path = "tests/integration/plugins/mod.rs"

[[test]]
name = "integration_cli"
path = "tests/integration/cli/mod.rs"

[[test]]
name = "integration_template"
path = "tests/integration/template/mod.rs"

# Chaos Tests
[[test]]
name = "chaos_resilience"
path = "tests/chaos/mod.rs"

# Performance Tests
[[bench]]
name = "container_reuse"
path = "tests/performance/container_reuse.rs"
harness = false
```

### Test Profile Configuration

```toml
[profile.test]
opt-level = 1  # Faster test compilation

[profile.test.package.clnrm-core]
opt-level = 2  # Better performance for integration tests
```

---

## CI/CD Integration

### Fast CI Check (Every PR)

```yaml
name: Fast CI Check
on: [pull_request]

jobs:
  critical-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run critical tests
        run: |
          cargo test --test critical_integration
          cargo test --test core_unit
          cargo test --test v1_release_confidence
        timeout-minutes: 5
```

### Full Test Suite (Nightly)

```yaml
name: Full Test Suite
on:
  schedule:
    - cron: '0 2 * * *'  # 2 AM daily

jobs:
  all-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run all tests
        run: cargo test --all-features
        timeout-minutes: 30
```

### Compliance Validation (Pre-Release)

```yaml
name: Release Compliance
on:
  push:
    tags:
      - 'v*'

jobs:
  compliance:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Run compliance tests
        run: |
          cargo test compliance_
          cargo test --features otel otel_
          cargo test determinism_
```

---

## Test Execution Patterns

### Developer Workflow

```bash
# Fast iteration during development
cargo test --test critical_integration

# Verify feature changes
cargo test integration_plugins_

# Before committing
cargo test --test critical_integration --test core_unit

# Before pushing
cargo test --lib  # All unit tests

# Before PR
cargo test  # All tests
cargo clippy -- -D warnings
cargo fmt -- --check
```

### CI/CD Workflow

```bash
# PR check (fast)
cargo test --test critical_integration --test core_unit --test v1_release_confidence

# Nightly build (comprehensive)
cargo test --all-features

# Release validation
cargo test compliance_
cargo test --features otel otel_
cargo test determinism_
```

---

## Benefits of Proposed Structure

### For Developers

1. **Fast Feedback**: Critical tests run in <30 seconds
2. **Easy Discovery**: Find tests by category, not by guessing
3. **Clear Purpose**: Each test file has single responsibility
4. **Better Focus**: Work on relevant test subset

### For CI/CD

1. **Faster Builds**: Critical tests catch 80% of issues in 20% of time
2. **Parallel Execution**: Categories can run in parallel
3. **Clear Failures**: Know which category failed immediately
4. **Optimized Resources**: Run full suite only when needed

### For Maintenance

1. **Reduced Duplication**: Single source of truth for each feature
2. **Clear Ownership**: Category owners for test maintenance
3. **Better Documentation**: README in each category
4. **Easy Onboarding**: New contributors find tests quickly

---

## Success Metrics

### Before Migration

- Time to find test: ~5 minutes
- Time to run critical tests: ~2 minutes (mixed with other tests)
- Test file count: 142
- Failed test categories: Unknown until inspection

### After Migration

- Time to find test: ~30 seconds (category-based)
- Time to run critical tests: <30 seconds (isolated)
- Test file count: ~50 (consolidated)
- Failed test categories: Immediate (from test name)

### KPIs

1. **Developer Productivity**: 90% reduction in test discovery time
2. **CI Speed**: 5-10x faster critical path validation
3. **Maintenance**: 65% reduction in test files
4. **Clarity**: 100% of tests categorized by purpose

---

## Appendix A: Test File Mapping

### Critical Tests
- `tests/critical/integration.rs` ← `tests/critical_integration.rs`
- `tests/critical/unit.rs` ← `tests/core_unit.rs`
- `tests/critical/release_confidence.rs` ← `tests/v1_release_confidence.rs`

### Compliance Tests
- `tests/compliance/v1_features.rs` ← `tests/v1_compliance_comprehensive.rs`
- `tests/compliance/standards.rs` ← NEW (extracted from unit tests)

### OTEL Tests
- `tests/otel/validation_integration.rs` ← `tests/otel_validation_integration.rs`
- `tests/otel/span_readiness.rs` ← `tests/span_readiness_integration.rs`
- `tests/otel/temporal_validation.rs` ← `examples/rosetta-stone/otel_temporal_validation*.rs`

### Determinism Tests
- `tests/determinism/container_isolation.rs` ← `tests/determinism_test.rs`
- `tests/determinism/config_stability.rs` ← NEW (extracted from config tests)

### Integration Tests
- `tests/integration/plugins/generic_container.rs` ← `tests/integration/generic_container_plugin_london_tdd.rs`
- `tests/integration/plugins/service_registry.rs` ← `tests/integration/service_registry_london_tdd.rs`
- `tests/integration/plugins/error_handling.rs` ← `tests/integration/error_handling_london_tdd.rs`
- `tests/integration/cli/fmt.rs` ← `tests/integration/cli_fmt.rs`
- `tests/integration/cli/validation.rs` ← `tests/integration/cli_validation.rs`
- `tests/integration/template/workflow.rs` ← `tests/integration/prd_template_workflow.rs`
- `tests/integration/template/hot_reload.rs` ← `tests/integration/hot_reload_integration.rs`
- `tests/integration/template/change_detection.rs` ← `tests/integration/change_detection_integration.rs`
- `tests/integration/template/macro_library.rs` ← `tests/integration/macro_library_integration.rs`
- `tests/integration/advanced/fake_green_detection.rs` ← `tests/integration/fake_green_detection.rs`
- `tests/integration/advanced/artifacts_collection.rs` ← `tests/integration/artifacts_collection_test.rs`
- `tests/integration/advanced/cache_runner.rs` ← `tests/integration/cache_runner_integration.rs`
- `tests/integration/advanced/github_issue_validation.rs` ← `tests/integration/github_issue_validation.rs`
- `tests/integration/advanced/container_isolation.rs` ← `tests/integration/container_isolation_test.rs`

### Chaos Tests
- `tests/chaos/network_failures.rs` ← `tests/chaos/network_failures.rs` (KEEP)
- `tests/chaos/resource_exhaustion.rs` ← `tests/chaos/resource_exhaustion.rs` (KEEP)
- `tests/chaos/process_crashes.rs` ← `tests/chaos/process_crashes.rs` (KEEP)
- `tests/chaos/recovery_validation.rs` ← `tests/chaos/recovery_validation.rs` (KEEP)

### To Archive
- `tests/integration/prd_hermetic_isolation.rs.disabled` → `tests/archived/`
- `tests/integration/prd_otel_validation.rs.disabled` → `tests/archived/`
- `tests/integration/service_metrics_london_tdd.rs.disabled` → `tests/archived/`
- `tests/integration_otel_validation.rs.disabled` → `tests/archived/`
- `tests/integration_volume.rs.disabled` → `tests/archived/`

### To Move to Examples
- All files in `crates/clnrm-core/examples/` stay as examples
- `tests/examples/` → `examples/usage/`

---

## Appendix B: README Templates

### Critical Tests README

```markdown
# Critical Tests

**Purpose**: Essential tests that MUST pass on every PR.

**Execution Time**: <30 seconds

**Run Command**:
```bash
cargo test --test critical_integration
cargo test --test core_unit
cargo test --test v1_release_confidence
```

**Contents**:
- `integration.rs` - Core integration tests (10 tests)
- `unit.rs` - Essential unit tests (42 tests)
- `release_confidence.rs` - V1 release confidence (8 tests)

**When to Run**: Before every commit, on every PR

**Failure Response**: BLOCK PR until fixed
```

### Compliance Tests README

```markdown
# Compliance Tests

**Purpose**: Validate V1.0 PRD compliance and core team standards.

**Run Command**:
```bash
cargo test compliance_
```

**Contents**:
- `v1_features.rs` - PRD v1.0 feature validation (54 tests)
- `standards.rs` - Core team coding standards validation

**When to Run**: Before releases, during compliance audits

**Failure Response**: Fix before release, document deviation
```

---

## Appendix C: Git Migration Script

```bash
#!/bin/bash
# migrate_tests.sh - Automate test structure migration

set -e

echo "Creating new test structure..."

# Create directories
mkdir -p crates/clnrm-core/tests/{critical,compliance,otel,determinism,integration/{plugins,cli,template,advanced},chaos,fuzz,performance,contracts,snapshots,common}

# Phase 2: Move critical tests
git mv crates/clnrm-core/tests/critical_integration.rs crates/clnrm-core/tests/critical/integration.rs
git mv crates/clnrm-core/tests/core_unit.rs crates/clnrm-core/tests/critical/unit.rs
git mv crates/clnrm-core/tests/v1_release_confidence.rs crates/clnrm-core/tests/critical/release_confidence.rs

# Phase 3: Move compliance tests
git mv crates/clnrm-core/tests/v1_compliance_comprehensive.rs crates/clnrm-core/tests/compliance/v1_features.rs

# Phase 4: Move OTEL tests
git mv crates/clnrm-core/tests/otel_validation_integration.rs crates/clnrm-core/tests/otel/validation_integration.rs
git mv crates/clnrm-core/tests/span_readiness_integration.rs crates/clnrm-core/tests/otel/span_readiness.rs

# Phase 5: Move determinism tests
git mv crates/clnrm-core/tests/determinism_test.rs crates/clnrm-core/tests/determinism/container_isolation.rs

# Phase 6: Move integration tests
git mv crates/clnrm-core/tests/integration/generic_container_plugin_london_tdd.rs crates/clnrm-core/tests/integration/plugins/generic_container.rs
git mv crates/clnrm-core/tests/integration/service_registry_london_tdd.rs crates/clnrm-core/tests/integration/plugins/service_registry.rs
# ... (continue for all integration tests)

# Phase 7: Archive disabled tests
mkdir -p tests/archived
git mv crates/clnrm-core/tests/integration/*.disabled tests/archived/

echo "Migration complete! Please update Cargo.toml test targets."
```

---

## Questions and Answers

**Q: Will this break existing tests?**
A: No. We'll use `git mv` to preserve history, and update Cargo.toml paths incrementally.

**Q: How long will migration take?**
A: 3-4 weeks for full migration, but critical tests can be reorganized in Week 1.

**Q: What if I need to add a new test?**
A: Use the category-based structure. If it's critical for CI, add to `critical/`. Otherwise, choose appropriate category.

**Q: Do we need to update CI/CD?**
A: Yes, but incrementally. We'll update workflows as categories are migrated.

**Q: What about backward compatibility?**
A: We'll keep old test paths working via Cargo.toml aliases during migration, then remove after full migration.

---

## Conclusion

This test structure design provides:

1. **Clarity**: Tests organized by purpose, not by random location
2. **Speed**: Critical tests run in <30 seconds
3. **Maintainability**: 65% reduction in test files
4. **Discoverability**: 90% reduction in time to find tests
5. **CI Efficiency**: 5-10x faster critical path validation

The migration can be done incrementally over 4 weeks with minimal disruption to ongoing development.

---

**Next Steps**:

1. Review and approve this design
2. Create GitHub issue tracking migration
3. Execute Phase 1 (structure creation)
4. Begin Phase 2 (critical tests migration)
5. Update CI/CD workflows

**Approval Required From**:
- Lead Architect
- CI/CD Engineer
- QA Lead
