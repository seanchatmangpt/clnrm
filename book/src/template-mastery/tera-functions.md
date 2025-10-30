# Tera Functions

Tera functions extend template capabilities beyond basic variable substitution. This chapter covers built-in and custom Tera functions available in clnrm.

## Overview

clnrm provides several built-in Tera functions:
- **Environment functions** - Access environment variables
- **Time functions** - Current time and formatting
- **Hash functions** - SHA-256 hashing for deterministic builds
- **TOML functions** - TOML encoding and parsing
- **Custom functions** - Extend with your own logic

## Built-in Functions

### Environment Function

Access environment variables in templates:

```toml
# Access environment variables
[test.metadata]
name = "{{ env(name=\"SERVICE_NAME\") }}_test"

[services.api]
image = "{{ env(name=\"API_IMAGE\") }}"
port = {{ env(name=\"API_PORT\") | default(value=80) }}

# Environment with fallback
[otel]
endpoint = "{{ env(name=\"OTEL_ENDPOINT\") | default(value=\"http://localhost:4318\") }}"
```

### Time Functions

Work with time and dates:

```toml
# Current time in RFC3339 format
[test.metadata]
timestamp = "{{ now_rfc3339() }}"

# Custom time format
[test.metadata]
build_time = "{{ now() | date(format=\"%Y-%m-%d %H:%M:%S\") }}"

# Freeze clock for deterministic testing
[determinism]
freeze_clock = "{{ now_rfc3339() | date(format=\"%Y-%m-%dT%H:%M:%SZ\") }}"
```

### Hash Functions

Generate deterministic hashes:

```toml
# SHA-256 hash for content
[test.metadata]
content_hash = "{{ sha256(value=\"test content\") }}"

# Hash for file content
[test.metadata]
file_hash = "{{ sha256_file(path=\"config/app.toml\") }}"

# Hash for template variables
[test.metadata]
config_hash = "{{ sha256(value=vars | toml_encode) }}"
```

### TOML Functions

Work with TOML data:

```toml
# Encode data as TOML
[test.metadata]
config = {{ toml_encode(value={
    "database": {"host": "localhost", "port": 5432},
    "api": {"port": 8080}
}) }}

# Parse TOML string
{% set config = toml_parse(value="[database]\nhost = \"localhost\"\nport = 5432") %}

[services.database]
host = "{{ config.database.host }}"
port = {{ config.database.port }}
```

## Custom Functions

### Creating Custom Functions

Extend Tera with custom functions:

```rust
use tera::{Function, Value, Result as TeraResult};
use std::collections::HashMap;

pub struct CustomFunction;

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

// Register the function
tera.register_function("custom", CustomFunction);
```

### Database Connection Function

Create a function for database connection strings:

```rust
pub struct DatabaseConnectionFunction;

impl Function for DatabaseConnectionFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let host = args.get("host")
            .and_then(|v| v.as_str())
            .unwrap_or("localhost");

        let port = args.get("port")
            .and_then(|v| v.as_u64())
            .unwrap_or(5432);

        let database = args.get("database")
            .and_then(|v| v.as_str())
            .unwrap_or("postgres");

        let username = args.get("username")
            .and_then(|v| v.as_str())
            .unwrap_or("postgres");

        let password = args.get("password")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let connection_string = if password.is_empty() {
            format!("postgresql://{}@{}:{}/{}", username, host, port, database)
        } else {
            format!("postgresql://{}:{}@{}:{}/{}", username, password, host, port, database)
        };

        Ok(Value::String(connection_string))
    }
}
```

### Configuration Validation Function

Validate configuration values:

```rust
pub struct ValidateConfigFunction;

impl Function for ValidateConfigFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let value = args.get("value")
            .and_then(|v| v.as_str())
            .unwrap_or("");

        let pattern = args.get("pattern")
            .and_then(|v| v.as_str())
            .unwrap_or(".*");

        // Simple regex validation (use proper regex crate in production)
        let is_valid = value.contains(pattern);

        Ok(Value::Bool(is_valid))
    }
}
```

## Function Composition

### Combining Functions

Combine functions for complex logic:

```toml
# Complex configuration with function composition
[test.metadata]
name = "{{ env(name=\"SERVICE_NAME\") | default(value=\"api\") }}_{{ env(name=\"ENV\") | default(value=\"test\") }}_test"

[services.{{ env(name=\"SERVICE_NAME\") | default(value=\"api\") }}]
image = "{{ env(name=\"IMAGE\") | default(value=\"nginx:alpine\") }}"
port = {{ env(name=\"PORT\") | default(value=80) }}

# Validate configuration
{% if validate_config(value=env(name=\"DATABASE_URL\"), pattern=\"postgresql://\") %}
[services.database]
connection_string = "{{ env(name=\"DATABASE_URL\") }}"
{% endif %}

# Generate deterministic hash
[test.metadata]
config_hash = "{{ sha256(value=toml_encode(value=vars)) }}"
```

