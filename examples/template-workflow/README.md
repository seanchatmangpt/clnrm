# Template Workflow Example

This directory demonstrates the CLI template rendering workflow integration.

## Quick Start

### 1. Generate OTEL Template

```bash
# Output to stdout
clnrm template otel

# Save to file
clnrm template otel -o my-otel-test.clnrm.toml
```

### 2. Run Template-Based Test

```bash
# Run with default environment
clnrm run otel-template-example.clnrm.toml

# Run with custom OTEL exporter
OTEL_EXPORTER=jaeger OTEL_ENDPOINT=http://jaeger:4318 \
  clnrm run otel-template-example.clnrm.toml

# Run in CI mode with determinism
CI=true clnrm run otel-template-example.clnrm.toml
```

### 3. Template Features

The example template demonstrates:
- **Environment variable access** via `env()` function
- **Default values** using `| default(value='...')`
- **Conditional blocks** with `{% if %}` statements
- **Variable substitution** with `{{ vars.name }}`

## Template Syntax

### Variables
```toml
name = "{{ vars.name | default(value='my_test') }}"
```

### Environment Variables
```toml
exporter = "{{ env(name='OTEL_EXPORTER') | default(value='stdout') }}"
```

### Conditionals
```toml
{% if env(name='CI') == 'true' %}
[determinism]
seed = 42
{% endif %}
```

## Backward Compatibility

Non-template files work unchanged:

```toml
# Regular TOML - no template processing
[test.metadata]
name = "my_test"
```

Template files are automatically detected:

```toml
# Template TOML - automatically rendered
[test.metadata]
name = "{{ vars.name | default(value='my_test') }}"
```

## Template Detection

Templates are detected by checking for Tera syntax:
- `{{ variable }}` - Variable substitution
- `{% for x in list %}` - Control structures
- `{# comment #}` - Comments

## Available Functions

- `env(name="VAR")` - Access environment variables
- `hash(value)` - SHA-256 hashing
- `now()` - Current timestamp (respects `freeze_clock`)
- `toml_encode(value)` - TOML encoding

## Error Handling

If a template variable is missing:
```
Error: Template rendering failed in 'test.clnrm.toml':
  Variable 'required_var' not found
```

Use `default()` filter to provide fallbacks:
```toml
name = "{{ vars.required_var | default(value='fallback') }}"
```

## See Also

- [CLI Template Workflow Guide](../../docs/CLI_TEMPLATE_WORKFLOW.md)
- [Template System Documentation](../../docs/TEMPLATE_SYSTEM.md)
- [OTEL Validation Guide](../../docs/OTEL_VALIDATION.md)
