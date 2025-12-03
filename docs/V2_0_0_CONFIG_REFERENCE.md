# clnrm v2.0.0 Configuration Reference

Complete reference for the canonical TOML configuration format.

## Overview

clnrm uses TOML files (`.clnrm.toml`) to define tests. The v2.0.0 format has three main sections:

1. `[test]` - Test metadata
2. `[containers.X]` - Container definitions
3. `[[steps]]` - Execution steps

## Minimal Example

```toml
[test]
name = "minimal_test"

[containers.alpine]
image = "alpine:latest"

[[steps]]
name = "hello"
container = "alpine"
exec = ["echo", "Hello, World!"]
```

## Complete Example

```toml
[test]
name = "complete_example"
description = "Demonstrates all configuration options"
timeout = "300s"
parallel = 4

[containers.postgres]
image = "postgres:15-alpine"
env = { POSTGRES_PASSWORD = "secret", POSTGRES_DB = "testdb" }
ports = ["5432:5432"]
healthcheck = "pg_isready -U postgres"
command = ["postgres", "-c", "log_statement=all"]
workdir = "/var/lib/postgresql/data"

[containers.app]
image = "myapp:latest"
env = { DATABASE_URL = "postgres://postgres:secret@postgres:5432/testdb" }
depends_on = ["postgres"]
volumes = [
    { host = "./config", container = "/app/config", readonly = true }
]

[[steps]]
name = "wait_for_db"
container = "postgres"
exec = ["pg_isready", "-U", "postgres"]
assert.exit_code = 0
retry = { max_attempts = 10, delay = "1s" }

[[steps]]
name = "run_migrations"
container = "app"
exec = ["./migrate.sh"]
depends_on = ["wait_for_db"]
assert.exit_code = 0

[[steps]]
name = "run_tests"
container = "app"
exec = ["./run_tests.sh"]
depends_on = ["run_migrations"]
assert.exit_code = 0
assert.stdout_contains = "All tests passed"
```

## Section Reference

### `[test]` - Test Metadata

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | Test name (used in reports) |
| `description` | String | No | Human-readable description |
| `timeout` | String | No | Test timeout (e.g., "60s", "5m") |
| `parallel` | Integer | No | Max parallel step execution |

```toml
[test]
name = "integration_test"
description = "Tests API integration with database"
timeout = "120s"
parallel = 2
```

### `[containers.X]` - Container Definitions

Each container is defined under `[containers.<name>]`.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `image` | String | Yes | Docker image (e.g., "alpine:latest") |
| `env` | Table | No | Environment variables |
| `ports` | Array | No | Port mappings |
| `volumes` | Array | No | Volume mounts |
| `healthcheck` | String | No | Health check command |
| `depends_on` | Array | No | Container dependencies |
| `command` | Array | No | Override container command |
| `workdir` | String | No | Working directory |

#### Image

```toml
[containers.app]
image = "myapp:1.2.3"  # Specific version
# image = "myapp:latest"  # Latest tag
# image = "ghcr.io/org/myapp:v1"  # Registry path
```

#### Environment Variables

```toml
[containers.app]
image = "myapp:latest"
env = {
    DATABASE_URL = "postgres://localhost:5432/db",
    API_KEY = "secret",
    DEBUG = "true"
}
```

Or expanded syntax:

```toml
[containers.app.env]
DATABASE_URL = "postgres://localhost:5432/db"
API_KEY = "secret"
DEBUG = "true"
```

#### Port Mappings

```toml
[containers.app]
image = "myapp:latest"
ports = [
    "8080:8080",      # host:container
    "9090",           # Same port on both
    "127.0.0.1:3000:3000"  # Bind to specific interface
]
```

#### Volume Mounts

```toml
[containers.app]
image = "myapp:latest"
volumes = [
    { host = "./data", container = "/app/data" },
    { host = "./config.json", container = "/app/config.json", readonly = true }
]
```

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `host` | String | Yes | Host path |
| `container` | String | Yes | Container path |
| `readonly` | Boolean | No | Mount as read-only (default: false) |

#### Health Check

```toml
[containers.postgres]
image = "postgres:15"
healthcheck = "pg_isready -U postgres"

[containers.redis]
image = "redis:7"
healthcheck = "redis-cli ping"

[containers.http_app]
image = "myapp:latest"
healthcheck = "curl -f http://localhost:8080/health"
```

#### Dependencies

```toml
[containers.redis]
image = "redis:7-alpine"

[containers.postgres]
image = "postgres:15-alpine"

[containers.app]
image = "myapp:latest"
depends_on = ["redis", "postgres"]  # Start redis and postgres first
```

