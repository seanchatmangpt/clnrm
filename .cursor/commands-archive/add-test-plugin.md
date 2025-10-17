# Add Test Plugin - Create New Service Plugin

Create a new service plugin for the clnrm testing framework with proper structure and boilerplate.

## What This Does

Generates a complete plugin implementation following clnrm standards:
1. **Plugin trait implementation** - ServicePlugin with sync methods
2. **Configuration structure** - Service configuration
3. **Health check** - Service health validation
4. **Tests** - Unit tests with AAA pattern
5. **Integration** - Registration in service manager
6. **Documentation** - Inline docs and examples

## Plugin Structure

```rust
// crates/clnrm-core/src/services/my_plugin.rs

use crate::{
    error::CleanroomError,
    cleanroom::ServicePlugin,
    backend::ServiceHandle,
};
use std::collections::HashMap;

/// MyService plugin for testing [description]
///
/// # Examples
///
/// ```
/// use clnrm_core::services::MyServicePlugin;
///
/// let plugin = MyServicePlugin::new("my-service", "image:tag");
/// ```
pub struct MyServicePlugin {
    name: String,
    image: String,
    config: HashMap<String, String>,
}

impl MyServicePlugin {
    /// Create a new MyService plugin
    pub fn new(name: impl Into<String>, image: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            image: image.into(),
            config: HashMap::new(),
        }
    }

    /// Set environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.config.insert(key.into(), value.into());
        self
    }
}

impl ServicePlugin for MyServicePlugin {
    fn start(&self) -> Result<ServiceHandle, CleanroomError> {
        // Use tokio::task::block_in_place for async operations
        tokio::task::block_in_place(|| {
            // Implementation
            Ok(ServiceHandle::new(/* ... */))
        })
    }

    fn stop(&self, handle: &ServiceHandle) -> Result<(), CleanroomError> {
        // Cleanup
        Ok(())
    }

    fn health_check(&self, handle: &ServiceHandle) -> Result<bool, CleanroomError> {
        // Health validation
        Ok(true)
    }

    fn service_type(&self) -> &str {
        "my_service"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_creation() {
        // Arrange
        let name = "test-service";
        let image = "test:latest";

        // Act
        let plugin = MyServicePlugin::new(name, image);

        // Assert
        assert_eq!(plugin.name, name);
        assert_eq!(plugin.image, image);
    }

    #[tokio::test]
    async fn test_plugin_start_succeeds() -> Result<(), CleanroomError> {
        // Arrange
        let plugin = MyServicePlugin::new("test", "alpine:latest");

        // Act
        let handle = plugin.start()?;

        // Assert
        assert!(plugin.health_check(&handle)?);
        plugin.stop(&handle)?;
        Ok(())
    }
}
```

## Critical Requirements

### ✅ Core Team Standards

1. **NO async trait methods** - Use `tokio::task::block_in_place` internally
2. **NO .unwrap() or .expect()** - Use proper Result error handling
3. **NO fake implementations** - No `Ok(())` stubs without real work
4. **AAA test pattern** - Arrange, Act, Assert
5. **Documentation** - All public items must have doc comments

### ⚠️ Common Mistakes to Avoid

```rust
// ❌ WRONG - async trait method breaks dyn compatibility
impl ServicePlugin for MyPlugin {
    async fn start(&self) -> Result<ServiceHandle> { }
}

// ✅ CORRECT - sync method with internal async
impl ServicePlugin for MyPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            // async work here
        })
    }
}

// ❌ WRONG - unwrap can panic
let result = operation().unwrap();

// ✅ CORRECT - proper error handling
let result = operation().map_err(|e|
    CleanroomError::internal_error(format!("Failed: {}", e))
)?;
```

## Steps to Add Plugin

1. **Create plugin file**: `crates/clnrm-core/src/services/my_plugin.rs`
2. **Implement ServicePlugin trait** with sync methods
3. **Add to mod.rs**: Export plugin in `src/services/mod.rs`
4. **Write tests** following AAA pattern
5. **Update documentation**: Add to plugin guide
6. **Integration test**: Add to `tests/integration/`

## Testing Your Plugin

```bash
# Unit tests
cargo test --lib my_plugin

# Integration test
cargo test --test integration_my_plugin

# Full validation
cargo clippy -- -D warnings
cargo fmt --check
```

## When to Use
- Creating new service integrations
- Adding database plugins
- Implementing custom containers
- Extending test framework
