# Weaver Migration Quick Reference

**Full Plan:** [WEAVER_REFACTOR_MIGRATION_PLAN.md](WEAVER_REFACTOR_MIGRATION_PLAN.md)

---

## TL;DR

**Problem:** v1.1.0 uses test-based validation (can have false positives)
**Solution:** v1.2.0 uses Weaver schema-based validation (proves runtime behavior)
**Timeline:** 10 weeks (Q1 2026)
**Effort:** 400 hours

---

## 5 Phases

| Phase | Weeks | Goal | Risk |
|-------|-------|------|------|
| 1. Schema Definition | 1-2 | Define telemetry schemas | Low |
| 2. Code Generation | 3-4 | Generate type-safe builders | Medium |
| 3. Engine Refactor | 5-6 | Migrate all components | High |
| 4. Weaver Integration | 7-8 | CI/CD validation | Medium |
| 5. Documentation | 9-10 | Finalize release | Low |

---

## Component Migration Order

**Week 5 (Priority 1 - Critical Path):**
1. TestEngine
2. Container backend
3. Service manager

**Week 6 (Priority 2-3 - Supporting):**
4. Plugin registry
5. Command executor
6. Step executor
7. Config loader
8. Validation system
9. CLI commands

---

## Migration Pattern (Per Component)

```bash
# 1. Identify telemetry callsites
grep -n "spans::" src/component_name/

# 2. Create branch
git checkout -b refactor/component-weaver-spans

# 3. Replace manual spans with generated builders
# Before:
use crate::telemetry::spans;
let span = spans::test_span(name);

# After:
use crate::telemetry::TestExecutionSpan;
let span = TestExecutionSpan::new(container_id, name, true);

# 4. Write mocks from schema (London TDD)
#[test]
fn test_span_has_required_attributes() {
    let span = TestExecutionSpan::new("ctr", "test", true);
}

# 5. Validate
cargo build --features otel
cargo test --features otel

# 6. Commit
git commit -m "refactor(component): Migrate to Weaver-generated spans"

# 7. Validate with Weaver
clnrm validate-telemetry --registry registry/ --tests tests/
```

---

## Key Commands

```bash
# Schema validation
weaver registry check -r registry/

# Code generation
weaver registry generate rust -r registry/ -t templates/registry/rust/

# Telemetry validation
clnrm validate-telemetry --registry registry/ --tests tests/

# Build with OTEL
cargo build --release --features otel
```

---

## Success Criteria (v1.2.0 Release)

- [ ] 90%+ schema coverage
- [ ] 0 manual span creation in production
- [ ] 100% CI/CD runs with Weaver validation
- [ ] <5% false positive rate
- [ ] All tests passing
- [ ] Documentation complete

---

## File Structure

```
clnrm/
├── registry/                    # ⭐ NEW: Telemetry schemas
│   ├── core/                   # Span schemas
│   ├── metrics/                # Metric schemas
│   └── events/                 # Event schemas
│
├── templates/registry/          # ⭐ NEW: Weaver templates
│   └── rust/                   # Rust codegen templates
│
└── crates/clnrm-core/src/
    └── telemetry/
        ├── generated/          # ⭐ NEW: Weaver-generated code
        │   ├── spans.rs       # Type-safe span builders
        │   └── metrics.rs     # Type-safe metric recorders
        └── validation.rs       # ⭐ NEW: Weaver live-check
```

---

## Before/After Comparison

### Before (v1.1.0) - Manual Spans

```rust
use crate::telemetry::spans;

pub async fn execute_test(&self, test: &Test) -> Result<TestResult> {
    let span = spans::test_span(&test.name);
    let _guard = span.enter();
    // ... test execution ...
}
```

**Problems:**
- ❌ No schema validation
- ❌ No type safety
- ❌ Can forget required attributes
- ❌ No CI/CD validation
- ❌ False positive risk

### After (v1.2.0) - Generated Builders

```rust
use crate::telemetry::TestExecutionSpan;

pub async fn execute_test(&self, test: &Test) -> Result<TestResult> {
    let container_id = self.container.id();
    let span = TestExecutionSpan::new(
        container_id,          // Required by schema
        &test.name,            // Required by schema
        true,                  // Required by schema
    );
    let _guard = span.enter();

    let result = // ... test execution ...

    span.set_result(TestResult::from(result)); // Type-safe enum
}
```

**Benefits:**
- ✅ Schema-validated
- ✅ Type-safe builders
- ✅ Compiler enforces required attributes
- ✅ CI/CD validates actual telemetry
- ✅ Zero false positive risk

---

## Risk Mitigation

| Risk | Mitigation |
|------|------------|
| Schema doesn't capture behaviors | Audit all existing telemetry first |
| Generated code doesn't integrate | Incremental migration per component |
| Weaver fails in CI/CD | Parallel validation initially |
| Performance overhead | Run on sample subset, cache results |

---

## Rollback Plan

- **Phase 1-2:** No code changes, safe to rollback
- **Phase 3:** Revert component migrations, keep legacy code
- **Phase 4:** Disable CI/CD validation, use manual validation
- **Phase 5:** Emergency rollback to v1.1.0

---

## Next Steps (Week 1)

```bash
# 1. Create registry structure
mkdir -p registry/{core,metrics,events}
mkdir -p templates/registry/{rust,docs}

# 2. Define registry manifest
vim registry/registry_manifest.yaml

# 3. Define test execution schema
vim registry/core/test_execution.yaml

# 4. Validate schema
weaver registry check -r registry/

# 5. Iterate until schema passes
```

---

## Documentation

- **Full Plan:** [WEAVER_REFACTOR_MIGRATION_PLAN.md](WEAVER_REFACTOR_MIGRATION_PLAN.md)
- **Weaver Integration:** [WEAVER_INTEGRATION_PLAN.md](WEAVER_INTEGRATION_PLAN.md)
- **v1.1.0 Status:** [V1.1.0_FINAL_STATUS_AND_V1.2.0_PATH.md](V1.1.0_FINAL_STATUS_AND_V1.2.0_PATH.md)

---

**Status:** READY FOR IMPLEMENTATION
**Priority:** CRITICAL
**Timeline:** 10 weeks
**Start Date:** Q1 2026

🚀 **Ship it!**
