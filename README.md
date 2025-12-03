# clnrm

Hermetic integration testing with Docker + OpenTelemetry validation.

## Install

```bash
cargo build --release --features otel
```

## Quick Start

```bash
# Initialize a new test
clnrm init my_test.clnrm.toml

# Run tests
clnrm run tests/

# Validate configuration
clnrm validate tests/my_test.clnrm.toml
```

## Test Format (v2.0.0)

```toml
[test]
name = "example"
timeout = "60s"

[containers.app]
image = "alpine:latest"
env = { MY_VAR = "hello" }

[[steps]]
name = "verify"
container = "app"
exec = ["sh", "-c", "echo $MY_VAR"]
assert.exit_code = 0
assert.stdout_contains = "hello"
```

**Key v2.0.0 Changes:**
- `[containers.X]` replaces `[services.X]`
- `container = "X"` in steps (required)
- `exec = [...]` replaces `command = "..."`
- Environment variables work correctly via `docker exec` semantics

See [Migration Guide](docs/V2_0_0_MIGRATION_GUIDE.md) for upgrading from v1.x.

## Features

- **Hermetic Isolation**: Each test runs in isolated Docker containers
- **Container Pooling**: 10x faster test execution via pre-warmed containers
- **OpenTelemetry**: Built-in OTEL tracing, metrics, and logging
- **Weaver Validation**: Schema-first telemetry validation
- **Parse-Time Validation**: All references validated before execution

## Documentation

- [Architecture](docs/V2_0_0_ARCHITECTURE.md) - System design and C4 diagrams
- [Config Reference](docs/V2_0_0_CONFIG_REFERENCE.md) - Complete TOML syntax
- [Migration Guide](docs/V2_0_0_MIGRATION_GUIDE.md) - v1.x to v2.0.0 migration
- [Doctest Guide](docs/DOCTEST_GUIDE.md) - Writing and running doctests

## Status

Production ready. v2.0.0 with docker exec semantics fix.
