# clnrm (Cleanroom)

The hermetic integration testing framework powered by gVisor.

[![Version](https://img.shields.io/badge/version-3.0.0-blue.svg)](CHANGELOG.md)
[![Backend](https://img.shields.io/badge/backend-gVisor-green.svg)](docs/MIGRATION_GUIDE_3.0.md)
[![License](https://img.shields.io/badge/license-MIT-lightgrey.svg)](LICENSE)

## Overview

clnrm is a high-performance testing framework designed for complex integration scenarios. It leverages multiple execution engines, including **gVisor** and **Docker**, to provide robust test environments. While gVisor provides strong isolation, **Docker is still required** for pulling images, executing CLI commands, running health checks, and for `testcontainers` backend support.

### Why clnrm?

- **Flexible Execution**: Support for gVisor sandboxes alongside standard Docker-based test environments.
- **Hybrid Dependencies**: While gVisor helps isolate tests, a local Docker daemon is still extensively used.
- **Shared State Architecture**: Uses `Arc<RwLock>` heavily for safe concurrent state management across the framework.
- **Determinism**: Sequential port allocation and predictable resource management.
- **Performance**: Optimized for fast container startup and low overhead.
- **Type-Safe**: Native Rust implementation with strong error handling.

## v3.0 Architecture State

Starting with v3.0, clnrm was intended to be gVisor-first, but **Docker remains deeply embedded** in the system's core CLI, `pull` commands, observability collectors, and health checks. `testcontainers` is fully supported behind a feature gate, and `Arc<RwLock>` is broadly used rather than pure message-passing concurrency.

See the [v3.0 Migration Guide](docs/MIGRATION_GUIDE_3.0.md) for details on upgrading.

## Quick Start

### 1. Install gVisor

Ensure `runsc` is in your PATH.

```bash
# macOS
brew install gvisor

# Linux
curl -fsSL https://gvisor.dev/install | bash
```

### 2. Install clnrm

#### Via Homebrew (macOS & Linux)

```bash
brew tap seanchatmangpt/clnrm
brew install clnrm
```

#### From Source

```bash
cargo install --path crates/clnrm-cli
```

### 3. Run Tests

```bash
# Run all tests in a directory
clnrm run tests/

# Run a specific test
clnrm run tests/my_test.clnrm.toml
```

# Validate configuration without running
clnrm validate tests/
```

## Configuration Format (v2.0.0)

clnrm uses a clean, declarative TOML format for defining test scenarios.

```toml
[test]
name = "api_integration"
description = "Tests API interaction with SurrealDB"
timeout = "60s"

[containers.db]
image = "surrealdb/surrealdb:latest"
healthcheck = "curl -f http://localhost:8000/health"

[containers.app]
image = "my-app:v1.2.3"
env = { DATABASE_URL = "http://db:8000" }
depends_on = ["db"]

[[steps]]
name = "check_db"
container = "db"
exec = ["/surreal", "version"]
assert.exit_code = 0

[[steps]]
name = "run_api_test"
container = "app"
exec = ["npm", "test"]
depends_on = ["check_db"]
assert.exit_code = 0
assert.stdout_contains = "All tests passed"
```

## Documentation

- [v3.0 Migration Guide](docs/MIGRATION_GUIDE_3.0.md) - **Read this first!**
- [v2.0.0 Config Reference](docs/V2_0_0_CONFIG_REFERENCE.md)
- [gVisor Integration Details](docs/GVISOR_README.md)
- [Code Standards](docs/CODE_STANDARDS.md)

## Code Standards

This project follows strict standards to eliminate Mura (inconsistency):

This project follows strict code standards to eliminate Mura (inconsistency):

- **Zero `unwrap()`**: All errors must be handled explicitly.
- **Result-Driven**: Use `CleanroomError` for all fallible operations.
- **High Coverage**: Minimum 80% test coverage required.
- **Idiomatic Rust**: Adherence to `cargo clippy` and `cargo fmt`.

## License

MIT - See [LICENSE](LICENSE) for details.
