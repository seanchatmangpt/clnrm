# Variable Resolution

Variable resolution determines how template variables, environment variables, and default values are combined. This chapter covers the precedence system and resolution strategies.

## Overview

clnrm uses a three-tier precedence system:
1. **Template variables** (highest priority)
2. **Environment variables**
3. **Default values** (lowest priority)

## Precedence System

### Template Variables (Highest Priority)

Variables defined in the template take precedence:

```toml
# Template variables override everything
[vars]
svc = "my_custom_service"
image = "nginx:custom"
port = 8080

[test.metadata]
name = "{{ svc }}_test"  # Uses "my_custom_service"

[services.{{ svc }}]
image = "{{ image }}"    # Uses "nginx:custom"
port = {{ port }}        # Uses 8080
```

### Environment Variables (Medium Priority)

Environment variables override defaults but not template variables:

```bash
# Set environment variables
export SERVICE_NAME="api"
export IMAGE_NAME="nginx:alpine"
export PORT_NUMBER="80"
```

```toml
# Uses environment variables when template vars not set
[test.metadata]
name = "{{ svc }}_test"  # Uses "api" from ENV

[services.{{ svc }}]
image = "{{ image }}"    # Uses "nginx:alpine" from ENV
port = {{ port }}        # Uses 80 from ENV
```

### Default Values (Lowest Priority)

Defaults are used when neither template nor environment variables are set:

```toml
# No template vars or ENV vars - uses defaults
[test.metadata]
name = "{{ svc }}_test"  # Uses "clnrm" default

[services.{{ svc }}]
image = "{{ image }}"    # Uses "alpine:latest" default
port = {{ port }}        # Uses 80 default
```

## Resolution Implementation

### Rust Resolution Logic

The resolution happens in Rust before template rendering:

```rust
use std::{collections::HashMap, env};

fn pick(vars: &HashMap<String, String>, key: &str, env_key: &str, default: &str) -> String {
    vars.get(key)
        .cloned()
        .or_else(|| env::var(env_key).ok())
        .unwrap_or_else(|| default.to_string())
}

fn resolve(vars: HashMap<String, String>) -> HashMap<String, String> {
    let mut out = HashMap::new();

    // Service configuration
    out.insert("svc".into(), pick(&vars, "svc", "SERVICE_NAME", "clnrm"));
    out.insert("image".into(), pick(&vars, "image", "IMAGE_NAME", "alpine:latest"));
    out.insert("port".into(), pick(&vars, "port", "PORT_NUMBER", "80"));

    // OTEL configuration
    out.insert("endpoint".into(), pick(&vars, "endpoint", "OTEL_ENDPOINT", "http://localhost:4318"));
    out.insert("exporter".into(), pick(&vars, "exporter", "OTEL_TRACES_EXPORTER", "otlp"));
    out.insert("token".into(), pick(&vars, "token", "OTEL_TOKEN", ""));

    // Environment and version
    out.insert("env".into(), pick(&vars, "env", "ENVIRONMENT", "test"));
    out.insert("version".into(), pick(&vars, "version", "VERSION", "1.0.0"));

    // Determinism
    out.insert("freeze_clock".into(), pick(&vars, "freeze_clock", "FREEZE_CLOCK", "2025-01-01T00:00:00Z"));
    out.insert("seed".into(), pick(&vars, "seed", "DETERMINISTIC_SEED", "42"));

    out
}
```

### Template Context Setup

Variables are injected into the Tera context:

```rust
fn render_template(template_glob: &str, template_name: &str, user_vars: HashMap<String, Value>) -> String {
    let resolved = resolve(user_vars.iter().map(|(k, v)| (k.clone(), v.clone())).collect());

    // Create Tera context
    let mut ctx = Context::new();

    // Inject resolved variables (no prefixes needed)
    for (k, v) in &resolved {
        ctx.insert(k, v);
    }

    // Inject vars table for authoring
    ctx.insert("vars", &resolved);

    // Inject matrix for advanced patterns
    ctx.insert("matrix", &HashMap::<String, Value>::new());

    // Render template
    tera.render(template_name, &ctx).unwrap()
}
```

## Variable Types

### String Variables

Basic string substitution:

```toml
[vars]
service_name = "my_api"
environment = "production"

[test.metadata]
name = "{{ service_name }}_{{ environment }}_test"

[services.{{ service_name }}]
image = "{{ service_name }}_image:latest"
env_vars = { "ENV" = "{{ environment }}" }
```

### Numeric Variables

Numeric values for ports, timeouts, etc.:

```toml
[vars]
port = 8080
timeout_seconds = 30
retry_count = 3

[services.api]
port = {{ port }}

[services.api.env_vars]
TIMEOUT_SECONDS = "{{ timeout_seconds }}"
RETRY_COUNT = "{{ retry_count }}"
```

