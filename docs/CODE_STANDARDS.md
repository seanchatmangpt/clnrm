# clnrm Code Standards - Eliminating Mura (Inconsistency)

**Date:** 2025-01-XX
**Purpose:** Define consistent standards to eliminate Mura (unevenness) in code quality, patterns, and style

---

## Executive Summary

This document establishes consistent standards for clnrm to eliminate Mura (inconsistency). All code must follow these standards to maintain quality, consistency, and maintainability.

**Mura Status:** HIGH - Significant inconsistencies identified across 463 files
**Quality Impact:** Inconsistent code increases cognitive load and maintenance cost
**Coverage:** Currently 22% test coverage with wide variance

---

## Style Standards

### Naming Conventions
- **Functions**: `snake_case` (Rust standard)
- **Types/Structs**: `PascalCase` (Rust standard)
- **Constants**: `SCREAMING_SNAKE_CASE` (Rust standard)
- **Modules**: `snake_case` (Rust standard)
- **Variables**: `snake_case` (Rust standard)

### Formatting
- **Tool**: `cargo fmt` (enforced)
- **Line length**: 100 characters maximum
- **Indentation**: 4 spaces (Rust default)
- **Bracing**: Consistent with Rust standard

### Import Organization
```rust
// Group 1: std imports (alphabetical)
use std::collections::HashMap;
use std::sync::Arc;

// Group 2: external crates (alphabetical)
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;

// Group 3: local crates/modules (alphabetical)
use crate::error::{CleanroomError, Result};
use crate::types::TestConfig;
```

---

## Error Handling Standards

### Error Types
- **Primary**: `Result<T, CleanroomError>` for all fallible operations
- **Never**: `unwrap()` or `expect()` in production code
- **Consistent**: All modules use `CleanroomError` for consistency

### Error Handling Pattern
```rust
// ✅ CORRECT
pub fn do_operation(config: &Config) -> Result<Output, CleanroomError> {
    let data = load_data(config)
        .map_err(|e| CleanroomError::config_error(
            format!("Failed to load data: {}", e)
        ))?;
    Ok(process_data(data))
}

// ❌ WRONG
pub fn do_operation(config: &Config) -> Output {
    let data = load_data(config).unwrap(); // NEVER
    process_data(data)
}
```

---

## Documentation Standards

### Public API Documentation
- **All pub functions** must have `///` documentation
- **Include examples** in doc comments where helpful
- **Document parameters** and return values
- **Document error conditions**

### Documentation Pattern
```rust
/// Process test configuration and validate all settings.
///
/// This function performs comprehensive validation of test configurations
/// including container settings, network requirements, and resource limits.
///
/// # Arguments
///
/// * `config` - The test configuration to validate
///
/// # Returns
///
/// Returns `Ok(())` if validation passes, or `Err(CleanroomError)` if validation fails.
///
/// # Errors
///
/// This function will return an error if:
/// - Container image is invalid
/// - Resource limits exceed system capacity
/// - Network configuration conflicts
///
/// # Examples
///
/// ```rust
/// use clnrm_core::config::validator::validate_config;
///
/// let config = TestConfig::default();
/// assert!(validate_config(&config).is_ok());
/// ```
pub fn validate_config(config: &TestConfig) -> Result<(), CleanroomError> {
    // implementation
}
```

---

## Testing Standards

### Coverage Requirements
- **Minimum**: 80% test coverage for all modules
- **Target**: 90%+ for critical paths
- **Measure**: `cargo tarpaulin` or similar coverage tools

### Test Organization
- **Pattern**: AAA (Arrange, Act, Assert)
- **Naming**: `test_descriptive_name_of_what_is_tested`
- **Structure**: Unit tests in same file, integration tests in `tests/` directory

### Test Quality
```rust
#[test]
fn test_validate_config_accepts_valid_config() {
    // Arrange
    let config = TestConfig {
        name: "valid-test".to_string(),
        image: "alpine:latest".to_string(),
        ..TestConfig::default()
    };

    // Act
    let result = validate_config(&config);

    // Assert
    assert!(result.is_ok(), "Valid config should be accepted");
}

#[test]
fn test_validate_config_rejects_empty_name() {
    // Arrange
    let config = TestConfig {
        name: "".to_string(),
        image: "alpine:latest".to_string(),
        ..TestConfig::default()
    };

    // Act
    let result = validate_config(&config);

    // Assert
    assert!(result.is_err(), "Empty name should be rejected");
    let err = result.unwrap_err();
    assert!(err.to_string().contains("name"), "Error should mention name field");
}
```

---

## Complexity Standards

### File Size Limits
- **Maximum lines**: 500 lines per file
- **Maximum functions**: 10 public functions per module
- **Reason**: Maintainability and readability

### Function Complexity
- **Maximum cyclomatic complexity**: 10
- **Maximum nesting depth**: 3 levels
- **Single responsibility**: One function = one responsibility

### Module Organization
```rust
// Good: Focused modules
pub mod config;
pub mod executor;
pub mod validator;

