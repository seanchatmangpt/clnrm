# Generate Documentation

## Overview
Generate comprehensive documentation for the cleanroom testing framework. Ensures all features, APIs, and usage patterns are properly documented following core team standards.

## Documentation Categories

### 1. API Documentation
```bash
# Generate Rust API documentation
cargo doc --no-deps --document-private-items

# Generate with examples
cargo doc --no-deps --document-private-items --examples

# Open in browser
cargo doc --no-deps --open

# Generate for specific package
cargo doc --package clnrm-core --no-deps
```

### 2. Usage Documentation
```bash
# Generate CLI help documentation
cargo run -- --help > docs/CLI_HELP.md

# Generate configuration examples
cargo run -- init --force
cp cleanroom.toml docs/examples/basic-config.toml

# Generate template examples
cargo run -- template otel > docs/examples/otel-template.toml
cargo run -- template matrix > docs/examples/matrix-template.toml
```

### 3. Architecture Documentation
```bash
# Generate module hierarchy documentation
find crates/ -name "*.rs" -type f | grep -v target | sort > docs/ARCHITECTURE.md

# Document plugin architecture
cargo run -- plugins > docs/PLUGIN_ARCHITECTURE.md

# Document error types and handling
grep -r "CleanroomError" src/ | grep -v "test" | head -20 > docs/ERROR_HANDLING.md
```

## Comprehensive Documentation Generation

### Generate Complete Documentation Suite
```bash
#!/bin/bash
# scripts/generate-complete-docs.sh

echo "📚 Generating complete documentation suite..."
echo "==========================================="

# Create docs directory structure
mkdir -p docs/api
mkdir -p docs/examples
mkdir -p docs/guides
mkdir -p docs/reference

# Generate API documentation
echo "🔧 Generating API documentation..."
cargo doc --no-deps --document-private-items
cp -r target/doc/* docs/api/

# Generate CLI documentation
echo "💻 Generating CLI documentation..."
cargo run -- --help > docs/CLI_REFERENCE.md

# Generate configuration examples
echo "⚙️ Generating configuration examples..."
cargo run -- init --force
cp cleanroom.toml docs/examples/basic-configuration.toml

# Generate template examples
echo "📄 Generating template examples..."
cargo run -- template otel > docs/examples/otel-validation.toml
cargo run -- template matrix > docs/examples/matrix-testing.toml
cargo run -- template full-validation > docs/examples/comprehensive-validation.toml

# Generate architecture documentation
echo "🏗️ Generating architecture documentation..."
find crates/ -name "*.rs" -type f | grep -v target | sort > docs/MODULE_HIERARCHY.md

# Generate error handling guide
echo "🚨 Generating error handling guide..."
grep -r "CleanroomError" src/ crates/ | grep -v "test" | head -30 > docs/ERROR_PATTERNS.md

# Generate testing patterns
echo "🧪 Generating testing patterns..."
grep -r "#\[tokio::test\]" tests/ src/ | wc -l > docs/TESTING_PATTERNS.md
grep -r "// Arrange" tests/ | wc -l >> docs/TESTING_PATTERNS.md
grep -r "// Act" tests/ | wc -l >> docs/TESTING_PATTERNS.md
grep -r "// Assert" tests/ | wc -l >> docs/TESTING_PATTERNS.md

# Generate best practices summary
echo "🎯 Generating best practices summary..."
cp .cursorrules docs/BEST_PRACTICES.md

# Generate changelog
echo "📝 Generating changelog..."
cp CHANGELOG.md docs/CHANGELOG.md

echo "==========================================="
echo "✅ Documentation generation completed!"
echo "📂 Documentation available in docs/ directory"
echo "🌐 API docs: docs/api/index.html"
echo "==========================================="
```

## Documentation Standards