#### Command Override

```toml
[containers.worker]
image = "alpine:latest"
command = ["sh", "-c", "while true; do sleep 1; done"]  # Keepalive
```

### `[[steps]]` - Execution Steps

Steps are defined as an array using `[[steps]]`.

| Field | Type | Required | Description |
|-------|------|----------|-------------|
| `name` | String | Yes | Step name |
| `container` | String | Yes | Target container |
| `exec` | Array | Yes | Command to execute |
| `depends_on` | Array | No | Step dependencies |
| `assert` | Table | No | Assertions |
| `retry` | Table | No | Retry configuration |

#### Basic Step

```toml
[[steps]]
name = "run_test"
container = "app"
exec = ["pytest", "-v", "tests/"]
```

#### Step Dependencies

```toml
[[steps]]
name = "setup"
container = "app"
exec = ["./setup.sh"]

[[steps]]
name = "test"
container = "app"
exec = ["./test.sh"]
depends_on = ["setup"]  # Run after setup

[[steps]]
name = "cleanup"
container = "app"
exec = ["./cleanup.sh"]
depends_on = ["test"]  # Run after test
```

#### Assertions

```toml
[[steps]]
name = "verify"
container = "app"
exec = ["./check.sh"]
assert.exit_code = 0
assert.stdout_contains = "SUCCESS"
assert.stderr_is_empty = true
```

| Assertion | Type | Description |
|-----------|------|-------------|
| `exit_code` | Integer | Expected exit code |
| `stdout_contains` | String | Stdout must contain this string |
| `stdout_matches` | String | Stdout must match this regex |
| `stderr_contains` | String | Stderr must contain this string |
| `stderr_is_empty` | Boolean | Stderr must be empty |
| `timeout` | String | Step-specific timeout |

#### Retry Configuration

```toml
[[steps]]
name = "wait_for_service"
container = "app"
exec = ["curl", "-f", "http://localhost:8080/health"]
retry = { max_attempts = 10, delay = "2s", backoff = "exponential" }
```

| Field | Type | Default | Description |
|-------|------|---------|-------------|
| `max_attempts` | Integer | 1 | Maximum retry attempts |
| `delay` | String | "1s" | Delay between retries |
| `backoff` | String | "constant" | Backoff strategy: "constant", "linear", "exponential" |

## Duration Format

Durations use human-readable format:

| Format | Meaning |
|--------|---------|
| `"30s"` | 30 seconds |
| `"5m"` | 5 minutes |
| `"1h"` | 1 hour |
| `"1m30s"` | 1 minute 30 seconds |

## Environment Variable Expansion

Environment variables from the host can be used:

```toml
[containers.app]
image = "myapp:latest"
env = {
    API_KEY = "${API_KEY}",           # From host env
    VERSION = "${VERSION:-1.0.0}"     # With default
}
```

## Templates (Advanced)

clnrm supports Tera templates for dynamic configuration:

```toml
[test]
name = "test_{{ env.ENVIRONMENT }}"

[containers.app]
image = "myapp:{{ vars.version }}"

[vars]
version = "1.2.3"
```

## Validation

Configs are validated at parse time:

1. **Container references**: `container = "X"` must exist in `[containers.X]`
2. **Step dependencies**: `depends_on = ["X"]` must reference existing steps
3. **Required fields**: `name`, `image`, `exec` are required
4. **Format validation**: Durations, ports, etc. must be valid

Run validation:

```bash
clnrm validate path/to/test.clnrm.toml
```

## Best Practices

### 1. Use Explicit Timeouts

```toml
[test]
timeout = "60s"  # Always set a timeout
```

### 2. Add Health Checks

```toml
[containers.postgres]
image = "postgres:15"
healthcheck = "pg_isready -U postgres"  # Ensure container is ready
```

### 3. Use Step Dependencies

```toml
[[steps]]
name = "migrate"
depends_on = ["wait_for_db"]  # Explicit ordering
```

### 4. Add Assertions

```toml
[[steps]]
name = "test"
exec = ["./test.sh"]
assert.exit_code = 0  # Verify success
```

### 5. Pin Image Versions

```toml
[containers.postgres]
image = "postgres:15.4-alpine"  # Specific version, not :latest
```

## Schema (JSON Schema)

For editor integration, see the JSON schema at:
`registry/schemas/clnrm-config.json`

---

**Last Updated:** 2025-12-03
**Version:** 2.0.0
