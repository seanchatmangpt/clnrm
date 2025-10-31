# Weaver Code Generation Guide

This guide explains how to use Weaver to generate type-safe telemetry code from semantic convention schemas in the clnrm project.

## Overview

Weaver is an OpenTelemetry tool that generates code from semantic convention schemas. In clnrm, we use it to create:

- **Type-safe span builders** - Enforce required attributes at compile time
- **Type-safe metrics recorders** - Guarantee schema conformance
- **Mock traits for London TDD** - Enable testing without real OTEL infrastructure
- **Event emitters** - Structured logging with schema validation

## Directory Structure

```
clnrm/
├── registry/                           # Semantic convention schemas
│   └── (created by Schema-Architect agent)
├── templates/registry/rust/            # Weaver templates
│   ├── weaver.yaml                     # Template configuration
│   ├── spans.rs.j2                     # Span builder template
│   ├── metrics.rs.j2                   # Metrics recorder template
│   ├── mocks.rs.j2                     # Mock trait template
│   └── events.rs.j2                    # Event emitter template
└── crates/clnrm-core/src/telemetry/
    └── generated/                      # Generated code output
        ├── mod.rs                      # Generated module root
        ├── spans.rs                    # Generated span builders
        ├── metrics.rs                  # Generated metrics recorders
        ├── mocks.rs                    # Generated mock traits
        └── events.rs                   # Generated event emitters
```

## Installation

Install Weaver CLI:

```bash
cargo install weaver-cli
```

Verify installation:

```bash
weaver --version
```

## Generating Code

### Manual Generation

Run Weaver to generate code from schemas:

```bash
weaver registry generate rust \
  --registry registry/ \
  --templates templates/registry/rust/ \
  --output crates/clnrm-core/src/telemetry/generated/
```

### Automatic Generation (Build Script)

The project includes a `build.rs` that automatically regenerates code during builds:

```rust
// build.rs
fn main() {
    Command::new("weaver")
        .args(&["registry", "generate", "rust", ...])
        .status()
        .expect("Failed to generate telemetry code");
}
```

The build script:
- Checks if weaver is installed
- Regenerates code if schemas or templates change
- Fails gracefully if weaver is not available

## Template Structure

### spans.rs.j2 - Span Builders

Generates type-safe span builder structs:

```rust
// Generated from schema
pub struct TestExecutionSpan {
    span: Span,
}

impl TestExecutionSpan {
    // Required attributes become constructor parameters
    pub fn new(test_name: &str, container_image: &str) -> Self { ... }

    // Optional attributes become setter methods
    pub fn set_isolated(&self, value: bool) -> &Self { ... }
    pub fn set_result(&self, value: &str) -> &Self { ... }

    pub fn enter(&self) -> tracing::span::Entered<'_> { ... }
    pub fn end(self) { ... }
}
```

**Key features:**
- Required attributes = constructor parameters (compile-time enforcement)
- Optional attributes = setter methods (fluent API)
- Type safety - wrong types won't compile
- Chainable setters for ergonomic usage

### metrics.rs.j2 - Metric Recorders

Generates metric recorder structs:

```rust
#[cfg(feature = "otel-metrics")]
pub struct TestExecutionMetric {
    histogram: Histogram<f64>,
}

impl TestExecutionMetric {
    pub fn new(meter: &Meter) -> Self { ... }

    // Required attributes in signature
    pub fn record(&self, value: f64, test_name: &str, result: &str) { ... }
}
```

**Key features:**
- Feature-gated (`otel-metrics`) with no-op fallback
- Type-safe attribute recording
- Instrument types from schema (counter, histogram, gauge)

### mocks.rs.j2 - Mock Traits

Generates mockall traits for London TDD:

```rust
#[cfg(test)]
#[automock]
pub trait TestExecutionSpanTrait: Send + Sync {
    fn set_isolated(&self, value: bool);
    fn set_result(&self, value: String);
    fn end(self: Box<Self>);
}
```

**Key features:**
- Only compiled in test mode
- Mockall-compatible trait definitions
- Enables behavior verification in tests

### events.rs.j2 - Event Emitters

Generates structured event emitters:

```rust
pub struct TestStartedEvent;

impl TestStartedEvent {
    pub fn emit(test_name: &str, container_image: &str) {
        event!(Level::INFO, ...);
    }

    pub fn emit_with_optional(
        test_name: &str,
        isolated: Option<bool>,
        ...
    ) { ... }
}
```

## Usage Examples

### Type-Safe Span Usage

```rust
use clnrm_core::telemetry::generated::spans::TestExecutionSpan;

// Required attributes enforced at compile time
let span = TestExecutionSpan::new(
    "my_test",           // test_name (required)
    "alpine:latest",     // container_image (required)
);

// Optional attributes via setters
span.set_isolated(true)
    .set_result("pass");

// Enter span context
let _guard = span.enter();

// Automatic cleanup on drop
span.end();
```

