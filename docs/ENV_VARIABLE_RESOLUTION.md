# Environment Variable Resolution in Template Rendering

This document describes how environment variables are resolved in the clnrm template rendering system, following the PRD v1.0 specification.

## Overview

The template rendering system supports automatic environment variable resolution with a well-defined precedence chain. This allows flexible configuration without requiring environment variable prefixes in templates.

## Variable Precedence

Variables are resolved with the following priority (highest to lowest):

1. **Template Variables** (user-provided) - Highest priority
2. **Environment Variables** - Middle priority
3. **Default Values** - Lowest priority

### Example

```rust
// Given:
// - Template var: svc = "template-service"
// - ENV var: SERVICE_NAME = "env-service"
// - Default: "clnrm"

// Result: "template-service" (template var wins)
```

## Supported Environment Variables

| Template Variable | Environment Variable | Default Value | Description |
|------------------|---------------------|---------------|-------------|
| `svc` | `SERVICE_NAME` | `clnrm` | Service name for OpenTelemetry resources |
| `env` | `ENV` | `ci` | Deployment environment (ci, staging, production) |
| `endpoint` | `OTEL_ENDPOINT` | `http://localhost:4318` | OpenTelemetry collector endpoint |
| `exporter` | `OTEL_TRACES_EXPORTER` | `otlp` | OTEL exporter type (otlp, jaeger, zipkin, stdout) |
| `image` | `CLNRM_IMAGE` | `registry/clnrm:1.0.0` | Default container image for tests |
| `freeze_clock` | `FREEZE_CLOCK` | `2025-01-01T00:00:00Z` | Deterministic timestamp for reproducible tests |
| `token` | `OTEL_TOKEN` | `""` (empty) | Authentication token for OTEL collector |

## Usage in Templates

Variables can be accessed in templates without any prefix:

```toml
# Template file: test.clnrm.toml
[meta]
name = "{{ svc }}_test"
version = "1.0.0"

[otel]
exporter = "{{ exporter }}"
endpoint = "{{ endpoint }}"

[otel.resources]
"service.name" = "{{ svc }}"
"deployment.environment" = "{{ env }}"

[service.test]
plugin = "generic_container"
image = "{{ image }}"

[determinism]
freeze_clock = "{{ freeze_clock }}"
seed = 42
```

## Conditional Rendering

Environment variables work seamlessly with Tera conditionals:

```toml
{% if token != "" %}
[otel.headers]
"Authorization" = "Bearer {{ token }}"
{% endif %}

{% if env == "production" %}
[otel.headers]
"X-Environment" = "production"
{% endif %}
```

## Programmatic Usage

### Using render_template Function

```rust
use clnrm_core::template::render_template;
use std::collections::HashMap;

// Render with defaults (reads from ENV)
let rendered = render_template(template_str, HashMap::new())?;

// Override with user variables
let mut user_vars = HashMap::new();
user_vars.insert("svc".to_string(), serde_json::json!("my-service"));
user_vars.insert("endpoint".to_string(), serde_json::json!("http://custom:4318"));

let rendered = render_template(template_str, user_vars)?;
```

### Using TemplateContext

```rust
use clnrm_core::template::TemplateContext;

// Create context with ENV resolution
let context = TemplateContext::with_defaults();

// Variables are automatically resolved:
// - From user vars if provided
// - From ENV if set
// - From defaults otherwise

// Add custom variable with precedence
let mut context = TemplateContext::new();
context.add_var_with_precedence("svc", "SERVICE_NAME", "clnrm");

// Merge user variables (highest priority)
let mut user_vars = HashMap::new();
user_vars.insert("svc".to_string(), serde_json::json!("override"));
context.merge_user_vars(user_vars);
```

## Configuration Examples

### Example 1: CI Environment (Defaults)

```bash
# No ENV variables set
```

```toml
# Template
[meta]
name = "{{ svc }}_test"

[otel.resources]
"deployment.environment" = "{{ env }}"
```

**Rendered Output:**
```toml
[meta]
name = "clnrm_test"

[otel.resources]
"deployment.environment" = "ci"
```

### Example 2: Production Environment (ENV Variables)

```bash
export SERVICE_NAME=payment-service
export ENV=production
export OTEL_ENDPOINT=http://otel.prod.example.com:4318
export OTEL_TOKEN=prod-api-key-xyz
```

```toml
# Template
[meta]
name = "{{ svc }}_test"

[otel]
endpoint = "{{ endpoint }}"

[otel.resources]
"service.name" = "{{ svc }}"
"deployment.environment" = "{{ env }}"

{% if token != "" %}
[otel.headers]
"Authorization" = "Bearer {{ token }}"
{% endif %}
```

**Rendered Output:**
```toml
[meta]
name = "payment-service_test"

[otel]
endpoint = "http://otel.prod.example.com:4318"

[otel.resources]
"service.name" = "payment-service"
"deployment.environment" = "production"

[otel.headers]
"Authorization" = "Bearer prod-api-key-xyz"
```

### Example 3: User Override (Template Variables)

```bash
export SERVICE_NAME=env-service
export ENV=staging
```

```rust
// User provides template variables
let mut user_vars = HashMap::new();
user_vars.insert("svc".to_string(), serde_json::json!("user-service"));
user_vars.insert("env".to_string(), serde_json::json!("development"));

let rendered = render_template(template, user_vars)?;
```

