# Cleanroom CLI Guide

This guide covers the complete CLI interface for the Cleanroom testing framework.

> **💡 Pro Tip:** See our [comprehensive examples](https://github.com/cleanroom-testing/clnrm/tree/main/examples) for real-world usage patterns and verification scripts that demonstrate every CLI feature.

## Installation

### Homebrew (Recommended)

```bash
# Install via custom tap (immediate availability)
brew tap seanchatmangpt/clnrm
brew install clnrm

# Verify installation
clnrm --version
# Output: clnrm 0.4.0
```

### Direct Installation

```bash
# Install the CLI tool (no Rust required)
curl -fsSL https://install.clnrm.dev | sh

# Verify installation
clnrm --version
# Output: clnrm 0.4.0
```

### Build from Source

```bash
# Clone and build (requires Rust)
git clone https://github.com/seanchatmangpt/clnrm.git
cd clnrm
cargo build --release
sudo cp target/release/clnrm /usr/local/bin/
```

## Command Reference

### `clnrm run` - Execute Tests

Run tests from TOML configuration files.

```bash
# Run a single test
clnrm run tests/user_registration.toml

# Run all tests in a directory
clnrm run tests/

# Parallel execution
clnrm run tests/ --parallel --jobs 4

# Watch mode for development
clnrm run tests/ --watch

# Interactive debugging
clnrm run tests/ --interactive

# Fail fast mode
clnrm run tests/ --fail-fast

# Multiple output formats
clnrm run tests/ --format json
clnrm run tests/ --format junit > results.xml
```

**Options:**
- `--parallel` - Run tests in parallel
- `--jobs N` - Number of parallel workers (default: 4)
- `--watch` - Rerun tests on file changes
- `--interactive` - Step through tests manually
- `--fail-fast` - Stop on first failure
- `--format FORMAT` - Output format (human, json, junit, tap)

### `clnrm validate` - Validate Configuration

Check if TOML test files are valid.

```bash
# Validate a single file
clnrm validate tests/user_registration.toml

# Validate all files in directory
clnrm validate tests/

# Batch validation with detailed output
clnrm validate tests/ --verbose
```

### `clnrm init` - Initialize Project

Create a new test project structure.

```bash
# Initialize with default name
clnrm init

# Initialize with custom name
clnrm init my-integration-tests

# Use specific template
clnrm init my-project --template advanced
```

### `clnrm services` - Service Management

Manage running services.

```bash
# Show service status
clnrm services status

# Show logs for a service
clnrm services logs database --lines 50

# Restart a service
clnrm services restart database
```

### `clnrm plugins` - List Available Plugins

Show available service plugins.

```bash
clnrm plugins

# Output:
# 📦 Available Service Plugins:
# ✅ generic_container (alpine, ubuntu, debian)
# ✅ network_tools (curl, wget)
# ✅ custom_plugins (user-defined)
```

### `clnrm report` - Generate Reports

Create test reports in various formats.

```bash
# HTML report
clnrm report tests/ --format html > report.html

# JUnit XML for CI
clnrm report tests/ --format junit > test-results.xml

# JSON report
clnrm report tests/ --format json > data.json
```

### `clnrm selftest` - Framework Self-Testing

Run built-in framework self-tests to verify functionality.

```bash
# Run all self-tests
clnrm selftest

# Run specific test suite
clnrm selftest --suite framework

# Run with detailed report
clnrm selftest --report

# Run container lifecycle tests
clnrm selftest --suite container
```

**Test suites available:**
- `framework` - Core framework functionality
- `container` - Container lifecycle management
- `plugin` - Plugin system functionality
- `cli` - CLI interface testing
- `otel` - Observability features

## Configuration

### Project Configuration (`cleanroom.toml`)

```toml
[cli]
# Default settings
parallel = true
jobs = 4
output_format = "human"
fail_fast = false

[services]
# Default service configurations
default_timeout = "30s"
health_check_interval = "5s"

[logging]
# Observability settings
enable_tracing = true
enable_metrics = true
log_level = "info"
```

### Test Configuration (TOML Files)

See the [TOML Reference](../TOML_REFERENCE.md) for complete configuration options.

## Output Formats

### Human-Readable (Default)
```
🚀 Starting test environment...
📦 Loading plugins...
🔌 Plugin 'alpine' loaded

📋 Running test 'container_lifecycle_test'

📋 Step: verify_container_startup
✅ Container started successfully (0.2s)

📋 Step: test_command_execution
🔍 Checking regex: "Command completed"
✅ Pattern found in output

🎉 Test 'container_lifecycle_test' PASSED in 1.3s
```

### JSON Format
```json
{
  "test_name": "container_lifecycle_test",
  "status": "passed",
  "duration_ms": 1300,
  "steps": [
    {
      "name": "verify_container_startup",
      "status": "passed",
      "duration_ms": 200,
      "output": "Container started successfully"
    }
  ]
}
```

### JUnit XML Format
```xml
<testsuite name="cleanroom" tests="1" failures="0" time="1.3">
  <testcase name="container_lifecycle_test" time="1.3"/>
</testsuite>
```

## CI/CD Integration

### GitHub Actions
```yaml
- name: Run Cleanroom Tests
  run: clnrm run tests/ --format junit > test-results.xml

- name: Upload Test Results
  uses: actions/upload-artifact@v3
  with:
    name: test-results
    path: test-results.xml
```

### GitLab CI
```yaml
stages:
  - test

cleanroom_tests:
  stage: test
  script:
    - clnrm run tests/ --parallel --jobs 8
  artifacts:
    reports:
      junit: test-results.xml
```

### Jenkins
```groovy
pipeline {
    stages {
        stage('Test') {
            steps {
                sh 'clnrm run tests/ --format junit > test-results.xml'
                junit 'test-results.xml'
            }
        }
    }
}
```

## Examples

### Basic Test Execution
```bash
# Initialize project
clnrm init my-tests

# Run tests
cd my-tests
clnrm run tests/

# Validate configuration
clnrm validate tests/
```

### Development Workflow
```bash
# Watch mode for development
clnrm run tests/ --watch

# Interactive debugging
clnrm run tests/ --interactive

# Parallel execution for speed
clnrm run tests/ --parallel --jobs 8
```

### CI/CD Pipeline
```bash
# Run tests and generate reports
clnrm run tests/ --parallel --jobs 8 --format junit > test-results.xml
clnrm report tests/ --format html > integration-report.html
```

## Troubleshooting

### Common Issues

**"Service not found" error:**
```bash
# Check available plugins
clnrm plugins

# Check service status
clnrm services status

# Restart services
clnrm services restart database
```

**"Configuration invalid" error:**
```bash
# Validate configuration
clnrm validate tests/my_test.toml

# Check TOML syntax
cat tests/my_test.toml
```

**"Container failed to start" error:**
```bash
# Check Docker status
docker ps

# Check service logs
clnrm services logs my_service --lines 100

# Restart service
clnrm services restart my_service
```

### Debug Mode

```bash
# Enable debug logging
clnrm run tests/ --verbose

# Interactive debugging
clnrm run tests/ --interactive

# Check service health
clnrm services status
```

## Performance Optimization

### Parallel Execution
```bash
# Run tests in parallel for speed
clnrm run tests/ --parallel --jobs 8
```

### Container Reuse
- Containers are automatically reused across tests
- First run creates containers (30-60s)
- Subsequent runs reuse containers (2-5ms)
- **10-50x performance improvement**

### Resource Management
- Automatic cleanup prevents resource leaks
- Service health monitoring prevents hung tests
- Timeout handling prevents infinite waits

## Best Practices

1. **Use descriptive test names** - Makes debugging easier
2. **Include timeout values** - Prevents hung tests
3. **Use regex patterns** - More flexible than exact string matching
4. **Validate configurations** - Catch errors early
5. **Use parallel execution** - Faster test runs
6. **Monitor service health** - Prevent flaky tests
7. **Generate reports** - Share results with team

## 📚 CLI Examples & Verification

Cleanroom provides comprehensive CLI examples that demonstrate every feature and verify all claims:

### 🚀 **Installation Verification Examples**

#### Verify CLI Installation
```bash
# Copy and run to verify CLI works
curl -fsSL https://raw.githubusercontent.com/cleanroom-testing/clnrm/main/examples/installation/verify-cli-installation.sh | bash
```

#### Complete Quick Start Demo
```bash
# Copy and run the complete quick start guide
curl -fsSL https://raw.githubusercontent.com/cleanroom-testing/clnrm/main/examples/installation/quick-start-demo.sh | bash
```

### 🎯 **CLI Feature Examples**

#### Advanced CLI Features Demo
```bash
# Demonstrates all advanced CLI features
./examples/cli-features/advanced-cli-demo.sh
```

**Features demonstrated:**
- ✅ Parallel execution with configurable jobs
- ✅ Watch mode for development workflow
- ✅ Comprehensive report generation
- ✅ Interactive debugging capabilities
- ✅ Service management commands
- ✅ Configuration validation
- ✅ JUnit XML output for CI/CD

#### Performance Verification
```bash
# Run real container reuse benchmarks
cargo run --example container-reuse-benchmark
```

**Shows actual performance improvements:**
- Container reuse statistics
- Parallel execution benefits
- Real performance measurements

### 📋 **CI/CD Integration Examples**

#### GitHub Actions Workflow
```bash
# Copy ready-to-use workflow
cp examples/ci-cd/github-actions-workflow.yml .github/workflows/
```

#### GitLab CI Pipeline
```bash
# Copy complete CI pipeline
cp examples/ci-cd/gitlab-ci-pipeline.yml .gitlab-ci.yml
```

### 🔗 **Verify All Claims**
```bash
# Run comprehensive verification of all README claims
cd examples && ./verify-all-claims.sh
```

**Verifies:**
- ✅ Installation works correctly
- ✅ Framework self-testing works
- ✅ TOML configuration works
- ✅ Performance claims are real
- ✅ Plugin system works
- ✅ Observability works
- ✅ CLI features work
- ✅ CI/CD integration works

### 💡 **Example Categories**

| Category | Files | Purpose |
|----------|-------|---------|
| **Installation** | 2 scripts | Verify CLI installation and quick start |
| **CLI Features** | 12 scripts | Demonstrate all CLI capabilities |
| **TOML Config** | 16 files | Show no-code testing examples |
| **Performance** | 5 files | Measure real performance benefits |
| **CI/CD** | 4 workflows | Ready-to-use integration examples |

**Total: 39 CLI-focused examples** that demonstrate and verify every CLI feature!

## Support

- **Documentation**: See [TOML Reference](../TOML_REFERENCE.md)
- **Examples**: See [comprehensive examples directory](../examples/)
- **Issues**: Report bugs and feature requests
- **Community**: Join discussions and get help
