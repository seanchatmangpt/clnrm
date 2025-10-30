# TOML Schema Reference

This chapter provides comprehensive documentation for the clnrm TOML configuration schema, including all supported fields, types, and validation rules.

## Schema Overview

clnrm configuration files use TOML format with the following top-level sections:

```toml
[test.metadata]           # Test identification and metadata
[services.*]              # Service definitions
[[steps]]                 # Test execution steps
[vars]                    # Template variables
[otel]                    # OpenTelemetry configuration
[expect.*]                # Validation expectations
[determinism]             # Deterministic execution settings
[report]                  # Report generation settings
[chaos]                   # Chaos engineering settings
[performance]             # Performance testing settings
```

## Test Metadata

### `[test.metadata]`

Defines test identification and basic properties.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | Test name (used for identification) |
| `description` | String | No | Human-readable test description |
| `version` | String | No | Test version (semver format) |
| `tags` | Array<String> | No | Test tags for categorization |
| `author` | String | No | Test author |
| `timeout_minutes` | Integer | No | Test timeout in minutes |

**Example:**

```toml
[test.metadata]
name = "api_integration_test"
description = "Test API integration with database"
version = "1.0.0"
tags = ["api", "integration", "database"]
author = "test-team@company.com"
timeout_minutes = 30
```

## Service Definitions

### `[services.<service_name>]`

Defines services used in the test.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `type` | String | Yes | Service type (`generic_container`, `external`) |
| `image` | String | No* | Docker image name:tag |
| `ports` | Array<Integer> | No | Port mappings |
| `env_vars` | Table | No | Environment variables |
| `volumes` | Array<String> | No | Volume mounts |
| `depends_on` | Array<String> | No | Service dependencies |
| `command` | Array<String> | No | Container command override |
| `working_directory` | String | No | Working directory in container |
| `user` | String | No | User to run as in container |
| `networks` | Array<String> | No | Networks to connect to |

*Required for `generic_container` type

**Example:**

```toml
[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [80, 443]
env_vars = { "API_KEY" = "secret", "LOG_LEVEL" = "info" }
volumes = ["/host/data:/container/data:ro"]
depends_on = ["database"]
command = ["nginx", "-g", "daemon off;"]
working_directory = "/app"
user = "nginx"
networks = ["test-network"]

[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]
env_vars = {
    "POSTGRES_DB" = "testdb",
    "POSTGRES_USER" = "testuser",
    "POSTGRES_PASSWORD" = "testpass"
}
```

## Test Steps

### `[[steps]]`

Defines individual test execution steps.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | Step name |
| `description` | String | No | Step description |
| `command` | Array<String> | Yes | Command to execute |
| `service` | String | No | Service to run command in |
| `working_directory` | String | No | Working directory for command |
| `environment` | Table | No | Environment variables for step |
| `expected_output_regex` | String | No | Expected output pattern |
| `expected_exit_code` | Integer | No | Expected exit code |
| `timeout_seconds` | Integer | No | Step timeout |
| `retry_count` | Integer | No | Number of retries |
| `retry_delay_seconds` | Integer | No | Delay between retries |

**Example:**

```toml
[[steps]]
name = "start_api"
description = "Start the API service"
command = ["echo", "Starting API server"]
service = "api"
expected_output_regex = "Starting API.*"

[[steps]]
name = "health_check"
description = "Check API health endpoint"
command = ["curl", "-f", "http://localhost:80/health"]
service = "api"
expected_output_regex = ".*healthy.*"
timeout_seconds = 10

[[steps]]
name = "integration_test"
description = "Run integration test"
command = ["./test-integration.sh"]
expected_exit_code = 0
timeout_seconds = 300
```

## Template Variables

### `[vars]`

Defines template variables for dynamic configuration.

| Field | Type | Description |
|-------|------|-------------|
| `<variable_name>` | Any | Variable value (string, number, boolean, object, array) |

**Example:**

```toml
[vars]
service_name = "api"
environment = "production"
port = 8080
timeout_seconds = 30

# Complex variables
services = [
    { name = "api", port = 80 },
    { name = "database", port = 5432 }
]

config = {
    api = { host = "localhost", port = 8080 },
    database = { host = "db", port = 5432 }
}
```

## OpenTelemetry Configuration

