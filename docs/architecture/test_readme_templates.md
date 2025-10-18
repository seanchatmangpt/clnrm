# Test README Templates

This file contains templates for README files in each test category directory.

---

## Template: tests/critical/README.md

```markdown
# Critical Tests

**Purpose**: Essential tests that MUST pass on every PR. These tests provide 80% of value with 20% of effort.

**Execution Time**: <30 seconds total

**Run Commands**:
```bash
# Run all critical tests
cargo test --test critical_integration --test core_unit --test v1_release_confidence

# Run specific test
cargo test --test critical_integration
cargo test --test core_unit
cargo test --test v1_release_confidence
```

## Contents

| File | Tests | Purpose | Execution Time |
|------|-------|---------|----------------|
| `integration.rs` | 10 | Core integration functionality | ~10s |
| `unit.rs` | 42 | Essential unit tests (config, error, backend, cache) | ~15s |
| `release_confidence.rs` | 8 | V1 release confidence validation | ~5s |

**Total**: 60 tests, <30 seconds

## When to Run

- **Every commit**: Before committing code
- **Every PR**: Automated CI check
- **Before push**: Local validation

## Failure Response

**BLOCK PR** until tests pass. These tests validate core functionality and must never fail.

## Test Requirements

All tests in this directory MUST:
1. Execute in <30 seconds total
2. Follow AAA pattern (Arrange, Act, Assert)
3. Use proper error handling (no unwrap/expect)
4. Have descriptive names explaining scenario and outcome
5. Be deterministic (no flaky tests)

## Adding New Tests

**DO NOT add tests to critical/ lightly**. Only add tests that:
1. Validate absolutely essential functionality
2. Execute quickly (<3 seconds)
3. Catch critical regressions
4. Provide high value

For other tests, use appropriate category:
- Compliance tests → `tests/compliance/`
- Integration tests → `tests/integration/`
- OTEL tests → `tests/otel/`

## Maintenance

**Owner**: Core Team
**Review Frequency**: Every PR
**Performance Target**: <30 seconds total
```

---

## Template: tests/compliance/README.md

```markdown
# Compliance Tests

**Purpose**: Validate V1.0 PRD compliance and core team coding standards.

**Run Commands**:
```bash
# Run all compliance tests
cargo test compliance_

# Run specific compliance category
cargo test --test v1_features_compliance
cargo test --test core_team_standards
```

## Contents

| File | Tests | Purpose |
|------|-------|---------|
| `v1_features.rs` | 54 | PRD v1.0 feature validation (10 core features, 31 CLI commands, acceptance criteria, performance) |
| `standards.rs` | TBD | Core team coding standards (error handling, async/sync, testing patterns, no false positives) |

## When to Run

- **Pre-release**: Before any version release
- **Weekly**: Nightly builds
- **On-demand**: Compliance audits

## Failure Response

**Document deviation** or **fix before release**. Compliance failures indicate:
- Missing PRD features
- Broken acceptance criteria
- Coding standards violations

## PRD v1.0 Coverage

### Core Features (10)
1. Tera template system
2. Variable precedence resolution
3. Macro library system
4. Hot-reload capability
5. Change detection
6. Snapshot testing
7. Contract testing
8. Chaos engineering
9. Fuzz testing
10. Performance benchmarking

### CLI Commands (31)
- Core: init, run, validate, etc.
- DX: fmt, lint, diff, dev-watch
- Advanced: chaos, fuzz, benchmark
- Templates: template, prd
- PRD v1: record, export, workflow

### Acceptance Criteria
- Core pipeline
- Developer experience
- Execution & isolation
- Template system
- Command coverage
- Quality assurance

## Maintenance

**Owner**: QA Lead
**Review Frequency**: Before releases
**Update Trigger**: PRD changes
```

---

## Template: tests/otel/README.md

```markdown
# OpenTelemetry (OTEL) Tests

**Purpose**: Validate OpenTelemetry integration for traces, metrics, and logs.

**Feature Flag**: Requires `--features otel`

**Run Commands**:
```bash
# Run all OTEL tests
cargo test --features otel otel_

