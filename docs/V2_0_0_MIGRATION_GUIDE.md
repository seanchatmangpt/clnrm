# clnrm v2.0.0 Migration Guide

This guide helps you migrate from v1.x to v2.0.0.

## Breaking Changes Summary

| Change | v1.x | v2.0.0 |
|--------|------|--------|
| Config format | `[services.X]` | `[containers.X]` |
| Step execution | New container per step | `docker exec` into running container |
| Environment variables | Lost between steps | Persist across steps |
| Container reference | `service = "X"` | `container = "X"` |
| Metadata section | `[test.metadata]` or `[meta]` | `[test]` |

## Config Format Migration

### Before (v1.x)

```toml
[test.metadata]
name = "my_test"
description = "Test description"

[services.postgres]
type = "generic_container"
image = "postgres:15"
plugin = "generic"  # Removed in v2.0.0

[[steps]]
name = "check_db"
command = "pg_isready -U postgres"  # String format
service = "postgres"
```

### After (v2.0.0)

```toml
[test]
name = "my_test"
description = "Test description"
timeout = "60s"

[containers.postgres]
image = "postgres:15"
env = { POSTGRES_PASSWORD = "test" }
healthcheck = "pg_isready -U postgres"

[[steps]]
name = "check_db"
container = "postgres"
exec = ["pg_isready", "-U", "postgres"]  # Array format
assert.exit_code = 0
```

## Key Migration Steps

### 1. Update `[test.metadata]` to `[test]`

```diff
- [test.metadata]
+ [test]
  name = "my_test"
+ timeout = "60s"  # Add explicit timeout
```

### 2. Rename `[services.X]` to `[containers.X]`

```diff
- [services.postgres]
- type = "generic_container"
+ [containers.postgres]
  image = "postgres:15"
- plugin = "generic"  # Remove this line
```

### 3. Update step references

```diff
  [[steps]]
  name = "run_query"
- service = "postgres"
+ container = "postgres"
- command = "psql -c 'SELECT 1'"
+ exec = ["psql", "-c", "SELECT 1"]
```

### 4. Add assertions

```diff
  [[steps]]
  name = "verify_db"
  container = "postgres"
  exec = ["pg_isready", "-U", "postgres"]
+ assert.exit_code = 0
+ assert.stdout_contains = "accepting connections"
```

### 5. Add container dependencies

```diff
  [containers.app]
  image = "myapp:latest"
+ depends_on = ["postgres", "redis"]
```

## Environment Variable Fix

The most important change in v2.0.0 is the fix for environment variables.

### v1.x Behavior (Broken)

```toml
[services.alpine]
image = "alpine:latest"
environment = { MY_VAR = "hello" }

[[steps]]
name = "echo_var"
service = "alpine"
command = "echo $MY_VAR"  # MY_VAR is NOT available!
```

In v1.x, each step created a NEW container, so env vars were lost.

### v2.0.0 Behavior (Fixed)

```toml
[containers.alpine]
image = "alpine:latest"
env = { MY_VAR = "hello" }
command = ["sh", "-c", "while true; do sleep 1; done"]

[[steps]]
name = "echo_var"
container = "alpine"
exec = ["sh", "-c", "echo $MY_VAR"]  # MY_VAR = "hello"
```

In v2.0.0, steps use `docker exec` into the RUNNING container.

### Container Keepalive Pattern

For containers that need to stay running:

```toml
[containers.worker]
image = "alpine:latest"
command = ["sh", "-c", "while true; do sleep 1; done"]  # Keepalive
env = { WORKER_ID = "1" }
```

## Step-by-Step Migration Checklist

- [ ] Backup your `.clnrm.toml` files
- [ ] Update `[test.metadata]` to `[test]`
- [ ] Add `timeout` to `[test]` section
- [ ] Rename `[services.X]` to `[containers.X]`
- [ ] Remove `type = "generic_container"` lines
- [ ] Remove `plugin = "..."` lines
- [ ] Rename `environment = {...}` to `env = {...}`
- [ ] Update `service = "X"` to `container = "X"` in steps
- [ ] Convert `command = "..."` to `exec = [...]` array format
- [ ] Add assertions (`assert.exit_code`, etc.)
- [ ] Add `depends_on` for container dependencies
- [ ] Add keepalive commands if containers exit immediately
- [ ] Run `clnrm validate` to check config
- [ ] Run tests and verify behavior

## Common Migration Patterns

### Database Container

```toml
# v2.0.0 PostgreSQL pattern
[containers.postgres]
image = "postgres:15-alpine"
env = { POSTGRES_PASSWORD = "test", POSTGRES_DB = "testdb" }
ports = ["5432:5432"]
healthcheck = "pg_isready -U postgres"

[[steps]]
name = "wait_for_db"
container = "postgres"
exec = ["pg_isready", "-U", "postgres"]
assert.exit_code = 0
retry = { max_attempts = 10, delay = "1s" }

[[steps]]
name = "create_table"
container = "postgres"
exec = ["psql", "-U", "postgres", "-c", "CREATE TABLE IF NOT EXISTS test (id INT);"]
depends_on = ["wait_for_db"]
```

### Application with Dependencies

```toml
[test]
name = "integration_test"
timeout = "120s"

[containers.redis]
image = "redis:7-alpine"
healthcheck = "redis-cli ping"

[containers.app]
image = "myapp:latest"
env = { REDIS_URL = "redis://redis:6379" }
depends_on = ["redis"]

[[steps]]
name = "health_check"
container = "app"
exec = ["curl", "-f", "http://localhost:8080/health"]
assert.exit_code = 0
```

### Multi-Step Workflow

```toml
[containers.worker]
image = "alpine:latest"
command = ["sh", "-c", "while true; do sleep 1; done"]
env = { DATA_DIR = "/data" }
volumes = [{ host = "./data", container = "/data" }]

[[steps]]
name = "setup"
container = "worker"
exec = ["mkdir", "-p", "/data/output"]

[[steps]]
name = "process"
container = "worker"
exec = ["sh", "-c", "echo 'processed' > /data/output/result.txt"]
depends_on = ["setup"]

[[steps]]
name = "verify"
container = "worker"
exec = ["cat", "/data/output/result.txt"]
depends_on = ["process"]
assert.stdout_contains = "processed"
```

## Validation

After migration, validate your configs:

```bash
# Validate single file
clnrm validate tests/my_test.clnrm.toml

# Validate all files
clnrm validate tests/

# Run tests
clnrm run tests/
```

## Deprecated Features

The following v1.x features are deprecated/removed:

| Feature | Status | Replacement |
|---------|--------|-------------|
| `type = "generic_container"` | Removed | All containers are generic |
| `plugin = "..."` | Removed | Plugins auto-detected from image |
| `[test.metadata]` | Deprecated | Use `[test]` |
| `[meta]` | Deprecated | Use `[test]` |
| `service = "X"` in steps | Deprecated | Use `container = "X"` |
| String `command` | Deprecated | Use array `exec` |

## Fallback Mode

For gradual migration, clnrm supports fallback mode:

```rust
// In your test runner
use clnrm_core::config::run_test_with_fallback;

// Tries v2.0.0 Config first, falls back to legacy TestConfig
let result = run_test_with_fallback(&path).await?;
```

## Getting Help

- **Validation errors**: Run `clnrm validate --verbose` for detailed errors
- **Migration questions**: See `docs/V2_0_0_CONFIG_REFERENCE.md`
- **Issues**: https://github.com/seanchatmangpt/clnrm/issues

---

**Last Updated:** 2025-12-03
**Version:** 2.0.0