### `[otel]`

Configures OpenTelemetry tracing and observability.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | Boolean | No | Enable OTEL tracing |
| `endpoint` | String | No | OTLP endpoint URL |
| `exporter` | String | No | Exporter type (`otlp`, `stdout`, `jaeger`) |
| `protocol` | String | No | Protocol (`http/protobuf`, `grpc`) |
| `sample_ratio` | Float | No | Sampling ratio (0.0-1.0) |
| `resources` | Table | No | Resource attributes |
| `headers` | Table | No | HTTP headers |

**Example:**

```toml
[otel]
enabled = true
endpoint = "http://localhost:4318"
exporter = "otlp"
protocol = "http/protobuf"
sample_ratio = 1.0

[otel.resources]
"service.name" = "clnrm"
"service.version" = "1.0.0"
"env" = "test"

[otel.headers]
"authorization" = "Bearer token123"
```

## Validation Expectations

### `[expect.span]`

Validates expected OTEL spans.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | Span name pattern |
| `kind` | String | No | Span kind (`internal`, `server`, `client`) |
| `parent` | String | No | Parent span name |
| `attrs.all` | Table | No | Required attributes |
| `attrs.any` | Table | No | Attributes that must match |
| `attrs.regex` | Table | No | Regex attribute patterns |

**Example:**

```toml
[[expect.span]]
name = "api.request"
kind = "server"
attrs.all = {
    "http.method" = "GET",
    "http.route" = "/api/users",
    "http.status_code" = "200"
}

[[expect.span]]
name = "db.query"
kind = "client"
parent = "api.request"
attrs.regex = {
    "db.table" = "users.*",
    "db.operation" = "SELECT"
}
```

### `[expect.count]`

Validates span counts.

| Field | Type | Description |
|-------|------|-------------|
| `by_kind.<kind>` | Table | Count by span kind |
| `by_name.<name>` | Table | Count by span name |

**Table Fields:**

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `min` | Integer | No | Minimum count |
| `max` | Integer | No | Maximum count |

**Example:**

```toml
[expect.count]
by_kind.server = { min = 1, max = 1 }
by_kind.client = { min = 2, max = 2 }
by_name."api.request" = { min = 1, max = 1 }
```

### `[expect.order]`

Validates span execution order.

| Field | Type | Description |
|-------|------|-------------|
| `must_precede` | Array<Array<String>> | Must-precede relationships |

**Example:**

```toml
[expect.order]
must_precede = [
    ["api.request", "db.query"],
    ["db.query", "api.response"]
]
```

### `[expect.window]`

Validates temporal constraints.

| Field | Type | Description |
|-------|------|-------------|
| `start_span` | String | Start span name |
| `end_span` | String | End span name |
| `min_duration_ms` | Integer | Minimum duration |
| `max_duration_ms` | Integer | Maximum duration |

**Example:**

```toml
[expect.window]
start_span = "api.request"
end_span = "api.response"
min_duration_ms = 100
max_duration_ms = 1000
```

### `[expect.graph]`

Validates trace graph structure.

| Field | Type | Description |
|-------|------|-------------|
| `must_include` | Array<Array<String>> | Required span relationships |
| `acyclic` | Boolean | Graph must be acyclic |
| `max_depth` | Integer | Maximum trace depth |

**Example:**

```toml
[expect.graph]
must_include = [
    ["clnrm.run", "api.request"],
    ["api.request", "db.query"],
    ["db.query", "api.response"]
]
acyclic = true
max_depth = 4
```

### `[expect.hermeticity]`

Validates hermetic isolation.

| Field | Type | Description |
|-------|------|-------------|
| `no_external_services` | Boolean | No external service calls |
| `resource_attrs.must_match` | Table | Required resource attributes |
| `resource_attrs.must_not_match` | Table | Forbidden resource attributes |

**Example:**

```toml
[expect.hermeticity]
no_external_services = true
resource_attrs.must_match = {
    "service.name" = "clnrm",
    "env" = "test"
}
```

## Determinism Settings

### `[determinism]`

Configures deterministic execution.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `seed` | Integer | No | Random seed for deterministic execution |
| `freeze_clock` | String | No | Freeze clock to specific time (RFC3339) |
| `deterministic_ports` | Boolean | No | Use deterministic port allocation |
| `deterministic_volumes` | Boolean | No | Use deterministic volume paths |