### Boolean Variables

Boolean flags for feature toggles:

```toml
[vars]
otel_enabled = true
debug_mode = false

{% if otel_enabled %}
[otel]
enabled = true
{% endif %}

{% if debug_mode %}
[debug]
verbose = true
{% endif %}
```

### Complex Variables

Nested objects and arrays:

```toml
[vars]
services = [
    { name = "api", port = 80 },
    { name = "database", port = 5432 }
]

config = {
    database = { host = "localhost", port = 5432 },
    api = { port = 80, timeout = 30 }
}

# Use in templates
{% for service in services %}
[services.{{ service.name }}]
port = {{ service.port }}
{% endfor %}

[services.database]
connection_string = "{{ config.database.host }}:{{ config.database.port }}"
```

## Environment Variable Integration

### Standard Environment Variables

Common environment variables supported:

```bash
# Service configuration
export SERVICE_NAME="api"
export IMAGE_NAME="nginx:alpine"
export PORT_NUMBER="80"

# OTEL configuration
export OTEL_ENDPOINT="http://localhost:4318"
export OTEL_TRACES_EXPORTER="otlp"
export OTEL_TOKEN="your-token"

# Environment
export ENVIRONMENT="production"
export VERSION="1.0.0"

# Determinism
export FREEZE_CLOCK="2025-01-01T00:00:00Z"
export DETERMINISTIC_SEED="42"
```

### Custom Environment Variables

Use custom environment variables:

```toml
# Custom environment variables
[vars]
custom_api_key = "{{ env(name=\"CUSTOM_API_KEY\") }}"
database_password = "{{ env(name=\"DB_PASSWORD\") }}"

[services.api]
env_vars = { "API_KEY" = "{{ custom_api_key }}" }

[services.database]
env_vars = { "DB_PASSWORD" = "{{ database_password }}" }
```

## Variable Validation

### Template Variable Validation

Validate template variables:

```toml
# Variable validation
[vars]
port = 80
timeout = 30

# Validate port is in valid range
{% if port < 1024 or port > 65535 %}
[test.errors]
invalid_port = "Port must be between 1024 and 65535"
{% endif %}

# Validate timeout is positive
{% if timeout <= 0 %}
[test.errors]
invalid_timeout = "Timeout must be positive"
{% endif %}
```

### Environment Variable Validation

Validate environment variables:

```toml
# Environment variable validation
{% if not env(name=\"DATABASE_URL\") %}
[test.errors]
missing_database_url = "DATABASE_URL environment variable is required"
{% endif %}

{% if env(name=\"DATABASE_URL\") and not env(name=\"DATABASE_URL\").starts_with(\"postgresql://\") %}
[test.errors]
invalid_database_url = "DATABASE_URL must be a valid PostgreSQL URL"
{% endif %}
```

## Advanced Resolution Patterns

### Conditional Resolution

Use different variables based on conditions:

```toml
# Conditional variable resolution
[vars]
environment = "production"

# Different images based on environment
{% if environment == "production" %}
image = "nginx:1.21-alpine"
{% elif environment == "staging" %}
image = "nginx:1.21-alpine"
{% else %}
image = "nginx:alpine"
{% endif %}

# Different configurations based on environment
{% if environment == "production" %}
port = 80
log_level = "info"
{% else %}
port = 8080
log_level = "debug"
{% endif %}

[services.api]
image = "{{ image }}"
port = {{ port }}
env_vars = { "LOG_LEVEL" = "{{ log_level }}" }
```

### Matrix Variable Resolution

Resolve variables for matrix testing:

```toml
# Matrix variable definition
[vars.matrix]
environments = ["test", "staging", "production"]
services = ["api", "database", "cache"]
versions = ["1.0", "1.1", "2.0"]

# Generate tests for each combination
{% for env in matrix.environments %}
{% for service in matrix.services %}
{% for version in matrix.versions %}
[test.{{ env }}_{{ service }}_{{ version }}.metadata]
name = "{{ service }}_{{ env }}_{{ version }}_test"

[services.{{ service }}]
image = "{{ service }}:{{ version }}"
env_vars = { "ENVIRONMENT" = "{{ env }}" }

# Service-specific configuration
{% if service == "api" %}
port = {{ env == "production" ? 80 : 8080 }}
{% elif service == "database" %}
port = 5432
{% elif service == "cache" %}
port = 6379
{% endif %}

{% endfor %}
{% endfor %}
{% endfor %}
```

### Dynamic Variable Generation

Generate variables dynamically:

