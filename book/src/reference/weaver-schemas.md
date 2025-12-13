# Weaver Schema Reference (v2.0.0)

This reference documents the schema structure for Weaver-based telemetry validation in clnrm v2.0.0.

## Overview

**OpenTelemetry Weaver** is the source of truth for validation in clnrm v2.0.0. This chapter explains schema-first validation and how to use Weaver live-check with proper health checks to prevent false positives.

## Key Concepts

### Schema-First Validation

```
Traditional Testing:
  Test passes ✅ → Assumes functionality works → FALSE POSITIVE ❌

Weaver v2.0.0:
  Test passes ✅ + Weaver validates schema ✅ → TRUE POSITIVE ✅
  Schema validation proves actual runtime behavior
```

### 80/20 Validation Strategy

**4 Critical Attributes** prove **80% of functionality**:

1. **Service Name** - Identifies the service generating telemetry
2. **Span Names** - Validates operation execution
3. **Span Attributes** - Confirms correct configuration
4. **Sample Count** - Ensures telemetry is actually generated

## Schema Registry

### Location
- **Registry Path**: `registry/` directory
- **Schema Files**: Comprehensive semantic conventions
- **Validation Engine**: Weaver live-check integration

### Schema Structure

```yaml
# Example span schema
- id: http_server_request
  type: span
  brief: "HTTP server request span"
  note: "Represents an HTTP server request"
  stability: stable
  attributes:
    - id: http.method
      type: string
      brief: "HTTP request method"
      examples: ["GET", "POST", "PUT"]
    - id: http.url
      type: string
      brief: "HTTP request URL"
      examples: ["https://example.com/api/users"]
    - id: http.status_code
      type: int
      brief: "HTTP response status code"
      examples: [200, 404, 500]
```

## Validation Configuration

### Basic Validation

```toml
# Enable Weaver validation
[otel]
exporter = "otlp-http"
endpoint = "http://weaver-collector:4318"

# Validation expectations
[expect.otel]
spans = [
    { name = "http_request", attributes = { "http.method" = "GET" } }
]
```

### Advanced Validation

```toml
[expect.otel]
# Required spans
spans = [
    {
        name = "http_server_request",
        kind = "server",
        attributes = {
            "http.method" = "POST",
            "http.route" = "/api/users"
        }
    },
    {
        name = "database_query",
        kind = "client",
        attributes = {
            "db.system" = "postgresql",
            "db.operation" = "SELECT"
        }
    }
]

# Required metrics
metrics = [
    {
        name = "http_requests_total",
        type = "counter",
        attributes = {
            "method" = "POST",
            "status" = "200"
        }
    }
]
```

## Health Check Integration

### v2.0.0 Health Checks

**CRITICAL**: v2.0.0 requires proper health checks for Weaver validation:

```toml
[containers.api]
image = "myapp:latest"
healthcheck = "curl -f http://localhost:8080/health"
ports = [8080]

[containers.weaver]
image = "otel/weaver:latest"
healthcheck = "weaver --version"
```

### Health Check Patterns

```toml
# HTTP health check
[containers.api]
healthcheck = "curl -f http://localhost:8080/health"

# Database health check
[containers.database]
healthcheck = "pg_isready -U user -d db"

# Custom health check
[containers.worker]
healthcheck = "./health-check.sh"
```

## Zero Sample Detection

### The False Positive Problem

**CRITICAL**: Tests can pass without generating telemetry:

```toml
# ❌ FALSE POSITIVE: Test passes but no telemetry generated
[[steps]]
name = "test_api"
container = "api"
exec = ["curl", "http://localhost:8080/api"]
expect = { exit_code = 0 }  # Passes even without OTEL instrumentation
```

### v2.0.0 Solution: Zero Sample Validation

```toml
# ✅ TRUE POSITIVE: Validation requires telemetry
[expect.otel]
spans = [
    {
        name = "http_client_request",
        sample_count = { min = 1 }  # REQUIRE telemetry generation
    }
]
```

### Sample Count Validation

```toml
[expect.otel.spans.0]
name = "http_request"
sample_count = { min = 1, max = 10 }  # Must generate 1-10 spans

[expect.otel.metrics.0]
name = "requests_total"
sample_count = { min = 5 }  # Must generate at least 5 metrics
```

## Validation Hierarchy

```
1. Weaver Schema Validation (HIGHEST AUTHORITY)
   ├─ Runtime telemetry MUST match schemas
   ├─ Exit code 1 = BUILD FAIL
   └─ Source of truth for production readiness

2. Compilation + Type Safety (SECOND AUTHORITY)
   ├─ Code must compile
   ├─ Type-safe telemetry builders
   └─ Zero clippy warnings

3. Traditional Tests (SUPPORTING EVIDENCE)
   ├─ Can have false positives
   ├─ Not sole source of truth
   └─ Validated by Weaver
```

## Common Validation Patterns

### HTTP API Testing

