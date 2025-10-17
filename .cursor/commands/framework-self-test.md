# Framework Self-Test

## Overview
Execute the framework's comprehensive self-test suite. The cleanroom testing framework should "eat its own dog food" by testing itself using its own capabilities. This validates that the framework works correctly and catches regressions early.

## Self-Test Categories

### 1. Core Framework Validation
```bash
# Test basic framework functionality
cargo test framework_self_test

# Test TOML configuration parsing
cargo test toml_config_parsing

# Test test discovery and execution
cargo test test_discovery_execution

# Test error handling and reporting
cargo test error_handling_reporting
```

### 2. Container Integration Testing
```bash
# Test container backend functionality
cargo test container_backend_integration

# Test plugin system integration
cargo test plugin_system_integration

# Test service lifecycle management
cargo test service_lifecycle_integration
```

### 3. OpenTelemetry Integration
```bash
# Test OTEL initialization and configuration
cargo test otel_initialization

# Test span creation and propagation
cargo test span_creation_propagation

# Test OTLP export (requires collector setup)
cargo test otlp_export_integration
```

### 4. CLI Interface Testing
```bash
# Test all CLI command functionality
cargo test cli_command_integration

# Test help and version commands
cargo test cli_help_version

# Test error handling in CLI
cargo test cli_error_handling
```

### 5. Template System Validation
```bash
# Test Tera template parsing and rendering
cargo test template_parsing_rendering

# Test template variable substitution
cargo test template_variable_substitution

# Test macro library functionality
cargo test macro_library_integration
```

## Comprehensive Self-Test Execution

### Run Complete Self-Test Suite
```bash
#!/bin/bash
# scripts/run-framework-self-test.sh

echo "🚀 Starting comprehensive framework self-test..."
echo "==============================================="

# Set up test environment
export RUST_BACKTRACE=1
export RUST_LOG=debug

# Run unit tests
echo "🧪 Running unit tests..."
cargo test --lib
if [ $? -ne 0 ]; then
    echo "❌ Unit tests failed"
    exit 1
fi

# Run integration tests
echo "🔗 Running integration tests..."
cargo test --test integration
if [ $? -ne 0 ]; then
    echo "❌ Integration tests failed"
    exit 1
fi

# Run framework self-tests
echo "🏗️ Running framework self-tests..."
cargo test framework_self_test
if [ $? -ne 0 ]; then
    echo "❌ Framework self-tests failed"
    exit 1
fi

# Test CLI functionality
echo "💻 Testing CLI interface..."
cargo run -- --help > /dev/null
cargo run -- --version > /dev/null
cargo run -- init --force
cargo run -- validate cleanroom.toml

# Test configuration validation
echo "⚙️ Testing configuration validation..."
cargo run -- validate cleanroom.toml
if [ $? -ne 0 ]; then
    echo "❌ Configuration validation failed"
    exit 1
fi

# Test template generation
echo "📄 Testing template generation..."
cargo run -- template otel > /dev/null
cargo run -- template matrix > /dev/null

echo "==============================================="
echo "🎉 Framework self-test completed successfully!"
echo "==============================================="
```

## Self-Test Validation Criteria

### Functional Requirements
- [ ] **TOML parsing** works correctly for all test files
- [ ] **Test discovery** finds all test files automatically
- [ ] **Command execution** runs successfully (on host currently)
- [ ] **Output validation** matches expected patterns
- [ ] **Error handling** provides meaningful error messages

### Architecture Validation
- [ ] **Plugin system** registration and discovery works
- [ ] **Service lifecycle** management functions properly
- [ ] **Error propagation** follows established patterns
- [ ] **Async operations** use proper patterns throughout

### Quality Assurance
- [ ] **No unwrap()/expect()** in framework code paths
- [ ] **All traits dyn compatible** for plugin architecture
- [ ] **Proper error context** provided for debugging
- [ ] **No fake implementations** - honest about limitations

### Performance Validation
- [ ] **Test execution time** within acceptable bounds
- [ ] **Memory usage** doesn't grow unbounded
- [ ] **No resource leaks** in container operations
- [ ] **Efficient parsing** of TOML configurations

## Debugging Self-Test Failures

### Common Failure Patterns