### Code Documentation
```rust
/// Comprehensive function documentation
///
/// # Arguments
/// * `config` - Configuration object with validation
/// * `environment` - Test environment for execution
///
/// # Returns
/// * `Result<TestResult, CleanroomError>` - Test execution results or error
///
/// # Errors
/// * `CleanroomError::ValidationError` - Invalid configuration
/// * `CleanroomError::ContainerError` - Container operation failure
///
/// # Examples
/// ```rust
/// use clnrm_core::{Config, TestEnvironments};
///
/// #[tokio::test]
/// async fn example_usage() -> Result<(), CleanroomError> {
///     let config = Config::default();
///     let environment = TestEnvironments::unit_test().await?;
///
///     let result = execute_test(&config, &environment).await?;
///     assert!(result.success);
///     Ok(())
/// }
/// ```
pub async fn execute_test(
    config: &Config,
    environment: &TestEnvironment,
) -> Result<TestResult, CleanroomError> {
    // Implementation
}
```

### Module Documentation
```rust
//! # Cleanroom Testing Framework - Core Library
//!
//! Comprehensive testing framework for integration testing with TOML-based
//! test definitions and container plugin architecture.
//!
//! ## Core Concepts
//!
//! - **Test Definition**: TOML files defining test steps and validation
//! - **Plugin Architecture**: Extensible system for different backends
//! - **Error Handling**: Structured error types with context
//! - **Async Patterns**: Proper async/await for I/O operations
//!
//! ## Quick Start
//!
//! ```toml
//! [test.metadata]
//! name = "example_test"
//! description = "Example test case"
//!
//! [[steps]]
//! name = "hello_world"
//! command = ["echo", "Hello, World!"]
//! expected_output_regex = "Hello"
//! ```
//!
//! ## Architecture Overview
//!
//! The framework consists of several key modules:
//! - `cli` - Command-line interface and argument parsing
//! - `config` - TOML configuration parsing and validation
//! - `execution` - Test orchestration and step execution
//! - `backend` - Container and service backend implementations
//! - `error` - Structured error handling throughout
```

### CLI Documentation
```markdown
# Cleanroom Testing Framework - CLI Reference

## Overview
Command-line interface for the cleanroom testing framework providing comprehensive testing capabilities.

## Commands

### `clnrm run`
Execute tests defined in TOML configuration files.

```bash
clnrm run [OPTIONS] [PATHS]...

Options:
    --parallel         Run tests in parallel
    --jobs <JOBS>      Number of parallel jobs
    --fail-fast        Stop on first failure
    --watch           Watch for file changes
    --interactive     Interactive mode
    --force           Force execution
    --shard <SHARD>   Shard execution
    --digest          Generate SHA-256 digests
    --report-junit <FILE>  Generate JUnit XML report
```

### `clnrm validate`
Validate TOML configuration files for syntax and structure.

```bash
clnrm validate <FILES>...
```

### `clnrm init`
Initialize a new project with sample configuration.

```bash
clnrm init [OPTIONS]

Options:
    --force      Overwrite existing files
    --config     Specify configuration template
```

### `clnrm self-test`
Execute framework self-tests to validate functionality.

```bash
clnrm self-test [OPTIONS]
```

## Examples

### Basic Test Execution
```bash
clnrm run tests/integration.toml
```

### Parallel Test Execution
```bash
clnrm run --parallel --jobs 4 tests/
```

### Watch Mode (Future Feature)
```bash
clnrm run --watch tests/
```

### Generate JUnit Report
```bash
clnrm run --report-junit results.xml tests/
```
```

## Documentation Validation

### Link Validation
```bash
# Check for broken internal links in documentation
cargo doc --no-deps 2>&1 | grep -i "warning\|error" || echo "No documentation warnings"

# Validate external links (if available)
cargo doc --no-deps && find target/doc -name "*.html" -exec grep -l "href=\"http" {} \; | head -5
```

### Example Validation
```bash
# Test that all code examples compile
cargo run --example config-loading-test
cargo run --example observability-demo
cargo run --example complete-dogfooding-suite

# Verify example output matches expectations
cargo run --example simple-framework-test > /tmp/example_output.txt
grep -q "Test passed" /tmp/example_output.txt && echo "✅ Examples working" || echo "❌ Examples broken"
```

### Cross-Reference Validation
```bash
# Ensure documentation references match implementation
grep -r "CleanroomError" docs/ | wc -l
grep -r "CleanroomError" src/ crates/ | wc -l

# Check that public APIs are documented
cargo doc --no-deps 2>&1 | grep -i "missing documentation" || echo "✅ All public APIs documented"
```

## Documentation Maintenance

### Regular Updates
- [ ] **API changes** reflected in documentation
- [ ] **New features** documented with examples
- [ ] **Deprecated features** marked appropriately
- [ ] **Breaking changes** clearly communicated

### Documentation Quality Gates
- [ ] **All public functions** have comprehensive documentation
- [ ] **Code examples** compile and run successfully
- **Module-level documentation** explains purpose and usage
- **Cross-references** are accurate and functional

### Documentation Testing
```bash
# Test documentation build process
cargo doc --no-deps

# Validate example code
cargo test --doc

# Check documentation coverage
cargo doc --no-deps --document-private-items 2>&1 | grep -c "warning" || echo "Documentation warnings found"
```

## Documentation Success Metrics

- **Zero broken links** in generated documentation
- **All public APIs documented** with examples
- **Documentation builds successfully** without warnings
- **Examples compile and run** correctly
- **Cross-references accurate** between modules
- **User feedback incorporated** from issues and discussions

This documentation command ensures the cleanroom testing framework maintains comprehensive, accurate, and accessible documentation that serves both developers and users effectively.
