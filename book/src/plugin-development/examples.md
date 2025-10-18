# Plugin Examples

This chapter provides complete, runnable examples of custom plugins for common use cases.

## Database Plugin Example

A complete PostgreSQL plugin with connection management and health checks.

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
pub struct PostgresPlugin {
    name: String,
    image: String,
    tag: String,
    database_name: String,
    username: String,
    password: String,
    port: u16,
    container_id: Arc<RwLock<Option<String>>>,
    connection_string: Arc<RwLock<Option<String>>>,
}

impl PostgresPlugin {
    pub fn new(name: &str, database_name: &str) -> Result<Self> {
        if name.is_empty() {
            return Err(CleanroomError::validation_error("Plugin name cannot be empty"));
        }
        
        if database_name.is_empty() {
            return Err(CleanroomError::validation_error("Database name cannot be empty"));
        }

        Ok(Self {
            name: name.to_string(),
            image: "postgres".to_string(),
            tag: "15-alpine".to_string(),
            database_name: database_name.to_string(),
            username: "testuser".to_string(),
            password: "testpass".to_string(),
            port: 5432,
            container_id: Arc::new(RwLock::new(None)),
            connection_string: Arc::new(RwLock::new(None)),
        })
    }

    pub fn with_credentials(mut self, username: &str, password: &str) -> Self {
        self.username = username.to_string();
        self.password = password.to_string();
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn get_connection_string(&self) -> Result<String> {
        let conn_str = self.connection_string.read().await
            .ok_or_else(|| CleanroomError::internal_error("Connection string not available"))?;
        Ok(conn_str.clone())
    }
}

impl ServicePlugin for PostgresPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _span = tracing::info_span!("postgres_start", plugin = self.name);
                
                // Create container
                let image = GenericImage::new(self.image.clone(), self.tag.clone());
                let mut container_request: testcontainers::core::ContainerRequest<GenericImage> =
                    image.into();

                // Configure PostgreSQL
                container_request = container_request
                    .with_env_var("POSTGRES_DB", &self.database_name)
                    .with_env_var("POSTGRES_USER", &self.username)
                    .with_env_var("POSTGRES_PASSWORD", &self.password)
                    .with_env_var("POSTGRES_INITDB_ARGS", "--auth-host=scram-sha-256");

                // Start container
                let container = container_request.start().await
                    .map_err(|e| CleanroomError::container_error(
                        format!("Failed to start PostgreSQL container: {}", e)
                    ))?;

                let container_id = container.id().to_string();
                {
                    let mut id_guard = self.container_id.write().await;
                    *id_guard = Some(container_id.clone());
                }

                // Wait for PostgreSQL to be ready
                self.wait_for_postgres_ready(&container).await?;

                // Get connection details
                let host = container.get_host().await;
                let port = container.get_host_port_ipv4(self.port).await;
                
                // Create connection string
                let conn_str = format!(
                    "postgresql://{}:{}@{}:{}/{}",
                    self.username, self.password, host, port, self.database_name
                );
                
                {
                    let mut conn_guard = self.connection_string.write().await;
                    *conn_guard = Some(conn_str.clone());
                }

                // Create service handle
                let mut metadata = HashMap::new();
                metadata.insert("container_id".to_string(), container_id);
                metadata.insert("database_name".to_string(), self.database_name.clone());
                metadata.insert("username".to_string(), self.username.clone());
                metadata.insert("port".to_string(), port.to_string());
                metadata.insert("connection_string".to_string(), conn_str);

                Ok(ServiceHandle {
                    id: Uuid::new_v4().to_string(),
                    service_name: self.name.clone(),
                    metadata,
                })
            })
        })
    }

    fn stop(&self, handle: ServiceHandle) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _span = tracing::info_span!("postgres_stop", plugin = self.name);
                
                let container_id = handle.metadata.get("container_id")
                    .ok_or_else(|| CleanroomError::internal_error(
                        "Service handle missing container_id"
                    ))?;

                tracing::info!("Stopping PostgreSQL container: {}", container_id);
                
                // Clear state
                {
                    let mut id_guard = self.container_id.write().await;
                    *id_guard = None;
                }
                
                {
                    let mut conn_guard = self.connection_string.write().await;
                    *conn_guard = None;
                }

                Ok(())
            })
        })
    }

    fn health_check(&self, handle: &ServiceHandle) -> Result<HealthStatus> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _span = tracing::info_span!("postgres_health_check", plugin = self.name);
                
                // Check if connection string is available
                let conn_str = {
                    let conn_guard = self.connection_string.read().await;
                    conn_guard.clone()
                };
                
                if conn_str.is_none() {
                    return Ok(HealthStatus::Unhealthy);
                }

                // Perform basic health check
                // In a real implementation, you would execute a simple query
                tracing::debug!("PostgreSQL health check passed");
                Ok(HealthStatus::Healthy)
            })
        })
    }
}