### Conditional Function Usage

Use functions conditionally:

```toml
# Environment-specific configuration
{% if env(name=\"ENV\") == \"production\" %}
[otel]
endpoint = "{{ env(name=\"OTEL_ENDPOINT\") }}"
sample_ratio = 0.1
{% else %}
[otel]
endpoint = "{{ env(name=\"OTEL_ENDPOINT\") | default(value=\"http://localhost:4318\") }}"
sample_ratio = 1.0
{% endif %}

# Feature flags
{% if env(name=\"FEATURE_OTEL\") == \"true\" %}
[otel]
enabled = true
exporter = "{{ env(name=\"OTEL_EXPORTER\") | default(value=\"stdout\") }}"
{% endif %}

# Custom function for feature detection
{% if custom_function(input=env(name=\"FEATURES\")) %}
[features]
enabled = true
{% endif %}
```

## Advanced Function Patterns

### Function-Based Configuration

Generate configuration using functions:

```toml
# Database configuration using functions
[services.database]
type = "generic_container"
image = "{{ env(name=\"DB_IMAGE\") | default(value=\"postgres:15-alpine\") }}"
connection_string = "{{ database_connection(
    host=env(name=\"DB_HOST\") | default(value=\"localhost\"),
    port=env(name=\"DB_PORT\") | default(value=5432),
    database=env(name=\"DB_NAME\") | default(value=\"testdb\"),
    username=env(name=\"DB_USER\") | default(value=\"testuser\"),
    password=env(name=\"DB_PASSWORD\") | default(value=\"testpass\")
) }}"

# Service ports using functions
[services.api]
ports = [{{ env(name=\"API_PORT\") | default(value=80) }}]

[services.cache]
ports = [{{ env(name=\"CACHE_PORT\") | default(value=6379) }}]

# Environment-specific settings
[services.{{ env(name=\"SERVICE_NAME\") | default(value=\"api\") }}]
{% if env(name=\"ENV\") == \"production\" %}
env_vars = { "LOG_LEVEL" = "info", "DEBUG" = "false" }
{% else %}
env_vars = { "LOG_LEVEL" = "debug", "DEBUG" = "true" }
{% endif %}
```

### Dynamic Service Generation

Generate services dynamically:

```toml
# Dynamic service generation
{% for service in services %}
[services.{{ service.name }}]
type = "{{ service.type }}"
image = "{{ service.image }}"
ports = {{ service.ports | toml_encode }}

{% if service.env_vars %}
env_vars = {{ service.env_vars | toml_encode }}
{% endif %}

{% if service.volumes %}
volumes = {{ service.volumes | toml_encode }}
{% endif %}

{% endfor %}

# Dynamic test steps
{% for test in tests %}
[[steps]]
name = "{{ test.name }}"
description = "{{ test.description }}"
command = {{ test.command | toml_encode }}

{% if test.expected_output %}
expected_output_regex = "{{ test.expected_output }}"
{% endif %}

{% if test.service %}
service = "{{ test.service }}"
{% endif %}

{% endfor %}
```

### Template Inheritance with Functions

Use functions in template inheritance:

```toml
# Base template (base.clnrm.toml.tera)
{% import "_macros.toml.tera" as m %}

[test.metadata]
name = "{{ svc }}_base_test"
version = "{{ version | default(value=\"1.0.0\") }}"

[services.{{ svc }}]
type = "generic_container"
image = "{{ image }}"
ports = [{{ port | default(value=80) }}]

# Common spans
{{ m::span("clnrm.run", kind="internal") }}
{{ m::span("{{ svc }}.start", kind="internal") }}

# Base configuration
[otel]
endpoint = "{{ env(name=\"OTEL_ENDPOINT\") | default(value=\"http://localhost:4318\") }}"
sample_ratio = {{ env(name=\"OTEL_SAMPLE_RATIO\") | default(value=1.0) }}

# Determinism
[determinism]
seed = {{ seed | default(value=42) }}
freeze_clock = "{{ now_rfc3339() }}"
```

```toml
# Extended template (api.clnrm.toml.tera)
{% extends "base.clnrm.toml.tera" %}

[test.metadata]
name = "api_extended_test"

[services.api]
# Inherit from base
# Add API-specific configuration
env_vars = { "API_KEY" = "{{ env(name=\"API_KEY\") }}" }

# Additional spans
{{ m::span("api.request", kind="server") }}
{{ m::span("api.response", kind="server") }}

# API-specific OTEL
[otel.resources]
"service.name" = "api"
"service.version" = "{{ version }}"

# API-specific steps
[[steps]]
name = "api_health_check"
command = ["curl", "-f", "http://localhost:{{ port }}/health"]
expected_output_regex = ".*"
```

