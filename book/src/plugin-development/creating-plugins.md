# Creating Custom Plugins

This chapter provides detailed guidance on creating custom plugins, from basic structure to advanced patterns.

## Plugin Architecture Overview

clnrm plugins follow a specific architecture pattern:

```
Plugin
├── Configuration (constructor parameters)
├── ServicePlugin trait implementation
├── Container management (testcontainers-rs)
├── Health checking
└── Resource cleanup
```

## Plugin Types

### 1. Database Plugins

For databases like PostgreSQL, MySQL, Redis, etc.

**Key Features:**
- Database-specific environment variables
- Connection string generation
- Health checks via database queries
- Data initialization scripts

**Example Structure:**
```rust
pub struct DatabasePlugin {
    name: String,
    image: String,
    database_name: String,
    username: String,
    password: String,
    init_scripts: Vec<String>,
}
```

### 2. API Service Plugins

For REST APIs, GraphQL services, microservices, etc.

**Key Features:**
- Port configuration
- Health endpoint monitoring
- Custom headers and authentication
- Request/response validation

**Example Structure:**
```rust
pub struct ApiServicePlugin {
    name: String,
    image: String,
    port: u16,
    health_endpoint: String,
    auth_token: Option<String>,
}
```

### 3. Message Queue Plugins

For RabbitMQ, Apache Kafka, Redis Streams, etc.

**Key Features:**
- Queue/topic configuration
- Producer/consumer setup
- Message validation
- Connection pooling

**Example Structure:**
```rust
pub struct MessageQueuePlugin {
    name: String,
    image: String,
    queue_name: String,
    exchange_type: String,
    routing_key: String,
}
```

## Step-by-Step Plugin Creation

### Step 1: Define Plugin Structure

Start with the basic structure:

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
pub struct MyCustomPlugin {
    // Plugin identification
    name: String,
    
    // Container configuration
    image: String,
    tag: String,
    
    // Service-specific configuration
    config: HashMap<String, String>,
    
    // Container state
    container_id: Arc<RwLock<Option<String>>>,
}
```

### Step 2: Implement Constructor

Create a constructor with validation:

```rust
impl MyCustomPlugin {
    pub fn new(name: &str, image: &str) -> Result<Self> {
        // Validate inputs
        if name.is_empty() {
            return Err(CleanroomError::validation_error("Plugin name cannot be empty"));
        }
        
        if image.is_empty() {
            return Err(CleanroomError::validation_error("Image name cannot be empty"));
        }

        // Parse image and tag
        let (image_name, image_tag) = if let Some((name, tag)) = image.split_once(':') {
            (name.to_string(), tag.to_string())
        } else {
            (image.to_string(), "latest".to_string())
        };

        Ok(Self {
            name: name.to_string(),
            image: image_name,
            tag: image_tag,
            config: HashMap::new(),
            container_id: Arc::new(RwLock::new(None)),
        })
    }

    // Builder pattern methods
    pub fn with_config(mut self, key: &str, value: &str) -> Self {
        self.config.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.config.insert(format!("ENV_{}", key), value.to_string());
        self
    }
}
```

### Step 3: Implement ServicePlugin Trait

Implement the core trait methods:

```rust
impl ServicePlugin for MyCustomPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        // Use tokio::task::block_in_place for async operations
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.start_async().await
            })
        })
    }

    fn stop(&self, handle: ServiceHandle) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.stop_async(handle).await
            })
        })
    }

    fn health_check(&self, handle: &ServiceHandle) -> Result<HealthStatus> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                self.health_check_async(handle).await
            })
        })
    }
}
```

### Step 4: Implement Async Methods

Separate async logic into private methods:

```rust
impl MyCustomPlugin {
    async fn start_async(&self) -> Result<ServiceHandle> {
        // Create container configuration
        let image = GenericImage::new(self.image.clone(), self.tag.clone());
        
        // Build container request
        let mut container_request: testcontainers::core::ContainerRequest<GenericImage> =
            image.into();

        // Add environment variables
        for (key, value) in &self.config {
            if key.starts_with("ENV_") {
                let env_key = &key[4..]; // Remove "ENV_" prefix
                container_request = container_request.with_env_var(env_key, value);
            }
        }

        // Start container
        let container = container_request.start().await
            .map_err(|e| CleanroomError::container_error(
                format!("Failed to start container: {}", e)
            ))?;

        // Store container ID
        let container_id = container.id().to_string();
        {
            let mut id_guard = self.container_id.write().await;
            *id_guard = Some(container_id.clone());
        }

        // Wait for service to be ready
        self.wait_for_service_ready(&container).await?;

        // Create service handle
        let mut metadata = HashMap::new();
        metadata.insert("container_id".to_string(), container_id);
        metadata.insert("image".to_string(), format!("{}:{}", self.image, self.tag));
        
        // Add service-specific metadata
        for (key, value) in &self.config {
            if !key.starts_with("ENV_") {
                metadata.insert(key.clone(), value.clone());
            }
        }

        Ok(ServiceHandle {
            id: Uuid::new_v4().to_string(),
            service_name: self.name.clone(),
            metadata,
        })
    }

