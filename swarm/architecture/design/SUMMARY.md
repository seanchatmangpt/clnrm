# Architecture Design Summary - CLNRM v0.6.0

**Designer**: System Designer (Architecture Sub-Coordinator)
**Date**: 2025-10-16
**Status**: Design Complete - Ready for Review

## Documents Delivered

### 1. System Architecture Design
**File**: `v0.6.0-system-architecture.md` (367 lines)

**Contents**:
- Executive summary and system overview
- Architecture principles (modular, dyn-compatible, environment-safe)
- Complete module structure (existing + new)
- Component architecture with detailed API specs
- Integration points (config loading pipeline)
- Plugin interfaces for template functions
- Error handling strategy with taxonomy
- Data flow diagrams
- Security architecture and threat model
- Performance considerations and optimization
- Testing strategy (unit, integration, property-based)
- Implementation roadmap (6 phases, 12-18 days)
- Acceptance criteria and DoD
- Complete function registry (4 existing + 13 new functions)

**Key Decisions**:
1. **Build on existing**: Extends current template system (40% implemented)
2. **Modular design**: All files < 500 lines
3. **dyn-compatible**: No async trait methods
4. **Backward compatible**: Zero breaking changes to existing TOML files
5. **Security first**: Sandboxed functions, no template injection
6. **Performance target**: < 100ms for 1000-step template

### 2. Module Dependency Graph
**File**: `module-dependency-graph.md` (227 lines)

**Contents**:
- Complete 7-layer dependency hierarchy
- Dependency rules (no circular, layer isolation, error independence)
- Template module detailed dependency graph
- External dependencies (uuid, rand NEW for v0.6.0)
- Module size constraints and current status
- Coupling metrics (afferent, efferent, instability)
- Dependency injection points
- Interface contracts
- Change impact analysis
- Cyclomatic complexity targets

**Key Metrics**:
- **Layers**: 7 layers from foundation (error.rs) to public API (lib.rs)
- **Coupling**: Low (error.rs is only high-Ca module, intentional)
- **Stability**: error.rs I=0.00 (maximally stable), lib.rs I>0.7 (abstract)
- **New modules**: 2 (generators.rs, registry.rs) - isolated, no breaking changes

### 3. Plugin Interface Specification
**File**: `plugin-interface-spec.md` (420 lines)

