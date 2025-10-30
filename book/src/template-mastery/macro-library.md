# Macro Library

The macro library provides reusable Tera macros for common TOML patterns. This chapter covers using and extending the macro library for efficient test configuration.

## Overview

The macro library includes 8 core macros:
- **`span()`** - Single span validation
- **`lifecycle()`** - Service start/exec/stop lifecycle
- **`edges()`** - Parent-child relationships
- **`window()`** - Time containment validation
- **`count()`** - Span count constraints
- **`multi_lifecycle()`** - Batch service lifecycles
- **`span_with_attrs()`** - Span with attributes
- **`attrs()`** - Inline attribute table

## Using Core Macros

### Span Validation Macro

Create span validation with attributes:

```toml
{% import "_macros.toml.tera" as m %}

# Single span with attributes
{{ m::span("api.request", kind="server", attrs={"http.method": "GET", "http.route": "/api/users"}) }}

# Multiple spans
{{ m::span("api.request", kind="server") }}
{{ m::span("api.response", kind="server") }}
{{ m::span("db.query", kind="client") }}
```

### Lifecycle Macro

Validate complete service lifecycle:

```toml
{% import "_macros.toml.tera" as m %}

# Service lifecycle for PostgreSQL
{{ m::lifecycle("postgres") }}

# Custom lifecycle with specific operations
{{ m::lifecycle("api", operations=["start", "exec", "stop"], attrs={"version": "1.0.0"}) }}
```

### Edge Relationships Macro

Define parent-child span relationships:

```toml
{% import "_macros.toml.tera" as m %}

# Simple parent-child relationship
{{ m::edges([["clnrm.run", "api.request"]]) }}

# Complex relationship graph
{{ m::edges([
    ["clnrm.run", "api.request"],
    ["api.request", "db.query"],
    ["db.query", "api.response"],
    ["api.request", "cache.get"]
]) }}
```

### Time Window Macro

Validate temporal constraints:

```toml
{% import "_macros.toml.tera" as m %}

# Simple time window
{{ m::window("api.request", "api.response") }}

# Time window with duration constraint
{{ m::window("api.request", "api.response", max_duration_ms=1000) }}

# Multiple time windows
{{ m::window("db.query", "cache.get", max_duration_ms=500) }}
{{ m::window("api.request", "db.query", max_duration_ms=100) }}
```

### Count Validation Macro

Validate span counts:

```toml
{% import "_macros.toml.tera" as m %}

# Count by span kind
{{ m::count("internal", min=2, max=5) }}

# Count by specific span name
{{ m::count("api.request", min=1, max=1) }}

# Multiple count constraints
{{ m::count("server", min=3, max=3) }}
{{ m::count("client", min=2, max=2) }}
```

## Advanced Macro Usage

### Multi-Service Lifecycle

Coordinate multiple service lifecycles:

```toml
{% import "_macros.toml.tera" as m %}

# Multi-service lifecycle
{{ m::multi_lifecycle(["api", "database", "cache"]) }}

# With custom ordering
{{ m::multi_lifecycle(["database", "cache", "api"], order="sequential") }}

# With dependency relationships
{{ m::multi_lifecycle(["database", "cache", "api"],
    dependencies={
        "api": ["database", "cache"],
        "cache": ["database"]
    }
) }}
```

### Complex Span Patterns

Create complex span validation patterns:

```toml
{% import "_macros.toml.tera" as m %}

# HTTP request pattern
{{ m::span_with_attrs("api.request", "server", {
    "http.method": "GET",
    "http.route": "/api/users",
    "http.user_agent": "curl/.*"
}) }}

{{ m::span_with_attrs("api.response", "server", {
    "http.status_code": "200",
    "http.response_size": "[0-9]+"
}) }}

# Database operation pattern
{{ m::span_with_attrs("db.query", "client", {
    "db.system": "postgresql",
    "db.operation": "SELECT",
    "db.table": "users"
}) }}

# Relationships between patterns
{{ m::edges([
    ["api.request", "db.query"],
    ["db.query", "api.response"]
]) }}

{{ m::window("api.request", "api.response", max_duration_ms=500) }}
```

### Attribute Table Macro

Create inline attribute tables:

```toml
{% import "_macros.toml.tera" as m %}

# Attribute validation using macro
{{ m::span("api.request", kind="server", attrs=m::attrs({
    "http.method": "GET",
    "http.route": "/api/users",
    "http.status_code": "200"
})) }}

# Complex attribute patterns
{{ m::span("db.query", kind="client", attrs=m::attrs({
    "db.system": "postgresql",
    "db.operation": "SELECT",
    "db.table": "users",
    "db.row_count": "[0-9]+"
})) }}
```

