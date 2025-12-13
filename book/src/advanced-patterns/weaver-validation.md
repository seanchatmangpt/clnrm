# Weaver Schema Validation (v2.0.0)

This chapter covers Weaver-based telemetry validation in clnrm v2.0.0.

## Overview

**OpenTelemetry Weaver** provides schema-first validation, eliminating false positives by validating actual telemetry generation.

## Key Features

### Zero Sample Detection

**CRITICAL**: Prevents tests from passing without telemetry:

```toml
[expect.otel]
spans = [
    {
        name = "http_request",
        sample_count = { min = 1 }  # REQUIRE telemetry
    }
]
```

### Schema Validation

```toml
[expect.otel]
spans = [
    {
        name = "http_server_request",
        attributes = {
            "http.method" = "GET",
            "http.status_code" = 200
        },
        sample_count = { min = 1 }
    }
]
```

## v2.0.0 Integration

- **Live Validation**: Real-time schema checking
- **Health Checks**: Required for proper validation
- **Registry Support**: Comprehensive schema library
- **CI/CD Integration**: Automated validation gates

## Setup

```bash
# Enable Weaver validation
clnrm run --validate --otel-exporter otlp-http test.clnrm.toml
```

## Configuration

```toml
[otel]
exporter = "otlp-http"
endpoint = "http://weaver:4318"

[expect.otel]
registry_path = "./registry"
spans = [
    { name = "operation", sample_count = { min = 1 } }
]
```

## Best Practices

- Always require `sample_count = { min = 1 }`
- Use specific attribute validation
- Combine with health checks
- Test failure scenarios