    async fn stop_async(&self, handle: ServiceHandle) -> Result<()> {
        // Get container ID
        let container_id = handle.metadata.get("container_id")
            .ok_or_else(|| CleanroomError::internal_error(
                "Service handle missing container_id"
            ))?;

        tracing::info!("Stopping container: {}", container_id);

        // Stop container (implementation depends on your container backend)
        // This is a simplified example
        tracing::debug!("Container {} stopped", container_id);

        // Clear container ID
        {
            let mut id_guard = self.container_id.write().await;
            *id_guard = None;
        }

        Ok(())
    }

    async fn health_check_async(&self, handle: &ServiceHandle) -> Result<HealthStatus> {
        let container_id = handle.metadata.get("container_id")
            .ok_or_else(|| CleanroomError::internal_error(
                "Service handle missing container_id"
            ))?;

        // Perform health check
        // This is a simplified example - real implementation would
        // execute a health check command or make an HTTP request
        tracing::debug!("Health checking container: {}", container_id);

        // For now, assume healthy if container ID exists
        Ok(HealthStatus::Healthy)
    }

    async fn wait_for_service_ready(&self, container: &testcontainers::Container<GenericImage>) -> Result<()> {
        // Wait for service to be ready
        // This is a simplified example - real implementation would
        // poll the service or check logs
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        tracing::info!("Service {} is ready", self.name);
        Ok(())
    }
}
```

## Advanced Plugin Patterns

### Plugin with Custom Validation

```rust
impl MyCustomPlugin {
    pub fn validate_config(&self) -> Result<()> {
        // Validate required configuration
        if self.config.get("required_param").is_none() {
            return Err(CleanroomError::validation_error(
                "required_param is required"
            ));
        }

        // Validate parameter values
        if let Some(port_str) = self.config.get("port") {
            let port: u16 = port_str.parse()
                .map_err(|_| CleanroomError::validation_error(
                    "Invalid port number"
                ))?;
            
            if port == 0 {
                return Err(CleanroomError::validation_error(
                    "Port cannot be 0"
                ));
            }
        }

        Ok(())
    }
}
```

### Plugin with OTEL Integration

```rust
use crate::telemetry::spans;

impl ServicePlugin for MyCustomPlugin {
    fn start(&self) -> Result<ServiceHandle> {
        // Create span for plugin start
        let _span = spans::plugin_start_span(&self.name);
        
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // Create span for container operations
                let _container_span = spans::container_start_span(&self.image);
                
                let handle = self.start_async().await?;
                
                // Record successful start
                tracing::info!("Plugin {} started successfully", self.name);
                
                Ok(handle)
            })
        })
    }
}
```

### Plugin with Resource Management

```rust
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug)]
pub struct ResourceManagedPlugin {
    name: String,
    image: String,
    container_id: Arc<RwLock<Option<String>>>,
    resources: Arc<RwLock<Vec<String>>>,
}

impl ResourceManagedPlugin {
    async fn allocate_resources(&self) -> Result<Vec<String>> {
        let mut resources = Vec::new();
        
        // Allocate resources (files, ports, etc.)
        resources.push("resource1".to_string());
        resources.push("resource2".to_string());
        
        {
            let mut res_guard = self.resources.write().await;
            *res_guard = resources.clone();
        }
        
        Ok(resources)
    }

