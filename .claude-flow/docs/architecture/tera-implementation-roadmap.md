# Tera Template System Implementation Roadmap

## Overview

This document provides a detailed, week-by-week implementation plan for integrating the Tera templating system into clnrm v0.6.0.

**Total Duration**: 4 weeks
**Team Size**: 1-2 developers
**Complexity**: Medium (leveraging existing Tera library)

---

## Pre-Implementation Checklist

Before starting implementation, ensure:

- [ ] Architecture document reviewed and approved
- [ ] Tera dependency license compatible (MIT - compatible with clnrm)
- [ ] Backward compatibility requirements understood
- [ ] Test infrastructure ready (Docker/Podman available)
- [ ] CI/CD pipeline configured for new tests

---

## Week 1: Core Tera Integration & Schema Extensions

### Phase 1A: Core Tera Integration (Days 1-3)

#### Tasks

**Day 1: Dependency Setup & Module Scaffolding**

1. Add Tera dependency to `Cargo.toml`:
   ```toml
   [dependencies]
   tera = "1.19"
   sha2 = "0.10"
   ```

2. Create module structure:
   ```bash
   mkdir -p crates/clnrm-core/src/template
   touch crates/clnrm-core/src/template/mod.rs
   touch crates/clnrm-core/src/template/context.rs
   touch crates/clnrm-core/src/template/functions.rs
   touch crates/clnrm-core/src/template/determinism.rs
   touch crates/clnrm-core/src/template/loader.rs
   ```

3. Update `crates/clnrm-core/src/lib.rs`:
   ```rust
   pub mod template;
   ```

**Day 2: Implement TemplateRenderer**

1. Implement `template/mod.rs`:
   - `TemplateRenderer` struct
   - `new()` and `with_determinism()` constructors
   - `render()` and `render_str()` methods

2. Implement `template/context.rs`:
   - `TemplateContext` struct
   - `from_config()` method
   - `to_tera_context()` method

3. Add error handling:
   - Extend `CleanroomError` enum with `TemplateError` variant
   - Implement `From<tera::Error> for CleanroomError`

**Day 3: Implement Custom Tera Functions**

1. Implement `template/functions.rs`:
   - `tera_env()` - Environment variable access
   - `tera_sha256()` - SHA-256 hashing
   - `tera_toml_encode()` - TOML encoding
   - `tera_now_rfc3339()` - Timestamp generation

2. Register functions in `TemplateRenderer::new()`

3. Write unit tests for each function (20+ tests)

#### Deliverables

- [ ] `template/` module structure created
- [ ] `TemplateRenderer` implemented and tested
- [ ] All custom Tera functions implemented
- [ ] Error handling integrated
- [ ] Unit tests passing (20+ tests)

#### Testing Strategy

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_template_renderer_basic() {
        let renderer = TemplateRenderer::new().unwrap();
        let template = "{{ vars.name }}";
        let mut context = TemplateContext::default();
        context.vars.insert("name".to_string(), "test".into());
        let result = renderer.render_str(template, context).unwrap();
        assert_eq!(result, "test");
    }

    #[test]
    fn test_env_function() {
        std::env::set_var("TEST_VAR", "value");
        // ... test env() function
    }
}
```

---

### Phase 1B: Configuration Schema Extensions (Days 4-5)

#### Tasks

**Day 4: Add New Config Structs**

1. Update `crates/clnrm-core/src/config/mod.rs`:
   - Add `DeterminismConfig`
   - Add `ReportConfig`
   - Add `LimitsConfig`
   - Add `OrderExpectationConfig`
   - Add `StatusExpectationConfig`
   - Add `OtelHeadersConfig`
   - Add `OtelPropagatorsConfig`

2. Update `TestConfig` struct:
   - Add `determinism: Option<DeterminismConfig>`
   - Add `report: Option<ReportConfig>`
   - Add `limits: Option<LimitsConfig>`

3. Update `ExpectConfig` struct:
   - Add `order: Option<OrderExpectationConfig>`
   - Add `status: Option<StatusExpectationConfig>`

4. Update `OtelConfig` struct:
   - Add `headers: Option<OtelHeadersConfig>`
   - Add `propagators: Option<OtelPropagatorsConfig>`

**Day 5: Integrate Template Rendering into Config Loading**

1. Modify `config/mod.rs`:
   - Add `is_template(content: &str) -> bool`
   - Add `render_template_if_needed(path: &Path) -> Result<String>`
   - Update `load_config_from_file()` to call rendering step

2. Implement template detection logic

3. Add integration tests for rendering + parsing

#### Deliverables

- [ ] All new config structs added
- [ ] Deserialization working correctly
- [ ] Template detection implemented
- [ ] Config loading pipeline updated
- [ ] Integration tests passing (10+ tests)

#### Testing Strategy

```rust
#[test]
fn test_load_template_config() {
    let template = r#"
        [template.vars]
        name = "test"

        [meta]
        name = "{{ vars.name }}"
    "#;

    std::fs::write("/tmp/test.toml", template).unwrap();
    let config = load_config_from_file(Path::new("/tmp/test.toml")).unwrap();
    assert_eq!(config.meta.name, "test");
}