## Macro Composition

### Building Complex Patterns

Combine macros for complex validation:

```toml
{% import "_macros.toml.tera" as m %}

# Complete API test pattern
{{ m::lifecycle("api") }}

{{ m::span_with_attrs("api.request", "server", m::attrs({
    "http.method": "GET",
    "http.route": "/api/health"
})) }}

{{ m::span_with_attrs("api.response", "server", m::attrs({
    "http.status_code": "200"
})) }}

{{ m::edges([
    ["api.request", "api.response"]
]) }}

{{ m::window("api.request", "api.response", max_duration_ms=100) }}

{{ m::count("server", min=2, max=2) }}
```

### Service Integration Pattern

Pattern for integrating multiple services:

```toml
{% import "_macros.toml.tera" as m %}

# Database service
{{ m::lifecycle("database") }}
{{ m::span("database.start", kind="internal") }}
{{ m::span("database.exec", kind="internal") }}
{{ m::span("database.stop", kind="internal") }}

# API service
{{ m::lifecycle("api") }}
{{ m::span("api.start", kind="internal") }}
{{ m::span("api.exec", kind="internal") }}
{{ m::span("api.stop", kind="internal") }}

# Service interaction
{{ m::span_with_attrs("api.request", "server", m::attrs({
    "http.method": "GET",
    "http.route": "/api/data"
})) }}

{{ m::span_with_attrs("db.query", "client", m::attrs({
    "db.system": "postgresql",
    "db.operation": "SELECT"
})) }}

# Relationships
{{ m::edges([
    ["clnrm.run", "database.start"],
    ["clnrm.run", "api.start"],
    ["api.request", "db.query"],
    ["db.query", "api.response"]
]) }}

{{ m::window("api.request", "db.query", max_duration_ms=50) }}
{{ m::window("db.query", "api.response", max_duration_ms=100) }}

# Count validation
{{ m::count("internal", min=4, max=4) }}
{{ m::count("server", min=1, max=1) }}
{{ m::count("client", min=1, max=1) }}
```

## Custom Macro Creation

### Extending the Macro Library

Create custom macros for your domain:

```rust
// In _macros.toml.tera or custom macro file
{% macro custom_api_test(service_name, endpoint, expected_status) %}
[[expect.span]]
name = "{{ service_name }}.request"
kind = "server"
attrs.all = { "http.route" = "{{ endpoint }}", "http.method" = "GET" }

[[expect.span]]
name = "{{ service_name }}.response"
kind = "server"
attrs.all = { "http.status_code" = "{{ expected_status }}" }

[expect.order]
must_precede = [["{{ service_name }}.request", "{{ service_name }}.response"]]

[expect.window]
start_span = "{{ service_name }}.request"
end_span = "{{ service_name }}.response"
max_duration_ms = 200

[expect.count]
by_name."{{ service_name }}.request" = { min = 1, max = 1 }
by_name."{{ service_name }}.response" = { min = 1, max = 1 }
{% endmacro custom_api_test %}
```

### Using Custom Macros

```toml
{% import "_macros.toml.tera" as m %}

# Use custom macro
{{ m::custom_api_test("api", "/api/health", "200") }}
{{ m::custom_api_test("api", "/api/users", "200") }}
{{ m::custom_api_test("api", "/api/error", "500") }}
```

## Best Practices

### 1. Use Macros for Common Patterns

```toml
# ✅ Good: Use lifecycle macro for common pattern
{% import "_macros.toml.tera" as m %}
{{ m::lifecycle("api") }}
{{ m::lifecycle("database") }}
{{ m::lifecycle("cache") }}

# ✅ Good: Use span macro for consistent validation
{{ m::span("clnrm.run", kind="internal", attrs={"result": "pass"}) }}
```

### 2. Combine Macros for Complex Validation

```toml
# ✅ Good: Combine macros for complete validation
{% import "_macros.toml.tera" as m %}

# Service lifecycle
{{ m::lifecycle("{{ service }}") }}

# Request/response pattern
{{ m::span_with_attrs("{{ service }}.request", "server", m::attrs({
    "http.method": "GET",
    "http.route": "{{ endpoint }}"
})) }}

{{ m::span_with_attrs("{{ service }}.response", "server", m::attrs({
    "http.status_code": "200"
})) }}

# Relationships and timing
{{ m::edges([["{{ service }}.request", "{{ service }}.response"]]) }}
{{ m::window("{{ service }}.request", "{{ service }}.response", max_duration_ms=100) }}
```

