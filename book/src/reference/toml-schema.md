# TOML Schema Reference (v2.0.0)

This chapter provides comprehensive documentation for the clnrm v2.0.0 TOML configuration schema, including all supported fields, types, and validation rules.

## Schema Overview

clnrm v2.0.0 configuration files use TOML format with the following top-level sections:

```toml
[test]                 # Test identification and metadata
[containers.*]         # Container definitions (BREAKING CHANGE)
[[steps]]              # Test execution steps
[vars]                 # Template variables
[otel]                 # OpenTelemetry configuration
[expect.*]             # Validation expectations
[determinism]          # Deterministic execution settings
[report]               # Report generation settings
[chaos]                # Chaos engineering settings
[performance]          # Performance testing settings
```

## v2.0.0 Breaking Changes

### Configuration Format Changes
- `[services.X]` → `[containers.X]`
- `service = "X"` → `container = "X"`
- `[test.metadata]` → `[test]`
- Removed `type = "generic_container"` field

### Execution Model Changes
- Commands execute via `docker exec` into running containers
- Environment variables persist across steps
- Container lifecycle is more predictable

## Test Metadata

### `[test]`

Defines test identification and basic properties.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | Test name (used for identification) |
| `description` | String | No | Human-readable test description |
| `version` | String | No | Test version (semver format) |
| `tags` | Array<String> | No | Test tags for categorization |
| `author` | String | No | Test author |
| `timeout` | String | No | Test timeout (e.g., "60s", "5m") |

**Example:**

```toml
[test]
name = "api_integration_test"
description = "Test API integration with database"
version = "1.0.0"
tags = ["api", "integration", "database"]
author = "test-team@company.com"
timeout = "5m"
```

## Container Definitions

### `[containers.<container_name>]`

Defines containers used in the test (BREAKING CHANGE from `[services.*]`).

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `image` | String | Yes | Docker image name:tag |
| `ports` | Array<Integer> | No | Port mappings |
| `env` | Table | No | Environment variables |
| `volumes` | Array<String> | No | Volume mounts |
| `depends_on` | Array<String> | No | Container dependencies |
| `command` | Array<String> | No | Container command override |
| `working_dir` | String | No | Working directory in container |
| `user` | String | No | User to run as in container |
| `networks` | Array<String> | No | Networks to connect to |
| `healthcheck` | String | No | Health check command |

**Example:**

```toml
[containers.api]
image = "nginx:alpine"
ports = [80, 443]
env = { "API_KEY" = "secret", "LOG_LEVEL" = "info" }
volumes = ["/host/data:/container/data:ro"]
depends_on = ["database"]
command = ["nginx", "-g", "daemon off;"]
working_dir = "/app"
user = "nginx"
networks = ["test-network"]
healthcheck = "curl -f http://localhost/health"

[containers.database]
image = "postgres:15-alpine"
ports = [5432]
env = {
    "POSTGRES_DB" = "testdb",
    "POSTGRES_USER" = "testuser",
    "POSTGRES_PASSWORD" = "testpass"
}
healthcheck = "pg_isready -U testuser"
```

## Test Steps

### `[[steps]]`

Defines individual test execution steps.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | Step name |
| `container` | String | Yes | Container to execute in (BREAKING CHANGE from `service`) |
| `exec` | Array<String> | Yes | Command to execute |
| `env` | Table | No | Step-specific environment variables |
| `working_dir` | String | No | Working directory for this step |
| `user` | String | No | User to run as for this step |
| `timeout` | String | No | Step timeout |
| `expect` | Table | No | Expected outcomes |

**Example:**

```toml
[[steps]]
name = "setup_database"
container = "database"
exec = ["psql", "-U", "testuser", "-d", "testdb", "-c", "CREATE TABLE users (id SERIAL PRIMARY KEY, name VARCHAR(255));"]
timeout = "30s"

[[steps]]
name = "start_api"
container = "api"
exec = ["nginx", "-g", "daemon off;"]
timeout = "10s"

[[steps]]
name = "test_api"
container = "api"
exec = ["curl", "-f", "http://localhost/api/health"]
expect = { exit_code = 0 }
timeout = "5s"
```

## Template Variables

### `[vars]`

Defines variables for template substitution.

```toml
[vars]
database_url = "postgresql://testuser:testpass@database:5432/testdb"
api_port = "8080"
log_level = "debug"
```

## OpenTelemetry Configuration

### `[otel]`