**Benefits:**
- Cannot forget required attributes (won't compile)
- Cannot use wrong types (won't compile)
- IDE autocomplete for all attributes
- Self-documenting code

### Type-Safe Metrics Usage

```rust
#[cfg(feature = "otel-metrics")]
use clnrm_core::telemetry::generated::metrics::TestExecutionMetric;
use opentelemetry::global;

#[cfg(feature = "otel-metrics")]
{
    let meter = global::meter("clnrm");
    let metric = TestExecutionMetric::new(&meter);

    // Required attributes in signature
    metric.record(
        125.5,          // duration in ms
        "my_test",      // test_name (required)
        "pass",         // result (required)
    );
}
```

### Mock Usage in Tests

```rust
#[cfg(test)]
use clnrm_core::telemetry::generated::mocks::MockTestExecutionSpanTrait;
use mockall::predicate::*;

#[test]
fn test_telemetry_behavior() {
    let mut mock = MockTestExecutionSpanTrait::new();

    // Expect method calls
    mock.expect_set_result()
        .with(eq("pass".to_string()))
        .times(1)
        .returning(|_| ());

    // Verify behavior
    mock.set_result("pass".to_string());
}
```

## Schema Changes

When schemas change:

1. **Update schema files** in `registry/`
2. **Regenerate code**:
   ```bash
   weaver registry generate rust \
     --registry registry/ \
     --templates templates/registry/rust/ \
     --output crates/clnrm-core/src/telemetry/generated/
   ```
3. **Review generated diffs** - check breaking changes
4. **Update usage** - fix any compilation errors
5. **Run tests** - verify behavior unchanged

## Build Integration

### Cargo.toml Configuration

```toml
[build-dependencies]
# No extra dependencies needed - weaver is CLI tool
```

### build.rs Configuration

The build script automatically:
- Detects schema changes via `cargo:rerun-if-changed`
- Checks for weaver installation
- Regenerates code if schemas/templates change
- Provides helpful warnings if weaver not found

### CI/CD Integration

In CI pipelines:

```yaml
# .github/workflows/ci.yml
- name: Install Weaver
  run: cargo install weaver-cli

- name: Generate telemetry code
  run: |
    weaver registry generate rust \
      --registry registry/ \
      --templates templates/registry/rust/ \
      --output crates/clnrm-core/src/telemetry/generated/

- name: Verify no uncommitted changes
  run: git diff --exit-code
```

## Troubleshooting

### Weaver Not Found

**Error:** `weaver: command not found`

**Solution:** Install weaver-cli:
```bash
cargo install weaver-cli
```

### Schema Validation Errors

**Error:** `Invalid schema: missing required field 'type'`

**Solution:** Validate schemas match OpenTelemetry spec:
```bash
weaver registry check --registry registry/
```

### Template Rendering Errors

**Error:** `Template error: undefined variable 'span_name'`

**Solution:** Check template uses correct schema field names from OTel spec

### Generated Code Won't Compile

**Error:** Rust compilation errors in generated code

**Solution:**
1. Check Jinja2 template syntax
2. Verify type mappings in template
3. Test with simple schema first

## Best Practices

1. **Version Control Generated Code**
   - Commit generated code to git
   - Enables code review of schema changes
   - Provides build fallback if weaver unavailable

2. **Schema First Development**
   - Design schemas before implementation
   - Generate code, then implement logic
   - Type system guides implementation

3. **Review Generated Diffs**
   - Schema changes visible in generated code diffs
   - Catch unintended changes early
   - Document breaking changes

4. **Test Generated Code**
   - Generated code includes basic tests
   - Add integration tests for actual usage
   - Use mocks for unit testing

5. **Keep Templates DRY**
   - Extract common patterns to macros
   - Share filters across templates
   - Document custom filters

## Advanced Features

### Custom Filters

Add Rust-specific filters to templates:

```jinja2
{# Custom filter for PascalCase #}
{{ group.id | pascal_case }}

{# Custom filter for snake_case #}
{{ attr.name | snake_case }}
```

### Conditional Generation

Generate different code based on schema:

```jinja2
{% if group.instrument == "counter" %}
counter.add(value, &attributes);
{% elif group.instrument == "histogram" %}
histogram.record(value, &attributes);
{% endif %}
```

### Multi-Language Support

Weaver supports multiple languages:

```bash
# Generate Python code
weaver registry generate python \
  --registry registry/ \
  --templates templates/registry/python/

# Generate Go code
weaver registry generate go \
  --registry registry/ \
  --templates templates/registry/go/
```

## Resources

- [Weaver Documentation](https://github.com/open-telemetry/weaver)
- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [Jinja2 Template Documentation](https://jinja.palletsprojects.com/)
- [clnrm Telemetry Architecture](./TELEMETRY_ARCHITECTURE.md)

## Summary

Weaver code generation provides:

✅ **Type Safety** - Compile-time enforcement of schemas
✅ **DRY Principle** - Single source of truth (schemas)
✅ **Consistency** - Generated code follows patterns
✅ **Maintainability** - Schema changes propagate automatically
✅ **Testability** - Generated mocks for London TDD
✅ **Documentation** - Self-documenting through types

Generated code is the bridge between semantic conventions and implementation, ensuring clnrm's telemetry matches industry standards.