// Bad: Kitchen sink module
pub mod utils; // Contains everything
```

---

## Async/Sync Standards

### Async Usage
- **When to use**: I/O operations, network calls, file operations
- **When not to use**: Pure computation, validation, data transformation
- **Pattern**: Use `tokio::task::block_in_place` for mixing sync traits with async

### Async Function Guidelines
```rust
// ✅ CORRECT: Async for I/O
pub async fn run_test(config: &TestConfig) -> Result<TestResult, CleanroomError> {
    let container = create_container(&config).await?;
    execute_test(container).await
}

// ✅ CORRECT: Sync for computation (with block_in_place if needed)
pub fn validate_config(config: &TestConfig) -> Result<(), CleanroomError> {
    // Pure validation logic - no async needed
    if config.name.is_empty() {
        return Err(CleanroomError::validation_error("Name cannot be empty"));
    }
    Ok(())
}
```

---

## Pattern Standards

### Result Handling
```rust
// ✅ CORRECT: Early return with ?
pub fn process_data(input: &str) -> Result<Output, CleanroomError> {
    let parsed = parse_input(input)?;
    let validated = validate_data(&parsed)?;
    Ok(transform_data(validated))
}

// ✅ CORRECT: Error context
pub fn load_config(path: &Path) -> Result<Config, CleanroomError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| CleanroomError::io_error(
            format!("Failed to read config file {}: {}", path.display(), e)
        ))?;
    Ok(serde_json::from_str(&content)?)
}
```

### Builder Pattern
```rust
// ✅ CORRECT: Builder for complex types
pub struct TestConfigBuilder {
    config: TestConfig,
}

impl TestConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: TestConfig::default(),
        }
    }

    pub fn name(mut self, name: impl Into<String>) -> Self {
        self.config.name = name.into();
        self
    }

    pub fn image(mut self, image: impl Into<String>) -> Self {
        self.config.image = image.into();
        self
    }

    pub fn build(self) -> TestConfig {
        self.config
    }
}
```

---

## Implementation Priority

### Phase 1: Critical Standards (Immediate)
1. **Formatting**: Apply `cargo fmt` universally
2. **Error handling**: Remove all `unwrap()` and `expect()` calls
3. **Testing**: Reach 80% minimum coverage
4. **CI enforcement**: Automated checks for standards

### Phase 2: Quality Standards (Next)
1. **Documentation**: Complete all public API docs
2. **Complexity**: Break up large files/modules
3. **Async/sync**: Consistent usage patterns

### Phase 3: Polish Standards (Future)
1. **Advanced patterns**: Builder patterns, type safety
2. **Performance**: Consistent optimization approaches
3. **Code review**: Checklist enforcement

---

## Enforcement

### Automated Checks (CI)
- `cargo fmt --check` - Formatting consistency
- `cargo clippy -- -D warnings` - Code quality
- Coverage threshold checks - Test consistency
- Custom lint: forbid unwrap/expect - Error handling consistency

### Code Review Checklist
- [ ] Code follows style standards
- [ ] Error handling uses Result<T,E> consistently
- [ ] Public APIs are fully documented
- [ ] Tests exist and follow AAA pattern
- [ ] Complexity limits are respected
- [ ] Async/sync usage is appropriate

---

## Metrics

### Current State (Mura Identified)
- **Files**: 463 total
- **Test coverage**: 22% (102 test files)
- **Unsafe errors**: 109 unwrap/expect calls
- **Documentation variance**: 0-296 documented functions per file
- **Complexity variance**: 1-38 public functions per file

### Target State (Mura Eliminated)
- **Test coverage**: 80% minimum
- **Unsafe errors**: 0 unwrap/expect calls
- **Documentation**: 100% public API coverage
- **Complexity**: All files under 500 lines, 10 pub fn max
- **Consistency**: <5% variance in quality metrics

---

## Continuous Improvement

**Kaizen Approach**: Improve consistency incrementally
- Fix 1 inconsistency type per week
- Automate enforcement as standards are established
- Regular audits to prevent Mura recurrence

**Principle**: "Consistency is more important than perfection" - Consistent good code beats perfect code in some places and poor code elsewhere.

---

**Document Version:** 1.0
**Last Updated:** 2025-01-XX
**Review Cycle:** Monthly
**Enforcement:** CI + Code Review