#[test]
fn test_non_template_backward_compatible() {
    let toml = r#"
        [meta]
        name = "test"
    "#;

    std::fs::write("/tmp/test2.toml", toml).unwrap();
    let config = load_config_from_file(Path::new("/tmp/test2.toml")).unwrap();
    assert_eq!(config.meta.name, "test");
}
```

---

## Week 2: Validators & Reporting

### Phase 2A: New Validators (Days 6-8)

#### Tasks

**Day 6: Implement OrderValidator**

1. Create `crates/clnrm-core/src/validation/order_validator.rs`:
   - `OrderValidator` struct
   - `new()` constructor
   - `validate()` method
   - `check_precedence()` helper

2. Implement temporal ordering logic:
   - Compare span start times
   - Support glob pattern matching
   - Generate detailed error messages

3. Write unit tests (15+ tests)

**Day 7: Implement StatusValidator**

1. Create `crates/clnrm-core/src/validation/status_validator.rs`:
   - `StatusValidator` struct
   - `new()` constructor
   - `validate()` method

2. Implement status validation logic:
   - Check global `all` constraint
   - Check per-pattern `by_name` constraints
   - Support glob matching

3. Write unit tests (15+ tests)

**Day 8: Register Validators**

1. Update `validation/mod.rs`:
   - Add `OrderValidator` to `create_validator_chain()`
   - Add `StatusValidator` to `create_validator_chain()`
   - Export new validator modules

2. Integration testing with full pipeline

#### Deliverables

- [ ] `OrderValidator` implemented and tested
- [ ] `StatusValidator` implemented and tested
- [ ] Validators registered in chain
- [ ] Integration tests passing (30+ validator tests)

#### Testing Strategy

```rust
#[test]
fn test_order_validator_detects_violation() {
    let config = OrderExpectationConfig {
        must_precede: Some(vec![("start".into(), "end".into())]),
        must_follow: None,
    };
    let validator = OrderValidator::new(&Some(config));

    let spans = vec![
        create_span("end", now),
        create_span("start", now + 1s),
    ];

    let errors = validator.validate(&spans);
    assert!(!errors.is_empty());
}

