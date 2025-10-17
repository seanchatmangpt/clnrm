# ADR-001: Tera Templating Integration for v0.6.0

**Status**: APPROVED
**Date**: 2025-10-16
**Deciders**: Architecture Sub-Coordinator, System Designer, API Architect
**Technical Story**: Integrate Tera templating for property-based testing and dynamic test generation

## Context and Problem Statement

CLNRM v0.4.x supports only static TOML configuration files. Users need to create hundreds of similar test scenarios manually, leading to:
- Code duplication (60-80% similar test configs)
- Manual maintenance burden
- Inability to generate property-based tests
- No support for dynamic test data generation

The existing template implementation (added in recent commits) provides basic Tera support but lacks:
- Comprehensive fake data generators (only has env, now_rfc3339, sha256, toml_encode)
- Integration with config loading pipeline
- Template validation and error handling
- Property-based testing capabilities

## Decision Drivers

1. **Developer Experience**: Reduce test configuration code by 80%+
2. **Property-Based Testing**: Enable thousands of test scenarios from templates
3. **Backward Compatibility**: Zero breaking changes to existing `.toml` files
4. **Core Team Standards**: No `.unwrap()`, sync traits, proper error handling
5. **Production Quality**: Comprehensive validation, testing, and error messages

## Considered Options

### Option 1: Extend Existing Template System (SELECTED)
- Build on existing `crates/clnrm-core/src/template/` module
- Add missing fake data generators to `functions.rs`
- Integrate with config loading pipeline
- Maintain backward compatibility via extension detection

### Option 2: External Templating Tool
- Use external tools (Jinja2, Handlebars CLI) to pre-process TOML
- Keep core framework simple
- **Rejected**: Poor integration, extra dependencies, manual workflow

### Option 3: Custom DSL
- Create custom domain-specific language for test generation
- Full control over syntax and features
- **Rejected**: High implementation cost, learning curve, ecosystem fragmentation

## Decision Outcome

**Chosen option**: "Option 1: Extend Existing Template System"

### Architecture Overview

```
┌─────────────────────────────────────────────────────────────────┐
│                   Template Rendering Pipeline                    │
└─────────────────────────────────────────────────────────────────┘

┌──────────────────┐
│ .clnrm.toml file │  ← Existing static TOML (unchanged)
└────────┬─────────┘
         │
         ▼
    [is_template()?]
         │
    ┌────┴────┐
    │ YES     │ NO (skip rendering)
    ▼         ▼
┌──────────┐  ┌────────────┐
│ Template │  │ Raw TOML   │
│ Render   │  └──────┬─────┘
└────┬─────┘         │
     │               │
     ▼               │
┌──────────┐         │
│ Rendered │         │
│ TOML     │         │
└────┬─────┘         │
     └───────────────┘
             │
             ▼
    ┌────────────────┐
    │ parse_toml()   │
    └────────┬───────┘
             │
             ▼
    ┌────────────────┐
    │ validate()     │
    └────────┬───────┘
             │
             ▼
        [TestConfig]
```

### Key Components

#### 1. Template Detection (mod.rs)
```rust
/// Check if content contains Tera template syntax
pub fn is_template(content: &str) -> bool {
    content.contains("{{") || content.contains("{%") || content.contains("{#")
}
```

**Decision**: Content-based detection (not file extension)
- **Rationale**: More flexible, works with any file
- **Trade-off**: Slight performance cost for scanning content
- **Mitigation**: O(n) scan only once at load time

#### 2. Function Registry (functions.rs)

**Existing Functions** (already implemented):
- `env(name)` - Environment variable access
- `now_rfc3339()` - Deterministic timestamps
- `sha256(s)` - SHA-256 hashing
- `toml_encode(value)` - TOML literal encoding

**NEW Functions** (to be added):
```rust
// Fake Data Generators
fake_uuid() -> String
fake_name() -> String
fake_email() -> String
fake_ipv4() -> String
fake_int(min, max) -> i64
fake_string(length) -> String
fake_bool() -> bool

// Property Testing
property_range(start, end) -> Vec<i64>
random_choice(items) -> String
```

**Decision**: Pure function approach (no side effects)
- **Rationale**: Deterministic, testable, safe
- **Trade-off**: Cannot perform I/O operations
- **Benefit**: Thread-safe, cache-friendly

#### 3. Error Handling