Configures OpenTelemetry integration.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `service_name` | String | No | Service name for traces |
| `service_version` | String | No | Service version |
| `exporter` | String | No | OTEL exporter (none, stdout, otlp-http, otlp-grpc) |
| `endpoint` | String | No | OTEL endpoint |
| `headers` | Table | No | Additional headers |

**Example:**

```toml
[otel]
service_name = "clnrm-test"
service_version = "2.0.0"
exporter = "otlp-http"
endpoint = "http://otel-collector:4318"
headers = { "authorization" = "Bearer token123" }
```

## Validation Expectations

### `[expect.<component>]`

Defines validation expectations for different components.

#### `[expect.otel]`
OpenTelemetry validation expectations.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `spans` | Array<Table> | No | Expected spans |
| `metrics` | Array<Table> | No | Expected metrics |
| `logs` | Array<Table> | No | Expected logs |

**Example:**

```toml
[expect.otel]
spans = [
    { name = "http_request", kind = "server" },
    { name = "database_query", kind = "client" }
]
metrics = [
    { name = "http_requests_total", type = "counter" }
]
```

#### `[expect.exit_codes]`
Expected exit codes for steps.

```toml
[expect.exit_codes]
"setup_database" = 0
"test_api" = 0
```

## Deterministic Execution

### `[determinism]`

Configures deterministic execution settings.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | Boolean | No | Enable deterministic execution |
| `seed` | Integer | No | Random seed for deterministic behavior |
| `isolation` | String | No | Isolation level |

**Example:**

```toml
[determinism]
enabled = true
seed = 42
isolation = "strict"
```

## Report Generation

### `[report]`

Configures report generation settings.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `format` | Array<String> | No | Report formats |
| `output_dir` | String | No | Output directory |
| `include_traces` | Boolean | No | Include OTEL traces |
| `include_metrics` | Boolean | No | Include metrics |

**Example:**

```toml
[report]
format = ["html", "json", "junit"]
output_dir = "test-reports"
include_traces = true
include_metrics = true
```

## Chaos Engineering

### `[chaos]`

Configures chaos engineering experiments.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | Boolean | No | Enable chaos experiments |
| `latency_ms` | Integer | No | Network latency injection |
| `packet_loss` | Float | No | Packet loss percentage |
| `cpu_stress` | Float | No | CPU stress level |

**Example:**

```toml
[chaos]
enabled = true
latency_ms = 100
packet_loss = 0.05
cpu_stress = 0.8
```

## Performance Testing

### `[performance]`

Configures performance testing settings.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `enabled` | Boolean | No | Enable performance testing |
| `duration` | String | No | Test duration |
| `concurrency` | Integer | No | Concurrent users |
| `ramp_up` | String | No | Ramp-up time |

**Example:**

```toml
[performance]
enabled = true
duration = "5m"
concurrency = 100
ramp_up = "30s"
```

## Migration from v1.x

### Automatic Migration Script

```bash
#!/bin/bash
# Migrate v1.x configs to v2.0.0

find . -name "*.clnrm.toml" -exec sed -i 's/\[services\./[containers./g' {} \;
find . -name "*.clnrm.toml" -exec sed -i 's/service = /container = /g' {} \;
find . -name "*.clnrm.toml" -exec sed -i 's/\[test\.metadata\]/[test]/g' {} \;
find . -name "*.clnrm.toml" -exec sed -i 's/command = /exec = /g' {} \;
find . -name "*.clnrm.toml" -exec sed -i '/type = "generic_container"/d' {} \;
```

### Manual Migration Steps

1. **Update section headers:**
   ```diff
   - [services.postgres]
   + [containers.postgres]
   ```

2. **Update step references:**
   ```diff
   - service = "postgres"
   + container = "postgres"
   ```

3. **Update metadata section:**
   ```diff
   - [test.metadata]
   + [test]
   ```

4. **Update command format:**
   ```diff
   - command = "psql -c 'SELECT 1'"
   + exec = ["psql", "-c", "SELECT 1"]
   ```

5. **Remove type fields:**
   ```diff
   - type = "generic_container"
   ```

## Validation

### Schema Validation

clnrm v2.0.0 includes comprehensive schema validation:

```bash
# Validate configuration
clnrm validate test.clnrm.toml

# Strict validation
clnrm validate --strict test.clnrm.toml
```

### Common Validation Errors

- **Missing container reference:** Steps must reference valid containers
- **Invalid image format:** Docker images must include tag
- **Circular dependencies:** Container dependencies cannot be circular
- **Invalid environment variables:** Env vars must be strings

## Examples

### Basic API Test (v2.0.0)