## Function Best Practices

### 1. Use Descriptive Function Names

```toml
# ✅ Good: Descriptive function usage
[services.database]
connection_string = "{{ database_connection(
    host=env(name=\"DB_HOST\"),
    port=env(name=\"DB_PORT\"),
    database=env(name=\"DB_NAME\")
) }}"
```

### 2. Provide Sensible Defaults

```toml
# ✅ Good: Sensible defaults
[services.api]
port = {{ env(name=\"API_PORT\") | default(value=80) }}
image = "{{ env(name=\"API_IMAGE\") | default(value=\"nginx:alpine\") }}"
```

### 3. Validate Function Inputs

```rust
// ✅ Good: Input validation in custom functions
impl Function for CustomFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let input = args.get("input")
            .and_then(|v| v.as_str())
            .ok_or_else(|| tera::Error::msg("input parameter is required"))?;

        if input.is_empty() {
            return Err(tera::Error::msg("input cannot be empty"));
        }

        // ... function logic
    }
}
```

### 4. Handle Errors Gracefully

```toml
# ✅ Good: Error handling
{% if validate_config(value=env(name=\"DATABASE_URL\"), pattern=\"postgresql://\") %}
[services.database]
connection_string = "{{ env(name=\"DATABASE_URL\") }}"
{% else %}
# Invalid database URL - use default
[services.database]
connection_string = "{{ database_connection(host=\"localhost\", port=5432) }}"
{% endif %}
```

## Common Function Patterns

### Multi-Environment Configuration

```toml
# Multi-environment configuration using functions
[test.metadata]
name = "{{ svc }}_{{ env(name=\"ENVIRONMENT\") | default(value=\"test\") }}_test"

[services.{{ svc }}]
image = "{{ env(name=\"IMAGE\") | default(value=\"nginx:alpine\") }}"

# Environment-specific settings
{% if env(name=\"ENVIRONMENT\") == \"production\" %}
env_vars = { "LOG_LEVEL" = "info", "DEBUG" = "false" }
{% elif env(name=\"ENVIRONMENT\") == \"staging\" %}
env_vars = { "LOG_LEVEL" = "debug", "DEBUG" = "true" }
{% else %}
env_vars = { "LOG_LEVEL" = "trace", "DEBUG" = "true" }
{% endif %}

# OTEL configuration by environment
[otel]
endpoint = "{{ env(name=\"OTEL_ENDPOINT\") | default(value=\"http://localhost:4318\") }}"
{% if env(name=\"ENVIRONMENT\") == \"production\" %}
sample_ratio = 0.1
{% else %}
sample_ratio = 1.0
{% endif %}
```

### Dynamic Service Discovery

```toml
# Dynamic service discovery using functions
{% set services = [
    {name: \"api\", type: \"generic_container\", image: \"nginx:alpine\", port: 80},
    {name: \"database\", type: \"generic_container\", image: \"postgres:15-alpine\", port: 5432},
    {name: \"cache\", type: \"generic_container\", image: \"redis:7-alpine\", port: 6379}
] %}

{% for service in services %}
[services.{{ service.name }}]
type = "{{ service.type }}"
image = "{{ service.image }}"
ports = [{{ service.port }}]

# Register service in discovery
[service_discovery.{{ service.name }}]
port = {{ service.port }}
health_endpoint = "{{ service.health_endpoint | default(value=\"/health\") }}"

{% endfor %}
```

### Configuration Validation

```toml
# Configuration validation using functions
{% set config_valid = true %}

{% if not validate_config(value=env(name=\"DATABASE_URL\"), pattern=\"postgresql://\") %}
{% set config_valid = false %}
[test.errors]
database_url_invalid = "DATABASE_URL must be a valid PostgreSQL URL"
{% endif %}

{% if not validate_config(value=env(name=\"API_PORT\"), pattern=\"^[0-9]+$\") %}
{% set config_valid = false %}
[test.errors]
api_port_invalid = "API_PORT must be a valid port number"
{% endif %}

{% if config_valid %}
# Valid configuration - proceed with test
[services.api]
image = "{{ env(name=\"API_IMAGE\") | default(value=\"nginx:alpine\") }}"
port = {{ env(name=\"API_PORT\") | default(value=80) }}

[services.database]
connection_string = "{{ env(name=\"DATABASE_URL\") }}"
{% endif %}
```

## Next Steps

Now that you understand Tera functions:

1. **Try the examples**: Run the function examples in this chapter
2. **Create custom functions**: Build functions for your specific needs
3. **Learn macro library**: Move on to [Macro Library](macro-library.md)
4. **Master variable resolution**: Learn about [Variable Resolution](variable-resolution.md)

## Further Reading

- [Tera Documentation](https://tera.netlify.app/)
- [Template System Mastery Overview](README.md)
- [Custom Function Implementation](../plugin-development/README.md)

