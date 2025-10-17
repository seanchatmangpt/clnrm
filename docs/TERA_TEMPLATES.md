# Tera Template Guide for Cleanroom v0.7.0

## Table of Contents

- [Introduction](#introduction)
- [Macro Library](#macro-library)
- [Basic Syntax](#basic-syntax)
- [Template Variables](#template-variables)
- [Custom Functions](#custom-functions)
- [Template Namespaces](#template-namespaces)
- [Common Patterns](#common-patterns)
- [Best Practices](#best-practices)
- [Examples](#examples)

## Introduction

Cleanroom v0.7.0 introduces **Tera templating** for dynamic test configuration. Tera uses Jinja2-like syntax to enable:

- Dynamic test names and descriptions
- Environment-based configuration
- Matrix testing (cross-product of variables)
- Reproducible timestamps and identifiers
- Conditional configuration blocks
- **NEW in v0.7.0**: Macro library for eliminating TOML boilerplate

## Macro Library

Cleanroom v0.7.0 introduces a comprehensive macro library (`_macros.toml.tera`) that eliminates TOML boilerplate for common OpenTelemetry validation patterns.

### Installation

The macro library is automatically installed to `~/.clnrm/templates/` when you run:

```bash
clnrm init
```

### Usage

Import the macro library in your `.clnrm.toml.tera` template files:

```tera
{% import "_macros.toml.tera" as m %}
```

### Available Macros

#### 1. `span(name, kind, attrs={})`

Generate a `[[expect.span]]` block for OpenTelemetry span validation.

**Parameters:**
- `name` (string): Span name to match
- `kind` (string): Span kind (`"internal"`, `"server"`, `"client"`, `"producer"`, `"consumer"`)
- `attrs` (object, optional): Attribute key-value pairs for validation

**Example:**

```tera
{{ m::span("http.request", "server", {"http.method": "GET", "http.status_code": "200"}) }}
```

**Produces:**

```toml
[[expect.span]]
name = "http.request"
kind = "server"
attrs.all = { "http.method" = "GET", "http.status_code" = "200" }
```

#### 2. `lifecycle(service)`

Generate complete service lifecycle span expectations with ordering constraints.

Creates three spans: `{service}.start`, `{service}.exec`, `{service}.stop` and ensures they execute in the correct order.

**Parameters:**
- `service` (string): Service name prefix

**Example:**

```tera
{{ m::lifecycle("postgres") }}
```

**Produces:**

```toml
# Service lifecycle: postgres
[[expect.span]]
name = "postgres.start"
kind = "internal"

[[expect.span]]
name = "postgres.exec"
kind = "internal"

[[expect.span]]
name = "postgres.stop"
kind = "internal"

[expect.order]
must_precede = [
  ["postgres.start", "postgres.exec"],
  ["postgres.exec", "postgres.stop"]
]
```

#### 3. `edges(pairs)`

Generate parent-child graph edge constraints for span relationships.

**Parameters:**
- `pairs` (array): Array of `[parent, child]` tuples

**Example:**

```tera
{{ m::edges([
  ["root", "child1"],
  ["root", "child2"],
  ["child1", "grandchild"]
]) }}
```

**Produces:**

```toml
[expect.graph]
must_include = [
  ["root", "child1"],
  ["root", "child2"],
  ["child1", "grandchild"]
]
```

#### 4. `window(start, end)`

Generate time window constraint ensuring one span contains another.

**Parameters:**
- `start` (string): Outer span name (must start before and end after)
- `end` (string): Inner span name (must be contained within outer span)

**Example:**

```tera
{{ m::window("transaction", "db.query") }}
```

**Produces:**

```toml
[expect.window]
"transaction" = { contains = ["db.query"] }
```

#### 5. `count(kind, min, max=none)`

Generate span count constraint by kind.

**Parameters:**
- `kind` (string): Span kind to count
- `min` (number): Minimum expected span count
- `max` (number, optional): Maximum expected span count

**Example:**

```tera
{{ m::count("server", 1, 5) }}
{{ m::count("internal", 3) }}
```

**Produces:**

```toml
[expect.count]
by_kind.server = { min = 1, max = 5 }

[expect.count]
by_kind.internal = { min = 3 }
```

#### 6. `multi_lifecycle(services)`

Generate lifecycle spans for multiple services at once.

**Parameters:**
- `services` (array): Array of service names

**Example:**

```tera
{{ m::multi_lifecycle(["postgres", "redis", "api"]) }}
```

**Produces:**

Complete lifecycle blocks for each service (see `lifecycle()` macro output).

#### 7. `span_with_attrs(name, kind, attr_pairs)`

Convenience wrapper combining `span()` with inline attributes.

**Parameters:**
- `name` (string): Span name
- `kind` (string): Span kind
- `attr_pairs` (object): Attribute key-value pairs

**Example:**

```tera
{{ m::span_with_attrs("api.request", "server", {
  "method": "POST",
  "endpoint": "/users"
}) }}
```

**Produces:**

```toml
[[expect.span]]
name = "api.request"
kind = "server"
attrs.all = { "method" = "POST", "endpoint" = "/users" }
```

#### 8. `attrs(pairs)`

Generate attribute constraints as a TOML inline table (helper macro).

**Parameters:**
- `pairs` (object): Key-value pairs of attributes

**Example:**

```tera
{{ m::attrs({"http.method": "GET", "http.status_code": "200"}) }}
```

**Produces:**

```toml
{ "http.method" = "GET", "http.status_code" = "200" }
```

### Complete Macro Example

```tera
{% import "_macros.toml.tera" as m %}

[test.metadata]
name = "microservices-otel-validation"
description = "Validate OpenTelemetry traces across microservices"

[services.api]
type = "generic_container"
image = "nginx:alpine"

[services.postgres]
type = "generic_container"
image = "postgres:15"

[services.redis]
type = "generic_container"
image = "redis:7-alpine"

# Generate lifecycle spans for all services
{{ m::lifecycle("api") }}
{{ m::lifecycle("postgres") }}
{{ m::lifecycle("redis") }}

# Define parent-child relationships
{{ m::edges([
  ["test-root", "api.start"],
  ["api.exec", "postgres.exec"],
  ["api.exec", "redis.exec"]
]) }}

# Ensure database queries happen within API execution window
{{ m::window("api.exec", "postgres.exec") }}
{{ m::window("api.exec", "redis.exec") }}

# Validate span counts
{{ m::count("internal", 9) }}
{{ m::count("client", 2, 4) }}

# Validate HTTP request span
{{ m::span("http.request", "server", {
  "http.method": "GET",
  "http.route": "/api/users"
}) }}

[[steps]]
name = "execute_test"
command = ["sh", "-c", "echo 'Test execution complete'"]

[assertions]
container_should_have_executed_commands = 1
execution_should_be_hermetic = true
```

### Macro Best Practices

#### 1. Use Lifecycle for Services

Instead of manually defining start/exec/stop spans:

```tera
# ❌ Verbose
[[expect.span]]
name = "postgres.start"
kind = "internal"

[[expect.span]]
name = "postgres.exec"
kind = "internal"

[[expect.span]]
name = "postgres.stop"
kind = "internal"

[expect.order]
must_precede = [
  ["postgres.start", "postgres.exec"],
  ["postgres.exec", "postgres.stop"]
]

# ✅ Concise
{{ m::lifecycle("postgres") }}
```

#### 2. Combine Macros for Complex Validation

```tera
# Define transaction span with attributes
{{ m::span("transaction.commit", "internal", {"tx.id": "12345"}) }}

# Ensure DB operations happen within transaction
{{ m::window("transaction.commit", "postgres.exec") }}
{{ m::window("transaction.commit", "redis.exec") }}

# Validate relationships
{{ m::edges([
  ["transaction.commit", "postgres.exec"],
  ["transaction.commit", "redis.exec"]
]) }}
```

#### 3. Parameterize Service Names

```tera
{% set db_service = "postgres" %}
{% set cache_service = "redis" %}

{{ m::lifecycle(db_service) }}
{{ m::lifecycle(cache_service) }}

{{ m::edges([[db_service ~ ".exec", cache_service ~ ".exec"]]) }}
```

#### 4. Use Multi-Lifecycle for Common Patterns

```tera
# ❌ Repetitive
{{ m::lifecycle("service1") }}
{{ m::lifecycle("service2") }}
{{ m::lifecycle("service3") }}

# ✅ Batch operation
{{ m::multi_lifecycle(["service1", "service2", "service3"]) }}
```

### Macro Troubleshooting

#### Macro Not Found

```
Error: Macro 'span' not found
```

**Solution**: Ensure you've imported the macro library:

```tera
{% import "_macros.toml.tera" as m %}
```

#### Invalid TOML Output

```
Error: Invalid TOML: duplicate key 'expect.graph'
```

**Solution**: Macros that generate section headers (`[expect.graph]`) can only be used once per file. Combine multiple edges into a single `edges()` call:

```tera
# ❌ Wrong - duplicate sections
{{ m::edges([["a", "b"]]) }}
{{ m::edges([["c", "d"]]) }}

# ✅ Correct - single call
{{ m::edges([["a", "b"], ["c", "d"]]) }}
```

#### Empty Attributes

```tera
# This works - attrs will be omitted
{{ m::span("my.span", "internal", {}) }}
{{ m::span("my.span", "internal") }}
```

Both produce the same output:

```toml
[[expect.span]]
name = "my.span"
kind = "internal"
```

## Basic Syntax

### Variable Substitution

```toml
[vars]
test_name = "my_test"
service_name = "api-service"

[meta]
name = "{{ vars.test_name }}"
description = "Test for {{ vars.service_name }}"
```

### Filters

Tera supports filters for transforming values:

```toml
[meta]
name = "{{ vars.test_name | upper }}"  # MY_TEST
description = "{{ vars.test_name | title }}"  # My Test
```

Common filters:
- `upper` / `lower` - Case conversion
- `title` - Title case
- `trim` - Remove whitespace
- `replace(from=":", to="-")` - String replacement
- `truncate(length=8, end="")` - Truncate strings
- `default(value="fallback")` - Provide default value

### Conditionals

```toml
{% if env(name="CI") %}
[otel]
exporter = "otlp-grpc"
endpoint = "https://collector.example.com"
{% else %}
[otel]
exporter = "stdout"
{% endif %}
```

### Loops

```toml
[vars]
test_count = 3

{% for i in range(end=vars.test_count) %}
[[scenario]]
name = "test_{{ i }}"
service = "api"
run = "echo 'Test {{ i }}'"
{% endfor %}
```

## Template Variables

Variables are defined in the `[vars]` section:

```toml
[vars]
test_name = "integration_test"
api_version = "v1"
timeout_ms = 5000
enable_tracing = true
endpoints = ["health", "status", "metrics"]
```

Access with `{{ vars.variable_name }}`:

```toml
[service.api]
image = "myapi:{{ vars.api_version }}"

[[scenario]]
name = "test_{{ vars.test_name }}"
timeout_ms = {{ vars.timeout_ms }}
```

## Custom Functions

Cleanroom provides 4 custom Tera functions:

### 1. `env(name="VAR_NAME")`

Read environment variables:

```toml
[otel]
exporter = "{{ env(name="OTEL_EXPORTER") | default(value="stdout") }}"
endpoint = "{{ env(name="OTEL_ENDPOINT") }}"
```

**Use with default filter** to provide fallback:

```toml
api_key = "{{ env(name="API_KEY") | default(value="dev-key") }}"
```

### 2. `now_rfc3339()`

Get current timestamp in RFC3339 format:

```toml
[vars]
test_timestamp = "{{ now_rfc3339() }}"

[otel.resources]
"test.started_at" = "{{ vars.test_timestamp }}"
```

Output: `2025-10-17T12:34:56.789Z`

**Determinism**: When `[determinism.freeze_clock]` is set, `now_rfc3339()` returns the frozen time.

### 3. `sha256(s="text")`

Compute SHA-256 hash:

```toml
[vars]
test_id = "{{ sha256(s=vars.test_name) }}"

[report]
digest = "reports/{{ vars.test_id }}.sha256"
```

Output: 64-character hexadecimal hash

### 4. `toml_encode(value)`

Encode values as TOML (useful for nested structures):

```toml
# Example usage for dynamic TOML generation
{% set config = {"key": "value", "number": 42} %}
{{ toml_encode(value=config) }}
```

## Template Namespaces

Cleanroom provides three template namespaces:

### 1. `vars.*` - User-Defined Variables

```toml
[vars]
service_name = "api"
version = "1.0"

[service.{{ vars.service_name }}]
image = "myapi:{{ vars.version }}"
```

### 2. `matrix.*` - Matrix Testing

Cross-product testing across multiple dimensions:

```toml
[matrix]
os = ["alpine", "ubuntu"]
version = ["3.18", "22.04"]

[service.test_runner]
image = "{{ matrix.os }}:{{ matrix.version }}"

[[scenario]]
name = "test_{{ matrix.os }}_{{ matrix.version }}"
run = "echo 'Testing on {{ matrix.os }}'"
```

**Matrix expansion** generates one test per combination:
- `alpine:3.18`
- `alpine:22.04`
- `ubuntu:3.18`
- `ubuntu:22.04`

### 3. `otel.*` - OpenTelemetry Context

Access OTEL configuration in templates:

```toml
[otel]
exporter = "stdout"

{% if otel.exporter == "otlp-grpc" %}
[otel.headers]
"x-api-key" = "{{ env(name="OTEL_API_KEY") }}"
{% endif %}
```

## Common Patterns

### Pattern 1: Environment-Based Configuration

```toml
[vars]
environment = "{{ env(name="ENV") | default(value="dev") }}"

[meta]
name = "test_{{ vars.environment }}"

[service.database]
image = "postgres:15"
env = {
  POSTGRES_DB = "testdb_{{ vars.environment }}",
  POSTGRES_PASSWORD = "{{ env(name="DB_PASSWORD") }}"
}
```

### Pattern 2: Unique Test Identifiers

```toml
[vars]
test_name = "api_integration"
test_id = "{{ sha256(s=vars.test_name) | truncate(length=8, end="") }}"

[report]
json = "reports/{{ vars.test_id }}.json"
```

### Pattern 3: Timestamped Reports

```toml
[vars]
timestamp = "{{ now_rfc3339() | replace(from=":", to="-") }}"

[report]
json = "reports/test_{{ vars.timestamp }}.json"
junit = "reports/junit_{{ vars.timestamp }}.xml"
```

### Pattern 4: Conditional Features

```toml
{% if env(name="ENABLE_TRACING") %}
[otel]
exporter = "otlp-grpc"
sample_ratio = 1.0
{% else %}
[otel]
exporter = "stdout"
{% endif %}
```

### Pattern 5: Matrix Testing

```toml
[matrix]
database = ["postgres", "mysql"]
cache = ["redis", "memcached"]

[service.db]
image = "{{ matrix.database }}:latest"

[service.cache]
image = "{{ matrix.cache }}:latest"

[[scenario]]
name = "test_{{ matrix.database }}_{{ matrix.cache }}"
run = "echo 'Testing with {{ matrix.database }} and {{ matrix.cache }}'"
```

## Best Practices

### 1. Use Variables for Reusability

**Good**:
```toml
[vars]
image_tag = "v1.2.3"

[service.api]
image = "myapi:{{ vars.image_tag }}"

[service.worker]
image = "myworker:{{ vars.image_tag }}"
```

**Avoid**:
```toml
[service.api]
image = "myapi:v1.2.3"

[service.worker]
image = "myworker:v1.2.3"  # Duplicated version
```

### 2. Provide Defaults for Environment Variables

**Good**:
```toml
exporter = "{{ env(name="OTEL_EXPORTER") | default(value="stdout") }}"
```

**Avoid**:
```toml
exporter = "{{ env(name="OTEL_EXPORTER") }}"  # Fails if not set
```

### 3. Use Determinism for Reproducibility

```toml
[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"

[vars]
# This will always return the frozen time
timestamp = "{{ now_rfc3339() }}"
```

### 4. Separate Configuration from Logic

**Good**:
```toml
[vars]
max_retries = 3
timeout_ms = 5000

[[scenario]]
max_retries = {{ vars.max_retries }}
timeout_ms = {{ vars.timeout_ms }}
```

**Avoid**:
```toml
[[scenario]]
max_retries = 3  # Hardcoded
timeout_ms = 5000  # Hardcoded
```

### 5. Document Template Variables

```toml
[vars]
# API version to test against
api_version = "v1"

# Maximum allowed response time in milliseconds
max_response_time_ms = 1000

# Enable detailed tracing (set to false in production)
enable_tracing = true
```

## Examples

### Example 1: Simple Dynamic Test

```toml
[meta]
name = "api_test_{{ env(name="ENV") | default(value="dev") }}"
version = "0.6.0"

[vars]
api_endpoint = "https://api.{{ env(name="ENV") }}.example.com"

[service.api]
plugin = "generic_container"
image = "nginx:alpine"

[[scenario]]
name = "health_check"
run = "curl {{ vars.api_endpoint }}/health"
```

### Example 2: Matrix Testing

```toml
[meta]
name = "cross_platform_test"
version = "0.6.0"

[matrix]
os = ["alpine", "ubuntu", "debian"]
arch = ["amd64", "arm64"]

[service.test_runner]
plugin = "generic_container"
image = "{{ matrix.os }}:latest"
platform = "linux/{{ matrix.arch }}"

[[scenario]]
name = "test_{{ matrix.os }}_{{ matrix.arch }}"
run = "uname -a"
```

### Example 3: Comprehensive OTEL Validation

```toml
[meta]
name = "otel_validation_{{ sha256(s=vars.test_name) | truncate(length=8, end="") }}"
version = "0.6.0"

[vars]
test_name = "comprehensive_otel_test"
service_name = "api-service"

[otel]
exporter = "{{ env(name="OTEL_EXPORTER") | default(value="stdout") }}"
sample_ratio = 1.0
resources = {
  "service.name" = "{{ vars.service_name }}",
  "test.timestamp" = "{{ now_rfc3339() }}",
  "test.id" = "{{ sha256(s=vars.test_name) }}"
}

[service.api]
plugin = "generic_container"
image = "nginx:alpine"

[[expect.span]]
name = "api.request"
kind = "server"
attrs.all = { "service.name" = "{{ vars.service_name }}" }

[expect.order]
must_precede = [
  ["api.start", "api.request"],
  ["api.request", "api.stop"]
]

[expect.status]
all = "ok"
by_name."api.*" = "ok"

[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"

[report]
json = "reports/{{ vars.test_name }}.json"
junit = "reports/junit.xml"
digest = "reports/{{ sha256(s=vars.test_name) }}.sha256"
```

### Example 4: Conditional Configuration

```toml
[meta]
name = "adaptive_test"
version = "0.6.0"

[vars]
is_ci = {{ env(name="CI") is defined }}
max_workers = {% if vars.is_ci %}10{% else %}4{% endif %}

{% if vars.is_ci %}
[otel]
exporter = "otlp-grpc"
endpoint = "{{ env(name="OTEL_COLLECTOR_ENDPOINT") }}"
{% else %}
[otel]
exporter = "stdout"
{% endif %}

[limits]
cpu_millicores = {% if vars.is_ci %}1000{% else %}500{% endif %}
memory_mb = {% if vars.is_ci %}1024{% else %}512{% endif %}
```

## Template Debugging

### Enable Template Rendering Output

Set environment variable to see rendered output:

```bash
CLNRM_DEBUG_TEMPLATES=1 clnrm run test.clnrm.toml
```

### Validate Template Syntax

```bash
clnrm validate test.clnrm.toml
```

### Test Template Rendering

Generate a template and inspect the output:

```bash
clnrm template otel | head -50
```

## Advanced Topics

### Custom Filters

Tera supports chaining filters:

```toml
name = "{{ vars.test_name | upper | replace(from="_", to="-") }}"
# "my_test" -> "MY-TEST"
```

### Macros

Reusable template blocks:

```toml
{% macro service_config(name, image) %}
[service.{{ name }}]
plugin = "generic_container"
image = "{{ image }}"
{% endmacro %}

{{ service_config(name="api", image="nginx:alpine") }}
{{ service_config(name="db", image="postgres:15") }}
```

### Include Files

Split configuration across files:

```toml
{% include "common_vars.tera" %}

[meta]
name = "{{ vars.test_name }}"
```

## Troubleshooting

### Template Not Rendering

**Problem**: Variables not substituted

**Solution**: Ensure Tera syntax is detected (contains `{{`, `{%`, or `{#`)

### Environment Variable Not Found

**Problem**: `env(name="VAR")` fails

**Solution**: Use default filter
```toml
{{ env(name="VAR") | default(value="fallback") }}
```

### Invalid Template Syntax

**Problem**: Parse errors

**Solution**: Check for:
- Unmatched `{{` / `}}`
- Unclosed `{% if %}` / `{% endif %}`
- Invalid filter syntax

## Resources

- **Tera Documentation**: https://keats.github.io/tera/docs/
- **Cleanroom Examples**: `examples/` directory
- **Template Generators**: `clnrm template --help`

---

**Next Steps**:
- Try the template generators: `clnrm template otel`
- Explore examples: `examples/optimus-prime-platform/`
- Read the migration guide: `docs/MIGRATION_v0.6.0.md`
