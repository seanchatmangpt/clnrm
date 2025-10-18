# Template System Mastery

This chapter covers advanced template usage with Tera templates, macro libraries, and variable resolution for creating reusable, maintainable test configurations.

## Overview

clnrm's template system enables:
- Dynamic configuration with Tera templates
- Reusable macro libraries (8 macros available)
- Variable precedence (template → ENV → defaults)
- Deterministic rendering
- Custom template functions

## Tera Template Basics

### Basic Template Syntax

```toml
# Basic variable substitution
[test.metadata]
name = "{{ svc }}_test"
description = "Test for {{ svc }} service"

[services.{{ svc }}]
type = "generic_container"
image = "{{ image }}"
ports = [{{ port }}]
```

### Conditional Logic

```toml
# Conditional configuration
{% if otel_enabled %}
[otel]
exporter = "{{ exporter }}"
endpoint = "{{ endpoint }}"
{% endif %}

{% if debug_mode %}
[debug]
verbose = true
log_level = "debug"
{% endif %}
```

### Loops and Iteration

```toml
# Loop through services
{% for service in services %}
[services.{{ service.name }}]
type = "{{ service.type }}"
image = "{{ service.image }}"
ports = [{{ service.port }}]
{% endfor %}
```

## Macro Library Usage

### Available Macros

The macro library provides 8 reusable macros:

| Macro | Purpose | Example |
|-------|---------|---------|
| `span(name, kind, attrs)` | Single span validation | `{{ m::span("api.request", "server") }}` |
| `lifecycle(service)` | Service start/exec/stop | `{{ m::lifecycle("postgres") }}` |
| `edges(pairs)` | Parent-child relationships | `{{ m::edges([["parent", "child"]]) }}` |
| `window(start, end)` | Time containment | `{{ m::window("tx", "query") }}` |
| `count(kind, min, max)` | Span count constraints | `{{ m::count("internal", 5) }}` |
| `multi_lifecycle(services)` | Batch lifecycles | `{{ m::multi_lifecycle(["db", "cache"]) }}` |
| `span_with_attrs(...)` | Span + attributes | `{{ m::span_with_attrs("req", "server", {...}) }}` |
| `attrs(pairs)` | Inline attribute table | `{{ m::attrs({"key": "val"}) }}` |

### Using Macros

```toml
{% import "_macros.toml.tera" as m %}

[test.metadata]
name = "{{ svc }}_macro_example"

# Service lifecycle using macro
[services.{{ svc }}]
type = "generic_container"
image = "{{ image }}"

{{ m::lifecycle("{{ svc }}") }}

# Span validation using macro
{{ m::span("clnrm.run", kind="internal", attrs={"result":"pass"}) }}

{{ m::span("{{ svc }}.request", kind="server", attrs={"http.method":"GET"}) }}

# Count validation using macro
{{ m::count("internal", 2, 2) }}
```

### Macro Composition

```toml
{% import "_macros.toml.tera" as m %}

# Multi-service lifecycle
{{ m::multi_lifecycle(["api", "database", "cache"]) }}

# Span relationships
{{ m::edges([
  ["clnrm.run", "api.start"],
  ["api.exec", "database.exec"],
  ["api.exec", "cache.exec"]
]) }}

# Temporal constraints
{{ m::window("api.exec", "database.exec") }}
{{ m::window("api.exec", "cache.exec") }}
```

## Variable Resolution

### Precedence Order

Variables are resolved in this order:
1. **Template variables** (highest priority)
2. **Environment variables**
3. **Default values** (lowest priority)

### Template Variables

```toml
# Template variables (highest priority)
[test.metadata]
name = "{{ svc }}_test"

[services.{{ svc }}]
image = "{{ image }}"
port = {{ port }}
```

### Environment Variables

```bash
# Set environment variables
export SERVICE_NAME=myapi
export IMAGE_NAME=nginx:alpine
export PORT_NUMBER=80
```

```toml
# Use environment variables
[test.metadata]
name = "{{ svc }}_test"  # Uses SERVICE_NAME from ENV

[services.{{ svc }}]
image = "{{ image }}"    # Uses IMAGE_NAME from ENV
port = {{ port }}        # Uses PORT_NUMBER from ENV
```

### Default Values

```rust
// Default values in Rust resolver
fn resolve(vars: HashMap<String,String>) -> HashMap<String,String> {
    let mut out = HashMap::new();
    out.insert("svc".into(), pick(&vars, "svc", "SERVICE_NAME", "clnrm"));
    out.insert("image".into(), pick(&vars, "image", "IMAGE_NAME", "alpine:latest"));
    out.insert("port".into(), pick(&vars, "port", "PORT_NUMBER", "80"));
    out
}
```

