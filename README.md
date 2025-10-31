# Cleanroom Testing Framework

[![Version](https://img.shields.io/badge/version-1.2.1-blue.svg)](https://github.com/seanchatmangpt/clnrm)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A hermetic integration testing framework that executes tests in isolated Docker containers with OpenTelemetry validation. Define tests declaratively using TOML configuration files and validate runtime behavior with Weaver schema validation.

## Installation

### Homebrew

```bash
brew tap seanchatmangpt/clnrm
brew install clnrm
```

### Cargo

```bash
cargo install clnrm
```

### Requirements

- Rust 1.70 or later (for building from source)
- Docker or Podman (for container execution)

## Quick Example

```bash
# Initialize a new test project
clnrm init
cd tests

# Run the generated test
clnrm run basic.clnrm.toml
```

The generated test file looks like this:

```toml
[test.metadata]
name = "basic_test"
description = "Basic integration test"

[services.test_container]
type = "generic_container"
image = "alpine:latest"

[[steps]]
name = "hello_world"
command = ["echo", "Hello from cleanroom!"]
expected_output_regex = "Hello from cleanroom!"
```

Each test step executes in a fresh Docker container, providing complete isolation between test runs.

## Features

**Core Testing**
- TOML-based test definitions
- Docker container isolation per test step
- Regex pattern validation for command output
- Automatic test discovery

**OpenTelemetry Integration**
- Weaver schema validation as source of truth
- Type-safe span creation from semantic conventions
- OTLP export for telemetry collection
- Runtime behavior validation

**CLI Commands**
- `clnrm init` - Initialize new test project
- `clnrm run` - Execute test files
- `clnrm validate` - Validate TOML configuration
- `clnrm plugins` - List available service plugins
- `clnrm self-test` - Run framework self-validation

## Documentation

- [Quick Start Guide](docs/quick-start.md) - Get started in 5 minutes
- [Advanced Users Guide](book/) - Comprehensive documentation (mdbook)
- [TOML Reference](book/src/reference/toml-schema.md) - Configuration format
- [Documentation Index](docs/INDEX.md) - Complete navigation hub

## Contributing

Contributions are welcome. See [CONTRIBUTING.md](CONTRIBUTING.md) for guidelines.

## License

Licensed under the MIT License. See [LICENSE](LICENSE) for details.

---

Repository: [github.com/seanchatmangpt/clnrm](https://github.com/seanchatmangpt/clnrm)