**Example:**

```toml
[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"
deterministic_ports = true
deterministic_volumes = true
```

## Report Configuration

### `[report]`

Configures test report generation.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `format` | String | No | Report format (`html`, `json`, `junit`, `markdown`) |
| `output` | String | No | Output file path |
| `include_spans` | Boolean | No | Include span details in report |
| `include_metrics` | Boolean | No | Include performance metrics |

**Example:**

```toml
[report]
format = "html"
output = "test-report.html"
include_spans = true
include_metrics = true
```

## Chaos Engineering

### `[chaos]`

Configures chaos engineering experiments.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | Boolean | No | Enable chaos experiments |
| `experiment` | String | No | Experiment type |
| `duration_seconds` | Integer | No | Experiment duration |

**Example:**

```toml
[chaos]
enabled = true
experiment = "network_latency"
duration_seconds = 60
```

### `[chaos.network_latency]`

Network latency injection settings.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `target_service` | String | Yes | Target service name |
| `latency_ms` | Integer | Yes | Latency to inject (ms) |
| `duration_seconds` | Integer | No | Injection duration |

**Example:**

```toml
[chaos.network_latency]
target_service = "api"
latency_ms = 1000
duration_seconds = 30
```

### `[chaos.container_kill]`

Container failure injection settings.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `target_service` | String | Yes | Target service name |
| `timing` | String | No | When to inject failure |
| `count` | Integer | No | Number of failures |

**Example:**

```toml
[chaos.container_kill]
target_service = "database"
timing = "after_steady_state_30s"
count = 1
```

## Performance Testing

### `[performance]`

Configures performance testing.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `baseline_name` | String | No | Baseline name for comparison |
| `regression_detection` | Boolean | No | Enable regression detection |
| `sample_size` | Integer | No | Sample size for statistical analysis |

**Example:**

```toml
[performance]
baseline_name = "v1_0_1"
regression_detection = true
sample_size = 1000
```

### `[performance.metrics]`

Performance metrics to collect.

| Field | Type | Description |
|-------|------|-------------|
| `p95_latency_ms` | Integer | 95th percentile latency |
| `p99_latency_ms` | Integer | 99th percentile latency |
| `throughput_rps` | Integer | Requests per second |
| `error_rate_percent` | Float | Error rate percentage |

**Example:**

```toml
[performance.metrics]
p95_latency_ms = 100
p99_latency_ms = 200
throughput_rps = 1000
error_rate_percent = 0.1
```

## Complete Example

Here's a complete example showing all schema sections:

```toml
[test.metadata]
name = "complete_example_test"
description = "Complete TOML schema example"
version = "1.0.0"
tags = ["example", "complete"]
author = "test-team@company.com"
timeout_minutes = 30

# Template variables
[vars]
service_name = "api"
environment = "test"
port = 8080

# Service definitions
[services.api]
type = "generic_container"
image = "nginx:alpine"
ports = [{{ port }}]
env_vars = { "SERVICE_NAME" = "{{ service_name }}" }

[services.database]
type = "generic_container"
image = "postgres:15-alpine"
ports = [5432]
env_vars = {
    "POSTGRES_DB" = "testdb",
    "POSTGRES_USER" = "testuser",
    "POSTGRES_PASSWORD" = "testpass"
}

# Test steps
[[steps]]
name = "api_start"
description = "Start API service"
command = ["echo", "Starting {{ service_name }}"]
service = "{{ service_name }}"
expected_output_regex = "Starting.*"

[[steps]]
name = "health_check"
description = "Check API health"
command = ["curl", "-f", "http://localhost:{{ port }}/health"]
service = "{{ service_name }}"
expected_output_regex = ".*healthy.*"

# OTEL configuration
[otel]
enabled = true
endpoint = "http://localhost:4318"
exporter = "stdout"
protocol = "http/protobuf"
sample_ratio = 1.0

[otel.resources]
"service.name" = "clnrm"
"service.version" = "1.0.0"
"env" = "{{ environment }}"

# Span validation
[[expect.span]]
name = "clnrm.run"
kind = "internal"
attrs.all = { "result" = "pass" }

[[expect.span]]
name = "api.start"
kind = "internal"

[[expect.span]]
name = "api.exec"
kind = "internal"

# Count validation
[expect.count]
by_kind.internal = { min = 2, max = 2 }

# Order validation
[expect.order]
must_precede = [
    ["api.start", "api.exec"],
    ["api.exec", "clnrm.run"]
]

# Hermeticity validation
[expect.hermeticity]
no_external_services = true
resource_attrs.must_match = {
    "service.name" = "clnrm",
    "env" = "{{ environment }}"
}

# Determinism
[determinism]
seed = 42
freeze_clock = "2025-01-01T00:00:00Z"

# Report configuration
[report]
format = "html"
output = "test-report.html"
include_spans = true
```

