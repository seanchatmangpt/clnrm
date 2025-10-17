# Tera Template Guide - clnrm v0.6.0

## Introduction

clnrm v0.6.0 introduces **Tera templating** for `.clnrm.toml` configuration files. This allows you to:

- Define reusable variables with `{{ vars.name }}`
- Generate multiple scenarios via loops `{% for %}`
- Conditionally include sections with `{% if %}`
- Share common configurations with `{% include %}`
- Create reusable blocks with `{% macro %}`

Tera uses Jinja2-like syntax, making it familiar to Python/Ansible users.

---

## Table of Contents

1. [Quick Start](#quick-start)
2. [Template Basics](#template-basics)
3. [Variable Substitution](#variable-substitution)
4. [Loops and Matrix Expansion](#loops-and-matrix-expansion)
5. [Conditionals](#conditionals)
6. [Includes and Reusability](#includes-and-reusability)
7. [Macros](#macros)
8. [Custom Functions](#custom-functions)
9. [Determinism](#determinism)
10. [Best Practices](#best-practices)
11. [Common Patterns](#common-patterns)
12. [Troubleshooting](#troubleshooting)

---

## Quick Start

### Non-Template File (v0.5.0 style)

```toml
[meta]
name = "simple_test"

[otel]
exporter = "stdout"

[[scenario]]
name = "test_1"
run = "clnrm run"
```

This works unchanged in v0.6.0.

### Template File (v0.6.0)

```toml
[template.vars]
exporter = "stdout"

[meta]
name = "simple_test"

[otel]
exporter = "{{ vars.exporter }}"

[[scenario]]
name = "test_1"
run = "clnrm run --otel-exporter {{ vars.exporter }}"
```

The template renders to the same output, but now you can change `exporter` in one place.

---

## Template Basics

### Detection

clnrm automatically detects templates by looking for:
- `{{` (variable substitution)
- `{%` (control structures)

If neither is found, the file is parsed as plain TOML (backward compatible).

### Rendering Pipeline

```
1. Load .clnrm.toml (as template)
2. Extract [template.vars], [template.matrix], [template.otel]
3. Render with Tera (expand {{ }} and {% %})
4. Parse rendered TOML
5. Execute tests
```

### Template Sections

Templates can define special sections consumed by Tera (NOT part of final TOML):

```toml
[template.vars]
# User-defined variables
service = "clnrm"
version = "0.6.0"

[template.matrix]
# Data for loops
exporters = ["stdout", "otlp", "jaeger"]

[template.otel]
# OTEL shortcuts
endpoint = "http://localhost:4318"
```

These sections are removed after rendering.

---

## Variable Substitution

### Basic Usage

```toml
[template.vars]
test_name = "my_test"
version = "0.6.0"

[meta]
name = "{{ vars.test_name }}"
description = "Version {{ vars.version }}"
```

**Renders to**:
```toml
[meta]
name = "my_test"
description = "Version 0.6.0"
```

### Nested Access

```toml
[template.otel]
endpoint = "http://localhost:4318"
service_name = "clnrm"

[otel]
exporter = "otlp"
endpoint = "{{ otel.endpoint }}"

[otel.resources]
"service.name" = "{{ otel.service_name }}"
```

### Default Values

```toml
[meta]
# Use environment variable, fallback to default
exporter = "{{ env(name='OTEL_EXPORTER', default='stdout') }}"

# Use variable with fallback
name = "{{ vars.name | default(value='unnamed_test') }}"
```

### Filters

Tera provides built-in filters:

```toml
[template.vars]
name = "my_test"

[meta]
# Uppercase filter
name = "{{ vars.name | upper }}"  # "MY_TEST"

# Length filter
count = "{{ matrix.items | length }}"  # Number of items

# Join filter
tags = "{{ vars.tags | join(sep=',') }}"  # "tag1,tag2,tag3"
```

See [Tera filters documentation](https://keats.github.io/tera/docs/#filters) for full list.

---

## Loops and Matrix Expansion

### Basic Loop

```toml
[template.matrix]
exporters = ["stdout", "otlp", "jaeger"]

{% for exporter in matrix.exporters %}
[[scenario]]
name = "test_{{ exporter }}"
run = "clnrm run --otel-exporter {{ exporter }}"
{% endfor %}
```

**Renders to**:
```toml
[[scenario]]
name = "test_stdout"
run = "clnrm run --otel-exporter stdout"

[[scenario]]
name = "test_otlp"
run = "clnrm run --otel-exporter otlp"

[[scenario]]
name = "test_jaeger"
run = "clnrm run --otel-exporter jaeger"
```

### Loop with Index

```toml
{% for i in range(end=3) %}
[[expect.span]]
name = "span_{{ i }}"
{% endfor %}
```

**Renders to**:
```toml
[[expect.span]]
name = "span_0"

[[expect.span]]
name = "span_1"

[[expect.span]]
name = "span_2"
```

### Complex Matrix

```toml
[template.matrix]
configs = [
    { env = "dev", exporter = "stdout", endpoint = "" },
    { env = "staging", exporter = "otlp", endpoint = "http://staging:4318" },
    { env = "prod", exporter = "otlp", endpoint = "http://prod:4318" },
]

{% for config in matrix.configs %}
[[scenario]]
name = "test_{{ config.env }}"
run = "clnrm run"

[otel]
exporter = "{{ config.exporter }}"
endpoint = "{{ config.endpoint }}"
{% endfor %}
```

### Nested Loops

```toml
[template.matrix]
environments = ["dev", "prod"]
exporters = ["stdout", "otlp"]

{% for env in matrix.environments %}
{% for exporter in matrix.exporters %}
[[scenario]]
name = "test_{{ env }}_{{ exporter }}"
{% endfor %}
{% endfor %}
```

**Generates 4 scenarios**: `dev_stdout`, `dev_otlp`, `prod_stdout`, `prod_otlp`

---

## Conditionals

### Basic If

```toml
[template.vars]
enable_tracing = true

{% if vars.enable_tracing %}
[otel]
exporter = "otlp"
endpoint = "http://localhost:4318"
{% endif %}
```

### If-Else

```toml
[template.vars]
env = "ci"

{% if vars.env == "ci" %}
[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"
{% else %}
# Use real time in development
{% endif %}
```

### If-Elif-Else

```toml
[template.vars]
log_level = "info"

[otel.resources]
{% if vars.log_level == "debug" %}
"log.level" = "DEBUG"
{% elif vars.log_level == "info" %}
"log.level" = "INFO"
{% else %}
"log.level" = "WARN"
{% endif %}
```

### Complex Conditions

```toml
{% if vars.version >= 2 and vars.experimental %}
[features]
new_api = true
{% endif %}

{% if vars.env == "prod" or vars.env == "staging" %}
[limits]
cpu_millicores = 4000
memory_mb = 8192
{% endif %}
```

### Conditional in Loop

```toml
[template.matrix]
services = [
    { name = "redis", enabled = true },
    { name = "postgres", enabled = false },
]

{% for service in matrix.services %}
{% if service.enabled %}
[services.{{ service.name }}]
type = "generic_container"
image = "{{ service.name }}:latest"
{% endif %}
{% endfor %}
```

---

## Includes and Reusability

### Basic Include

**File: `templates/partials/postgres.toml`**
```toml
[services.postgres]
type = "generic_container"
image = "postgres:15"
env = { POSTGRES_PASSWORD = "secret" }
```

**File: `my-test.clnrm.toml`**
```toml
[meta]
name = "test_with_postgres"

{% include "templates/partials/postgres.toml" %}

[[scenario]]
name = "test_db"
run = "psql -h postgres -U postgres -c 'SELECT 1'"
```

### Conditional Include

```toml
[template.vars]
use_redis = true
use_postgres = false

{% if vars.use_redis %}
{% include "templates/partials/redis.toml" %}
{% endif %}

{% if vars.use_postgres %}
{% include "templates/partials/postgres.toml" %}
{% endif %}
```

### Include with Variables

**File: `templates/partials/service.toml`**
```toml
[services.{{ vars.service_name }}]
type = "generic_container"
image = "{{ vars.service_image }}"
```

**File: `my-test.clnrm.toml`**
```toml
[template.vars]
service_name = "redis"
service_image = "redis:7"

{% include "templates/partials/service.toml" %}
```

### Include Path Resolution

Includes are resolved relative to the template file's directory:

```
project/
├── tests/
│   └── my-test.clnrm.toml           # {% include "../templates/common.toml" %}
└── templates/
    ├── common.toml
    └── partials/
        └── service.toml             # {% include "service.toml" %}
```

---

## Macros

Macros are reusable template blocks with parameters.

### Define and Use Macro

```toml
{% macro container_lifecycle() %}
["container.start", "container.exec", "container.stop"]
{% endmacro %}

[[expect.span]]
name = "clnrm.step:*"
events.any = {{ container_lifecycle() }}
```

### Macro with Parameters

```toml
{% macro span_matcher(name, max_duration) %}
{
  "name": "{{ name }}",
  "duration_ms": { "lte": {{ max_duration }} }
}
{% endmacro %}

[[expect.span]]
{{ span_matcher(name="fast_query", max_duration=100) }}

[[expect.span]]
{{ span_matcher(name="slow_query", max_duration=5000) }}
```

### Complex Macro

```toml
{% macro otel_config(exporter, endpoint, service_name) %}
[otel]
exporter = "{{ exporter }}"
{% if endpoint %}
endpoint = "{{ endpoint }}"
{% endif %}
resources = { "service.name" = "{{ service_name }}" }
{% endmacro %}

{{ otel_config(exporter="otlp", endpoint="http://localhost:4318", service_name="clnrm") }}
```

---

## Custom Functions

clnrm provides custom Tera functions for common operations.

### `env(name, default="")`

Read environment variable.

```toml
[otel]
endpoint = "{{ env(name='OTEL_ENDPOINT', default='http://localhost:4318') }}"

[meta]
run_id = "{{ env(name='CI_JOB_ID') }}"
```

### `now_rfc3339()`

Get current timestamp (respects determinism).

```toml
[meta]
created_at = "{{ now_rfc3339() }}"

[[scenario]]
name = "test_{{ now_rfc3339() }}"  # Unique name per run
```

### `sha256(input)`

Compute SHA-256 hash (hex encoded).

```toml
[meta]
config_hash = "{{ sha256(input='my_unique_seed') }}"

[[expect.span]]
name = "test_{{ sha256(input=vars.test_id) }}"
```

### `toml_encode(value)`

Encode value as TOML literal.

```toml
{% set my_data = {"key": "value", "count": 42} %}

[some_section]
data = {{ toml_encode(value=my_data) }}
```

**Renders to**:
```toml
[some_section]
data = { key = "value", count = 42 }
```

---

## Determinism

Make tests reproducible by freezing time and seeding randomness.

### Freeze Clock

```toml
[determinism]
freeze_clock = "2025-01-01T00:00:00Z"

[meta]
created_at = "{{ now_rfc3339() }}"  # Always "2025-01-01T00:00:00Z"
```

### Seed Random Number Generator

```toml
[determinism]
seed = 42
```

### Combined

```toml
[template.vars]
env = "ci"

{% if vars.env == "ci" %}
[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"
{% endif %}
```

---

## Best Practices

### 1. Use Variables for Reusability

**Bad** (repetition):
```toml
[[scenario]]
name = "test_1"
run = "clnrm run --otel-exporter otlp --otel-endpoint http://localhost:4318"

[[scenario]]
name = "test_2"
run = "clnrm run --otel-exporter otlp --otel-endpoint http://localhost:4318"
```

**Good** (DRY):
```toml
[template.vars]
exporter = "otlp"
endpoint = "http://localhost:4318"

[[scenario]]
name = "test_1"
run = "clnrm run --otel-exporter {{ vars.exporter }} --otel-endpoint {{ vars.endpoint }}"

[[scenario]]
name = "test_2"
run = "clnrm run --otel-exporter {{ vars.exporter }} --otel-endpoint {{ vars.endpoint }}"
```

### 2. Avoid Deep Nesting

**Bad**:
```toml
{% for env in matrix.envs %}
{% for region in matrix.regions %}
{% for exporter in matrix.exporters %}
{% if exporter == "otlp" %}
...
{% endif %}
{% endfor %}
{% endfor %}
{% endfor %}
```

**Good**:
```toml
{% for config in matrix.configs %}  # Flatten matrix
[[scenario]]
name = "{{ config.name }}"
{% endfor %}
```

### 3. Use Includes for Shared Config

Create a `templates/` directory for reusable blocks:

```
project/
├── tests/
│   ├── test-a.clnrm.toml
│   └── test-b.clnrm.toml
└── templates/
    ├── otel-prod.toml
    ├── otel-dev.toml
    └── partials/
        ├── postgres.toml
        └── redis.toml
```

### 4. Comment Template Logic

```toml
{% # This loop generates scenarios for each exporter %}
{% for exporter in matrix.exporters %}
[[scenario]]
name = "test_{{ exporter }}"
{% endfor %}
```

### 5. Validate Rendered Output

Run `clnrm template render <file>` to see rendered TOML before execution.

---

## Common Patterns

### Pattern 1: Multi-Environment Testing

```toml
[template.matrix]
environments = [
    { name = "dev", endpoint = "http://dev:4318" },
    { name = "staging", endpoint = "http://staging:4318" },
    { name = "prod", endpoint = "http://prod:4318" },
]

{% for env in matrix.environments %}
[[scenario]]
name = "test_{{ env.name }}"
run = "clnrm run"

[otel]
exporter = "otlp"
endpoint = "{{ env.endpoint }}"
resources = { "env" = "{{ env.name }}" }
{% endfor %}
```

### Pattern 2: Feature Flags

```toml
[template.vars]
enable_tracing = true
enable_metrics = false
enable_logs = false

{% if vars.enable_tracing %}
[otel]
exporter = "otlp"
{% endif %}

{% if vars.enable_metrics %}
[otel.metrics]
enabled = true
{% endif %}
```

### Pattern 3: Reusable Validators

```toml
{% macro fast_span_matcher() %}
{
  "duration_ms": { "lte": 100 }
}
{% endmacro %}

[[expect.span]]
name = "query:*"
{{ fast_span_matcher() }}

[[expect.span]]
name = "api:*"
{{ fast_span_matcher() }}
```

### Pattern 4: CI/CD Integration

```toml
[template.vars]
ci = "{{ env(name='CI', default='false') }}"

{% if vars.ci == "true" %}
[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"

[report]
json = "results/report.json"
junit = "results/junit.xml"
{% endif %}
```

### Pattern 5: Service Mesh Testing

```toml
[template.matrix]
services = [
    { name = "frontend", image = "frontend:latest", port = 3000 },
    { name = "backend", image = "backend:latest", port = 8080 },
    { name = "database", image = "postgres:15", port = 5432 },
]

{% for service in matrix.services %}
[services.{{ service.name }}]
type = "generic_container"
image = "{{ service.image }}"
ports = [{{ service.port }}]
{% endfor %}

[[scenario]]
name = "test_service_mesh"
run = """
curl http://frontend:3000/api/health
curl http://backend:8080/health
"""
```

---

## Troubleshooting

### Error: Variable not found

```
Error: Variable 'vars.undefined_var' does not exist in context.
```

**Fix**: Define the variable in `[template.vars]`:
```toml
[template.vars]
undefined_var = "value"
```

### Error: Syntax error

```
Error: Template rendering failed

  Syntax error: unexpected token '}}'

  15 | name = "{{ vars.name }}"
     |                       ^^ expected expression
```

**Fix**: Check for typos in template syntax. Common issues:
- Mismatched braces: `{{ vars.name }` (missing `}`)
- Wrong delimiter: `{ vars.name }` (missing `{`)
- Unclosed block: `{% for x in y %}` (missing `{% endfor %}`)

### Error: Template too complex

```
Error: Template exceeds maximum size (1 MB)
```

**Fix**: Break large templates into smaller includes:
```toml
{% include "part1.toml" %}
{% include "part2.toml" %}
```

### Error: Circular include

```
Error: Circular include detected: a.toml -> b.toml -> a.toml
```

**Fix**: Restructure includes to avoid circular dependencies.

### Debugging Tips

1. **Render template first**:
   ```bash
   clnrm template render my-test.clnrm.toml
   ```

2. **Check Tera syntax**:
   Use Tera playground: https://keats.github.io/tera/docs/

3. **Validate TOML**:
   After rendering, validate TOML syntax with online validators.

4. **Enable debug logging**:
   ```bash
   RUST_LOG=debug clnrm run my-test.clnrm.toml
   ```

---

## Advanced Topics

### Custom Filters (Future)

clnrm may support custom filters in future versions:

```toml
# Hypothetical syntax
name = "{{ vars.name | slugify }}"  # "My Test" -> "my-test"
```

### Template Inheritance (Future)

Tera supports template inheritance, which may be added:

```toml
# base.toml
{% block otel %}
[otel]
exporter = "stdout"
{% endblock %}

# child.toml
{% extends "base.toml" %}
{% block otel %}
[otel]
exporter = "otlp"  # Override
{% endblock %}
```

---

## Reference

### Tera Syntax Summary

| Syntax | Purpose | Example |
|--------|---------|---------|
| `{{ }}` | Variable substitution | `{{ vars.name }}` |
| `{% %}` | Control structure | `{% if %} ... {% endif %}` |
| `{# #}` | Comment | `{# This is a comment #}` |

### Control Structures

- `{% for item in list %} ... {% endfor %}`
- `{% if condition %} ... {% elif %} ... {% else %} ... {% endif %}`
- `{% include "path" %}`
- `{% macro name(args) %} ... {% endmacro %}`
- `{% set var = value %}`

### Custom Functions

- `env(name, default="")`
- `now_rfc3339()`
- `sha256(input)`
- `toml_encode(value)`

### Built-in Filters

See [Tera filters documentation](https://keats.github.io/tera/docs/#filters) for complete list:
- `upper`, `lower`, `capitalize`
- `length`, `reverse`, `sort`
- `join(sep=",")`, `split(pat=" ")`
- `default(value="fallback")`
- And many more...

---

## Examples

Complete examples are available in `examples/templates/`:

- `simple-variables.clnrm.toml` - Basic variable substitution
- `matrix-expansion.clnrm.toml` - Loop-based scenario generation
- `multi-environment.clnrm.toml` - Environment-specific configurations
- `service-mesh.clnrm.toml` - Complex service orchestration
- `ci-integration.clnrm.toml` - CI/CD pipeline integration

---

## Further Reading

- [Tera Documentation](https://keats.github.io/tera/docs/)
- [TOML Specification](https://toml.io/)
- [clnrm TOML Reference](./TOML_REFERENCE.md)
- [clnrm Testing Guide](./TESTING.md)

---

## Support

For template-related issues:
1. Check this guide's [Troubleshooting](#troubleshooting) section
2. Review [examples/templates/](../examples/templates/)
3. Open an issue: https://github.com/seanchatmangpt/clnrm/issues
