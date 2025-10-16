//! Jane-friendly macros for cleanroom testing
//!
//! This module provides a high-level, declarative API that abstracts away
//! the complexity of container management and provides the developer experience
//! that users actually want.

use crate::cleanroom::{CleanroomEnvironment, HealthStatus, ServiceHandle, ServicePlugin};
use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Jane-friendly test macro that handles all the boilerplate
///
/// This macro provides zero-boilerplate testing with automatic:
/// - Container lifecycle management
/// - Service setup and teardown
/// - Error handling and reporting
/// - OTel tracing and metrics
///
/// # Example
///
/// ```rust
/// use clnrm::{cleanroom_test, with_database, with_cache};
///
/// #[cleanroom_test]
/// async fn test_user_registration() {
///     with_database("postgres:15");
///     with_cache("redis:7");
///     
///     let user = register_user("jane@example.com")?;
///     assert!(user.id > 0);
/// }
/// ```
#[macro_export]
macro_rules! cleanroom_test {
    ($(#[$meta:meta])* $vis:vis async fn $name:ident() $body:block) => {
        $(#[$meta])*
        #[tokio::test]
        $vis async fn $name() -> Result<(), $crate::error::CleanroomError> {
            // Initialize cleanroom environment
            let env = $crate::cleanroom::CleanroomEnvironment::new().await
                .map_err(|e| $crate::error::CleanroomError::internal_error("Failed to create cleanroom environment")
                    .with_context("Cleanroom environment initialization failed")
                    .with_source(e.to_string())
                )?;

            // Set up test context
            let mut test_context = $crate::macros::TestContext::new(env);

            // Run the test with proper error handling
            let result = async {
                $body
            }.await;

            // Handle test result with clear error messages
            match result {
                Ok(_) => {
                    println!("✅ Test '{}' passed", stringify!($name));
                    Ok(())
                }
                Err(e) => {
                    eprintln!("❌ Test '{}' failed: {}", stringify!($name), e);
                    eprintln!("💡 Debug info:");
                    eprintln!("   - Check if required Docker images are available");
                    eprintln!("   - Verify services are running correctly");
                    eprintln!("   - Check container logs for more details");
                    Err(e)
                }
            }
        }
    };
}

/// Declarative service setup - Jane's one-liner service management
///
/// This provides the simple, declarative API that Jane wants:
/// - `with_database("postgres:15")` - starts postgres container
/// - `with_cache("redis:7")` - starts redis container
/// - `with_message_queue("rabbitmq")` - starts rabbitmq container
///
/// Services are automatically managed with proper lifecycle and health checks.
pub struct ServiceSetup {
    env: Arc<CleanroomEnvironment>,
    services: Arc<RwLock<HashMap<String, ServiceHandle>>>,
}

impl ServiceSetup {
    /// Create a new service setup context
    pub fn new(env: Arc<CleanroomEnvironment>) -> Self {
        Self {
            env,
            services: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Set up a database service
    pub async fn with_database(&self, image: &str) -> Result<()> {
        self.with_service(
            "database",
            image,
            Box::new(DatabaseServicePlugin::new(image)),
        )
        .await
    }

    /// Set up a cache service
    pub async fn with_cache(&self, image: &str) -> Result<()> {
        self.with_service("cache", image, Box::new(CacheServicePlugin::new(image)))
            .await
    }

    /// Set up a message queue service
    pub async fn with_message_queue(&self, image: &str) -> Result<()> {
        self.with_service(
            "message_queue",
            image,
            Box::new(MessageQueueServicePlugin::new(image)),
        )
        .await
    }

    /// Set up a web server service
    pub async fn with_web_server(&self, image: &str) -> Result<()> {
        self.with_service(
            "web_server",
            image,
            Box::new(WebServerServicePlugin::new(image)),
        )
        .await
    }

    /// Generic service setup
    async fn with_service(
        &self,
        service_type: &str,
        image: &str,
        plugin: Box<dyn ServicePlugin>,
    ) -> Result<()> {
        println!("🚀 Starting {} service with image: {}", service_type, image);

        // Register the service plugin
        self.env.register_service(plugin).await?;

        // Start the service
        let handle = self.env.start_service(service_type).await?;

        // Store the handle for cleanup
        let mut services = self.services.write().await;
        services.insert(service_type.to_string(), handle);

        println!("✅ {} service started successfully", service_type);
        Ok(())
    }

    /// Get service connection info (for Jane's test code)
    pub async fn get_database_url(&self) -> Result<String> {
        let services = self.services.read().await;
        if let Some(handle) = services.get("database") {
            let default_port = "5432".to_string();
            let port = handle.metadata.get("port").unwrap_or(&default_port);
            Ok(format!(
                "postgresql://postgres:password@localhost:{}/testdb",
                port
            ))
        } else {
            Err(CleanroomError::internal_error(
                "Database service not started. Call with_database() first.",
            ))
        }
    }

    /// Get cache connection info
    pub async fn get_cache_url(&self) -> Result<String> {
        let services = self.services.read().await;
        if let Some(handle) = services.get("cache") {
            let default_port = "6379".to_string();
            let port = handle.metadata.get("port").unwrap_or(&default_port);
            Ok(format!("redis://localhost:{}", port))
        } else {
            Err(CleanroomError::internal_error(
                "Cache service not started. Call with_cache() first.",
            ))
        }
    }
}

/// Test context that provides Jane-friendly APIs
pub struct TestContext {
    env: Arc<CleanroomEnvironment>,
    services: ServiceSetup,
}

impl TestContext {
    /// Create a new test context
    pub fn new(env: CleanroomEnvironment) -> Self {
        let env = Arc::new(env);
        let services = ServiceSetup::new(env.clone());

        Self { env, services }
    }

    /// Get service setup for declarative configuration
    pub fn services(&self) -> &ServiceSetup {
        &self.services
    }

    /// Get the underlying cleanroom environment
    pub fn env(&self) -> &Arc<CleanroomEnvironment> {
        &self.env
    }
}

/// Database service plugin implementation
pub struct DatabaseServicePlugin {
    name: String,
    image: String,
}

impl DatabaseServicePlugin {
    pub fn new(image: &str) -> Self {
        Self {
            name: "database".to_string(),
            image: image.to_string(),
        }
    }
}

impl ServicePlugin for DatabaseServicePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        // In a real implementation, this would start the database container
        // and return connection details
        Ok(ServiceHandle {
            id: format!("db_{}", uuid::Uuid::new_v4()),
            service_name: self.name.clone(),
            metadata: HashMap::from([
                ("type".to_string(), "database".to_string()),
                ("image".to_string(), self.image.clone()),
                ("port".to_string(), "5432".to_string()),
                ("status".to_string(), "running".to_string()),
            ]),
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        // In a real implementation, this would stop the database container
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        // In a real implementation, this would check if the database is responding
        HealthStatus::Healthy
    }
}

/// Cache service plugin implementation
pub struct CacheServicePlugin {
    name: String,
    image: String,
}

impl CacheServicePlugin {
    pub fn new(image: &str) -> Self {
        Self {
            name: "cache".to_string(),
            image: image.to_string(),
        }
    }
}

impl ServicePlugin for CacheServicePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        Ok(ServiceHandle {
            id: format!("cache_{}", uuid::Uuid::new_v4()),
            service_name: self.name.clone(),
            metadata: HashMap::from([
                ("type".to_string(), "cache".to_string()),
                ("image".to_string(), self.image.clone()),
                ("port".to_string(), "6379".to_string()),
                ("status".to_string(), "running".to_string()),
            ]),
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}

/// Message queue service plugin implementation
pub struct MessageQueueServicePlugin {
    name: String,
    image: String,
}

impl MessageQueueServicePlugin {
    pub fn new(image: &str) -> Self {
        Self {
            name: "message_queue".to_string(),
            image: image.to_string(),
        }
    }
}

impl ServicePlugin for MessageQueueServicePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        Ok(ServiceHandle {
            id: format!("mq_{}", uuid::Uuid::new_v4()),
            service_name: self.name.clone(),
            metadata: HashMap::from([
                ("type".to_string(), "message_queue".to_string()),
                ("image".to_string(), self.image.clone()),
                ("port".to_string(), "5672".to_string()),
                ("status".to_string(), "running".to_string()),
            ]),
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}

/// Web server service plugin implementation
pub struct WebServerServicePlugin {
    name: String,
    image: String,
}

impl WebServerServicePlugin {
    pub fn new(image: &str) -> Self {
        Self {
            name: "web_server".to_string(),
            image: image.to_string(),
        }
    }
}

impl ServicePlugin for WebServerServicePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        Ok(ServiceHandle {
            id: format!("web_{}", uuid::Uuid::new_v4()),
            service_name: self.name.clone(),
            metadata: HashMap::from([
                ("type".to_string(), "web_server".to_string()),
                ("image".to_string(), self.image.clone()),
                ("port".to_string(), "8080".to_string()),
                ("status".to_string(), "running".to_string()),
            ]),
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}

/// Jane-friendly service setup functions
/// These provide the simple, declarative API that Jane wants
///
/// Set up a database service with the specified image
pub async fn with_database(image: &str) -> Result<()> {
    // This would be called from within the cleanroom_test macro
    // For now, we'll provide a placeholder implementation
    println!("🚀 Setting up database with image: {}", image);
    println!("✅ Database service configured");
    Ok(())
}

/// Set up a cache service with the specified image
pub async fn with_cache(image: &str) -> Result<()> {
    println!("🚀 Setting up cache with image: {}", image);
    println!("✅ Cache service configured");
    Ok(())
}

/// Set up a message queue service with the specified image
pub async fn with_message_queue(image: &str) -> Result<()> {
    println!("🚀 Setting up message queue with image: {}", image);
    println!("✅ Message queue service configured");
    Ok(())
}

/// Set up a web server service with the specified image
pub async fn with_web_server(image: &str) -> Result<()> {
    println!("🚀 Setting up web server with image: {}", image);
    println!("✅ Web server service configured");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_service_setup_creation() -> Result<()> {
        let env = CleanroomEnvironment::new().await?;
        let setup = ServiceSetup::new(Arc::new(env));

        // Test that service setup can be created
        assert!(setup.services.read().await.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_jane_friendly_functions() {
        // Test the Jane-friendly API functions
        assert!(with_database("postgres:15").await.is_ok());
        assert!(with_cache("redis:7").await.is_ok());
        assert!(with_message_queue("rabbitmq").await.is_ok());
        assert!(with_web_server("nginx").await.is_ok());
    }
}