```toml
[expect.otel]
spans = [
    {
        name = "http_server_request",
        kind = "server",
        attributes = {
            "http.method" = "GET",
            "http.route" = "/api/users",
            "http.status_code" = 200
        },
        sample_count = { min = 1 }
    },
    {
        name = "database_query",
        kind = "client",
        attributes = {
            "db.system" = "postgresql",
            "db.operation" = "SELECT"
        },
        sample_count = { min = 1 }
    }
]
```

### Database Operation Validation

```toml
[expect.otel]
spans = [
    {
        name = "database_query",
        attributes = {
            "db.statement" = "SELECT * FROM users WHERE id = ?",
            "db.operation" = "SELECT"
        },
        sample_count = { min = 1 }
    }
]
```

### Message Queue Processing

```toml
[expect.otel]
spans = [
    {
        name = "messaging_publish",
        attributes = {
            "messaging.system" = "rabbitmq",
            "messaging.destination" = "user_events"
        },
        sample_count = { min = 1 }
    },
    {
        name = "messaging_receive",
        attributes = {
            "messaging.system" = "rabbitmq",
            "messaging.operation" = "receive"
        },
        sample_count = { min = 1 }
    }
]
```

## Weaver Live-Check Integration

### Setup Requirements

1. **Weaver Installation**:
   ```bash
   # Install Weaver
   cargo install weaver

   # Start Weaver collector
   weaver collector --port 4318
   ```

2. **Configuration**:
   ```toml
   [otel]
   exporter = "otlp-http"
   endpoint = "http://localhost:4318"

   # Enable Weaver validation
   [expect.otel]
   registry_path = "./registry"  # Path to schema registry
   ```

### Validation Commands

```bash
# Validate with Weaver
clnrm run --validate --otel-exporter otlp-http test.clnrm.toml

# Live validation during test execution
clnrm run --live-check --registry-path ./registry test.clnrm.toml
```

## Troubleshooting

### Common Issues

**Schema validation fails:**
```bash
# Check schema registry
ls registry/

# Validate schema syntax
weaver validate registry/
```

**Zero sample count:**
```bash
# Check OTEL instrumentation
curl http://localhost:4318/v1/traces

# Verify span generation
clnrm run --otel-exporter stdout test.clnrm.toml
```

**Health check failures:**
```bash
# Check container health
docker ps

# Test health endpoints
curl http://localhost:8080/health
```

## Migration from v1.x

### v1.x OTEL Validation (Deprecated)

```toml
# OLD: TOML-based expectations (v1.x)
[otel.expect]
spans = ["http_request", "db_query"]
```

### v2.0.0 Weaver Validation (Recommended)

```toml
# NEW: Schema-based validation (v2.0.0)
[expect.otel]
spans = [
    { name = "http_server_request", sample_count = { min = 1 } },
    { name = "database_query", sample_count = { min = 1 } }
]
```

## Best Practices

### 1. Always Require Sample Count

```toml
# ✅ Good: Require telemetry generation
[expect.otel.spans.0]
name = "operation"
sample_count = { min = 1 }

# ❌ Bad: Allow zero telemetry
[expect.otel.spans.0]
name = "operation"
```

### 2. Use Specific Attributes

```toml
# ✅ Good: Specific validation
attributes = {
    "http.method" = "POST",
    "http.status_code" = 201
}

# ❌ Bad: Generic validation
attributes = {}
```

### 3. Combine with Health Checks

```toml
# ✅ Good: Health checks + validation
[containers.api]
healthcheck = "curl -f http://localhost/health"

[expect.otel]
spans = [{ name = "health_check", sample_count = { min = 1 } }]
```

### 4. Use Registry Path

```toml
# ✅ Good: Explicit registry
[expect.otel]
registry_path = "./registry"

# ❌ Bad: Default registry
[expect.otel]
# Uses default path
```

## Performance Considerations

### Validation Overhead

- **Schema loading**: Minimal (cached)
- **Validation**: Proportional to telemetry volume
- **Network**: OTLP export adds latency

### Optimization Tips

```toml
# Use batch export
[otel]
exporter = "otlp-grpc"  # More efficient than HTTP

# Limit validation scope
[expect.otel]
spans = [
    { name = "critical_operation", sample_count = { min = 1 } }
    # Don't validate every span
]
```

## Advanced Features

### Custom Schema Extensions

```yaml
# custom-schemas.yaml
- id: my_custom_span
  type: span
  attributes:
    - id: my.custom.attribute
      type: string
      brief: "Custom attribute"
```

### Conditional Validation

```toml
# Validate based on test type
[expect.otel]
spans = [
    { name = "smoke_test_span", condition = "test.tags contains 'smoke'" },
    { name = "integration_test_span", condition = "test.tags contains 'integration'" }
]
```

## Next Steps

- [TOML Schema](toml-schema.md) - Complete configuration reference
- [Error Handling](error-handling.md) - Troubleshooting validation issues
- [Migration Guide](../docs/V2_0_0_MIGRATION_GUIDE.md) - Upgrade from v1.x
- [Registry Documentation](../../registry/) - Schema registry details