#### TOML Parsing Issues
```bash
# Debug TOML parsing with detailed output
RUST_LOG=debug cargo test toml_config_parsing -v

# Check specific TOML file syntax
cargo run -- validate path/to/test.toml
```

#### Container Operation Failures
```bash
# Debug container backend issues
RUST_LOG=debug cargo test container_backend_integration -v

# Check container logs for errors
docker logs <container_id> 2>&1 || echo "No container logs available"
```

#### Async Pattern Issues
```bash
# Debug async execution problems
RUST_LOG=debug cargo test async_pattern_validation -v

# Check for proper tokio runtime usage
grep -r "Runtime::new" src/ | head -5
```

#### Plugin System Issues
```bash
# Debug plugin registration and discovery
RUST_LOG=debug cargo test plugin_system_integration -v

# Check plugin metadata and configuration
cargo run -- plugins
```

## Self-Test Artifacts

### Generated Test Reports
- **JUnit XML** - for CI/CD integration
- **JSON reports** - for detailed analysis
- **Console output** - for immediate feedback
- **Performance metrics** - for benchmarking

### Test Data and Fixtures
- **Sample TOML files** - for configuration testing
- **Test containers** - for integration testing
- **Mock services** - for plugin testing
- **Template examples** - for template validation

## Continuous Self-Testing

### CI/CD Integration
```yaml
# In .github/workflows/ci.yml
- name: Framework Self-Test
  run: |
    ./scripts/run-framework-self-test.sh

- name: Upload Test Results
  uses: actions/upload-artifact@v3
  with:
    name: test-results
    path: |
      target/test-results/
      junit.xml
```

### Local Development Workflow
```bash
# Run self-tests during development
cargo test framework_self_test

# Run specific self-test categories
cargo test container_backend_integration
cargo test otel_integration
cargo test plugin_system_integration

# Run with detailed output for debugging
RUST_LOG=debug cargo test framework_self_test -v
```

## Self-Test Success Metrics

### Reliability Metrics
- **Test pass rate** - percentage of tests passing consistently
- **Flaky test detection** - identification of intermittent failures
- **Regression detection** - early identification of breaking changes
- **Performance consistency** - stable execution times

### Quality Metrics
- **Error handling coverage** - comprehensive error scenario testing
- **Edge case validation** - boundary condition testing
- **Integration coverage** - cross-component interaction testing
- **Documentation accuracy** - alignment between docs and implementation

## Framework "Dog Food" Principle

### What This Means
The framework should use its own testing capabilities to validate itself, demonstrating that:

1. **The framework works** - can successfully test its own functionality
2. **Testing approach is sound** - if it can't test itself, how can it test others?
3. **Best practices are followed** - uses proper error handling, async patterns, etc.
4. **Documentation is accurate** - examples actually work

### Current Implementation Status
- **TOML parsing** - ✅ Working (can parse its own config files)
- **Test discovery** - ✅ Working (finds its own test files)
- **Command execution** - ✅ Working (can run its own CLI commands)
- **Error handling** - ✅ Working (uses CleanroomError properly)
- **Plugin system** - 🚧 Partial (registration works, execution incomplete)
- **Container testing** - ❌ Not working (executes on host, not in containers)

### Path to Full Self-Testing
1. **Complete container execution** - run tests in actual containers
2. **Implement plugin lifecycle** - full start/stop/cleanup cycles
3. **Add OTEL validation** - comprehensive telemetry testing
4. **Framework stress testing** - test itself under load
5. **Cross-version compatibility** - test against multiple versions

## Troubleshooting Guide

### Self-Test Environment Issues
```bash
# Check Rust toolchain version
rustc --version
cargo --version

# Verify dependencies are available
cargo check

# Check for conflicting versions
cargo tree | grep -i conflict || echo "No dependency conflicts"
```

### Test Execution Problems
```bash
# Run tests in single-threaded mode for debugging
cargo test -- --test-threads=1

# Run with backtrace for better error context
RUST_BACKTRACE=1 cargo test

# Check for resource exhaustion
ulimit -a
```

### Container-Related Issues
```bash
# Check Docker/Podman availability
docker --version || podman --version || echo "No container runtime found"

# Verify container networking
docker network ls

# Check available images
docker images | head -10
```

This self-test command ensures the framework maintains its own quality standards and serves as a living example of how to properly test complex systems.