#[test]
fn test_status_validator_all_ok() {
    let config = StatusExpectationConfig {
        all: Some("OK".into()),
        by_name: None,
    };
    let validator = StatusValidator::new(&Some(config));

    let spans = vec![
        create_span_with_status("span1", "OK"),
        create_span_with_status("span2", "ERROR"),
    ];

    let errors = validator.validate(&spans);
    assert_eq!(errors.len(), 1); // span2 should fail
}
```

---

### Phase 2B: Reporting System (Days 9-10)

#### Tasks

**Day 9: Implement Core Reporters**

1. Create `crates/clnrm-core/src/reporting/` module:
   ```bash
   mkdir -p crates/clnrm-core/src/reporting
   touch crates/clnrm-core/src/reporting/mod.rs
   touch crates/clnrm-core/src/reporting/json.rs
   touch crates/clnrm-core/src/reporting/junit.rs
   touch crates/clnrm-core/src/reporting/digest.rs
   ```

2. Implement `reporting/mod.rs`:
   - `Reporter` trait
   - `TestResults` struct
   - `ScenarioResult` struct
   - `generate_reports()` function

3. Implement `reporting/json.rs`:
   - `JsonReporter` struct
   - JSON serialization logic

**Day 10: Implement Additional Reporters**

1. Implement `reporting/junit.rs`:
   - `JUnitReporter` struct
   - JUnit XML generation
   - XML escaping

2. Implement `reporting/digest.rs`:
   - `DigestReporter` struct
   - Canonical representation
   - SHA-256 hashing

3. Update CLI to use reporters

#### Deliverables

- [ ] `reporting/` module created
- [ ] All three reporters implemented (JSON, JUnit, Digest)
- [ ] CLI integration complete
- [ ] Reporter tests passing (20+ tests)

#### Testing Strategy

```rust
#[test]
fn test_json_reporter() {
    let results = create_test_results();
    let reporter = JsonReporter;
    let output = reporter.generate(&results).unwrap();

    let json: serde_json::Value = serde_json::from_str(&output).unwrap();
    assert_eq!(json["summary"]["total"], 3);
}

#[test]
fn test_junit_reporter_xml_valid() {
    let results = create_test_results();
    let reporter = JUnitReporter;
    let output = reporter.generate(&results).unwrap();

    // Validate XML parsing
    assert!(output.starts_with("<?xml"));
    assert!(output.contains("<testsuites"));
}

#[test]
fn test_digest_reporter_deterministic() {
    let results = create_test_results();
    let reporter = DigestReporter;

    let digest1 = reporter.generate(&results).unwrap();
    let digest2 = reporter.generate(&results).unwrap();

    assert_eq!(digest1, digest2);
}
```

---

## Week 3: Documentation & Examples

### Phase 3A: Documentation (Days 11-13)

#### Tasks

**Day 11: TERA_TEMPLATE_GUIDE.md** (COMPLETED)

- [x] Write comprehensive template guide
- [x] Include syntax reference
- [x] Add troubleshooting section
- [x] Document all custom functions

**Day 12: Update Existing Documentation**

1. Update `README.md`:
   - Add v0.6.0 features overview
   - Link to Tera template guide
   - Update quick start examples

2. Update `docs/TOML_REFERENCE.md`:
   - Document new config sections
   - Document new expectation blocks
   - Add template examples

3. Create `docs/MIGRATION_v0.5_to_v0.6.md`:
   - Backward compatibility notes
   - Template migration guide
   - Breaking changes (if any)

**Day 13: Create Example Templates** (COMPLETED)

- [x] simple-variables.clnrm.toml
- [x] matrix-expansion.clnrm.toml
- [x] multi-environment.clnrm.toml
- [x] service-mesh.clnrm.toml
- [x] ci-integration.clnrm.toml
- [x] macros-and-includes.clnrm.toml
- [x] advanced-validators.clnrm.toml
- [x] examples/templates/README.md

#### Deliverables

- [x] TERA_TEMPLATE_GUIDE.md complete
- [ ] README.md updated
- [ ] TOML_REFERENCE.md updated
- [ ] Migration guide created
- [x] 7+ example templates created
- [x] examples/templates/README.md written

---

### Phase 3B: Testing & Validation (Days 14-15)

#### Tasks

**Day 14: E2E Tests**

1. Create `crates/clnrm-core/tests/e2e_template.rs`:
   - Test full pipeline (template → execution → validation)
   - Test each example template
   - Test error handling

2. Test matrix expansion generating multiple scenarios

3. Test conditional includes

**Day 15: Red-Team Testing**

1. False positive detection:
   - Validators should NOT pass invalid data
   - Template errors should NOT be silently ignored

2. Edge case testing:
   - Empty templates
   - Large templates (near 1 MB limit)
   - Deeply nested includes
   - Complex macro recursion

3. Performance testing:
   - Benchmark template rendering
   - Benchmark validator execution
   - Memory usage profiling

#### Deliverables

- [ ] E2E tests passing (10+ tests)
- [ ] Red-team tests passing (15+ tests)
- [ ] Performance benchmarks documented
- [ ] No false positives detected

#### Testing Strategy

```rust
#[tokio::test]
async fn test_e2e_matrix_expansion() {
    let config = load_config_from_file("examples/templates/matrix-expansion.clnrm.toml").unwrap();

    // Should generate 3 scenarios from template
    assert_eq!(config.scenarios.len(), 3);

    let results = execute_test(&config).await.unwrap();
    assert!(results.validation_errors.is_empty());
}

