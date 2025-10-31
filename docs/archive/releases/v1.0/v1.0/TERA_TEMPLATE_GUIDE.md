# Cleanroom v1.0 Tera Template Guide

## 🎯 Overview

**✅ IMPLEMENTED** in v0.7.0+ - Cleanroom v1.0 uses **Tera** (Rust implementation of Jinja2) for templating with a simplified variable model. Variables are resolved in Rust before template rendering, eliminating the need for prefixes like `vars.` in templates.

## 🚀 Key Features

### No-Prefix Variables
```toml
# ✅ v0.7.0+ style (implemented)
name = "{{ svc }}_test"
```

### Clean Variable Resolution
Variables are resolved in Rust using this precedence:
1. **Template variables** (highest)
2. **Environment variables**
3. **Default values** (lowest)

### Simplified Syntax
```toml
# Simple variable
endpoint = "{{ endpoint }}"

# With default (if Tera filters supported)
endpoint = "{{ endpoint | default(value='http://localhost:4318') }}"

# Conditional content
{% if token != "" %}
[otel.headers]
Authorization = "Bearer {{ token }}"
{% endif %}
```

## 📋 Available Variables

| Variable | Description | Example |
|----------|-------------|---------|
| `svc` | Service name | `"myapp"` |
| `env` | Environment | `"prod"`, `"ci"`, `"dev"` |
| `endpoint` | OTEL endpoint | `"http://localhost:4318"` |
| `exporter` | OTEL exporter | `"otlp"`, `"stdout"` |
| `image` | Container image | `"myapp:latest"` |
| `freeze_clock` | Deterministic timestamp | `"2025-01-01T00:00:00Z"` |
| `token` | OTEL authentication token | `"abc123..."` |

## 🎨 Template Examples

### Basic Service Configuration
```toml
[meta]
name = "{{ svc }}_integration_test"
version = "1.0"
description = "{{ svc }} service tests"

[service.{{ svc }}]
plugin = "generic_container"
image = "{{ image }}"
args = ["test", "--service", "{{ svc }}"]
env = {
  "SERVICE_NAME" = "{{ svc }}",
  "ENVIRONMENT" = "{{ env }}"
}
wait_for_span = "{{ svc }}.ready"

[[scenario]]
name = "{{ svc }}_basic_test"
service = "{{ svc }}"
run = "./test.sh {{ svc }}"
artifacts.collect = ["spans:default"]
```

### Multi-Environment Configuration
```toml
[otel]
exporter = "{{ exporter }}"
{% if env == "prod" %}
endpoint = "https://otel-collector.prod.company.com:4318"
sample_ratio = 0.1
{% else %}
endpoint = "{{ endpoint }}"
sample_ratio = 1.0
{% endif %}
resources = {
  "service.name" = "{{ svc }}",
  "deployment.env" = "{{ env }}"
}
```

### Dynamic Scenario Generation
```toml
# Generate multiple scenarios based on conditions
{% for test_type in ["unit", "integration", "e2e"] %}
[[scenario]]
name = "{{ svc }}_{{ test_type }}_test"
service = "{{ svc }}"
run = "test --type={{ test_type }}"
artifacts.collect = ["spans:{{ test_type }}"]
{% endfor %}
```

### Conditional Headers
```toml
[otel.headers]
{% if token != "" %}
Authorization = "Bearer {{ token }}"
{% endif %}
X-Service = "{{ svc }}"
X-Environment = "{{ env }}"
```

## 🔧 Template Functions

### Built-in Tera Functions
- `default(value)` - Provide default value
- `length` - Get string/array length
- `trim` - Remove whitespace
- `upper`, `lower` - Case conversion
- `replace(from, to)` - String replacement

### Custom Cleanroom Functions
- `now_rfc3339()` - Current timestamp (respects determinism)
- `sha256(s)` - SHA-256 hash
- `toml_encode(value)` - TOML encoding
- `env(name)` - Environment variable access

## 📚 Template Structure

### File Organization
```
tests/
├── api.clnrm.toml           # Static configuration
├── database.clnrm.toml      # Static configuration
├── templates/
│   ├── base.toml.tera       # Base template
│   ├── api.toml.tera        # API-specific template
│   └── db.toml.tera         # Database-specific template
└── scenarios/
    └── load-test.toml.tera  # Scenario template
```

### Template Composition
```toml
# templates/base.toml.tera
[meta]
name = "{{ svc }}_test"
version = "1.0"

[vars]
svc = "{{ svc }}"
env = "{{ env }}"

[otel]
exporter = "{{ exporter }}"
endpoint = "{{ endpoint }}"

# Include service-specific configuration
{% include "service.toml.tera" %}
```

