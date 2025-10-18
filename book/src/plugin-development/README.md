# Plugin Development

Plugin development is the most powerful way to extend clnrm's capabilities. This chapter covers creating custom service plugins, integrating with external systems, and following core team standards.

## Overview

clnrm's plugin system allows you to:
- Create custom service plugins for any Docker image
- Integrate with databases, APIs, and external services
- Implement custom validation logic
- Extend the framework without modifying core code

## Core Concepts

### ServicePlugin Trait

All plugins implement the `ServicePlugin` trait:

```rust
pub trait ServicePlugin: Send + Sync + std::fmt::Debug {
    fn name(&self) -> &str;
    fn start(&self) -> Result<ServiceHandle>;
    fn stop(&self, handle: ServiceHandle) -> Result<()>;
    fn health_check(&self, handle: &ServiceHandle) -> Result<HealthStatus>;
}
```

### Key Principles

1. **dyn-Compatible**: All traits use sync methods only
2. **Error Handling**: Return `Result<T, CleanroomError>` from all operations
3. **OTEL Integration**: Emit spans for observability
4. **Container Integration**: Use testcontainers-rs for container management

## Creating Your First Plugin

Let's create a custom database plugin step by step.

### Step 1: Define the Plugin Structure

```rust
use crate::cleanroom::{HealthStatus, ServiceHandle, ServicePlugin};
use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug)]
pub struct CustomDatabasePlugin {
    name: String,
    image: String,
    tag: String,
    database_name: String,
    username: String,
    password: String,
    container_id: Arc<RwLock<Option<String>>>,
    env_vars: HashMap<String, String>,
}

impl CustomDatabasePlugin {
    pub fn new(name: &str, image: &str, database_name: &str) -> Self {
        let (image_name, image_tag) = if let Some((name, tag)) = image.split_once(':') {
            (name.to_string(), tag.to_string())
        } else {
            (image.to_string(), "latest".to_string())
        };

        Self {
            name: name.to_string(),
            image: image_name,
            tag: image_tag,
            database_name: database_name.to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            container_id: Arc::new(RwLock::new(None)),
            env_vars: HashMap::new(),
        }
    }

    pub fn with_credentials(mut self, username: &str, password: &str) -> Self {
        self.username = username.to_string();
        self.password = password.to_string();
        self
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }
}
```

### Step 2: Implement ServicePlugin

```rust
impl ServicePlugin for CustomDatabasePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        // Use tokio::task::block_in_place for async operations
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Create container configuration
                let image = GenericImage::new(self.image.clone(), self.tag.clone());

                // Build container request with database-specific environment
                let mut container_request: testcontainers::core::ContainerRequest<GenericImage> =
                    image.into();

                // Set database environment variables
                container_request = container_request
                    .with_env_var("POSTGRES_DB", &self.database_name)
                    .with_env_var("POSTGRES_USER", &self.username)
                    .with_env_var("POSTGRES_PASSWORD", &self.password);

                // Add custom environment variables
                for (key, value) in &self.env_vars {
                    container_request = container_request.with_env_var(key, value);
                }

                // Start container
                let container = container_request.start().await
                    .map_err(|e| CleanroomError::container_error(
                        format!("Failed to start database container: {}", e)
                    ))?;

                // Store container ID
                let container_id = container.id().to_string();
                {
                    let mut id_guard = self.container_id.write().await;
                    *id_guard = Some(container_id.clone());
                }

                // Wait for database to be ready
                self.wait_for_database_ready(&container).await?;

                Ok(ServiceHandle {
                    id: Uuid::new_v4().to_string(),
                    service_name: self.name.clone(),
                    metadata: HashMap::from([
                        ("container_id".to_string(), container_id),
                        ("database_name".to_string(), self.database_name.clone()),
                        ("username".to_string(), self.username.clone()),
                    ]),
                })
            })
        })
    }

    fn stop(&self, handle: ServiceHandle) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Get container ID from metadata
                let container_id = handle.metadata.get("container_id")
                    .ok_or_else(|| CleanroomError::internal_error(
                        "Service handle missing container_id"
                    ))?;

                // Stop container (implementation depends on your container backend)
                // This is a simplified example - real implementation would use
                // the actual container reference
                tracing::info!("Stopping database container: {}", container_id);
                
                Ok(())
            })
        })
    }

    fn health_check(&self, handle: &ServiceHandle) -> Result<HealthStatus> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Perform database health check
                let container_id = handle.metadata.get("container_id")
                    .ok_or_else(|| CleanroomError::internal_error(
                        "Service handle missing container_id"
                    ))?;

                // Check if container is running
                // This is a simplified example - real implementation would
                // execute a health check command in the container
                tracing::debug!("Health checking database container: {}", container_id);
                
                Ok(HealthStatus::Healthy)
            })
        })
    }
}

impl CustomDatabasePlugin {
    async fn wait_for_database_ready(&self, container: &testcontainers::Container<GenericImage>) -> Result<()> {
        // Wait for database to be ready by checking logs or executing a test query
        // This is a simplified example - real implementation would poll the database
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        tracing::info!("Database container is ready");
        Ok(())
    }
}
```