## Validation Rules

### Required Fields

- `test.metadata.name` - Must be present and non-empty
- `services.*.type` - Must be present for each service
- `steps.*.name` - Must be present for each step
- `steps.*.command` - Must be present for each step

### Field Types

- **String**: Text values, may contain template variables
- **Integer**: Whole numbers (ports, timeouts, counts)
- **Float**: Decimal numbers (sample ratios, percentages)
- **Boolean**: true/false values
- **Array**: Lists of values
- **Table**: Key-value mappings

### Template Variable Usage

Template variables (`{{ variable }}`) are allowed in:
- String values
- Array elements
- Table keys and values

### Environment Variable Resolution

Environment variables are resolved in this order:
1. Template variables (`[vars]`)
2. Environment variables (system `ENV`)
3. Default values (hardcoded in resolver)

## Best Practices

### 1. Use Descriptive Names

```toml
# ✅ Good: Descriptive names
[test.metadata]
name = "user_authentication_integration_test"

[services.user_api]
type = "generic_container"
image = "user-service:latest"
```

### 2. Organize Configuration

```toml
# ✅ Good: Organized configuration
[test.metadata]
name = "test_name"

# Services
[services.database]
# Database config

[services.api]
# API config

# Test steps
[[steps]]
name = "setup"
# Setup steps

[[steps]]
name = "test"
# Test steps

[[steps]]
name = "cleanup"
# Cleanup steps
```

### 3. Use Template Variables

```toml
# ✅ Good: Template variables for reusability
[vars]
service_name = "api"
port = 8080

[services.{{ service_name }}]
port = {{ port }}

[[steps]]
name = "test_{{ service_name }}"
command = ["curl", "http://localhost:{{ port }}/health"]
```

### 4. Validate Configuration

```bash
# ✅ Good: Validate before using
clnrm validate test.toml
```

## Common Patterns

### Multi-Service Test

```toml
[test.metadata]
name = "multi_service_test"

[services.database]
type = "generic_container"
image = "postgres:15-alpine"

[services.api]
type = "generic_container"
image = "nginx:alpine"
depends_on = ["database"]

[[steps]]
name = "test_integration"
command = ["curl", "http://localhost:80/api/data"]
service = "api"
```

### Performance Test

```toml
[test.metadata]
name = "performance_test"

[services.api]
type = "generic_container"
image = "nginx:alpine"

[performance]
baseline_name = "v1_0_0"
regression_detection = true

[[steps]]
name = "load_test"
command = ["ab", "-n", "1000", "-c", "10", "http://localhost:80/"]

[expect.performance]
max_p95_latency_ms = 200
min_throughput_rps = 800
```

### Chaos Test

```toml
[test.metadata]
name = "chaos_test"

[services.api]
type = "generic_container"
image = "nginx:alpine"

[chaos]
enabled = true
experiment = "network_latency"

[chaos.network_latency]
target_service = "api"
latency_ms = 1000

[[steps]]
name = "chaos_test"
command = ["curl", "http://localhost:80/health"]

[expect.resilience]
max_response_time_ms = 2000
min_success_rate = 0.95
```

## Next Steps

Now that you understand the TOML schema:

1. **Try the examples**: Create test files using the schema examples
2. **Validate configuration**: Use `clnrm validate` to check your files
3. **Learn error handling**: Move on to [Error Handling](error-handling.md)
4. **Master advanced patterns**: Review the other chapters for advanced usage

## Further Reading

- [TOML Specification](https://toml.io/en/)
- [Configuration Management Best Practices](https://12factor.net/config)
- [Schema Validation](https://json-schema.org/)