    async fn deallocate_resources(&self) -> Result<()> {
        let resources = {
            let res_guard = self.resources.read().await;
            res_guard.clone()
        };
        
        // Deallocate resources
        for resource in resources {
            tracing::debug!("Deallocating resource: {}", resource);
        }
        
        {
            let mut res_guard = self.resources.write().await;
            res_guard.clear();
        }
        
        Ok(())
    }
}
```

## Testing Your Plugin

### Basic Test Structure

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_plugin_lifecycle() -> Result<(), CleanroomError> {
        // Arrange
        let plugin = MyCustomPlugin::new("test-plugin", "alpine:latest")?
            .with_config("param1", "value1")
            .with_env("TEST_VAR", "test_value");

        // Act - Start plugin
        let handle = plugin.start()?;
        
        // Assert - Verify handle
        assert_eq!(handle.service_name, "test-plugin");
        assert!(handle.metadata.contains_key("container_id"));
        assert_eq!(handle.metadata.get("param1"), Some(&"value1".to_string()));
        
        // Act - Health check
        let health = plugin.health_check(&handle)?;
        
        // Assert - Verify health
        assert_eq!(health, HealthStatus::Healthy);
        
        // Act - Stop plugin
        plugin.stop(handle)?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_plugin_validation() {
        // Test invalid name
        let result = MyCustomPlugin::new("", "alpine:latest");
        assert!(result.is_err());
        
        // Test invalid image
        let result = MyCustomPlugin::new("test", "");
        assert!(result.is_err());
        
        // Test valid configuration
        let result = MyCustomPlugin::new("test", "alpine:latest");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_plugin_with_custom_config() -> Result<(), CleanroomError> {
        // Arrange
        let plugin = MyCustomPlugin::new("test-plugin", "alpine:latest")?
            .with_config("custom_param", "custom_value")
            .with_env("CUSTOM_ENV", "env_value");

        // Act
        let handle = plugin.start()?;
        
        // Assert
        assert_eq!(handle.metadata.get("custom_param"), Some(&"custom_value".to_string()));
        
        // Cleanup
        plugin.stop(handle)?;
        
        Ok(())
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_plugin_integration() -> Result<(), CleanroomError> {
    // Test plugin with real container
    let plugin = MyCustomPlugin::new("integration-test", "alpine:latest")?;
    
    // Start plugin
    let handle = plugin.start()?;
    
    // Verify container is running
    let container_id = handle.metadata.get("container_id").unwrap();
    tracing::info!("Container ID: {}", container_id);
    
    // Perform integration test
    // ... test logic here ...
    
    // Cleanup
    plugin.stop(handle)?;
    
    Ok(())
}
```

## Common Patterns

### Builder Pattern

```rust
impl MyCustomPlugin {
    pub fn builder() -> MyCustomPluginBuilder {
        MyCustomPluginBuilder::new()
    }
}

#[derive(Debug)]
pub struct MyCustomPluginBuilder {
    name: Option<String>,
    image: Option<String>,
    config: HashMap<String, String>,
}

impl MyCustomPluginBuilder {
    pub fn new() -> Self {
        Self {
            name: None,
            image: None,
            config: HashMap::new(),
        }
    }

    pub fn name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    pub fn image(mut self, image: &str) -> Self {
        self.image = Some(image.to_string());
        self
    }

    pub fn config(mut self, key: &str, value: &str) -> Self {
        self.config.insert(key.to_string(), value.to_string());
        self
    }

    pub fn build(self) -> Result<MyCustomPlugin> {
        let name = self.name.ok_or_else(|| {
            CleanroomError::validation_error("Plugin name is required")
        })?;
        
        let image = self.image.ok_or_else(|| {
            CleanroomError::validation_error("Plugin image is required")
        })?;

        let mut plugin = MyCustomPlugin::new(&name, &image)?;
        
        for (key, value) in self.config {
            plugin = plugin.with_config(&key, &value);
        }
        
        Ok(plugin)
    }
}

// Usage
let plugin = MyCustomPlugin::builder()
    .name("my-plugin")
    .image("alpine:latest")
    .config("param1", "value1")
    .config("param2", "value2")
    .build()?;
```

### Configuration Validation

```rust
impl MyCustomPlugin {
    pub fn validate_required_config(&self) -> Result<()> {
        let required_keys = ["database_url", "api_key"];
        
        for key in &required_keys {
            if !self.config.contains_key(*key) {
                return Err(CleanroomError::validation_error(
                    &format!("Required configuration key '{}' is missing", key)
                ));
            }
        }
        
        Ok(())
    }

    pub fn validate_config_values(&self) -> Result<()> {
        // Validate database URL format
        if let Some(db_url) = self.config.get("database_url") {
            if !db_url.starts_with("postgresql://") {
                return Err(CleanroomError::validation_error(
                    "database_url must be a valid PostgreSQL URL"
                ));
            }
        }
        
        // Validate port number
        if let Some(port_str) = self.config.get("port") {
            let port: u16 = port_str.parse()
                .map_err(|_| CleanroomError::validation_error(
                    "port must be a valid number"
                ))?;
            
            if port < 1024 {
                return Err(CleanroomError::validation_error(
                    "port must be >= 1024"
                ));
            }
        }
        
        Ok(())
    }
}
```

## Next Steps

Now that you can create custom plugins:

1. **Try the examples**: Run the code samples in this chapter
2. **Create your own plugin**: Build a plugin for your specific use case
3. **Learn lifecycle management**: Move on to [Plugin Lifecycle Management](plugin-lifecycle.md)
4. **See real examples**: Check out [Plugin Examples](examples.md)

## Further Reading

- [Plugin Architecture Design](../docs/architecture/plugin_system.md)
- [Core Team Standards](../CLAUDE.md)
- [Error Handling Reference](reference/error-handling.md)