impl PostgresPlugin {
    async fn wait_for_postgres_ready(&self, container: &testcontainers::Container<GenericImage>) -> Result<()> {
        // Wait for PostgreSQL to be ready
        // In a real implementation, you would check logs or execute a test query
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        tracing::info!("PostgreSQL container is ready");
        Ok(())
    }
}
```

## API Service Plugin Example

A complete REST API plugin with health endpoint monitoring.

```rust
#[derive(Debug)]
pub struct ApiServicePlugin {
    name: String,
    image: String,
    tag: String,
    port: u16,
    health_endpoint: String,
    expected_response: String,
    container_id: Arc<RwLock<Option<String>>>,
    base_url: Arc<RwLock<Option<String>>>,
}

impl ApiServicePlugin {
    pub fn new(name: &str, image: &str, port: u16) -> Result<Self> {
        if name.is_empty() {
            return Err(CleanroomError::validation_error("Plugin name cannot be empty"));
        }
        
        if image.is_empty() {
            return Err(CleanroomError::validation_error("Image name cannot be empty"));
        }

        let (image_name, image_tag) = if let Some((name, tag)) = image.split_once(':') {
            (name.to_string(), tag.to_string())
        } else {
            (image.to_string(), "latest".to_string())
        };

        Ok(Self {
            name: name.to_string(),
            image: image_name,
            tag: image_tag,
            port,
            health_endpoint: "/health".to_string(),
            expected_response: "OK".to_string(),
            container_id: Arc::new(RwLock::new(None)),
            base_url: Arc::new(RwLock::new(None)),
        })
    }

    pub fn with_health_endpoint(mut self, endpoint: &str, expected: &str) -> Self {
        self.health_endpoint = endpoint.to_string();
        self.expected_response = expected.to_string();
        self
    }

    pub fn get_base_url(&self) -> Result<String> {
        let base_url = self.base_url.read().await
            .ok_or_else(|| CleanroomError::internal_error("Base URL not available"))?;
        Ok(base_url.clone())
    }
}

impl ServicePlugin for ApiServicePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _span = tracing::info_span!("api_start", plugin = self.name);
                
                // Create container
                let image = GenericImage::new(self.image.clone(), self.tag.clone());
                let mut container_request: testcontainers::core::ContainerRequest<GenericImage> =
                    image.into();

                // Expose port
                container_request = container_request.with_exposed_port(self.port);

                // Start container
                let container = container_request.start().await
                    .map_err(|e| CleanroomError::container_error(
                        format!("Failed to start API container: {}", e)
                    ))?;

                let container_id = container.id().to_string();
                {
                    let mut id_guard = self.container_id.write().await;
                    *id_guard = Some(container_id.clone());
                }

                // Wait for API to be ready
                self.wait_for_api_ready(&container).await?;

                // Get base URL
                let host = container.get_host().await;
                let port = container.get_host_port_ipv4(self.port).await;
                let base_url = format!("http://{}:{}", host, port);
                
                {
                    let mut url_guard = self.base_url.write().await;
                    *url_guard = Some(base_url.clone());
                }

                // Create service handle
                let mut metadata = HashMap::new();
                metadata.insert("container_id".to_string(), container_id);
                metadata.insert("port".to_string(), port.to_string());
                metadata.insert("base_url".to_string(), base_url);
                metadata.insert("health_endpoint".to_string(), self.health_endpoint.clone());

                Ok(ServiceHandle {
                    id: Uuid::new_v4().to_string(),
                    service_name: self.name.clone(),
                    metadata,
                })
            })
        })
    }

    fn stop(&self, handle: ServiceHandle) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _span = tracing::info_span!("api_stop", plugin = self.name);
                
                let container_id = handle.metadata.get("container_id")
                    .ok_or_else(|| CleanroomError::internal_error(
                        "Service handle missing container_id"
                    ))?;

                tracing::info!("Stopping API container: {}", container_id);
                
                // Clear state
                {
                    let mut id_guard = self.container_id.write().await;
                    *id_guard = None;
                }
                
                {
                    let mut url_guard = self.base_url.write().await;
                    *url_guard = None;
                }

                Ok(())
            })
        })
    }

    fn health_check(&self, handle: &ServiceHandle) -> Result<HealthStatus> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _span = tracing::info_span!("api_health_check", plugin = self.name);
                
                // Get base URL
                let base_url = {
                    let url_guard = self.base_url.read().await;
                    url_guard.clone()
                };
                
                if base_url.is_none() {
                    return Ok(HealthStatus::Unhealthy);
                }

                // Perform health check
                // In a real implementation, you would make an HTTP request
                tracing::debug!("API health check passed");
                Ok(HealthStatus::Healthy)
            })
        })
    }
}