## Custom Template Functions

### Built-in Functions

clnrm provides several built-in template functions:

```toml
# Environment function
[test.metadata]
name = "{{ env(name="SERVICE_NAME") }}_test"

# Current time function
[test.metadata]
timestamp = "{{ now_rfc3339() }}"

# SHA-256 hash function
[test.metadata]
hash = "{{ sha256("test_data") }}"

# TOML encoding function
[test.metadata]
config = {{ toml_encode({"key": "value"}) }}
```

### Custom Function Example

```rust
// Custom template function
struct CustomFunction;
impl Function for CustomFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let input = args.get("input")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        
        // Custom logic here
        let result = format!("custom_{}", input);
        Ok(Value::String(result))
    }
}

// Register function
tera.register_function("custom", CustomFunction);
```

```toml
# Use custom function
[test.metadata]
name = "{{ custom(input="test") }}_service"
```

## Multi-Environment Templates

### Environment-Specific Configuration

```toml
{% import "_macros.toml.tera" as m %}

[test.metadata]
name = "{{ svc }}_{{ env }}_test"
description = "{{ svc }} test for {{ env }} environment"

# Environment-specific service configuration
[services.{{ svc }}]
type = "generic_container"
image = "{{ image }}"
ports = [{{ port }}]

{% if env == "production" %}
env_vars = { 
    "LOG_LEVEL" = "info",
    "DEBUG" = "false"
}
{% elif env == "staging" %}
env_vars = { 
    "LOG_LEVEL" = "debug",
    "DEBUG" = "true"
}
{% else %}
env_vars = { 
    "LOG_LEVEL" = "trace",
    "DEBUG" = "true"
}
{% endif %}

# Environment-specific OTEL configuration
{% if otel_enabled %}
[otel]
exporter = "{{ exporter }}"
endpoint = "{{ endpoint }}"
{% if env == "production" %}
sample_ratio = 0.1
{% else %}
sample_ratio = 1.0
{% endif %}
{% endif %}

# Service lifecycle
{{ m::lifecycle("{{ svc }}") }}

# Span validation
{{ m::span("clnrm.run", kind="internal", attrs={"result":"pass"}) }}
{{ m::span("{{ svc }}.start", kind="internal", attrs={"env":"{{ env }}"}) }}
```

### Matrix Testing

```toml
{% import "_macros.toml.tera" as m %}

# Matrix testing across multiple dimensions
{% for service in matrix.services %}
{% for env in matrix.environments %}
[test.metadata]
name = "{{ service.name }}_{{ env }}_matrix_test"

[services.{{ service.name }}]
type = "{{ service.type }}"
image = "{{ service.image }}"
env_vars = { "ENVIRONMENT" = "{{ env }}" }

{{ m::lifecycle("{{ service.name }}") }}
{{ m::span("{{ service.name }}.start", kind="internal", attrs={"env":"{{ env }}"}) }}

{% endfor %}
{% endfor %}
```

## Deterministic Rendering

### Ensuring Determinism

```toml
# Deterministic configuration
[determinism]
seed = 42
freeze_clock = "{{ freeze_clock }}"

[test.metadata]
name = "{{ svc }}_deterministic_test"
timestamp = "{{ freeze_clock }}"
```

### Idempotent Templates

```toml
# Idempotent template (renders same output multiple times)
[test.metadata]
name = "{{ svc }}_idempotent_test"

# Use deterministic values
[services.{{ svc }}]
image = "{{ image }}"
ports = [{{ port }}]
env_vars = { 
    "SERVICE_NAME" = "{{ svc }}",
    "ENVIRONMENT" = "{{ env }}",
    "VERSION" = "{{ version }}"
}
```

## Advanced Patterns

### Dynamic Test Generation

```toml
{% import "_macros.toml.tera" as m %}

# Generate tests dynamically based on configuration
{% for test_case in test_cases %}
[test.metadata]
name = "{{ test_case.name }}_test"
description = "{{ test_case.description }}"

[services.{{ test_case.service }}]
type = "generic_container"
image = "{{ test_case.image }}"

{% for step in test_case.steps %}
[[steps]]
name = "{{ step.name }}"
command = {{ step.command | toml_encode }}
expected_output_regex = "{{ step.expected }}"
{% endfor %}

{{ m::lifecycle("{{ test_case.service }}") }}
{{ m::span("{{ test_case.service }}.test", kind="internal", attrs={"test_case":"{{ test_case.name }}"}) }}

{% endfor %}
```