# Run specific OTEL test
cargo test --features otel --test otel_validation_integration
cargo test --features otel --test otel_span_readiness
cargo test --features otel --test otel_temporal_validation
```

## Contents

| File | Purpose | Validates |
|------|---------|-----------|
| `validation_integration.rs` | OTEL validation framework | Span processors, validators, assertions |
| `span_readiness.rs` | Span export validation | Span data structure, export mechanisms |
| `temporal_validation.rs` | Temporal validation | Time-based span ordering, trace coherence |

## OTEL Features Tested

### Traces
- Span creation and lifecycle
- Span processors (stdout, OTLP)
- Span validation (attributes, events, status)
- Trace context propagation

### Metrics
- Counter metrics
- Gauge metrics
- Histogram metrics
- Metric export (stdout, OTLP)

### Logs
- Log emission
- Log correlation with traces
- Log export

### Exporters
- Stdout exporter (testing)
- OTLP HTTP exporter
- OTLP gRPC exporter
- Jaeger exporter (legacy)

## When to Run

- **When modifying telemetry code**: Always
- **Pre-release**: Validate OTEL integration
- **Weekly**: Nightly builds with OTEL features

## Failure Response

Fix immediately if:
- Span validation fails
- Export mechanisms broken
- Temporal ordering violated

## Configuration

Tests use environment variables:
- `OTEL_EXPORTER_OTLP_ENDPOINT` - OTLP endpoint
- `OTEL_SERVICE_NAME` - Service name
- `OTEL_TRACES_SAMPLER` - Sampler (always_on/always_off)

## Maintenance

**Owner**: Observability Team
**Review Frequency**: When modifying telemetry code
**Performance Target**: <1 minute total
```

---

## Template: tests/determinism/README.md

```markdown
# Determinism Tests

**Purpose**: Validate hermetic isolation and deterministic behavior.

**Pattern**: Run tests 5 times and verify identical output (kcura pattern).

**Run Commands**:
```bash
# Run all determinism tests
cargo test determinism_

# Run specific determinism test
cargo test --test determinism_container_isolation
cargo test --test determinism_config_stability
```

## Contents

| File | Purpose | Iterations |
|------|---------|------------|
| `container_isolation.rs` | Container execution determinism | 5x runs |
| `config_stability.rs` | Configuration parsing stability | 5x runs |

## Determinism Validation

### What We Validate
1. **Container Execution**: Same input → Same output (5x)
2. **Service Lifecycle**: Deterministic startup/shutdown
3. **Configuration Parsing**: Stable TOML parsing
4. **Test Output**: Consistent results (excluding timestamps)

### Normalization
Tests normalize output by removing:
- Timestamps
- Session IDs
- Duration measurements
- Dynamic container IDs

### Hash Validation
Tests use `DefaultHasher` to compute deterministic hashes:
```rust
fn calculate_hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}
```

## When to Run

- **When modifying core isolation logic**: Always
- **Pre-release**: Validate hermetic guarantees
- **Weekly**: Detect non-determinism regressions

## Failure Response

**CRITICAL**: Fix immediately if determinism tests fail.

Determinism failures indicate:
- Broken hermetic isolation
- Non-deterministic execution
- Container reuse issues
- Configuration instability

## Maintenance

**Owner**: Core Team
**Review Frequency**: When modifying isolation logic
**Performance Target**: <1 minute (5x runs)
```

---

## Template: tests/integration/README.md

```markdown
# Integration Tests

**Purpose**: Non-critical integration tests organized by feature area.

**Run Commands**:
```bash
# Run all integration tests
cargo test integration_