### 3. Create Domain-Specific Macros

```toml
# ✅ Good: Domain-specific macro for HTTP APIs
{% macro http_api_test(base_url, endpoint, method="GET", expected_status="200") %}
[[expect.span]]
name = "http.request"
kind = "client"
attrs.all = {
    "http.method" = "{{ method }}",
    "http.url" = "{{ base_url }}{{ endpoint }}",
    "http.status_code" = "{{ expected_status }}"
}

[expect.order]
must_precede = [["http.request", "http.response"]]

[expect.window]
start_span = "http.request"
end_span = "http.response"
max_duration_ms = 1000
{% endmacro http_api_test %}
```

### 4. Validate Macro Output

```bash
# ✅ Good: Validate macro rendering
clnrm template render test.clnrm.toml.tera > test.clnrm.toml
clnrm validate test.clnrm.toml
```

## Common Macro Patterns

### Database Test Pattern

```toml
{% import "_macros.toml.tera" as m %}

# Database service lifecycle
{{ m::lifecycle("postgres") }}

# Database operations
{{ m::span_with_attrs("db.connection", "client", m::attrs({
    "db.system": "postgresql",
    "db.operation": "connect"
})) }}

{{ m::span_with_attrs("db.query", "client", m::attrs({
    "db.system": "postgresql",
    "db.operation": "SELECT",
    "db.table": "users"
})) }}

{{ m::span_with_attrs("db.response", "client", m::attrs({
    "db.system": "postgresql",
    "db.row_count": "[0-9]+"
})) }}

# Database relationships
{{ m::edges([
    ["db.connection", "db.query"],
    ["db.query", "db.response"]
]) }}

{{ m::window("db.query", "db.response", max_duration_ms=100) }}

# Count validation
{{ m::count("client", min=3, max=3) }}
```

### Microservices Pattern

```toml
{% import "_macros.toml.tera" as m %}

# Multiple service lifecycles
{{ m::multi_lifecycle(["api", "auth", "user", "order"]) }}

# Service interactions
{{ m::span_with_attrs("api.request", "server", m::attrs({
    "http.method": "GET",
    "http.route": "/api/order"
})) }}

{{ m::span_with_attrs("auth.check", "internal", m::attrs({
    "auth.user_id": "12345"
})) }}

{{ m::span_with_attrs("user.get", "client", m::attrs({
    "user.id": "12345"
})) }}

{{ m::span_with_attrs("order.create", "client", m::attrs({
    "order.user_id": "12345"
})) }}

# Complex relationships
{{ m::edges([
    ["api.request", "auth.check"],
    ["auth.check", "user.get"],
    ["user.get", "order.create"],
    ["order.create", "api.response"]
]) }}

# Temporal constraints
{{ m::window("api.request", "auth.check", max_duration_ms=10) }}
{{ m::window("auth.check", "user.get", max_duration_ms=20) }}
{{ m::window("user.get", "order.create", max_duration_ms=30) }}
{{ m::window("order.create", "api.response", max_duration_ms=50) }}
```

### Performance Test Pattern

```toml
{% import "_macros.toml.tera" as m %}

# Performance test lifecycle
{{ m::lifecycle("load_generator") }}

# Performance spans
{{ m::span_with_attrs("load.start", "internal", m::attrs({
    "load.target_rps": "1000",
    "load.duration_seconds": "60"
})) }}

{{ m::span_with_attrs("load.requests", "internal", m::attrs({
    "http.method": "GET",
    "http.url": "http://localhost:80/api/test"
})) }}

{{ m::span_with_attrs("load.end", "internal", m::attrs({
    "load.total_requests": "[0-9]+",
    "load.errors": "[0-9]+"
})) }}

# Performance relationships
{{ m::edges([
    ["load.start", "load.requests"],
    ["load.requests", "load.end"]
]) }}

# Performance validation
{{ m::count("internal", min=3, max=3) }}

[expect.performance]
spans = ["load.requests"]
max_p95_latency_ms = 200
min_throughput_rps = 800
```

## Next Steps

Now that you understand the macro library:

1. **Try the examples**: Run the macro examples in this chapter
2. **Create custom macros**: Build macros for your specific patterns
3. **Learn variable resolution**: Move on to [Variable Resolution](variable-resolution.md)
4. **Master advanced patterns**: Review [Advanced Testing Patterns](../advanced-patterns/README.md)

## Further Reading

- [Tera Macro Documentation](https://tera.netlify.app/docs/templates/macros/)
- [Macro Library Implementation](../templates/_macros.toml.tera)
- [Template System Overview](README.md)
