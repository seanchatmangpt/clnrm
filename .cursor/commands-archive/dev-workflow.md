# Development Workflow

## Overview
Standardized development workflow for the cleanroom testing framework. Follows core team best practices for efficient, high-quality development cycles.

## Daily Development Cycle

### 1. Start Development Session
```bash
# Update dependencies and ensure clean state
cargo update
cargo check

# Run baseline tests to ensure nothing broken
cargo test --quiet

# Check for any new clippy warnings
cargo clippy --quiet
```

### 2. Feature Development Process

#### Plan the Feature
- [ ] **Understand requirements** - clarify what needs to be built
- [ ] **Design approach** - consider error handling, async patterns, testing
- [ ] **Write failing test first** - TDD approach following London School
- [ ] **Plan implementation steps** - break into small, testable increments

#### Implement Following Best Practices
```rust
// Start with proper error handling structure
pub fn my_new_feature(config: &Config) -> Result<MyResult, CleanroomError> {
    // Implementation will go here
    unimplemented!("my_new_feature: implement core logic, error handling, and validation")
}
```

#### Follow TDD Pattern
```rust
#[tokio::test]
async fn test_my_new_feature() -> Result<(), CleanroomError> {
    // Arrange
    let config = TestConfig::valid_config();
    let test_environment = TestEnvironments::unit_test().await?;

    // Act
    let result = my_new_feature(&config)?;

    // Assert
    assert_eq!(result.status, ExpectedStatus::Success);
    assert!(!result.output.is_empty());

    Ok(())
}
```

### 3. Quality Assurance Gates

#### Code Quality Checks
```bash
# Verify no unwrap()/expect() in new code
git diff --name-only | xargs grep -l "unwrap()" | head -5

# Check for proper error handling patterns
git diff --name-only | xargs grep -l "CleanroomError" | wc -l

# Validate async patterns
git diff --name-only | xargs grep -l "async fn" | xargs grep -L "impl.*for"
```

#### Testing Validation
```bash
# Run new tests
cargo test my_new_feature

# Run full test suite to check for regressions
cargo test

# Run integration tests
cargo test --test integration

# Check test coverage (if available)
cargo test -- --nocapture | grep -E "test result|running"
```

#### Performance Validation
```bash
# Run benchmarks to ensure no performance regressions
cargo bench

# Check for memory leaks or performance issues
valgrind --tool=memcheck target/debug/clnrm --help 2>/dev/null || echo "Valgrind not available"
```

### 4. Code Review Preparation

#### Pre-Commit Checklist
- [ ] **Tests pass** - `cargo test` successful
- [ ] **No clippy warnings** - `cargo clippy` clean
- [ ] **No unwrap()/expect()** - proper error handling
- [ ] **Dyn compatibility** - no async trait methods
- [ ] **Honest implementation** - incomplete features use `unimplemented!()`
- [ ] **Documentation** - functions have proper docs
- [ ] **Error messages** - meaningful and contextual

#### Commit Message Standards
```
feat: add container lifecycle management

- Implement proper container startup/shutdown
- Add error handling for container failures
- Include integration tests for lifecycle
- Follow async patterns for I/O operations
- Update documentation with examples

Resolves: #123
```

### 5. Integration & Validation

#### Pre-Merge Validation
```bash
# Full test suite
cargo test --release

# Clippy with strict warnings
cargo clippy -- -D warnings

# Format check (if enabled)
cargo fmt --check

# Security audit (if available)
cargo audit

# Documentation build
cargo doc --no-deps
```

#### Best Practices Validation
```bash
# Run the best practices script
./scripts/check-best-practices.sh

# Validate no anti-patterns introduced
./scripts/validate-best-practices.sh

# Check for production readiness
./scripts/production-readiness-validation.sh
```

## Hot Reload Development (Future Feature)

When `dev --watch` is implemented:

```bash
# Start development with hot reload
clnrm dev --watch

# Run tests on file changes
clnrm dev --watch --test

# Format and lint on save
clnrm dev --watch --fmt --clippy
```

## Debugging Workflows

### Common Debugging Patterns

#### Async Issues
```rust
// Debug async code with proper tracing
tracing::info!("Starting operation: {}", operation_name);

// Use block_in_place for async operations in sync contexts
tokio::task::block_in_place(|| {
    tokio::runtime::Handle::current().block_on(async {
        // Your async code here
    })
});
```

#### Container Issues
```rust
// Debug container operations
let container = environment.create_container("debug").await?;
tracing::debug!("Container created: {:?}", container.id());

// Check container logs
let logs = container.logs().await?;
tracing::debug!("Container logs: {}", logs);
```

#### Error Context
```rust
// Provide rich error context for debugging
some_operation().map_err(|e| {
    CleanroomError::container_error("Container operation failed")
        .with_context(format!("Operation: {}, Container: {}", operation, container_id))
        .with_source(e.to_string())
})?;
```

## Performance Optimization

### Benchmark Integration
```bash
# Run performance benchmarks
cargo bench

# Profile specific operations
cargo flamegraph --bin clnrm -- run tests/

# Memory profiling
cargo build --release
valgrind --tool=massif target/release/clnrm run tests/
```

### Hot Path Optimization
- **Test discovery** - fast TOML parsing
- **Container operations** - efficient lifecycle management
- **Error handling** - minimal overhead paths
- **Async operations** - proper task scheduling

## Success Metrics

- **Cycle time** - Feature implementation to production deployment
- **Quality gates** - Zero best practice violations
- **Test reliability** - Consistent test pass rates
- **Performance** - No regressions in benchmarks
- **Error rates** - Proper error handling reduces runtime issues

## Team Coordination

### Code Review Standards
- **Small PRs** - easier to review, faster feedback
- **Clear descriptions** - explain what, why, and how
- **Test coverage** - demonstrate functionality works
- **Best practices** - follow established patterns

### Knowledge Sharing
- **Document decisions** - why certain approaches chosen
- **Share learnings** - what worked, what didn't
- **Update guidelines** - improve processes based on experience
- **Mentor others** - help team members adopt best practices