# Run specific category
cargo test integration_plugins_
cargo test integration_cli_
cargo test integration_template_
cargo test integration_advanced_
```

## Directory Structure

```
integration/
├── plugins/          # Plugin system tests
│   ├── generic_container.rs
│   ├── service_registry.rs
│   └── error_handling.rs
├── cli/             # CLI command tests
│   ├── fmt.rs
│   ├── validation.rs
│   ├── hot_reload.rs
│   └── report_formats.rs
├── template/        # Template system tests
│   ├── workflow.rs
│   ├── change_detection.rs
│   └── macro_library.rs
└── advanced/        # Advanced features
    ├── fake_green_detection.rs
    ├── artifacts_collection.rs
    ├── cache_runner.rs
    ├── github_issue_validation.rs
    └── container_isolation.rs
```

## Categories

### Plugins (`plugins/`)
Tests for plugin system functionality:
- Generic container plugin
- Service registry
- Error handling

### CLI (`cli/`)
Tests for CLI commands:
- Formatting (clnrm fmt)
- Validation (clnrm validate)
- Hot reload (clnrm dev-watch)
- Report formats (clnrm report)

### Template (`template/`)
Tests for template system:
- Tera template workflow
- Change detection
- Macro library

### Advanced (`advanced/`)
Tests for advanced features:
- Fake green detection
- Artifacts collection
- Cache runner
- GitHub issue validation
- Container isolation

## When to Run

- **Feature development**: When working on specific features
- **Full test suite**: Nightly builds
- **Pre-release**: Comprehensive validation

## Failure Response

Investigate and fix. Integration tests validate feature functionality.

## Adding New Tests

1. Identify appropriate category (plugins/cli/template/advanced)
2. Create test file following naming convention
3. Follow AAA pattern
4. Add to Cargo.toml with `[[test]]` entry
5. Update category README

## Maintenance

**Owner**: Feature Team
**Review Frequency**: Weekly
**Performance Target**: <5 minutes total
```

---

## Template: tests/chaos/README.md

```markdown
# Chaos Engineering Tests

**Purpose**: Validate system resilience under failure conditions.

**Run Commands**:
```bash
# Run all chaos tests
cargo test chaos_

# Run specific chaos test
cargo test --test chaos_network_failures
cargo test --test chaos_resource_exhaustion
cargo test --test chaos_process_crashes
cargo test --test chaos_recovery_validation
```

## Contents

| File | Purpose | Scenarios |
|------|---------|-----------|
| `network_failures.rs` | Network failure injection | Timeouts, disconnections, latency |
| `resource_exhaustion.rs` | Resource limit testing | Memory, CPU, disk, file descriptors |
| `process_crashes.rs` | Process crash scenarios | Container crashes, service failures |
| `recovery_validation.rs` | Recovery validation | Automatic recovery, state restoration |

## Failure Scenarios

### Network Failures
- Connection timeouts
- Network disconnections
- High latency
- Packet loss

### Resource Exhaustion
- Memory limits
- CPU saturation
- Disk space exhaustion
- File descriptor limits

### Process Crashes
- Container crashes
- Service failures
- Abrupt terminations

### Recovery
- Automatic service restart
- State restoration
- Error handling
- Graceful degradation

## When to Run

- **Weekly**: Nightly builds
- **Pre-release**: Validate resilience
- **After major changes**: Verify stability

## Failure Response

Fix if:
- System doesn't recover gracefully
- Error handling is inadequate
- State corruption occurs

## Maintenance

**Owner**: Reliability Team
**Review Frequency**: Weekly
**Performance Target**: <2 minutes total
```

---

## Template: tests/fuzz/README.md

```markdown
# Fuzz Testing

**Purpose**: Property-based and fuzz testing for input validation.

**Run Commands**:
```bash
# Run fuzz tests (requires proptest feature)
cargo test --features proptest fuzz_