```toml
# Dynamic variable generation
{% set timestamp = now_rfc3339() %}
{% set random_seed = sha256(value=timestamp) %}

[vars]
generated_timestamp = "{{ timestamp }}"
generated_seed = "{{ random_seed }}"

[test.metadata]
name = "dynamic_{{ generated_timestamp | date(format=\"%Y%m%d_%H%M%S\") }}_test"

[determinism]
seed = {{ generated_seed | slice(start=0, end=8) | int(base=16) }}
freeze_clock = "{{ timestamp }}"
```

## Variable Scoping

### Global Variables

Variables available throughout the template:

```toml
[vars]
service_name = "api"
environment = "test"

# Available everywhere
[test.metadata]
name = "{{ service_name }}_{{ environment }}_test"

[services.{{ service_name }}]
env_vars = { "SERVICE_NAME" = "{{ service_name }}" }

[otel.resources]
"service.name" = "{{ service_name }}"
"env" = "{{ environment }}"
```

### Local Variables

Variables scoped to specific sections:

```toml
# Global variables
[vars]
service_name = "api"

# Section-specific variables
[vars.services.api]
port = 80
timeout = 30

[vars.services.database]
port = 5432
connection_timeout = 10

# Use variables in respective sections
[services.api]
port = {{ vars.services.api.port }}

[services.database]
port = {{ vars.services.database.port }}
```

## Best Practices

### 1. Use Descriptive Variable Names

```toml
# ✅ Good: Descriptive variable names
[vars]
api_service_name = "user_api"
database_host = "localhost"
cache_port = 6379

[services.{{ api_service_name }}]
# Clear and readable
```

### 2. Provide Sensible Defaults

```toml
# ✅ Good: Sensible defaults
[vars]
port = 80
timeout_seconds = 30
log_level = "info"

# Override only what needs to be different
```

### 3. Validate Variable Values

```toml
# ✅ Good: Variable validation
{% if port < 1024 or port > 65535 %}
[test.errors]
invalid_port = "Port must be between 1024 and 65535"
{% endif %}
```

### 4. Use Environment Variables for Secrets

```toml
# ✅ Good: Use ENV for secrets
[vars]
api_key = "{{ env(name=\"API_KEY\") }}"
database_password = "{{ env(name=\"DB_PASSWORD\") }}"

[services.api]
env_vars = { "API_KEY" = "{{ api_key }}" }
```

## Common Patterns

### Multi-Environment Configuration

```toml
# Multi-environment configuration
[vars]
environment = "production"

# Environment-specific variables
{% if environment == "production" %}
[vars.services]
api_port = 80
database_host = "prod-db.example.com"
log_level = "info"
{% elif environment == "staging" %}
[vars.services]
api_port = 8080
database_host = "staging-db.example.com"
log_level = "debug"
{% else %}
[vars.services]
api_port = 3000
database_host = "localhost"
log_level = "trace"
{% endif %}

[services.api]
port = {{ vars.services.api_port }}
env_vars = { "LOG_LEVEL" = "{{ vars.services.log_level }}" }

[services.database]
host = "{{ vars.services.database_host }}"
```

### Feature Flags

```toml
# Feature flags using variables
[vars]
features = {
    otel_enabled = true,
    debug_mode = false,
    performance_monitoring = true
}

# Conditional configuration based on features
{% if features.otel_enabled %}
[otel]
enabled = true
endpoint = "{{ env(name=\"OTEL_ENDPOINT\") | default(value=\"http://localhost:4318\") }}"
{% endif %}

{% if features.debug_mode %}
[debug]
verbose = true
log_level = "debug"
{% endif %}

{% if features.performance_monitoring %}
[performance]
enabled = true
metrics = ["latency", "throughput", "error_rate"]
{% endif %}
```

### Dynamic Service Configuration

```toml
# Dynamic service configuration
[vars]
services = [
    { name = "api", type = "nginx", port = 80 },
    { name = "database", type = "postgres", port = 5432 },
    { name = "cache", type = "redis", port = 6379 }
]

# Generate services dynamically
{% for service in services %}
[services.{{ service.name }}]
type = "generic_container"
image = "{{ service.type }}:alpine"
ports = [{{ service.port }}]

# Service-specific configuration
{% if service.name == "database" %}
env_vars = { "POSTGRES_DB" = "testdb" }
{% elif service.name == "cache" %}
env_vars = { "REDIS_DB" = "0" }
{% endif %}

{% endfor %}
```

## Next Steps

Now that you understand variable resolution:

1. **Try the examples**: Run the variable resolution examples in this chapter
2. **Create your own patterns**: Build variable patterns for your use cases
3. **Master production deployment**: Move on to [Production Deployment](../production-deployment/README.md)
4. **Learn advanced patterns**: Review [Advanced Testing Patterns](../advanced-patterns/README.md)

## Further Reading

- [Variable Resolution Implementation](../crates/clnrm-core/src/template/resolver.rs)
- [Environment Variable Integration](../crates/clnrm-core/src/config/)
- [Template System Overview](README.md)