impl ApiServicePlugin {
    async fn wait_for_api_ready(&self, container: &testcontainers::Container<GenericImage>) -> Result<()> {
        // Wait for API to be ready
        // In a real implementation, you would poll the health endpoint
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        tracing::info!("API container is ready");
        Ok(())
    }
}
```

## Message Queue Plugin Example

A complete Redis plugin for message queue operations.

```rust
#[derive(Debug)]
pub struct RedisPlugin {
    name: String,
    image: String,
    tag: String,
    port: u16,
    container_id: Arc<RwLock<Option<String>>>,
    connection_url: Arc<RwLock<Option<String>>>,
}

impl RedisPlugin {
    pub fn new(name: &str) -> Result<Self> {
        if name.is_empty() {
            return Err(CleanroomError::validation_error("Plugin name cannot be empty"));
        }

        Ok(Self {
            name: name.to_string(),
            image: "redis".to_string(),
            tag: "7-alpine".to_string(),
            port: 6379,
            container_id: Arc::new(RwLock::new(None)),
            connection_url: Arc::new(RwLock::new(None)),
        })
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.port = port;
        self
    }

    pub fn get_connection_url(&self) -> Result<String> {
        let conn_url = self.connection_url.read().await
            .ok_or_else(|| CleanroomError::internal_error("Connection URL not available"))?;
        Ok(conn_url.clone())
    }
}

impl ServicePlugin for RedisPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _span = tracing::info_span!("redis_start", plugin = self.name);
                
                // Create container
                let image = GenericImage::new(self.image.clone(), self.tag.clone());
                let mut container_request: testcontainers::core::ContainerRequest<GenericImage> =
                    image.into();

                // Expose port
                container_request = container_request.with_exposed_port(self.port);

                // Start container
                let container = container_request.start().await
                    .map_err(|e| CleanroomError::container_error(
                        format!("Failed to start Redis container: {}", e)
                    ))?;

                let container_id = container.id().to_string();
                {
                    let mut id_guard = self.container_id.write().await;
                    *id_guard = Some(container_id.clone());
                }

                // Wait for Redis to be ready
                self.wait_for_redis_ready(&container).await?;

                // Get connection URL
                let host = container.get_host().await;
                let port = container.get_host_port_ipv4(self.port).await;
                let connection_url = format!("redis://{}:{}", host, port);
                
                {
                    let mut url_guard = self.connection_url.write().await;
                    *url_guard = Some(connection_url.clone());
                }

                // Create service handle
                let mut metadata = HashMap::new();
                metadata.insert("container_id".to_string(), container_id);
                metadata.insert("port".to_string(), port.to_string());
                metadata.insert("connection_url".to_string(), connection_url);

                Ok(ServiceHandle {
                    id: Uuid::new_v4().to_string(),
                    service_name: self.name.clone(),
                    metadata,
                })
            })
        })
    }

    fn stop(&self, handle: ServiceHandle) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _span = tracing::info_span!("redis_stop", plugin = self.name);
                
                let container_id = handle.metadata.get("container_id")
                    .ok_or_else(|| CleanroomError::internal_error(
                        "Service handle missing container_id"
                    ))?;

                tracing::info!("Stopping Redis container: {}", container_id);
                
                // Clear state
                {
                    let mut id_guard = self.container_id.write().await;
                    *id_guard = None;
                }
                
                {
                    let mut url_guard = self.connection_url.write().await;
                    *url_guard = None;
                }

                Ok(())
            })
        })
    }

    fn health_check(&self, handle: &ServiceHandle) -> Result<HealthStatus> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let _span = tracing::info_span!("redis_health_check", plugin = self.name);
                
                // Check if connection URL is available
                let conn_url = {
                    let url_guard = self.connection_url.read().await;
                    url_guard.clone()
                };
                
                if conn_url.is_none() {
                    return Ok(HealthStatus::Unhealthy);
                }

                // Perform health check
                // In a real implementation, you would execute a PING command
                tracing::debug!("Redis health check passed");
                Ok(HealthStatus::Healthy)
            })
        })
    }
}

