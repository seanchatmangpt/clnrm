# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.3.0] - 2025-01-15

### Added
- **Container Command Execution**: Full implementation of `execute_in_container()` method with proper error handling and observability
- **TOML Test Execution Pipeline**: Complete CLI integration for running tests from TOML configuration files
- **Service Plugin System**: 
  - `GenericContainerPlugin` for running any Docker image with environment variables and port mapping
  - `SurrealDbPlugin` for SurrealDB database services with WebSocket support
- **Parallel Test Execution**: `--parallel` flag for running multiple tests concurrently
- **CLI Commands**:
  - `clnrm run <files>` - Execute TOML test files
  - `clnrm validate <files>` - Validate TOML configuration syntax
  - `clnrm plugins` - List available service plugins
  - `clnrm self-test` - Run framework self-tests
  - `clnrm init <name>` - Initialize new test projects
- **Framework Self-Testing**: Comprehensive self-test suite demonstrating "eat your own dog food" philosophy
- **Regex Validation**: Pattern matching for container output validation
- **Structured Logging**: Proper tracing integration with configurable verbosity levels
- **Error Handling**: Comprehensive error types with context and source information

### Fixed
- **Runtime Conflicts**: Resolved tokio runtime conflicts in container execution using `spawn_blocking`
- **False Positives**: Eliminated fake `Ok(())` returns from incomplete implementations
- **Logging Setup**: Fixed tracing integration in CLI for proper output visibility
- **Backend Compatibility**: Made backend trait `dyn` compatible for proper abstraction

### Changed
- **Backend Architecture**: Switched from `Box<dyn Backend>` to `Arc<dyn Backend>` for better concurrency
- **Container Execution**: Each command now creates a fresh container for maximum isolation
- **Error Messages**: Improved error messages with context and source information
- **CLI Output**: Enhanced CLI output with structured logging and progress indicators

### Removed
- **Mock Implementations**: Removed fake service plugins that returned success without doing work
- **Unused Code**: Cleaned up dead code and unused imports
- **False Claims**: Removed documentation claims for unimplemented features

### Security
- **Input Validation**: Proper validation of TOML configuration files
- **Container Isolation**: Each test runs in a fresh, isolated container
- **Error Sanitization**: Error messages don't leak sensitive internal information

### Performance
- **Parallel Execution**: Tests can run concurrently for improved performance
- **Container Reuse Infrastructure**: Foundation for future container reuse optimization
- **Efficient Backend**: Optimized container startup and command execution

### Documentation
- **API Documentation**: Comprehensive documentation for all public APIs
- **Examples**: Working examples demonstrating framework capabilities
- **Self-Testing Guide**: Documentation on how the framework tests itself

## [0.2.0] - 2024-12-01

### Added
- Initial CLI structure with clap integration
- Basic TOML configuration parsing
- Testcontainers backend foundation
- Service plugin architecture design
- Observability framework with OpenTelemetry integration

### Changed
- Project structure reorganization
- Dependency management improvements

## [0.1.0] - 2024-11-01

### Added
- Initial project setup
- Basic crate structure
- Core type definitions
- Initial documentation

---

## Release Notes for v0.3.0

This release represents a major milestone in the Cleanroom Testing Framework development. v0.3.0 delivers a fully functional testing framework with:

### 🎯 **Core Functionality**
- **Working CLI**: Complete command-line interface for test execution
- **TOML Configuration**: Declarative test definitions without code
- **Container Execution**: Real containerized test execution with Docker
- **Parallel Testing**: Concurrent test execution for performance

### 🔧 **Service Plugins**
- **GenericContainerPlugin**: Run any Docker image with custom configuration
- **SurrealDbPlugin**: Database service with WebSocket support
- **Extensible Architecture**: Easy to add new service types

### 🧪 **Framework Self-Testing**
- **"Eat Your Own Dog Food"**: The framework tests itself
- **Comprehensive Validation**: All core features are self-validated
- **Real Examples**: Working examples users can copy and use

### 🚀 **Production Ready**
- **Error Handling**: Comprehensive error types with context
- **Observability**: Built-in tracing and metrics collection
- **Documentation**: Complete API documentation and examples
- **Testing**: Extensive test coverage including integration tests

### 🎉 **Key Achievements**
1. **No False Positives**: All features work as documented
2. **Real Container Execution**: Actual Docker containers, not mocks
3. **Parallel Performance**: Tests run concurrently for speed
4. **Self-Validating**: Framework proves its own capabilities
5. **Extensible**: Plugin architecture for future growth

This release establishes Cleanroom as a production-ready testing framework that delivers on its promises of hermetic, isolated, and efficient integration testing.