# Run specific fuzz test
cargo +nightly fuzz run fuzz_toml_parser
cargo +nightly fuzz run fuzz_cli_args
cargo +nightly fuzz run fuzz_regex_patterns
```

## Contents

| File | Purpose | Input Space |
|------|---------|-------------|
| `targets/toml_parser.rs` | TOML parser fuzzing | Random TOML input |
| `targets/cli_args.rs` | CLI argument fuzzing | Random CLI arguments |
| `targets/regex_patterns.rs` | Regex pattern fuzzing | Random regex patterns |

## Fuzz Targets

### TOML Parser
Tests TOML parsing with:
- Random valid TOML
- Invalid TOML syntax
- Edge cases (empty, huge files)
- Malformed structures

### CLI Arguments
Tests CLI parsing with:
- Random argument combinations
- Invalid flags
- Missing required arguments
- Edge cases

### Regex Patterns
Tests regex matching with:
- Random patterns
- Invalid regex syntax
- Catastrophic backtracking
- Edge cases

## Corpus Management

Fuzz corpus stored in `corpus/`:
```
fuzz/corpus/
├── fuzz_toml_parser/
├── fuzz_cli_args/
└── fuzz_regex_patterns/
```

## When to Run

- **Nightly**: Continuous fuzzing
- **Security audits**: Comprehensive testing
- **After parser changes**: Validate robustness

## Failure Response

**CRITICAL**: Fix crashes and panics immediately.

Fuzz failures indicate:
- Input validation bugs
- Panic scenarios
- Security vulnerabilities

## Maintenance

**Owner**: Security Team
**Review Frequency**: Weekly
**Performance Target**: Continuous (nightly)
```

---

## Template: tests/performance/README.md

```markdown
# Performance Tests

**Purpose**: Performance benchmarks and regression detection.

**Run Commands**:
```bash
# Run all benchmarks
cargo bench

# Run specific benchmark
cargo bench container_reuse
cargo bench hot_reload_critical_path
```

## Contents

| File | Purpose | Metrics |
|------|---------|---------|
| `container_reuse.rs` | Container reuse performance | Creation vs. reuse time |
| `hot_reload_critical_path.rs` | Hot reload performance | Edit-to-feedback latency |

## Benchmarks

### Container Reuse
Measures:
- Container creation time
- Container reuse time
- Reuse efficiency (speedup factor)

**Target**: 10x speedup for reused containers

### Hot Reload Critical Path
Measures:
- File change detection latency
- Test re-execution time
- Total edit-to-feedback latency

**Target**: <500ms edit-to-feedback

## Performance Targets

| Metric | Target | Current |
|--------|--------|---------|
| Container creation | <2s | TBD |
| Container reuse | <200ms | TBD |
| Hot reload latency | <500ms | TBD |
| Test suite (critical) | <30s | TBD |
| Test suite (full) | <5min | TBD |

## When to Run

- **Weekly**: Performance regression detection
- **Pre-release**: Validate performance targets
- **After optimization**: Measure improvements

## Failure Response

Investigate if:
- Performance regresses >10%
- Targets not met
- Anomalous results

## Maintenance

**Owner**: Performance Team
**Review Frequency**: Weekly
**Baseline Updates**: Every release
```

---

## Template: tests/common/README.md

```markdown
# Common Test Utilities

**Purpose**: Shared test utilities, fixtures, factories, and assertions.

**Usage**: Import from test code
```rust
use clnrm_core::tests::common::{fixtures, factories, assertions};
```

## Contents

| File | Purpose |
|------|---------|
| `mod.rs` | Common test helpers and utilities |
| `fixtures.rs` | Test fixtures (sample data, configs) |
| `factories.rs` | Test data factories (builders) |
| `assertions.rs` | Custom assertions for tests |

## Utilities

### Fixtures
Pre-defined test data:
- Sample TOML configurations
- Container configurations
- Service definitions

### Factories
Test data builders:
- `TestEnvironmentFactory` - Create test environments
- `ConfigFactory` - Create test configurations
- `ServiceFactory` - Create test services

### Assertions
Custom assertions:
- `assert_container_running()`
- `assert_service_healthy()`
- `assert_config_valid()`

## Best Practices

1. **Reuse common utilities**: Don't duplicate test helpers
2. **Add to common/**: Share utilities across tests
3. **Document utilities**: Add doc comments
4. **Keep DRY**: Extract common patterns

## Maintenance

**Owner**: Test Infrastructure Team
**Review Frequency**: As needed
```

---

These README templates provide clear documentation for each test category, making it easy for developers to understand test organization and find relevant tests.
