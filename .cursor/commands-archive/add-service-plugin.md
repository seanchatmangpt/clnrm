# Add New Service Plugin

Create a new service plugin for the clnrm framework following the plugin architecture pattern.

## Plugin Architecture Overview

clnrm uses a plugin-based architecture where each service (database, message queue, etc.) is implemented as a separate plugin that implements the `ServicePlugin` trait.

## ServicePlugin Trait Requirements

```rust
pub trait ServicePlugin: Send + Sync {
    /// Start the service (MUST be sync for dyn compatibility)
    fn start(&self) -> Result<ServiceHandle>;

    /// Stop the service (MUST be sync for dyn compatibility)
    fn stop(&self, handle: &ServiceHandle) -> Result<()>;

    /// Check service health
    fn health_check(&self, handle: &ServiceHandle) -> Result<bool>;

    /// Get service type identifier
    fn service_type(&self) -> &str;
}
```

## Critical Rules

1. **SYNC METHODS ONLY** - All trait methods must be sync for `dyn ServicePlugin` compatibility
2. **Use `block_in_place`** - For async operations inside trait methods:
   ```rust
   fn start(&self) -> Result<ServiceHandle> {
       tokio::task::block_in_place(|| {
           tokio::runtime::Handle::current().block_on(async {
               // async code here
           })
       })
   }
   ```

3. **Proper Error Handling** - NO `.unwrap()` or `.expect()`:
   ```rust
   operation().map_err(|e| {
       CleanroomError::service_error(format!("Failed: {}", e))
   })?
   ```

## Plugin Template

```rust
use crate::{ServicePlugin, ServiceHandle, Result, CleanroomError};
use testcontainers::{Container, GenericImage};

pub struct MyServicePlugin {
    name: String,
    image: String,
    config: MyServiceConfig,
}

#[derive(Debug, Clone)]
pub struct MyServiceConfig {
    pub port: u16,
    pub environment: HashMap<String, String>,
}

impl MyServicePlugin {
    pub fn new(name: impl Into<String>, config: MyServiceConfig) -> Self {
        Self {
            name: name.into(),
            image: "my-service:latest".to_string(),
            config,
        }
    }
}

impl ServicePlugin for MyServicePlugin {
    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Create container
                let image = GenericImage::new(&self.image, "latest")
                    .with_exposed_port(self.config.port);

                let container = image.start().await.map_err(|e| {
                    CleanroomError::service_error(format!(
                        "Failed to start {}: {}", self.name, e
                    ))
                })?;

                // Return handle
                Ok(ServiceHandle::new(
                    self.name.clone(),
                    container,
                ))
            })
        })
    }

    fn stop(&self, handle: &ServiceHandle) -> Result<()> {
        // Cleanup logic
        Ok(())
    }

    fn health_check(&self, handle: &ServiceHandle) -> Result<bool> {
        // Health check logic
        Ok(true)
    }

    fn service_type(&self) -> &str {
        "my-service"
    }
}
```

## Testing Your Plugin

```rust
#[tokio::test]
async fn test_my_service_plugin_starts_successfully() -> Result<()> {
    // Arrange
    let config = MyServiceConfig {
        port: 8080,
        environment: HashMap::new(),
    };
    let plugin = MyServicePlugin::new("test-service", config);

    // Act
    let handle = plugin.start()?;

    // Assert
    assert!(plugin.health_check(&handle)?);
    assert_eq!(plugin.service_type(), "my-service");

    plugin.stop(&handle)?;
    Ok(())
}
```

## Steps to Create Plugin

1. **Create plugin file** in `crates/clnrm-core/src/services/`
2. **Implement ServicePlugin trait** (sync methods only)
3. **Add to mod.rs** in services module
4. **Write tests** following AAA pattern
5. **Add example** in `examples/`
6. **Update documentation**

## Built-in Plugins for Reference

- `generic.rs` - GenericContainerPlugin (any Docker image)
- `surrealdb.rs` - SurrealDB plugin
- `ollama.rs` - Ollama LLM plugin

## Commands to Run

```bash
# Test your plugin
cargo test -p clnrm-core my_service

# Integration test
cargo test --test integration_my_service

# Build with plugin
cargo build --features my-service
```

## What to Provide

Specify:
1. **Service name** (e.g., "PostgreSQL", "Redis", "Kafka")
2. **Docker image** to use
3. **Configuration options** needed
4. **Health check method** (HTTP endpoint, TCP port, etc.)