**Result:** Template variables `user-service` and `development` are used, ENV variables are ignored.

## Implementation Details

### Precedence Resolution

The `TemplateContext::add_var_with_precedence` method implements the precedence chain:

```rust
pub fn add_var_with_precedence(&mut self, key: &str, env_key: &str, default: &str) {
    // 1. Check if variable already exists (template var - highest priority)
    if self.vars.contains_key(key) {
        return;
    }

    // 2. Try environment variable (second priority)
    if let Ok(env_value) = std::env::var(env_key) {
        self.vars.insert(key.to_string(), Value::String(env_value));
        return;
    }

    // 3. Use default (lowest priority)
    self.vars.insert(key.to_string(), Value::String(default.to_string()));
}
```

### Default Context Initialization

```rust
pub fn with_defaults() -> Self {
    let mut ctx = Self::new();

    // Resolve standard variables using precedence
    ctx.add_var_with_precedence("svc", "SERVICE_NAME", "clnrm");
    ctx.add_var_with_precedence("env", "ENV", "ci");
    ctx.add_var_with_precedence("endpoint", "OTEL_ENDPOINT", "http://localhost:4318");
    ctx.add_var_with_precedence("exporter", "OTEL_TRACES_EXPORTER", "otlp");
    ctx.add_var_with_precedence("image", "CLNRM_IMAGE", "registry/clnrm:1.0.0");
    ctx.add_var_with_precedence("freeze_clock", "FREEZE_CLOCK", "2025-01-01T00:00:00Z");
    ctx.add_var_with_precedence("token", "OTEL_TOKEN", "");

    ctx
}
```

### Tera Context Injection

Variables are injected at two levels for flexible access:

```rust
pub fn to_tera_context(&self) -> Result<Context> {
    let mut ctx = Context::new();

    // Top-level injection (no prefix) - allows {{ svc }}, {{ env }}, etc.
    for (key, value) in &self.vars {
        ctx.insert(key, value);
    }

    // Nested injection for authoring - allows {{ vars.svc }}, etc.
    ctx.insert("vars", &self.vars);
    ctx.insert("matrix", &self.matrix);
    ctx.insert("otel", &self.otel);

    Ok(ctx)
}
```

## Error Handling

All template rendering functions return `Result<String, CleanroomError>` with proper error handling:

```rust
// Template rendering errors
let result = render_template(invalid_template, vars);
match result {
    Ok(rendered) => println!("Success: {}", rendered),
    Err(e) => {
        // Error type: CleanroomError::TemplateError
        eprintln!("Template error: {}", e);
    }
}
```

No `.unwrap()` or `.expect()` calls are used in production code paths.

## Testing

Comprehensive tests validate the precedence chain and ENV resolution:

- `tests/env_variable_resolution_test.rs` - Full ENV resolution tests
- `tests/template_vars_rendering_test.rs` - Template variable tests
- `src/template/context.rs` - Unit tests for TemplateContext

### Running Tests

```bash
# Run all template tests
cargo test template

# Run ENV resolution tests
cargo test env_variable_resolution

# Run specific test
cargo test test_precedence_template_vars_override_env
```

## Best Practices

1. **Use ENV variables for deployment-specific config** (endpoints, credentials)
2. **Use template variables for test-specific overrides** (custom service names)
3. **Rely on defaults for local development** (no ENV setup needed)
4. **Keep templates portable** - avoid hardcoding environment-specific values

## Security Considerations

- **Never commit ENV values to version control** - use `.env` files (gitignored)
- **Use `OTEL_TOKEN` for sensitive credentials** - it defaults to empty for safety
- **Validate ENV values** - ensure they match expected formats
- **Use conditionals** - only include sensitive headers when token is set

## Migration Guide

### From Hardcoded Values

**Before:**
```toml
[otel]
endpoint = "http://localhost:4318"  # Hardcoded

[otel.resources]
"service.name" = "my-service"  # Hardcoded
```

**After:**
```toml
[otel]
endpoint = "{{ endpoint }}"  # ENV: OTEL_ENDPOINT

[otel.resources]
"service.name" = "{{ svc }}"  # ENV: SERVICE_NAME
```

### From env() Function

**Before:**
```toml
endpoint = "{{ env(name="OTEL_ENDPOINT") | default(value="http://localhost:4318") }}"
```

**After:**
```toml
endpoint = "{{ endpoint }}"  # Automatic ENV resolution with default
```

## Troubleshooting

### Variables Not Resolving

1. Check ENV variable is set: `echo $SERVICE_NAME`
2. Verify variable name matches mapping table
3. Check precedence - template vars override ENV
4. Review rendered output: ENV vars should be substituted

### Template Markers Still Visible

If `{{ var }}` appears in output:
- Variable not found in context
- Typo in variable name
- Missing from default context

### ENV Not Overriding Default

- Ensure ENV variable is exported: `export SERVICE_NAME=value`
- Verify ENV name matches table (e.g., `SERVICE_NAME` not `SVC`)
- Check if template var is overriding ENV

## References

- PRD v1.0: Template Variable Resolution Specification
- Tera Documentation: https://keats.github.io/tera/
- clnrm Template System: `crates/clnrm-core/src/template/`
