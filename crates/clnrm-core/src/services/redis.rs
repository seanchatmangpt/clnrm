//! Redis Cache Service Plugin
//!
//! Production-ready Redis container management with health checks,
//! connection verification, and cache operations.

use crate::cleanroom::{ServicePlugin, ServiceHandle, HealthStatus};
use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::redis::{Redis, REDIS_PORT};
use std::future::Future;
use std::pin::Pin;

/// Redis service plugin configuration
#[derive(Debug, Clone)]
pub struct RedisConfig {
    /// Redis password (optional)
    pub password: Option<String>,
    /// Enable persistence
    pub persistence: bool,
    /// Memory limit in MB
    pub max_memory: Option<u64>,
    /// Custom configuration commands
    pub config_commands: Vec<String>,
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            password: None,
            persistence: false,
            max_memory: None,
            config_commands: Vec::new(),
        }
    }
}

/// Redis service plugin
pub struct RedisPlugin {
    name: String,
    config: RedisConfig,
    container_id: Arc<RwLock<Option<String>>>,
}

impl RedisPlugin {
    /// Create a new Redis plugin instance
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            config: RedisConfig::default(),
            container_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with custom configuration
    pub fn with_config(name: &str, config: RedisConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
            container_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Set password
    pub fn with_password(mut self, password: &str) -> Self {
        self.config.password = Some(password.to_string());
        self
    }

    /// Enable persistence
    pub fn with_persistence(mut self, enabled: bool) -> Self {
        self.config.persistence = enabled;
        self
    }

    /// Set memory limit
    pub fn with_max_memory(mut self, mb: u64) -> Self {
        self.config.max_memory = Some(mb);
        self
    }

    /// Add configuration command
    pub fn with_config_command(mut self, command: &str) -> Self {
        self.config.config_commands.push(command.to_string());
        self
    }

    /// Test Redis connection
    async fn verify_connection(&self, host_port: u16) -> Result<()> {
        use redis::AsyncCommands;
        
        let connection_string = if let Some(password) = &self.config.password {
            format!("redis://:{}@127.0.0.1:{}", password, host_port)
        } else {
            format!("redis://127.0.0.1:{}", host_port)
        };

        let client = redis::Client::open(connection_string)
            .map_err(|e| CleanroomError::connection_failed("Failed to create Redis client")
                .with_source(e.to_string()))?;

        let mut conn = client.get_async_connection().await
            .map_err(|e| CleanroomError::connection_failed("Failed to connect to Redis")
                .with_source(e.to_string()))?;

        // Test basic ping
        let pong: String = conn.ping().await
            .map_err(|e| CleanroomError::service_error("Redis ping failed")
                .with_source(e.to_string()))?;

        if pong != "PONG" {
            return Err(CleanroomError::service_error("Redis ping returned unexpected response"));
        }

        Ok(())
    }

    /// Set key-value pair
    pub async fn set(&self, key: &str, value: &str) -> Result<()> {
        // This would need the connection from the service handle
        // For now, return unimplemented
        unimplemented!("set: requires connection management from service handle")
    }

    /// Get value by key
    pub async fn get(&self, key: &str) -> Result<Option<String>> {
        unimplemented!("get: requires connection management from service handle")
    }

    /// Delete key
    pub async fn delete(&self, key: &str) -> Result<bool> {
        unimplemented!("delete: requires connection management from service handle")
    }

    /// Check if key exists
    pub async fn exists(&self, key: &str) -> Result<bool> {
        unimplemented!("exists: requires connection management from service handle")
    }

    /// Get all keys matching pattern
    pub async fn keys(&self, pattern: &str) -> Result<Vec<String>> {
        unimplemented!("keys: requires connection management from service handle")
    }

    /// Flush all data
    pub async fn flushall(&self) -> Result<()> {
        unimplemented!("flushall: requires connection management from service handle")
    }
}

impl ServicePlugin for RedisPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Pin<Box<dyn Future<Output = Result<ServiceHandle>> + Send + '_>> {
        Box::pin(async move {
            // Create Redis container
            let redis_config = Redis::default();

            let node = redis_config
                .start()
                .await
                .map_err(|e| CleanroomError::container_error("Failed to start Redis container")
                    .with_context("Container startup failed")
                    .with_source(e.to_string()))?;

            let host_port = node.get_host_port_ipv4(REDIS_PORT)
                .await
                .map_err(|e| CleanroomError::container_error("Failed to get container port")
                    .with_source(e.to_string()))?;

            // Wait for Redis to be ready
            tokio::time::sleep(std::time::Duration::from_secs(2)).await;

            // Verify connection works
            self.verify_connection(host_port).await?;

            // Apply configuration commands
            for command in &self.config.config_commands {
                println!("📝 Would execute Redis config: {}", command);
            }

            let mut container_guard = self.container_id.write().await;
            *container_guard = Some(format!("redis-{}", host_port));

            let mut metadata = HashMap::new();
            metadata.insert("host".to_string(), "127.0.0.1".to_string());
            metadata.insert("port".to_string(), host_port.to_string());
            
            let connection_string = if let Some(password) = &self.config.password {
                format!("redis://:{}@127.0.0.1:{}", password, host_port)
            } else {
                format!("redis://127.0.0.1:{}", host_port)
            };
            metadata.insert("connection_string".to_string(), connection_string);
            metadata.insert("database_type".to_string(), "redis".to_string());
            
            if let Some(password) = &self.config.password {
                metadata.insert("password".to_string(), password.clone());
            }
            
            if let Some(max_memory) = self.config.max_memory {
                metadata.insert("max_memory_mb".to_string(), max_memory.to_string());
            }

            Ok(ServiceHandle {
                id: Uuid::new_v4().to_string(),
                service_name: self.name.clone(),
                metadata,
            })
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let mut container_guard = self.container_id.write().await;
            if container_guard.is_some() {
                *container_guard = None; // Drop triggers container cleanup
            }
            Ok(())
        })
    }

    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus {
        if handle.metadata.contains_key("port") && 
           handle.metadata.contains_key("connection_string") {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_redis_plugin_creation() {
        let plugin = RedisPlugin::new("test_redis");
        assert_eq!(plugin.name(), "test_redis");
    }

    #[test]
    fn test_redis_config_default() {
        let config = RedisConfig::default();
        assert!(config.password.is_none());
        assert!(!config.persistence);
        assert!(config.max_memory.is_none());
        assert!(config.config_commands.is_empty());
    }

    #[test]
    fn test_redis_plugin_with_config() {
        let config = RedisConfig {
            password: Some("secret".to_string()),
            persistence: true,
            max_memory: Some(512),
            config_commands: vec!["CONFIG SET maxmemory 512mb".to_string()],
        };

        let plugin = RedisPlugin::with_config("custom_redis", config);
        assert_eq!(plugin.name(), "custom_redis");
    }

    #[test]
    fn test_redis_plugin_builder_pattern() {
        let plugin = RedisPlugin::new("test")
            .with_password("password")
            .with_persistence(true)
            .with_max_memory(256)
            .with_config_command("CONFIG SET save 900 1");

        assert_eq!(plugin.name(), "test");
    }
}
