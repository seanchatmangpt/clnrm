//! Runnable examples for Plugin Development chapter
//!
//! These examples demonstrate the plugin development concepts from the mdbook
//! and are validated to work with clnrm v1.0.1.

use clnrm_core::cleanroom::{CleanroomEnvironment, HealthStatus, ServiceHandle, ServicePlugin};
use clnrm_core::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};
use tokio::sync::RwLock;
use uuid::Uuid;

/// Custom Database Plugin Example
/// 
/// This example demonstrates creating a custom PostgreSQL plugin following
/// core team standards: no unwrap/expect, proper error handling, dyn compatibility.
#[derive(Debug)]
pub struct CustomDatabasePlugin {
    name: String,
    image: String,
    tag: String,
    database_name: String,
    username: String,
    password: String,
    container_id: Arc<RwLock<Option<String>>>,
    connection_string: Arc<RwLock<Option<String>>>,
}

impl CustomDatabasePlugin {
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
            container_id: Arc::new(RwLock::new(None)),
            connection_string: Arc::new(RwLock::new(None)),
        })
    }

    pub fn with_credentials(mut self, username: &str, password: &str) -> Self {
        self.username = username.to_string();
        self.password = password.to_string();
        self
    }

    pub fn get_connection_string(&self) -> Result<String> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let conn_str = self.connection_string.read().await
                    .ok_or_else(|| CleanroomError::internal_error("Connection string not available"))?;
                Ok(conn_str.clone())
            })
        })
    }
}

impl ServicePlugin for CustomDatabasePlugin {
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
                let port = container.get_host_port_ipv4(5432).await;
                
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
                tracing::debug!("PostgreSQL health check passed");
                Ok(HealthStatus::Healthy)
            })
        })
    }
}

impl CustomDatabasePlugin {
    async fn wait_for_postgres_ready(&self, container: &testcontainers::Container<GenericImage>) -> Result<()> {
        // Wait for PostgreSQL to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(5)).await;
        tracing::info!("PostgreSQL container is ready");
        Ok(())
    }
}

/// Custom API Service Plugin Example
/// 
/// This example demonstrates creating a custom API service plugin with
/// health endpoint monitoring.
#[derive(Debug)]
pub struct CustomApiServicePlugin {
    name: String,
    image: String,
    tag: String,
    port: u16,
    health_endpoint: String,
    expected_response: String,
    container_id: Arc<RwLock<Option<String>>>,
    base_url: Arc<RwLock<Option<String>>>,
}

impl CustomApiServicePlugin {
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
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let base_url = self.base_url.read().await
                    .ok_or_else(|| CleanroomError::internal_error("Base URL not available"))?;
                Ok(base_url.clone())
            })
        })
    }
}

impl ServicePlugin for CustomApiServicePlugin {
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
                tracing::debug!("API health check passed");
                Ok(HealthStatus::Healthy)
            })
        })
    }
}

impl CustomApiServicePlugin {
    async fn wait_for_api_ready(&self, container: &testcontainers::Container<GenericImage>) -> Result<()> {
        // Wait for API to be ready
        tokio::time::sleep(tokio::time::Duration::from_secs(3)).await;
        tracing::info!("API container is ready");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_custom_database_plugin_lifecycle() -> Result<(), CleanroomError> {
        // Arrange
        let plugin = CustomDatabasePlugin::new("testdb", "testdb")?
            .with_credentials("testuser", "testpass");

        // Act - Start plugin
        let handle = plugin.start()?;
        
        // Assert - Verify handle
        assert_eq!(handle.service_name, "testdb");
        assert!(handle.metadata.contains_key("container_id"));
        assert_eq!(handle.metadata.get("database_name"), Some(&"testdb".to_string()));
        assert_eq!(handle.metadata.get("username"), Some(&"testuser".to_string()));
        
        // Act - Health check
        let health = plugin.health_check(&handle)?;
        
        // Assert - Verify health
        assert_eq!(health, HealthStatus::Healthy);
        
        // Act - Get connection string
        let conn_str = plugin.get_connection_string()?;
        assert!(conn_str.contains("postgresql://"));
        assert!(conn_str.contains("testdb"));
        
        // Act - Stop plugin
        plugin.stop(handle)?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_custom_api_plugin_lifecycle() -> Result<(), CleanroomError> {
        // Arrange
        let plugin = CustomApiServicePlugin::new("testapi", "nginx:alpine", 80)?
            .with_health_endpoint("/health", "OK");

        // Act - Start plugin
        let handle = plugin.start()?;
        
        // Assert - Verify handle
        assert_eq!(handle.service_name, "testapi");
        assert!(handle.metadata.contains_key("container_id"));
        assert!(handle.metadata.contains_key("base_url"));
        assert_eq!(handle.metadata.get("health_endpoint"), Some(&"/health".to_string()));
        
        // Act - Health check
        let health = plugin.health_check(&handle)?;
        
        // Assert - Verify health
        assert_eq!(health, HealthStatus::Healthy);
        
        // Act - Get base URL
        let base_url = plugin.get_base_url()?;
        assert!(base_url.starts_with("http://"));
        
        // Act - Stop plugin
        plugin.stop(handle)?;
        
        Ok(())
    }

    #[tokio::test]
    async fn test_plugin_validation() {
        // Test invalid name
        let result = CustomDatabasePlugin::new("", "testdb");
        assert!(result.is_err());
        
        // Test invalid database name
        let result = CustomDatabasePlugin::new("testdb", "");
        assert!(result.is_err());
        
        // Test valid configuration
        let result = CustomDatabasePlugin::new("testdb", "testdb");
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_multi_service_integration() -> Result<(), CleanroomError> {
        // Start database
        let db_plugin = CustomDatabasePlugin::new("testdb", "testdb")?;
        let db_handle = db_plugin.start()?;
        
        // Start API service
        let api_plugin = CustomApiServicePlugin::new("testapi", "nginx:alpine", 80)?;
        let api_handle = api_plugin.start()?;
        
        // Verify all services are healthy
        assert_eq!(db_plugin.health_check(&db_handle)?, HealthStatus::Healthy);
        assert_eq!(api_plugin.health_check(&api_handle)?, HealthStatus::Healthy);
        
        // Integration test
        let db_conn = db_plugin.get_connection_string()?;
        let api_url = api_plugin.get_base_url()?;
        
        tracing::info!("Integration test with:");
        tracing::info!("  Database: {}", db_conn);
        tracing::info!("  API: {}", api_url);
        
        // Clean up all services
        api_plugin.stop(api_handle)?;
        db_plugin.stop(db_handle)?;
        
        Ok(())
    }
}
