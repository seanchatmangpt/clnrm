# CLI Template Rendering Workflow Integration

## Overview

The CLI now supports automatic template rendering for `.clnrm.toml` configuration files. This enables dynamic test generation with Tera templating syntax while maintaining full backward compatibility.

## Key Features

### 1. Automatic Template Detection

The `load_config_from_file` function automatically detects if a file contains template syntax:

```rust
// In config.rs
pub fn load_config_from_file(path: &std::path::Path) -> Result<TestConfig> {
    use crate::template::{TemplateRenderer, is_template};

    let content = std::fs::read_to_string(path)?;

    // Check if template rendering is needed
    let toml_content = if is_template(&content) {
        let mut renderer = TemplateRenderer::new()?;
        renderer.render_str(&content, path.to_str().unwrap_or("config"))?
    } else {
        content  // Use as-is for backward compatibility
    };

    let config = parse_toml_config(&toml_content)?;
    config.validate()?;
    Ok(config)
}
```

### 2. Template Detection Logic

Templates are detected by checking for Tera syntax:
- `{{ variable }}` - Variable substitution
- `{% for x in list %}` - Control structures
- `{# comment #}` - Comments

### 3. Backward Compatibility

**Non-template files work unchanged:**
```toml
# Regular TOML - no template processing
[test.metadata]
name = "my_test"
description = "Standard test"
```

**Template files use Tera syntax:**
```toml
# Template TOML - automatically rendered
[test.metadata]
name = "{{ vars.name | default(value='my_test') }}"
description = "Test with {{ env(name='ENVIRONMENT') | default(value='dev') }}"
```

## CLI Commands

### Generate OTEL Template

Generate a complete OTEL validation template:

```bash
# Output to stdout
clnrm template otel

# Save to file
clnrm template otel -o tests/otel-validation.clnrm.toml

# Generate lifecycle matcher
clnrm template lifecycle-matcher -o macros/lifecycle.tera
```

### Run Tests with Templates

Templates are automatically detected and rendered during `clnrm run`:

```bash
# Run template-based test
clnrm run tests/templated-test.clnrm.toml

# Templates support environment variables
OTEL_EXPORTER=jaeger clnrm run tests/otel-template.clnrm.toml
```

## Template Examples

### Basic OTEL Template

```toml
# clnrm OTEL validation template (v0.6.0)
[meta]
name = "{{ vars.name | default(value='otel_validation') }}"
version = "0.6.0"

[otel]
exporter = "{{ env(name='OTEL_EXPORTER') | default(value='stdout') }}"
{% if otel.endpoint %}
endpoint = "{{ otel.endpoint }}"
{% endif %}

[service.clnrm]
plugin = "generic_container"
image = "{{ vars.image | default(value='alpine:latest') }}"

{% if vars.deterministic %}
[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"
{% endif %}
```

### Lifecycle Matcher Macro

```toml
{% macro container_lifecycle_events() %}
["container.start", "container.exec", "container.stop"]
{% endmacro %}

[[expect.span]]
name = "{{ container_lifecycle_events()[0] }}"
kind = "internal"
```

## Template Functions

Templates have access to custom Tera functions:

- `env(name="VAR")` - Access environment variables
- `hash(value)` - SHA-256 hashing
- `now()` - Current timestamp (respects `freeze_clock`)
- `toml_encode(value)` - TOML encoding

## Error Handling

Template rendering errors are reported with context:

```
Error: Template rendering failed in 'tests/my-test.clnrm.toml':
  Variable 'required_var' not found

  Template context:
  - vars: {...}
  - env: {...}
```

## Implementation Details

### File Structure

```
crates/clnrm-core/src/
├── config.rs                    # Updated load_config_from_file
├── cli/
│   ├── mod.rs                  # Updated Template command handler
│   ├── types.rs                # Added output parameter
│   └── commands/
│       ├── mod.rs              # Export template functions
│       └── template.rs         # Template generators
└── template/
    ├── mod.rs                  # TemplateRenderer, is_template()
    ├── context.rs              # TemplateContext
    ├── determinism.rs          # DeterminismConfig
    └── functions.rs            # Custom Tera functions
```

### Key Integration Points

1. **Config Loading** (`config.rs:load_config_from_file`)
   - Automatic template detection
   - Renders templates before TOML parsing
   - Validates rendered output

2. **CLI Handler** (`cli/mod.rs`)
   - `template otel` - Generate OTEL template
   - `template lifecycle-matcher` - Generate macro
   - Regular templates unchanged

3. **Template Command** (`cli/commands/template.rs`)
   - `generate_otel_template()` - OTEL template
   - `generate_lifecycle_matcher()` - Lifecycle macro
   - `generate_from_template()` - Project templates

## Testing

### Unit Tests

```rust
#[test]
fn test_template_detection() {
    assert!(is_template("{{ var }}"));
    assert!(is_template("{% for x in list %}"));
    assert!(!is_template("[test]\nname = 'value'"));
}
```

### Integration Tests

```bash
# Test template rendering
cargo test --test template_integration

# Test CLI workflow
cargo run -- template otel -o /tmp/test.toml
cargo run -- run /tmp/test.toml
```

## Migration Guide

### From Static TOML to Templates

**Before:**
```toml
[test.metadata]
name = "my_test"

[service.app]
image = "myapp:latest"
```

**After (with templates):**
```toml
[test.metadata]
name = "{{ vars.test_name | default(value='my_test') }}"

[service.app]
image = "{{ env(name='APP_IMAGE') | default(value='myapp:latest') }}"
```

### Environment-Based Configuration

**Use case:** Different config per environment

```toml
[otel]
exporter = "{{ env(name='OTEL_EXPORTER') | default(value='stdout') }}"
endpoint = "{{ env(name='OTEL_ENDPOINT') | default(value='http://localhost:4318') }}"

{% if env(name='CI') == 'true' %}
[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"
{% endif %}
```

## Performance

- **No overhead for non-templates** - Direct TOML parsing
- **Template caching** - Tera compiles templates once
- **Fast rendering** - ~1-2ms for typical templates

## Future Enhancements

- [ ] Template variable injection via CLI flags
- [ ] Template includes and inheritance
- [ ] Pre-compiled template bundles
- [ ] Template marketplace integration

## Troubleshooting

### Template Syntax Errors

```
Error: Template rendering failed in 'test.clnrm.toml':
  unexpected end of input, expected `}}`
```

**Fix:** Check for unclosed `{{ }}` or `{% %}`

### Missing Variables

```
Error: Variable 'required_var' not found
```

**Fix:** Provide variable via context or use `default()` filter:
```toml
name = "{{ vars.required_var | default(value='fallback') }}"
```

### TOML Parse Errors After Rendering

```
Error: TOML parse error: invalid key
```

**Fix:** Ensure template output is valid TOML. Debug by running:
```bash
clnrm template otel  # View rendered output
```

## See Also

- [Template System Documentation](TEMPLATE_SYSTEM.md)
- [OTEL Validation Guide](OTEL_VALIDATION.md)
- [CLI Reference](CLI_GUIDE.md)