**Current**: `CleanroomError::template_error()` exists (line 197 in error.rs)

**Enhancement**: Rich error context
```rust
pub fn template_error(message: impl Into<String>) -> Self {
    Self::new(ErrorKind::TemplateError, message)
        .with_context("Template rendering phase")
}
```

**Decision**: Structured error messages with location info
- **Rationale**: Developer experience is critical
- **Implementation**: Tera's error messages provide line/column info
- **Example**:
  ```
  Error: Template rendering failed
    ┌─ test.clnrm.toml:15:20
    │
  15│ name = "{{ unknown_func() }}"
    │             ^^^^^^^^^^^^ unknown function
    │
    = help: Available: env, now_rfc3339, sha256, fake_uuid, ...
  ```

### Integration Points

#### 1. Config Loading Pipeline (config/mod.rs)

**Current**: Direct TOML parsing
```rust
pub fn load_config_from_file(path: &Path) -> Result<TestConfig> {
    let content = std::fs::read_to_string(path)?;
    let config = parse_toml_config(&content)?;
    config.validate()?;
    Ok(config)
}
```

**Enhanced**: Template-aware loading
```rust
pub fn load_config_from_file(path: &Path) -> Result<TestConfig> {
    let content = std::fs::read_to_string(path)?;

    // NEW: Render template if detected
    let rendered = if template::is_template(&content) {
        let mut renderer = TemplateRenderer::new()?;
        renderer.render_str(&content, path.to_str().unwrap_or("unknown"))?
    } else {
        content
    };

    let config = parse_toml_config(&rendered)?;
    config.validate()?;
    Ok(config)
}
```

**Decision**: Transparent template rendering before parsing
- **Rationale**: Zero changes to parsing/validation logic
- **Benefit**: Backward compatible, minimal code changes
- **Performance**: ~1-50ms overhead for templates (acceptable)

#### 2. Library Exports (lib.rs)

**Current**: Template exports exist (line 22, 48)
```rust
pub mod template;
pub use template::{DeterminismConfig, TemplateContext, TemplateRenderer};
```

**Enhancement**: Export additional helpers
```rust
pub use template::{
    DeterminismConfig,
    TemplateContext,
    TemplateRenderer,
    is_template,  // NEW: Helper for detection
    fake_data,    // NEW: Fake data module
};
```

### Backward Compatibility Strategy

**Guarantee**: ALL existing `.clnrm.toml` files work unchanged

| File Content | Behavior | Impact |
|-------------|----------|--------|
| No template syntax | Parse directly | ✅ Zero change |
| Contains `{{ }}` | Render, then parse | ✅ New feature |
| Contains `{% %}` | Render, then parse | ✅ New feature |

**Decision**: Content-based detection (not extension-based)
- **Rationale**: More flexible than `.tera` extension requirement
- **Trade-off**: Potential false positives if `{{` appears in TOML strings
- **Mitigation**: Rare occurrence, documented escape patterns

### Testing Strategy (London TDD)

**Phase 1: Unit Tests** (functions.rs)
```rust
#[test]
fn test_fake_uuid_generates_valid_uuid() {
    let uuid = fake_uuid();
    assert!(Uuid::parse_str(&uuid).is_ok());
}

#[test]
fn test_fake_int_stays_in_bounds() {
    for _ in 0..1000 {
        let val = fake_int(10, 20);
        assert!(val >= 10 && val <= 20);
    }
}
```

**Phase 2: Integration Tests** (template module)
```rust
#[tokio::test]
async fn test_template_rendering_with_fake_data() {
    let template = r#"
        [test.metadata]
        name = "{{ fake_name() }}"
        id = "{{ fake_uuid() }}"
    "#;

    let mut renderer = TemplateRenderer::new().unwrap();
    let rendered = renderer.render_str(template, "test").unwrap();

    assert!(rendered.contains("name = "));
    assert!(rendered.contains("id = "));
}
```

**Phase 3: Property-Based Tests** (proptest)
```rust
proptest! {
    #[test]
    fn fake_int_never_panics(min in -1000i64..1000, max in -1000i64..1000) {
        if min <= max {
            let val = fake_int(min, max);
            assert!(val >= min && val <= max);
        }
    }
}
```

