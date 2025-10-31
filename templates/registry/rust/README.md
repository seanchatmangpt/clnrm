# Rust Code Generation Templates for Weaver

This directory contains Jinja2 templates for generating type-safe Rust telemetry code from OpenTelemetry semantic convention schemas.

## Files

- **weaver.yaml** - Template configuration defining output files and filters
- **spans.rs.j2** - Generates type-safe span builder structs
- **metrics.rs.j2** - Generates metric recorder structs with feature gates
- **mocks.rs.j2** - Generates mockall traits for London TDD testing
- **events.rs.j2** - Generates structured event emitters

## Usage

### Generate Code

```bash
weaver registry generate rust \
  --registry ../../registry/ \
  --templates . \
  --output ../../crates/clnrm-core/src/telemetry/generated/
```

### Automatic Generation

Code is automatically generated during `cargo build` via `build.rs` if:
1. Weaver CLI is installed (`cargo install weaver-cli`)
2. Registry directory exists with schemas
3. Templates directory exists

## Template Structure

### spans.rs.j2

Generates span builder structs with:
- **Required attributes** → Constructor parameters (compile-time enforcement)
- **Optional attributes** → Setter methods (fluent API)
- **Utility methods** → enter(), span(), into_span(), end()
- **Unit tests** → Basic smoke tests

**Example Output:**
```rust
pub struct TestExecutionSpan {
    span: Span,
}

impl TestExecutionSpan {
    pub fn new(test_name: &str, container_image: &str) -> Self { ... }
    pub fn set_isolated(&self, value: bool) -> &Self { ... }
    pub fn enter(&self) -> tracing::span::Entered<'_> { ... }
}
```

### metrics.rs.j2

Generates metric recorder structs with:
- **Feature gates** → `#[cfg(feature = "otel-metrics")]`
- **No-op fallback** → Works without OTEL features
- **Instrument types** → Counter, Histogram, Gauge
- **Required attributes** → Method parameters

**Example Output:**
```rust
#[cfg(feature = "otel-metrics")]
pub struct TestExecutionMetric {
    histogram: Histogram<f64>,
}

impl TestExecutionMetric {
    pub fn record(&self, value: f64, test_name: &str, result: &str) { ... }
}
```

### mocks.rs.j2

Generates mockall-compatible trait definitions:
- **Test-only** → `#[cfg(test)]`
- **Mockall integration** → `#[automock]`
- **Send + Sync** → Thread-safe mocks
- **Unit tests** → Mock usage examples

**Example Output:**
```rust
#[cfg(test)]
#[automock]
pub trait TestExecutionSpanTrait: Send + Sync {
    fn set_isolated(&self, value: bool);
    fn set_result(&self, value: String);
}
```

### events.rs.j2

Generates event emitter structs:
- **Simple emit()** → Required attributes only
- **emit_with_optional()** → Includes optional attributes
- **tracing::event!** → Uses tracing macros
- **Structured logging** → Key-value attributes

**Example Output:**
```rust
pub struct TestStartedEvent;

impl TestStartedEvent {
    pub fn emit(test_name: &str, container_image: &str) { ... }
}
```

## Type Mappings

Templates map OpenTelemetry types to Rust types:

| OTel Type | Rust Type | Notes |
|-----------|-----------|-------|
| `string` | `&str` | Borrowed string |
| `int` | `i64` | 64-bit signed integer |
| `double` | `f64` | 64-bit float |
| `boolean` | `bool` | Boolean |
| `string[]` | `Vec<String>` | Vector of strings |
| `enum` | `impl Into<String>` | Custom types |

## Custom Filters

Templates use custom Jinja2 filters:

- **`pascal_case`** - Convert to PascalCase (e.g., `test_execution` → `TestExecution`)
- **`snake_case`** - Convert to snake_case (e.g., `TestExecution` → `test_execution`)
- **`selectattr`** - Filter list by attribute value
- **`equalto`** - Equality comparison in filters

**Example Usage:**
```jinja2
{% set struct_name = group.id | pascal_case %}
{% for attr in group.attributes | selectattr("requirement_level", "equalto", "required") %}
```

## Schema Format

Templates expect OpenTelemetry semantic convention schemas:

```yaml
groups:
  - id: test.execution
    type: span
    brief: "Describes a test execution"
    span_name: "test.execution"
    attributes:
      - id: test.name
        type: string
        requirement_level: required
        brief: "Name of the test"
      - id: test.isolated
        type: boolean
        requirement_level: optional
        brief: "Whether test runs in isolation"
```

## Conditional Generation

Templates use Jinja2 conditionals for different code paths:

```jinja2
{% if group.type == "span" %}
// Generate span builder
{% elif group.type == "metric" %}
// Generate metric recorder
{% endif %}

{% if attr.requirement_level == "required" %}
// Constructor parameter
{% else %}
// Setter method
{% endif %}
```

## Feature Gates

Metrics are feature-gated for conditional compilation:

```jinja2
#[cfg(feature = "otel-metrics")]
pub struct {{ struct_name }}Metric { ... }

#[cfg(not(feature = "otel-metrics"))]
pub struct {{ struct_name }}Metric; // No-op
```

## Documentation Generation

Templates generate Rust doc comments from schema documentation:

```jinja2
/// {{ group.brief }}
{% if group.note %}
///
/// {{ group.note }}
{% endif %}
pub struct {{ struct_name }}Span { ... }
```

## Validation

Templates include basic validation:

```jinja2
{% if not group.id %}
{{ raise("Group missing required 'id' field") }}
{% endif %}
```

## Best Practices

1. **DRY Templates** - Extract common patterns to Jinja2 macros
2. **Type Safety** - Map OTel types precisely to Rust types
3. **Feature Gates** - Use for optional dependencies
4. **Documentation** - Generate from schema descriptions
5. **Unit Tests** - Include in generated code
6. **Error Handling** - Handle missing fields gracefully

## Troubleshooting

### Template Syntax Error

**Error:** `TemplateSyntaxError: unexpected '}}'`

**Solution:** Check Jinja2 syntax - braces must be balanced, use `{#` for comments

### Undefined Variable

**Error:** `UndefinedError: 'group' is undefined`

**Solution:** Verify variable exists in schema context, check filter pipeline

### Type Mismatch

**Error:** Generated code has type errors

**Solution:** Check type mapping table, verify OTel type matches Rust type

### Missing Filter

**Error:** `FilterError: No filter named 'pascal_case'`

**Solution:** Ensure Weaver has custom filters enabled, update weaver version

## Resources

- [Weaver Documentation](https://github.com/open-telemetry/weaver)
- [Jinja2 Documentation](https://jinja.palletsprojects.com/)
- [OpenTelemetry Semantic Conventions](https://opentelemetry.io/docs/specs/semconv/)
- [clnrm Usage Examples](../../../docs/USAGE_EXAMPLES.md)
- [clnrm Codegen Guide](../../../docs/WEAVER_CODEGEN_GUIDE.md)

## Contributing

When modifying templates:

1. Test with sample schemas
2. Verify generated code compiles
3. Run `cargo test` to verify functionality
4. Update documentation with changes
5. Run verification script: `tests/verify_weaver_setup.sh`

## License

Same as clnrm project - see LICENSE file in repository root.