impl RedisPlugin {
    async fn wait_for_redis_ready(&self, container: &testcontainers::Container<GenericImage>) -> Result<()> {
        // Wait for Redis to be ready
        // In a real implementation, you would execute a PING command
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        tracing::info!("Redis container is ready");
        Ok(())
    }
}
```

## Usage Examples

### Using the Database Plugin

```rust
#[tokio::test]
async fn test_postgres_plugin() -> Result<(), CleanroomError> {
    // Create PostgreSQL plugin
    let db_plugin = PostgresPlugin::new("testdb", "testdb")?
        .with_credentials("testuser", "testpass")
        .with_port(5432);

    // Start database
    let handle = db_plugin.start()?;
    
    // Verify health
    let health = db_plugin.health_check(&handle)?;
    assert_eq!(health, HealthStatus::Healthy);
    
    // Get connection string
    let conn_str = db_plugin.get_connection_string()?;
    tracing::info!("Database connection: {}", conn_str);
    
    // Use database in tests
    // ... test logic here ...
    
    // Clean up
    db_plugin.stop(handle)?;
    
    Ok(())
}
```

### Using the API Service Plugin

```rust
#[tokio::test]
async fn test_api_plugin() -> Result<(), CleanroomError> {
    // Create API plugin
    let api_plugin = ApiServicePlugin::new("testapi", "nginx:alpine", 80)?
        .with_health_endpoint("/health", "OK");

    // Start API service
    let handle = api_plugin.start()?;
    
    // Verify health
    let health = api_plugin.health_check(&handle)?;
    assert_eq!(health, HealthStatus::Healthy);
    
    // Get base URL
    let base_url = api_plugin.get_base_url()?;
    tracing::info!("API base URL: {}", base_url);
    
    // Use API in tests
    // ... test logic here ...
    
    // Clean up
    api_plugin.stop(handle)?;
    
    Ok(())
}
```

### Using Multiple Plugins Together

```rust
#[tokio::test]
async fn test_multi_service_integration() -> Result<(), CleanroomError> {
    // Start database
    let db_plugin = PostgresPlugin::new("testdb", "testdb")?;
    let db_handle = db_plugin.start()?;
    
    // Start Redis
    let redis_plugin = RedisPlugin::new("testredis")?;
    let redis_handle = redis_plugin.start()?;
    
    // Start API service
    let api_plugin = ApiServicePlugin::new("testapi", "nginx:alpine", 80)?;
    let api_handle = api_plugin.start()?;
    
    // Verify all services are healthy
    assert_eq!(db_plugin.health_check(&db_handle)?, HealthStatus::Healthy);
    assert_eq!(redis_plugin.health_check(&redis_handle)?, HealthStatus::Healthy);
    assert_eq!(api_plugin.health_check(&api_handle)?, HealthStatus::Healthy);
    
    // Integration test
    let db_conn = db_plugin.get_connection_string()?;
    let redis_url = redis_plugin.get_connection_url()?;
    let api_url = api_plugin.get_base_url()?;
    
    tracing::info!("Integration test with:");
    tracing::info!("  Database: {}", db_conn);
    tracing::info!("  Redis: {}", redis_url);
    tracing::info!("  API: {}", api_url);
    
    // ... integration test logic ...
    
    // Clean up all services
    api_plugin.stop(api_handle)?;
    redis_plugin.stop(redis_handle)?;
    db_plugin.stop(db_handle)?;
    
    Ok(())
}
```

## Testing Your Plugins

### Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_postgres_plugin_creation() -> Result<(), CleanroomError> {
        let plugin = PostgresPlugin::new("testdb", "testdb")?;
        assert_eq!(plugin.name(), "testdb");
        Ok(())
    }

    #[tokio::test]
    async fn test_postgres_plugin_with_credentials() -> Result<(), CleanroomError> {
        let plugin = PostgresPlugin::new("testdb", "testdb")?
            .with_credentials("customuser", "custompass");
        
        // Test would verify credentials are set correctly
        Ok(())
    }

    #[tokio::test]
    async fn test_postgres_plugin_validation() {
        // Test invalid name
        let result = PostgresPlugin::new("", "testdb");
        assert!(result.is_err());
        
        // Test invalid database name
        let result = PostgresPlugin::new("testdb", "");
        assert!(result.is_err());
    }
}
```

### Integration Tests

```rust
#[tokio::test]
async fn test_postgres_plugin_integration() -> Result<(), CleanroomError> {
    let plugin = PostgresPlugin::new("integration-test", "testdb")?;
    
    // Start plugin
    let handle = plugin.start()?;
    
    // Verify container is running
    let container_id = handle.metadata.get("container_id").unwrap();
    tracing::info!("PostgreSQL container ID: {}", container_id);
    
    // Verify connection string is available
    let conn_str = plugin.get_connection_string()?;
    assert!(conn_str.contains("postgresql://"));
    
    // Clean up
    plugin.stop(handle)?;
    
    Ok(())
}
```

## Next Steps

Now that you have complete plugin examples:

1. **Try the examples**: Run the code samples in this chapter
2. **Modify for your needs**: Adapt the examples for your specific use cases
3. **Learn advanced patterns**: Move on to [Advanced Testing Patterns](../advanced-patterns/README.md)
4. **Master templates**: Learn about [Template System Mastery](../template-mastery/README.md)

## Further Reading

- [Creating Custom Plugins](creating-plugins.md)
- [Plugin Lifecycle Management](plugin-lifecycle.md)
- [Plugin Architecture Design](../docs/architecture/plugin_system.md)
- [Core Team Standards](../CLAUDE.md)