**Contents**:
- Template function plugin interface (Tera's Function trait)
- Template filter plugin interface (Tera's Filter trait)
- Generator plugin interface (pure functions)
- Registration API with detailed examples
- Plugin lifecycle (registration → rendering)
- Error handling contracts
- Testing requirements (unit, integration, property-based)
- Example implementations (simple, parameterized, filters)
- Plugin development checklist

**Key Contracts**:
1. **Thread safety**: All functions Sync + Send
2. **No panics**: Return tera::Error on failure
3. **Parameter validation**: Validate all required params
4. **Determinism**: Seeded variants produce identical output
5. **Performance**: < 10ms per function call

## Architecture Principles Adherence

### ✅ Modular Design (Files < 500 Lines)

| Module | Target | Status |
|--------|--------|--------|
| template/mod.rs | < 200 | ✅ 147 lines |
| template/context.rs | < 200 | ✅ 170 lines |
| template/determinism.rs | < 200 | ✅ 178 lines |
| template/functions.rs | < 500 | ✅ 382 lines |
| template/generators.rs | < 400 | 🔴 NEW (to implement) |
| template/registry.rs | < 150 | 🔴 NEW (to implement) |

**Result**: All existing modules meet constraint, new modules designed to fit.

### ✅ dyn-Compatible Traits (No Async Methods)

```rust
// ✅ All template functions use sync trait
pub trait Function {
    fn call(&self, args: &HashMap<String, Value>) -> tera::Result<Value>;
}

// ✅ All generators are pure sync functions
pub fn fake_uuid() -> String;
pub fn random_int(min: i64, max: i64) -> i64;
```

**Result**: No async methods in any trait or interface.

### ✅ Clean Separation of Concerns

```
Layer 1: error.rs (foundation)
Layer 2: template/generators.rs (pure data generation)
Layer 3: template/registry.rs (function registration)
Layer 4: config.rs (TOML loading + template rendering)
Layer 5: runtime (backend, services, cleanroom)
```

**Result**: Clear layering with no circular dependencies.

### ✅ Environment Safety (No Hardcoded Secrets)

```rust
// ✅ Environment variable access via function
{{ env(name="DB_PASSWORD") }}

// ✅ Lint warnings for hardcoded secrets
password = "secret123"  // ⚠️ WARNING: Hardcoded secret
```

**Result**: No hardcoded secrets, env_var() function for safe access.

## Error Handling Strategy

### Error Taxonomy

```
CleanroomError::TemplateError (already exists in error.rs)
├── Template Syntax Error
├── Function Execution Error
├── Rendering Error
└── Post-Rendering Error
```

### Error Handling Patterns

1. **Function errors**: Return tera::Error with context
2. **Generator errors**: Result<T> only if fallible, else infallible
3. **Config errors**: CleanroomError::template_error with file path
4. **Fail fast**: No recovery, abort on template errors

## Integration Strategy

### Modified Components

1. **config.rs**: Add `is_template_file()` and template rendering step (+50 lines)
2. **template/mod.rs**: Call `registry::register_all_functions()` (no size change)

### New Components

1. **template/generators.rs**: 13 generator functions (< 400 lines)
2. **template/registry.rs**: Function registration logic (< 150 lines)

### Unmodified Components

- error.rs (TemplateError already exists)
- template/context.rs (no changes)
- template/determinism.rs (no changes)
- template/functions.rs (no changes)
- All other modules (no impact)

## Testing Strategy

### Coverage Targets

| Module | Unit Tests | Integration | Property | Coverage |
|--------|-----------|-------------|----------|----------|
| generators.rs | 🔴 NEW (30+ tests) | 🔴 NEW | 🔴 NEW | > 90% |
| registry.rs | 🔴 NEW (10+ tests) | 🔴 NEW | N/A | > 90% |
| config.rs | ✅ Existing | ✅ Existing | N/A | Maintain |

### Test Types

1. **Unit tests**: Per-function validation, determinism, bounds
2. **Integration tests**: Full template rendering pipeline
3. **Property tests**: 160K+ generated test cases
4. **Performance tests**: < 100ms for 1000-step template

## Security Audit

### Threat Mitigation

| Threat | Mitigation | Status |
|--------|-----------|--------|
| Template injection | Sandboxed functions | ✅ Design |
| Secrets in templates | env_var() + lint | ✅ Design |
| Arbitrary code exec | Pure functions only | ✅ Design |
| DoS (infinite loops) | Tera timeouts | ✅ Built-in |
| Path traversal | No file system access | ✅ Design |

**Result**: All major threats mitigated by design.

## Performance Targets

| Metric | Target | Measurement |
|--------|--------|-------------|
| Empty template | < 1ms | Benchmark |
| Simple template (10 vars) | < 5ms | Benchmark |
| Medium template (100 steps) | < 50ms | Benchmark |
| Large template (1000 steps) | < 100ms | Benchmark |
| Function call | < 10ms | Unit test |

## Implementation Roadmap

### Phase 1: Fake Data Generators (3-4 days)
- Create generators.rs
- Implement UUID, name, email, timestamp, IP generators
- Add deterministic seeded variants
- 100+ unit tests

### Phase 2: Random Generators (2-3 days)
- Implement random_int, random_string, random_bool, random_choice
- Add seeded variants
- Property-based tests

### Phase 3: Function Registry (1-2 days)
- Create registry.rs
- Register all functions with Tera
- Integration tests

### Phase 4: Config Integration (2-3 days)
- Modify load_config_from_file()
- Add is_template_file()
- Pipeline tests

### Phase 5: Documentation (2-3 days)
- TEMPLATE_GUIDE.md
- Update TOML_REFERENCE.md
- Example templates

### Phase 6: Testing & Validation (2-3 days)
- Full test suite
- Performance benchmarks
- Security audit

**Total**: 12-18 days (2.5-3.5 weeks)

## Dependencies

### New Cargo Dependencies

```toml
[dependencies]
uuid = { version = "1.10", features = ["v4", "serde"] }  # 🔴 NEW
rand = "0.8"                                              # 🔴 NEW
```

### No Breaking Changes

All existing dependencies remain unchanged:
- tera = "1.19" ✅
- sha2 = "0.10" ✅
- chrono = "0.4" ✅

## Acceptance Criteria

### Definition of Done

- [ ] All generators implemented (13 functions)
- [ ] Function registry complete
- [ ] config.rs modified for template rendering
- [ ] File extension detection working (.tera, .toml.tera)
- [ ] All errors handled gracefully
- [ ] Unit tests (> 90% coverage)
- [ ] Integration tests (full pipeline)
- [ ] Property tests (160K+ cases)
- [ ] E2E test (100+ generated scenarios)
- [ ] Documentation complete
- [ ] No breaking changes
- [ ] `cargo clippy -- -D warnings` passes
- [ ] `cargo test` passes
- [ ] Framework self-test validates templating
- [ ] Performance: < 100ms for 1000 steps
- [ ] Security: No injection vulnerabilities
- [ ] All files < 500 lines
- [ ] No `.unwrap()` in production code

## Risks & Mitigation

| Risk | Impact | Probability | Mitigation |
|------|--------|-------------|------------|
| Performance regression | Medium | Low | Benchmark tests, caching |
| Template injection | High | Low | Sandboxed functions, security audit |
| Breaking changes | High | Low | Integration tests, backward compat |
| Complexity creep | Medium | Medium | File size limits, code review |

## Next Steps

1. **Review**: Architecture Sub-Coordinator reviews design
2. **Approval**: Get stakeholder sign-off
3. **Implementation**: Begin Phase 1 (Fake Data Generators)
4. **Iteration**: Adjust based on testing feedback
5. **Delivery**: v0.6.0 release with templating support

## Questions for Review

1. Are the module boundaries appropriate?
2. Is the error handling strategy sufficient?
3. Are performance targets realistic?
4. Should we add more security controls?
5. Is the implementation timeline reasonable?

---

**Designer Sign-Off**: System Designer
**Date**: 2025-10-16
**Status**: ✅ Ready for Architecture Review
