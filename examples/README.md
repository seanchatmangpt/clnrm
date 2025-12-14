# Cleanroom Testing Examples

This directory contains **real working examples** that demonstrate the Cleanroom testing framework. Unlike the previous examples which were non-functional, these examples are designed to compile, run, and demonstrate actual capabilities.

## Quick Start

```bash
# Run a simple TOML-based test
clnrm run examples/simple-working-test.clnrm.toml

# Run the CLI usage demo
./examples/cli-usage-demo.sh

# Compile and run a Rust example
rustc examples/simple-working-rust-example.rs --extern clnrm_core=target/debug/deps/libclnrm_core-*.rlib -L target/debug/deps
./simple-working-rust-example
```

## Examples Overview

### TOML Configuration Examples

#### `simple-working-test.clnrm.toml`
- **Purpose**: Minimal working test demonstrating basic container execution
- **Features**: Single container, simple command execution, output assertion
- **Use case**: Hello world style test to verify basic functionality

#### `comprehensive-working-test.clnrm.toml`
- **Purpose**: Multi-container test with environment variables and networking
- **Features**: Multiple containers, port mapping, environment variables, health checks
- **Use case**: Integration testing with API server and database

### Rust API Examples

#### `simple-working-rust-example.rs`
- **Purpose**: Basic usage of the CleanroomEnvironment API
- **Features**: Environment creation, health checks, metrics collection, test execution
- **Use case**: Programmatic testing with direct API access

#### `advanced-rust-example.rs`
- **Purpose**: Advanced API usage with custom service plugins
- **Features**: Custom service plugins, multiple test execution, health monitoring
- **Use case**: Extending the framework with custom services

### CLI Usage Examples

#### `cli-usage-demo.sh`
- **Purpose**: Interactive demo of CLI commands
- **Features**: Command examples, help text, validation
- **Use case**: Learning the CLI interface

## Key Concepts Demonstrated

### Container Lifecycle Management
- Automatic container startup and cleanup
- Health checking and service dependencies
- Environment variable injection

### Declarative Configuration
- TOML-based test definitions
- Container networking and port mapping
- Assertion-based validation

### Programmatic API
- Direct Rust API access
- Custom service plugins
- Metrics and observability

### Error Handling
- Proper Result types throughout
- No unwrap/expect in production code
- Actionable error messages

## Running Examples

### Prerequisites
- Docker must be installed and running
- Rust toolchain installed
- Framework compiled (`cargo build`)

### TOML Tests
```bash
clnrm run examples/simple-working-test.clnrm.toml
```

### Rust Examples
```bash
# Compile with framework dependencies
rustc examples/simple-working-rust-example.rs \
  --extern clnrm_core=target/debug/deps/libclnrm_core-*.rlib \
  -L target/debug/deps

# Run the compiled example
./simple-working-rust-example
```

### CLI Demo
```bash
./examples/cli-usage-demo.sh
```

## Framework Architecture

These examples demonstrate the core Cleanroom architecture:

- **CleanroomEnvironment**: Main API for test execution
- **Service Plugins**: Extensible service architecture
- **Container Registry**: Automatic container reuse and lifecycle management
- **Metrics Collection**: Built-in observability and performance tracking
- **Hermetic Isolation**: Unique session IDs and isolated test environments

## Contributing

When adding new examples:
- Ensure they compile and run successfully
- Follow the established patterns
- Include clear documentation
- Test with different configurations
- Demonstrate real framework capabilities

## Previous Examples

The previous examples in this directory were non-functional "lies" that claimed to demonstrate features but didn't actually work. These have been replaced with real, working examples that users can copy, modify, and rely upon.