```toml
[test]
name = "api_health_check"
description = "Test API health endpoint"
timeout = "2m"

[containers.api]
image = "nginx:alpine"
ports = [80]
healthcheck = "curl -f http://localhost"

[[steps]]
name = "check_health"
container = "api"
exec = ["curl", "-f", "http://localhost/health"]
expect = { exit_code = 0 }
```

### Database Integration Test (v2.0.0)

```toml
[test]
name = "database_integration"
description = "Test database operations"
timeout = "5m"

[containers.database]
image = "postgres:15-alpine"
env = {
    "POSTGRES_DB" = "testdb",
    "POSTGRES_USER" = "testuser",
    "POSTGRES_PASSWORD" = "testpass"
}
ports = [5432]
healthcheck = "pg_isready -U testuser"

[containers.app]
image = "myapp:latest"
env = { "DATABASE_URL" = "postgresql://testuser:testpass@database:5432/testdb" }
depends_on = ["database"]

[[steps]]
name = "migrate_db"
container = "app"
exec = ["./migrate", "up"]
timeout = "30s"

[[steps]]
name = "run_tests"
container = "app"
exec = ["./test", "--integration"]
expect = { exit_code = 0 }
```

### Multi-Service Orchestration (v2.0.0)

```toml
[test]
name = "microservices_test"
description = "Test microservices interaction"
timeout = "10m"

[containers.auth]
image = "auth-service:v1.2.3"
ports = [3001]
env = { "JWT_SECRET" = "secret" }
healthcheck = "curl -f http://localhost:3001/health"

[containers.api]
image = "api-gateway:v2.1.0"
ports = [3000]
env = { "AUTH_URL" = "http://auth:3001" }
depends_on = ["auth"]
healthcheck = "curl -f http://localhost:3000/health"

[containers.frontend]
image = "frontend:v3.0.0"
ports = [8080]
env = { "API_URL" = "http://api:3000" }
depends_on = ["api"]

[[steps]]
name = "test_auth_flow"
container = "frontend"
exec = ["npm", "test", "--", "--testNamePattern", "auth"]
expect = { exit_code = 0 }

[[steps]]
name = "test_api_integration"
container = "frontend"
exec = ["npm", "test", "--", "--testNamePattern", "api"]
expect = { exit_code = 0 }
```

## Best Practices

### 1. Use Descriptive Names

```toml
# ✅ Good
[containers.postgres_primary]
image = "postgres:15"

[containers.redis_cache]
image = "redis:7-alpine"

# ❌ Bad
[containers.c1]
image = "postgres:15"

[containers.c2]
image = "redis:7"
```

### 2. Define Health Checks

```toml
# ✅ Good - explicit health checks
[containers.api]
image = "myapp:latest"
healthcheck = "curl -f http://localhost:8080/health"

# ❌ Bad - no health check
[containers.api]
image = "myapp:latest"
```

### 3. Use Environment Variables

```toml
# ✅ Good - environment variables
[containers.app]
image = "myapp:latest"
env = {
    "DATABASE_URL" = "postgresql://user:pass@db:5432/app",
    "REDIS_URL" = "redis://cache:6379"
}

# ❌ Bad - hardcoded values
[containers.app]
image = "myapp:latest"
env = { "DB_HOST" = "localhost" }
```

### 4. Structure Complex Tests

```toml
# ✅ Good - logical step ordering
[[steps]]
name = "setup_infrastructure"
container = "app"
exec = ["./scripts/setup.sh"]

[[steps]]
name = "run_business_logic_tests"
container = "app"
exec = ["./test", "business"]

[[steps]]
name = "run_integration_tests"
container = "app"
exec = ["./test", "integration"]

[[steps]]
name = "cleanup"
container = "app"
exec = ["./scripts/cleanup.sh"]
```

## Troubleshooting

### Common Issues

**Container not found:**
- Ensure container name matches `[containers.<name>]`
- Check for typos in step `container` references

**Environment variables not working:**
- In v2.0.0, env vars persist across steps
- Set them in container definition, not step definition

**Command execution fails:**
- Use array format for `exec`: `exec = ["command", "arg1", "arg2"]`
- Not string format: `exec = "command arg1 arg2"`

**Dependency issues:**
- Use `depends_on` to ensure startup order
- Define health checks for dependency validation

## Next Steps

- [CLI Reference](cli-reference.md) - Learn about command-line usage
- [Weaver Schemas](weaver-schemas.md) - Understand OTEL validation
- [Error Handling](error-handling.md) - Learn about troubleshooting
- [Migration Guide](../docs/V2_0_0_MIGRATION_GUIDE.md) - Complete upgrade guide