#[test]
fn test_red_team_undefined_variable() {
    let template = r#"
        [meta]
        name = "{{ vars.undefined }}"
    "#;

    let result = render_template(template);
    assert!(result.is_err()); // MUST fail
}

#[test]
fn test_red_team_order_validator_false_positive() {
    // Valid ordering should NOT fail
    let validator = OrderValidator::new(...);
    let valid_spans = create_correctly_ordered_spans();
    let errors = validator.validate(&valid_spans);
    assert_eq!(errors.len(), 0);
}
```

---

## Week 4: Release Preparation

### Phase 4A: Code Quality & Optimization (Days 16-18)

#### Tasks

**Day 16: Code Review**

1. Self-review all new code:
   - No `.unwrap()` or `.expect()` in production code
   - All functions return `Result<T, CleanroomError>`
   - Proper error messages with context
   - No `println!` (use `tracing` macros)

2. Run quality checks:
   ```bash
   cargo clippy -- -D warnings
   cargo fmt --check
   cargo test
   ```

3. Address clippy warnings and formatting issues

**Day 17: Performance Optimization**

1. Profile template rendering:
   - Identify bottlenecks
   - Optimize hot paths
   - Add caching if needed

2. Profile validator execution:
   - Optimize ordering checks
   - Optimize status checks

3. Memory usage optimization:
   - Reduce allocations
   - Reuse Tera instances

**Day 18: Security Audit**

1. Review template injection risks:
   - Validate `env()` function safety
   - Check for filesystem access vulnerabilities
   - Validate resource limits enforcement

2. Review error message leakage:
   - Ensure secrets not logged
   - Validate error messages don't expose internals

3. Dependency audit:
   ```bash
   cargo audit
   ```

#### Deliverables

- [ ] All clippy warnings resolved
- [ ] Code formatting correct
- [ ] Performance optimizations applied
- [ ] Security audit complete
- [ ] No critical vulnerabilities

---

### Phase 4B: Release (Days 19-20)

#### Tasks

**Day 19: Release Candidate**

1. Update version in `Cargo.toml`:
   ```toml
   version = "0.6.0-rc.1"
   ```

2. Generate changelog:
   - List all new features
   - List breaking changes (if any)
   - List bug fixes
   - List performance improvements

3. Build release candidate:
   ```bash
   cargo build --release
   cargo test --release
   ```

4. Test installation:
   ```bash
   cargo install --path crates/clnrm
   clnrm --version
   ```

**Day 20: Final Release**

1. Tag release:
   ```bash
   git tag v0.6.0
   git push origin v0.6.0
   ```

2. Publish to crates.io:
   ```bash
   cargo publish -p clnrm-core
   cargo publish -p clnrm
   ```

3. Update documentation:
   - Update README badges
   - Update version references
   - Publish release notes

4. Announce release:
   - GitHub release notes
   - Community channels
   - Update examples

#### Deliverables

- [ ] Version bumped to 0.6.0
- [ ] Changelog complete
- [ ] Release tagged
- [ ] Published to crates.io
- [ ] Release notes published

---

## Risk Mitigation

### Risk 1: Tera API Changes

**Likelihood**: Low
**Impact**: Medium
**Mitigation**: Pin Tera version to `1.19`, test thoroughly before upgrading

### Risk 2: Backward Compatibility Issues

**Likelihood**: Medium
**Impact**: High
**Mitigation**: Comprehensive backward compatibility tests, clear migration guide

### Risk 3: Performance Regression

**Likelihood**: Low
**Impact**: Medium
**Mitigation**: Benchmark before/after, optimize hot paths, add caching

### Risk 4: Template Injection Vulnerabilities

**Likelihood**: Low
**Impact**: High
**Mitigation**: Security audit, resource limits, input validation

---

## Success Criteria

### Functional Requirements

- [ ] All existing tests pass (no regression)
- [ ] Template rendering works correctly
- [ ] New validators work correctly
- [ ] Reporters generate valid output
- [ ] CLI integration seamless

### Quality Requirements

- [ ] Zero clippy warnings with `-D warnings`
- [ ] Test coverage >80%
- [ ] No `.unwrap()` in production code
- [ ] All error messages meaningful

### Performance Requirements

- [ ] Template rendering <1ms for typical files
- [ ] No memory leaks
- [ ] CI tests complete in <5 minutes

### Documentation Requirements

- [ ] All features documented
- [ ] Examples comprehensive
- [ ] Migration guide clear
- [ ] API documentation complete

---

## Post-Release Tasks

### Week 5+

1. Monitor GitHub issues for bug reports
2. Gather user feedback on templates
3. Plan v0.7.0 features based on feedback
4. Consider additional reporters (HTML, TAP, etc.)
5. Explore template inheritance support

---

## Team Communication

### Daily Standups

- What was completed yesterday?
- What is planned for today?
- Any blockers?

### Weekly Demos

- End of each week: Demo new features
- Gather feedback from stakeholders
- Adjust roadmap if needed

### Documentation

- Update this roadmap as tasks complete
- Track blockers in separate BLOCKERS.md
- Maintain CHANGELOG.md

---

## Appendix A: Dependency Checklist

Before starting, ensure these are available:

- [ ] Rust 1.70+ installed
- [ ] Docker or Podman running
- [ ] GitHub access for CI/CD
- [ ] crates.io publish permissions
- [ ] Code editor with Rust support

---

## Appendix B: Testing Checklist

### Unit Tests

- [ ] Template functions (20+ tests)
- [ ] Validators (30+ tests)
- [ ] Reporters (20+ tests)
- [ ] Config parsing (15+ tests)

### Integration Tests

- [ ] Template rendering + parsing (10+ tests)
- [ ] Full pipeline (10+ tests)

### E2E Tests

- [ ] Example templates (7 tests)
- [ ] Error handling (5+ tests)

### Red-Team Tests

- [ ] False positive detection (10+ tests)
- [ ] Edge cases (10+ tests)

### Performance Tests

- [ ] Rendering benchmarks (3+ tests)
- [ ] Validator benchmarks (3+ tests)

**Total Test Count**: 150+ tests

---

## Appendix C: Release Checklist

- [ ] Version bumped
- [ ] Changelog updated
- [ ] All tests passing
- [ ] Documentation updated
- [ ] Examples tested
- [ ] Performance benchmarks run
- [ ] Security audit complete
- [ ] Git tagged
- [ ] Published to crates.io
- [ ] Release notes written
- [ ] Announcement posted

---

## Conclusion

This roadmap provides a structured, week-by-week plan for implementing Tera templating in clnrm v0.6.0. By following this plan, the team can deliver a high-quality, well-tested, and well-documented feature on schedule.

**Estimated Effort**: 4 weeks (1-2 developers)
**Complexity**: Medium
**Risk**: Low (leveraging proven library)
**Value**: High (significant DX improvement)