**Phase 4: E2E Tests** (dogfooding)
```rust
#[tokio::test]
async fn test_property_based_load_test_execution() {
    // Generate 100 test scenarios from template
    let config = load_config_from_file("tests/load-test.clnrm.toml").unwrap();
    assert_eq!(config.steps.len(), 100);

    // Execute all scenarios
    let env = CleanroomEnvironment::new().await.unwrap();
    let results = env.execute_config(config).await.unwrap();
    assert!(results.all_passed());
}
```

**Decision**: Test-first development with mocking
- **Rationale**: London TDD style with test doubles
- **Benefit**: Fast feedback, isolated unit tests
- **Trade-off**: More mocking code, potential coupling

### Performance Considerations

**Benchmark Results** (estimated):

| Scenario | Template Size | Rendering Time | Memory |
|----------|--------------|----------------|--------|
| Empty template | N/A | ~0.1ms | <1MB |
| Small (10 functions) | 50 lines | ~1ms | ~1MB |
| Medium (100 iterations) | 500 lines | ~10ms | ~5MB |
| Large (1000 iterations) | 5000 lines | ~50ms | ~20MB |

**Decision**: Acceptable overhead for testing use case
- **Rationale**: Rendering happens once at config load
- **Optimization**: Cache rendered templates (future enhancement)
- **Monitoring**: Add metrics for template rendering time

### Security Considerations

**Threat Model**:
1. **Template Injection**: User-provided template strings
   - **Mitigation**: Templates are source-controlled files
   - **Risk**: LOW

2. **Arbitrary Code Execution**: Malicious Tera functions
   - **Mitigation**: All functions pure, no I/O
   - **Risk**: LOW

3. **Secrets Leakage**: Hardcoded passwords in templates
   - **Mitigation**: `env()` function for secrets, linting
   - **Risk**: MEDIUM (user error)

**Decision**: Sandboxed function execution
```rust
// All functions are pure - no side effects
impl Function for FakeUuidFunction {
    fn call(&self, _args: &HashMap<String, Value>) -> tera::Result<Value> {
        // Pure computation only - no I/O, network, filesystem
        Ok(Value::String(Uuid::new_v4().to_string()))
    }
}
```

## Consequences

### Positive

1. **80%+ Code Reduction**: Template loops replace repetitive TOML
2. **Property-Based Testing**: Generate thousands of test scenarios
3. **Backward Compatible**: Zero breaking changes
4. **Production Quality**: Comprehensive error handling and testing
5. **Extensible**: Easy to add new template functions

### Negative

1. **Complexity**: Additional abstraction layer (template → TOML)
2. **Learning Curve**: Developers must learn Tera syntax
3. **Debugging**: Template errors less intuitive than TOML errors
4. **Performance**: 1-50ms overhead for template rendering

### Neutral

1. **Dependency**: Adds `tera` crate (already added in v0.4.x)
2. **Testing Burden**: More test coverage needed for templates
3. **Documentation**: Need template guide and examples

## Implementation Roadmap

### Phase 1: Fake Data Functions (Week 1)
- [ ] Add `fake_uuid()`, `fake_name()`, `fake_email()` to functions.rs
- [ ] Add `fake_int()`, `fake_string()`, `fake_bool()`
- [ ] Add `property_range()`, `random_choice()`
- [ ] Unit tests for all functions (>90% coverage)

### Phase 2: Integration (Week 2)
- [ ] Enhance `load_config_from_file()` with template rendering
- [ ] Add template detection logic
- [ ] Integration tests with full TOML templates
- [ ] Error handling with rich context

### Phase 3: Testing & Docs (Week 3)
- [ ] Property-based tests for random generators
- [ ] E2E tests with 100+ generated scenarios
- [ ] Template guide documentation
- [ ] Example templates in `examples/templating/`

### Total Timeline: 3 weeks

## Links

- [Tera Templating Architecture](./tera-templating-architecture.md) - Full design document
- [Tera Templating Quick Reference](./tera-templating-quick-reference.md) - Developer guide
- [Template Implementation Summary](./tera-templating-implementation-summary.md) - Summary
- [OTEL Validation ADR](./ADR-002-otel-validation-enhancement.md) - Related decision

## Notes

- Template module already exists in `crates/clnrm-core/src/template/`
- Basic Tera integration complete (env, now_rfc3339, sha256, toml_encode)
- Need to add comprehensive fake data generators
- Need to integrate with config loading pipeline
- Existing `CleanroomError::template_error()` in error.rs (line 197)
