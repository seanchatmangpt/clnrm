# clnrm (Cleanroom)

The hermetic integration testing framework powered by gVisor.

[![Version](https://img.shields.io/badge/version-3.0.0-blue.svg)](CHANGELOG.md)
[![Backend](https://img.shields.io/badge/backend-gVisor-green.svg)](docs/MIGRATION_GUIDE_3.0.md)
[![License](https://img.shields.io/badge/license-MIT-lightgrey.svg)](LICENSE)

## Overview

clnrm is a high-performance, hermetic testing framework designed for complex integration scenarios. It leverages **gVisor** as its primary execution engine to provide strong isolation, deterministic execution, and zero-dependency environments (no Docker required).

### Why clnrm?

- **Hermeticity**: Every test run is isolated in a fresh gVisor sandbox.
- **Zero Docker**: No Docker daemon dependency. Direct OCI image execution via `runsc`.
- **Determinism**: Sequential port allocation and predictable resource management.
- **Performance**: Optimized for fast container startup and low overhead.
- **Type-Safe**: Native Rust implementation with strong error handling.

## v3.0 gVisor-First Architecture

Starting with v3.0, clnrm has moved to a **gVisor-first architecture**. gVisor is now the default and only supported backend for production-grade isolation. Legacy `testcontainers` support is deprecated and available only via optional feature gates.

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

### 2. Run Tests

```bash
# Run all tests in a directory
clnrm run tests/

# Run a specific test
clnrm run tests/my_test.clnrm.toml

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
