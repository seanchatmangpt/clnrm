# Create Test Following AAA Pattern

Generate a new test following clnrm core team testing standards with proper AAA pattern and descriptive naming.

## Test Naming Convention

Use descriptive names that explain:
- **What** is being tested
- **How** it's being tested
- **What** the expected outcome is

### Good Examples:
- `test_container_creation_with_valid_image_succeeds()`
- `test_service_plugin_start_returns_error_when_container_fails()`
- `test_cleanup_removes_all_containers_on_drop()`

### Bad Examples:
- `test_1()`
- `test_container()`
- `test_works()`

## AAA Pattern (MANDATORY)

Every test must follow the Arrange-Act-Assert pattern:

```rust
#[tokio::test]
async fn test_container_creation_with_valid_image_succeeds() -> Result<()> {
    // Arrange
    let environment = TestEnvironments::unit_test().await?;
    let expected_image = "alpine:latest";

    // Act
    let container = environment.create_container(expected_image).await?;

    // Assert
    assert!(container.is_running());
    assert_eq!(container.image(), expected_image);
    Ok(())
}
```

## Test Structure Checklist

- [ ] Test name is descriptive and explains behavior
- [ ] Uses `#[tokio::test]` for async tests or `#[test]` for sync
- [ ] Returns `Result<()>` for error propagation
- [ ] Clear "Arrange" section with setup
- [ ] Clear "Act" section with operation being tested
- [ ] Clear "Assert" section with expectations
- [ ] Uses `?` operator instead of `.unwrap()` or `.expect()`

## Integration Test Template

```rust
use clnrm_core::{CleanroomEnvironment, Result};
use clnrm_core::services::GenericContainerPlugin;

#[tokio::test]
async fn test_[what]_[condition]_[expected_result]() -> Result<()> {
    // Arrange
    let env = CleanroomEnvironment::new().await?;
    // ... setup test environment

    // Act
    // ... perform the operation being tested

    // Assert
    // ... verify the expected outcome
    Ok(())
}
```

## Property-Based Test Template

```rust
#[cfg(feature = "proptest")]
use proptest::prelude::*;

#[cfg(feature = "proptest")]
proptest! {
    #[test]
    fn test_property_[what_property]_holds(
        input in any::<InputType>()
    ) {
        // Arrange
        let expected = expected_property(input);

        // Act
        let result = function_under_test(input);

        // Assert
        prop_assert_eq!(result, expected);
    }
}
```

## Commands to Run

```bash
# Run single test
cargo test test_name

# Run all tests in module
cargo test module_name::

# Run with output
cargo test test_name -- --nocapture

# Run integration test
cargo test --test integration_test_name
```

## What to Provide

Specify:
1. **What component** you want to test
2. **What scenario** should be tested
3. **What the expected outcome** is

Example: "Test CleanroomEnvironment container cleanup when service fails to start"