### Step 3: Using Your Plugin

```rust
use clnrm_core::cleanroom::CleanroomEnvironment;

#[tokio::test]
async fn test_custom_database_plugin() -> Result<(), CleanroomError> {
    // Create plugin
    let db_plugin = CustomDatabasePlugin::new(
        "testdb",
        "postgres:15-alpine",
        "testdb"
    )
    .with_credentials("testuser", "testpass")
    .with_env("POSTGRES_INITDB_ARGS", "--auth-host=scram-sha-256");

    // Start database service
    let handle = db_plugin.start()?;
    
    // Verify health
    let health = db_plugin.health_check(&handle)?;
    assert_eq!(health, HealthStatus::Healthy);
    
    // Use the database in your tests
    // ... test logic here ...
    
    // Clean up
    db_plugin.stop(handle)?;
    
    Ok(())
}
```

## Advanced Plugin Patterns

### Plugin with Custom Validation

```rust
#[derive(Debug)]
pub struct ApiServicePlugin {
    name: String,
    image: String,
    tag: String,
    port: u16,
    health_endpoint: String,
    expected_response: String,
}

impl ApiServicePlugin {
    pub fn new(name: &str, image: &str, port: u16) -> Self {
        Self {
            name: name.to_string(),
            image: image.to_string(),
            tag: "latest".to_string(),
            port,
            health_endpoint: "/health".to_string(),
            expected_response: "OK".to_string(),
        }
    }

    pub fn with_health_check(mut self, endpoint: &str, expected: &str) -> Self {
        self.health_endpoint = endpoint.to_string();
        self.expected_response = expected.to_string();
        self
    }
}

impl ServicePlugin for ApiServicePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        // Implementation similar to database plugin
        // but with API-specific configuration
        unimplemented!("API plugin implementation")
    }

    fn stop(&self, handle: ServiceHandle) -> Result<()> {
        // Stop API service
        unimplemented!("API plugin stop implementation")
    }

    fn health_check(&self, handle: &ServiceHandle) -> Result<HealthStatus> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Perform HTTP health check
                let health_url = format!("http://localhost:{}{}", self.port, self.health_endpoint);
                
                // Make HTTP request to health endpoint
                // This is a simplified example - real implementation would
                // use reqwest or similar HTTP client
                tracing::debug!("Checking API health at: {}", health_url);
                
                // For now, assume healthy
                Ok(HealthStatus::Healthy)
            })
        })
    }
}
```

### Plugin with OTEL Integration

```rust
use crate::telemetry::spans;

impl ServicePlugin for CustomDatabasePlugin {
    fn start(&self) -> Result<ServiceHandle> {
        // Create span for plugin start
        let _span = spans::plugin_start_span(&self.name);
        
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Create span for container start
                let _container_span = spans::container_start_span(&self.image);
                
                // ... container start logic ...
                
                tracing::info!("Database plugin started successfully");
                Ok(ServiceHandle {
                    // ... handle creation ...
                })
            })
        })
    }
}
```

## Best Practices

### 1. Error Handling

Always use proper error handling with context:

```rust
// ❌ WRONG - will panic
let result = container.start().await.unwrap();

// ✅ CORRECT - proper error handling
let result = container.start().await
    .map_err(|e| CleanroomError::container_error(
        format!("Failed to start container: {}", e)
    ))?;
```

### 2. Resource Cleanup

