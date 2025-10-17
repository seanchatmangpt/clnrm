# Variable Resolution Guide for Cleanroom v1.0.0

## Overview

Cleanroom v1.0.0 introduces a **Rust-based variable resolution system** that handles variable precedence in Rust code rather than in templates. This approach provides:

- **Clear precedence chain**: Template variables → Environment variables → Defaults
- **Type safety**: Variables are resolved with proper error handling
- **Performance**: Resolution happens once at startup, not during template rendering
- **Maintainability**: Complex logic is in Rust, templates stay simple

## Precedence Chain

Variables are resolved in this order (highest to lowest priority):

### 1. Template Variables (Highest Priority)

Define variables directly in the `[vars]` section of your template:

```toml
[vars]
svc = "my-custom-service"
endpoint = "https://otel.custom.example.com"
exporter = "otlp-grpc"

[meta]
name = "{{ svc }}_test"  # Uses "my-custom-service"
```

Template variables override both environment variables and defaults.

### 2. Environment Variables

If no template variable is defined, Cleanroom checks environment variables:

```toml
[meta]
name = "{{ svc }}_test"  # Uses $SERVICE_NAME if no template var
```

**Common Environment Variables:**
- `$SERVICE_NAME` → `svc` variable
- `$OTEL_ENDPOINT` → `endpoint` variable
- `$OTEL_TRACES_EXPORTER` → `exporter` variable
- `$CLNRM_IMAGE` → `image` variable
- `$FREEZE_CLOCK` → `freeze_clock` variable
- `$OTEL_TOKEN` → `token` variable

### 3. Built-in Defaults (Lowest Priority)

If neither template nor environment variables are set, Cleanroom uses sensible defaults:

```toml
[meta]
name = "{{ svc }}_test"  # Uses "clnrm" if no template or ENV var
```

## Available Variables

### Core Variables

| Variable | ENV Var | Default | Description |
|----------|---------|---------|-------------|
| `svc` | `SERVICE_NAME` | `"clnrm"` | Service name for telemetry |
| `env` | `ENV` | `"ci"` | Environment (dev, staging, prod) |
| `endpoint` | `OTEL_ENDPOINT` | `"http://localhost:4318"` | OTLP endpoint URL |
| `exporter` | `OTEL_TRACES_EXPORTER` | `"otlp"` | Exporter type ("stdout" or "otlp") |
| `image` | `CLNRM_IMAGE` | `"registry/clnrm:1.0.0"` | Container image to use |
| `freeze_clock` | `FREEZE_CLOCK` | `"2025-01-01T00:00:00Z"` | Frozen time for determinism |
| `token` | `OTEL_TOKEN` | `""` | Authentication token for OTLP |

## Resolution Examples

### Example 1: Template Variable Override

```toml
[vars]
endpoint = "https://otel.staging.example.com"

[otel]
endpoint = "{{ endpoint }}"  # Uses template var: "https://otel.staging.example.com"
```

### Example 2: Environment Variable Fallback

```toml
# No template variable defined
[otel]
endpoint = "{{ endpoint }}"  # Uses $OTEL_ENDPOINT if set, otherwise "http://localhost:4318"
```

### Example 3: Default Value

```toml
# No template or ENV variable
[meta]
name = "{{ svc }}_test"  # Uses default: "clnrm_test"
```

### Example 4: Mixed Precedence

```toml
[vars]
svc = "my-api"  # Template variable (highest priority)

[otel]
endpoint = "{{ endpoint }}"  # Uses $OTEL_ENDPOINT or "http://localhost:4318"
exporter = "{{ exporter }}"  # Uses $OTEL_TRACES_EXPORTER or "otlp"
```

## Advanced Usage

### Conditional Configuration

```toml
[otel.headers]
{% if token != "" %}
Authorization = "Bearer {{ token }}"
{% endif %}
```

The `token` variable follows the same precedence chain as other variables.

### Template Variable Documentation

```toml
[vars]  # Shows what values will be resolved to (authoring only)
svc = "{{ svc }}"           # Will show actual resolved service name
endpoint = "{{ endpoint }}" # Will show actual resolved endpoint
exporter = "{{ exporter }}" # Will show actual resolved exporter
```

This `[vars]` section is ignored at runtime but helps authors understand resolved values.

## Implementation Details

### Rust Resolution Code

The variable resolution happens in Rust with this logic:

```rust
fn pick(vars: &HashMap<String,String>, key: &str, env_key: &str, default: &str) -> String {
    vars.get(key)
        .cloned()
        .or_else(|| env::var(env_key).ok())
        .unwrap_or_else(|| default.to_string())
}

fn resolve(user_vars: HashMap<String,String>) -> HashMap<String,String> {
    let mut resolved = HashMap::new();
    resolved.insert("svc".into(), pick(&user_vars, "svc", "SERVICE_NAME", "clnrm"));
    resolved.insert("endpoint".into(), pick(&user_vars, "endpoint", "OTEL_ENDPOINT", "http://localhost:4318"));
    // ... other variables
    resolved
}
```

### Template Context Injection

Resolved variables are injected into the Tera context:

```rust
let mut context = Context::new();
for (key, value) in &resolved {
    context.insert(key, value);
}
context.insert("vars", &resolved);  // For [vars] section documentation
```

## Best Practices

### 1. Use Template Variables for Overrides

```toml
[vars]
endpoint = "https://otel.enterprise.com"  # Override for enterprise deployment

[meta]
name = "{{ svc }}_enterprise_test"
```

### 2. Rely on ENV for Dynamic Configuration

```toml
# No template vars - uses ENV for different environments
[meta]
name = "{{ svc }}_{{ env }}_test"  # Uses $SERVICE_NAME and $ENV
```

### 3. Use Defaults for Development

```toml
# No template or ENV vars - uses sensible defaults for local development
[meta]
name = "{{ svc }}_test"  # Uses "clnrm_test"

[otel]
endpoint = "{{ endpoint }}"  # Uses "http://localhost:4318"
```

### 4. Document Resolved Values

```toml
[vars]  # Shows what values will actually be used
svc = "{{ svc }}"
endpoint = "{{ endpoint }}"
env = "{{ env }}"

[meta]
name = "{{ svc }}_{{ env }}_test"  # Uses resolved values from above
```

## Troubleshooting

### Variable Not Resolved

**Problem**: Variable shows as empty or wrong value

**Solution**: Check precedence order:
1. Template variable in `[vars]` section?
2. Environment variable set?
3. Using correct default value?

### Environment Variable Not Read

**Problem**: `$MY_VAR` not being used

**Solution**: Ensure environment variable is exported:
```bash
export MY_VAR="my-value"
# Then run: clnrm run my-test.clnrm.toml
```

### Template Variable Override Not Working

**Problem**: Template variable not taking effect

**Solution**: Check variable name matches exactly (case-sensitive):
```toml
[vars]
endpoint = "custom-endpoint"  # Should be lowercase "endpoint"
```

## Migration from v0.7.0

In v0.7.0, variables used complex namespaces like `vars.service_name`. In v1.0.0:

- **Before**: `{{ vars.service_name }}`
- **After**: `{{ svc }}` (no namespace)

The Rust resolution system handles the complexity, keeping templates simple.

## Performance Benefits

- **Single resolution**: Variables resolved once at startup, not per template render
- **No runtime overhead**: Template rendering is fast since variables are pre-resolved
- **Memory efficient**: No need to pass large context objects around

This approach makes Cleanroom v1.0.0 both simpler to use and more performant than complex template-based variable systems.

---

**Key Innovation**: Rust-based variable resolution with clear precedence (template → ENV → defaults) makes configuration both powerful and predictable.
