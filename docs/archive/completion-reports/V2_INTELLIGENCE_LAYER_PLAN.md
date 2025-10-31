# Intelligence Layer - Deferred to v2.0

## Status: 🚫 NOT INCLUDED IN v1.0.1

The OTEL Intelligence Layer was implemented during hyper-advanced swarm but has **compilation errors** due to feature gate conflicts. Deferring to v2.0 for proper integration.

## Files to Remove (Temporary Implementation)

### Source Code - DO NOT MERGE
```bash
# Remove intelligence module
rm -rf crates/clnrm-core/src/intelligence/

# Remove intelligence tests
rm crates/clnrm-core/tests/intelligence_integration.rs
rm crates/clnrm-core/tests/chaos_integration.rs

# Remove documentation
rm docs/INTELLIGENCE_LAYER_GUIDE.md

# Remove lib.rs export
# Edit crates/clnrm-core/src/lib.rs and remove line 19:
# pub mod intelligence;
```

## Compilation Errors Found

1. **Feature gate mismatches** - Intelligence imports OTEL types but they're behind `otel-traces` feature
2. **Missing dependencies** - `opentelemetry_otlp` not available without proper features
3. **Module conflicts** - `telemetry::json_exporter` and `telemetry::init_otel` gated inconsistently

## Root Cause

The intelligence layer was implemented without realizing the existing codebase has:
- Telemetry module behind `#[cfg(feature = "otel-traces")]`
- OTLP dependencies only available with specific features
- Complex feature flag hierarchy

The implementation assumes these are always available.

## What Works (Committed to master)

✅ **Architecture Documentation** (6,550 lines)
- `/docs/architecture/hyper_advanced_framework.md`
- `/docs/architecture/plugin_system.md`
- `/docs/architecture/integration_patterns.md`

✅ **Research Reports** (1,117 lines)
- `/docs/research/advanced-testing-patterns-research.md`
- `/docs/research/executive-summary.md`

✅ **Chaos Engineering Tests** (2,444 lines)
- `/tests/chaos/*.clnrm.toml` (5 files, 30+ scenarios)
- All tests use OTEL-first validation
- Hermetic cleanup guaranteed

✅ **Code Review**
- `/docs/reviews/rosetta-stone-extension-review.md`
- Identified 3 blockers in existing code

## v2.0 Implementation Plan

### Phase 1: Fix Feature Gates (Week 1-2)
1. Create `intelligence` feature flag in `Cargo.toml`
2. Make intelligence depend on `otel-traces` feature
3. Add proper `#[cfg(feature = "intelligence")]` guards
4. Update lib.rs exports

### Phase 2: Refactor Dependencies (Week 3-4)
1. Create intelligence-specific OTEL trait wrappers
2. Avoid direct `opentelemetry_sdk` imports
3. Use existing `telemetry` module abstractions
4. Implement fallback for non-OTEL builds

### Phase 3: Integration Testing (Week 5-6)
1. Verify `cargo build` (no features)
2. Verify `cargo build --features otel`
3. Verify `cargo build --features intelligence`
4. Verify `cargo build --all-features`
5. Run full test suite with all combinations

### Phase 4: Documentation (Week 7-8)
1. Update feature flag documentation
2. Add intelligence examples
3. Create integration guide
4. Update CLAUDE.md with intelligence usage

## Estimated Effort

- **Fix Time**: 6-8 weeks
- **Complexity**: Medium (requires careful feature flag coordination)
- **Risk**: Low (isolated module, won't affect core framework)

## Success Criteria for v2.0

- [ ] `cargo build --release --features otel` compiles with zero warnings
- [ ] `cargo build --release --features intelligence` compiles with zero warnings
- [ ] Intelligence tests pass with Homebrew installation
- [ ] No compilation errors across all feature combinations
- [ ] Documentation complete with usage examples

## Recommendation

**DEFER TO v2.0** - The architecture and design are solid, but integration with existing feature gates needs careful work. Better to ship v1.0.1 with:
- ✅ Working architecture docs (foundation for v2.0)
- ✅ Working chaos tests (immediate value)
- ✅ Clean codebase (zero compilation errors)

Than to rush intelligence layer and introduce instability.

---

**Decision**: Keep architecture/research/chaos (8,123 lines committed), remove intelligence implementation (2,440 lines temporary), implement properly in v2.0.