Always clean up resources in the `stop` method:

```rust
fn stop(&self, handle: ServiceHandle) -> Result<()> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            // Get container reference
            let container_id = handle.metadata.get("container_id")
                .ok_or_else(|| CleanroomError::internal_error(
                    "Missing container_id in service handle"
                ))?;

            // Stop container
            tracing::info!("Stopping container: {}", container_id);
            
            // Clear container ID
            {
                let mut id_guard = self.container_id.write().await;
                *id_guard = None;
            }
            
            Ok(())
        })
    })
}
```

### 3. Configuration Validation

Validate configuration in your plugin constructor:

```rust
impl CustomDatabasePlugin {
    pub fn new(name: &str, image: &str, database_name: &str) -> Result<Self> {
        // Validate inputs
        if name.is_empty() {
            return Err(CleanroomError::validation_error("Plugin name cannot be empty"));
        }
        
        if database_name.is_empty() {
            return Err(CleanroomError::validation_error("Database name cannot be empty"));
        }

        // ... rest of constructor
        Ok(Self {
            name: name.to_string(),
            // ... other fields
        })
    }
}
```

### 4. Testing Your Plugin

Create comprehensive tests for your plugin:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_database_plugin_lifecycle() -> Result<(), CleanroomError> {
        // Arrange
        let plugin = CustomDatabasePlugin::new("testdb", "postgres:15-alpine", "testdb")?;

        // Act - Start plugin
        let handle = plugin.start()?;
        
        // Assert - Verify handle
        assert_eq!(handle.service_name, "testdb");
        assert!(handle.metadata.contains_key("container_id"));
        
        // Act - Health check
        let health = plugin.health_check(&handle)?;
        
        // Assert - Verify health
        assert_eq!(health, HealthStatus::Healthy);
        
        // Act - Stop plugin
        plugin.stop(handle)?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_database_plugin_with_credentials() -> Result<(), CleanroomError> {
        // Arrange
        let plugin = CustomDatabasePlugin::new("testdb", "postgres:15-alpine", "testdb")?
            .with_credentials("customuser", "custompass");

        // Act
        let handle = plugin.start()?;
        
        // Assert
        assert_eq!(handle.metadata.get("username"), Some(&"customuser".to_string()));
        
        // Cleanup
        plugin.stop(handle)?;
        
        Ok(())
    }
}
```

## Common Pitfalls

### 1. Breaking dyn Compatibility

```rust
// ❌ WRONG - async trait method breaks dyn compatibility
pub trait ServicePlugin {
    async fn start(&self) -> Result<ServiceHandle>;
}

// ✅ CORRECT - sync trait with async inside
pub trait ServicePlugin {
    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Async work here
            })
        })
    }
}
```

### 2. Forgetting Resource Cleanup

```rust
// ❌ WRONG - container keeps running
fn stop(&self, handle: ServiceHandle) -> Result<()> {
    tracing::info!("Stopping service");
    Ok(()) // Container still running!
}

// ✅ CORRECT - proper cleanup
fn stop(&self, handle: ServiceHandle) -> Result<()> {
    tokio::task::block_in_place(|| {
        tokio::runtime::Handle::current().block_on(async {
            let container_id = handle.metadata.get("container_id")?;
            // Actually stop the container
            tracing::info!("Stopping container: {}", container_id);
            Ok(())
        })
    })
}
```

### 3. Not Handling Errors Properly

```rust
// ❌ WRONG - panic on error
let result = container.start().await.unwrap();

// ✅ CORRECT - proper error handling
let result = container.start().await
    .map_err(|e| CleanroomError::container_error(
        format!("Container start failed: {}", e)
    ))?;
```

## Next Steps

Now that you understand plugin development basics:

1. **Try the examples**: Run the code samples in this chapter
2. **Create your own plugin**: Build a plugin for your specific use case
3. **Learn advanced patterns**: Move on to [Advanced Testing Patterns](advanced-patterns/README.md)
4. **Master templates**: Learn about [Template System Mastery](template-mastery/README.md)

## Further Reading

- [Plugin Architecture Design](../docs/architecture/plugin_system.md)
- [Core Team Standards](../CLAUDE.md)
- [Error Handling Reference](reference/error-handling.md)
- [Examples Directory](../examples/plugins/)