### Template Inheritance

```toml
# Base template (base.clnrm.toml.tera)
{% import "_macros.toml.tera" as m %}

[test.metadata]
name = "{{ svc }}_base_test"

[services.{{ svc }}]
type = "generic_container"
image = "{{ image }}"

{{ m::lifecycle("{{ svc }}") }}
{{ m::span("clnrm.run", kind="internal") }}
```

```toml
# Extended template (extended.clnrm.toml.tera)
{% extends "base.clnrm.toml.tera" %}
{% import "_macros.toml.tera" as m %}

[test.metadata]
name = "{{ svc }}_extended_test"

# Additional service
[services.{{ svc }}_cache]
type = "generic_container"
image = "redis:alpine"

# Additional spans
{{ m::span("{{ svc }}_cache.start", kind="internal") }}
{{ m::count("internal", 3, 3) }}
```

## Best Practices

### 1. Use Descriptive Variable Names

```toml
# ✅ Good: Descriptive variable names
[test.metadata]
name = "{{ service_name }}_{{ environment }}_test"

[services.{{ service_name }}]
image = "{{ service_image }}"
port = {{ service_port }}
```

### 2. Provide Sensible Defaults

```rust
// ✅ Good: Sensible defaults
out.insert("svc".into(), pick(&vars, "svc", "SERVICE_NAME", "clnrm"));
out.insert("env".into(), pick(&vars, "env", "ENVIRONMENT", "test"));
out.insert("port".into(), pick(&vars, "port", "PORT_NUMBER", "80"));
```

### 3. Use Macros for Common Patterns

```toml
# ✅ Good: Use macros for common patterns
{% import "_macros.toml.tera" as m %}

{{ m::lifecycle("{{ svc }}") }}
{{ m::span("clnrm.run", kind="internal") }}
{{ m::count("internal", 2, 2) }}
```

### 4. Validate Template Output

```bash
# ✅ Good: Validate template rendering
clnrm template render test.clnrm.toml.tera > test.clnrm.toml
clnrm validate test.clnrm.toml
```

## Common Patterns

### Multi-Service Template

```toml
{% import "_macros.toml.tera" as m %}

[test.metadata]
name = "{{ svc }}_multi_service_test"

# Database service
[services.{{ svc }}_db]
type = "generic_container"
image = "postgres:15-alpine"

# Cache service
[services.{{ svc }}_cache]
type = "generic_container"
image = "redis:7-alpine"

# API service
[services.{{ svc }}_api]
type = "generic_container"
image = "nginx:alpine"

# Service lifecycles
{{ m::multi_lifecycle(["{{ svc }}_db", "{{ svc }}_cache", "{{ svc }}_api"]) }}

# Span relationships
{{ m::edges([
  ["clnrm.run", "{{ svc }}_db.start"],
  ["{{ svc }}_db.start", "{{ svc }}_cache.start"],
  ["{{ svc }}_cache.start", "{{ svc }}_api.start"]
]) }}

# Temporal constraints
{{ m::window("{{ svc }}_db.start", "{{ svc }}_cache.start") }}
{{ m::window("{{ svc }}_cache.start", "{{ svc }}_api.start") }}
```

### Environment-Specific Template

```toml
{% import "_macros.toml.tera" as m %}

[test.metadata]
name = "{{ svc }}_{{ env }}_test"

[services.{{ svc }}]
type = "generic_container"
image = "{{ image }}"

{% if env == "production" %}
env_vars = { 
    "LOG_LEVEL" = "info",
    "DEBUG" = "false",
    "SAMPLE_RATIO" = "0.1"
}
{% else %}
env_vars = { 
    "LOG_LEVEL" = "debug",
    "DEBUG" = "true",
    "SAMPLE_RATIO" = "1.0"
}
{% endif %}

{{ m::lifecycle("{{ svc }}") }}
{{ m::span("{{ svc }}.start", kind="internal", attrs={"env":"{{ env }}"}) }}
```

## Next Steps

Now that you understand template system mastery:

1. **Try the examples**: Run the code samples in this chapter
2. **Create your own templates**: Build reusable templates for your use cases
3. **Learn production deployment**: Move on to [Production Deployment](../production-deployment/README.md)
4. **Master advanced patterns**: Review [Advanced Testing Patterns](../advanced-patterns/README.md)

## Further Reading

- [Plugin Development](../plugin-development/README.md)
- [Advanced Testing Patterns](../advanced-patterns/README.md)
- [Production Deployment](../production-deployment/README.md)
- [Template Reference](../docs/PRD-v1.md)