## 🎯 Best Practices

### Variable Usage
```toml
# ✅ Good: Use resolved variables directly
endpoint = "{{ endpoint }}"

# ✅ Good: Use in strings
run = "curl {{ endpoint }}/health"

# ❌ Avoid: Complex logic in templates (resolve in Rust)
# Complex conditional logic should be in Rust variable resolution
```

### Template Organization
```toml
# ✅ Good: Group related content
[meta]
name = "{{ svc }}_{{ env }}_test"
description = "{{ svc }} tests for {{ env }} environment"

[service.{{ svc }}]
# ... service configuration

# ✅ Good: Use clear variable names
{% if enable_debug %}
[otel]
sample_ratio = 1.0
{% else %}
[otel]
sample_ratio = 0.1
{% endif %}
```

### Error Handling
```toml
# Templates should be simple - complex logic in Rust
# If a variable might not resolve, handle it in Rust variable resolution
# rather than in template conditionals
```

## 🔄 Variable Resolution Flow

### 1. Rust Resolution
```rust
// Variables resolved before template rendering
let resolved = resolve(user_vars); // template vars → ENV → defaults

// Example resolution
resolved = {
  "svc": "myapp",              // from template vars or SERVICE_NAME or "clnrm"
  "env": "prod",               // from ENV or "ci"
  "endpoint": "https://otel.prod.company.com:4318", // from OTEL_ENDPOINT
  // ... other variables
}
```

### 2. Template Rendering
```rust
// Variables injected into Tera context
let mut context = Context::new();
for (k, v) in &resolved {
    context.insert(k, v);
}
context.insert("vars", &resolved); // Optional nested access

// Render template
let rendered = tera.render("template.toml.tera", &context)?;
```

### 3. Runtime Execution
```toml
# Rendered output (no template syntax)
[meta]
name = "myapp_prod_test"
version = "1.0"

[otel]
endpoint = "https://otel.prod.company.com:4318"
exporter = "otlp"
# ... resolved configuration
```

## 📊 Performance Considerations

### Template Complexity
- Keep templates simple and focused
- Complex logic should be in Rust variable resolution
- Use template includes for reusability

### Variable Access
- Variables are resolved once in Rust (fast)
- Template rendering is optimized
- No runtime variable resolution overhead

## 🚀 Migration from v0.7.0

### Before (v0.7.0)
```toml
[vars]
svc = "myapp"
env = "prod"

[meta]
name = "{{ vars.svc }}_{{ vars.env }}_test"

[otel]
endpoint = "{{ vars.endpoint | default(value='http://localhost:4318') }}"
```

### After (v1.0)
```toml
[meta]
name = "{{ svc }}_{{ env }}_test"

[otel]
endpoint = "{{ endpoint }}"
```

### Key Changes
- Remove `[vars]` table (runtime ignores it anyway)
- Remove `vars.` prefixes from variable references
- Variables resolved in Rust before template rendering
- Simpler, cleaner template syntax

## 🎯 Advanced Patterns

### Environment-Specific Configuration
```toml
{% if env == "prod" %}
[otel]
exporter = "otlp"
endpoint = "https://otel-collector.prod.company.com:4318"
sample_ratio = 0.1
{% else %}
[otel]
exporter = "stdout"
sample_ratio = 1.0
{% endif %}
```

### Service Matrix Testing
```toml
{% for service in ["api", "worker", "scheduler"] %}
[service.{{ service }}]
plugin = "generic_container"
image = "{{ service }}-service:{{ env }}"
wait_for_span = "{{ service }}.ready"

[[scenario]]
name = "{{ service }}_health_check"
service = "{{ service }}"
run = "curl http://{{ service }}:8080/health"
{% endfor %}
```

## 🔧 Troubleshooting

### Common Issues

**"Variable not found"**
- Check variable name spelling
- Ensure variable is in the resolution table
- Variables must be defined in Rust resolution logic

**"Template syntax error"**
- Validate Tera syntax with `clnrm dry-run`
- Check for unmatched braces or quotes
- Use `clnrm render --map` to debug variable resolution

**"Conditional logic not working"**
- Complex conditionals should be in Rust
- Keep template logic simple
- Use `clnrm dry-run` to validate templates

---

*Tera templating in v1.0 emphasizes simplicity and performance. Variables are resolved in Rust, templates focus on clean syntax, and complex logic stays in the application layer.*
