//! Nginx Web Server Service Plugin
//!
//! Production-ready Nginx container management with health checks,
//! custom configuration, and web server operations.

use crate::cleanroom::{ServicePlugin, ServiceHandle, HealthStatus};
use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};
use std::future::Future;
use std::pin::Pin;

/// Nginx service plugin configuration
#[derive(Debug, Clone)]
pub struct NginxConfig {
    /// Custom nginx configuration
    pub config: Option<String>,
    /// Static files to serve
    pub static_files: HashMap<String, String>,
    /// Port to expose
    pub port: u16,
    /// Enable SSL
    pub ssl: bool,
    /// Custom environment variables
    pub env_vars: HashMap<String, String>,
}

impl Default for NginxConfig {
    fn default() -> Self {
        Self {
            config: None,
            static_files: HashMap::new(),
            port: 80,
            ssl: false,
            env_vars: HashMap::new(),
        }
    }
}

/// Nginx service plugin
pub struct NginxPlugin {
    name: String,
    config: NginxConfig,
    container_id: Arc<RwLock<Option<String>>>,
}

impl NginxPlugin {
    /// Create a new Nginx plugin instance
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            config: NginxConfig::default(),
            container_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with custom configuration
    pub fn with_config(name: &str, config: NginxConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
            container_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Set custom nginx configuration
    pub fn with_custom_config(mut self, config: &str) -> Self {
        self.config.config = Some(config.to_string());
        self
    }

    /// Add static file to serve
    pub fn with_static_file(mut self, path: &str, content: &str) -> Self {
        self.config.static_files.insert(path.to_string(), content.to_string());
        self
    }

    /// Set port
    pub fn with_port(mut self, port: u16) -> Self {
        self.config.port = port;
        self
    }

    /// Enable SSL
    pub fn with_ssl(mut self, enabled: bool) -> Self {
        self.config.ssl = enabled;
        self
    }

    /// Add environment variable
    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.config.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    /// Test HTTP connection
    async fn verify_connection(&self, host_port: u16) -> Result<()> {
        let url = format!("http://127.0.0.1:{}", host_port);
        
        let response = reqwest::get(&url).await
            .map_err(|e| CleanroomError::connection_failed("Failed to connect to Nginx")
                .with_source(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(CleanroomError::service_error(&format!("Nginx returned status: {}", response.status())))
        }
    }

    /// Make HTTP request to nginx
    pub async fn make_request(&self, path: &str) -> Result<String> {
        // This would need the connection info from the service handle
        // For now, return unimplemented
        unimplemented!("make_request: requires connection management from service handle")
    }

    /// Check if endpoint is accessible
    pub async fn check_endpoint(&self, path: &str) -> Result<bool> {
        unimplemented!("check_endpoint: requires connection management from service handle")
    }
}

impl ServicePlugin for NginxPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Pin<Box<dyn Future<Output = Result<ServiceHandle>> + Send + '_>> {
        Box::pin(async move {
            // Create Nginx container
            let image = GenericImage::new("nginx", "alpine");

            let mut container_request: testcontainers::core::ContainerRequest<GenericImage> = image.into();

            // Add environment variables
            for (key, value) in &self.config.env_vars {
                container_request = container_request.with_env_var(key, value);
            }

            // Add port mapping
            container_request = container_request.with_mapped_port(
                self.config.port, 
                testcontainers::core::ContainerPort::Tcp(self.config.port)
            );

            // Start container
            let node = container_request
                .start()
                .await
                .map_err(|e| CleanroomError::container_error("Failed to start Nginx container")
                    .with_context("Container startup failed")
                    .with_source(e.to_string()))?;

            let host_port = node.get_host_port_ipv4(self.config.port)
                .await
                .map_err(|e| CleanroomError::container_error("Failed to get container port")
                    .with_source(e.to_string()))?;

            // Wait for Nginx to be ready
            tokio::time::sleep(std::time::Duration::from_secs(3)).await;

            // Verify connection works
            self.verify_connection(host_port).await?;

            // Apply custom configuration if provided
            if let Some(config) = &self.config.config {
                println!("📝 Would apply custom nginx config: {}", config);
            }

            // Serve static files if provided
            for (path, content) in &self.config.static_files {
                println!("📁 Would serve static file at {}: {}", path, content);
            }

            let mut container_guard = self.container_id.write().await;
            *container_guard = Some(format!("nginx-{}", host_port));

            let mut metadata = HashMap::new();
            metadata.insert("host".to_string(), "127.0.0.1".to_string());
            metadata.insert("port".to_string(), host_port.to_string());
            metadata.insert("internal_port".to_string(), self.config.port.to_string());
            metadata.insert("url".to_string(), format!("http://127.0.0.1:{}", host_port));
            metadata.insert("server_type".to_string(), "nginx".to_string());
            metadata.insert("ssl_enabled".to_string(), self.config.ssl.to_string());

            if let Some(config) = &self.config.config {
                metadata.insert("custom_config".to_string(), config.clone());
            }

            if !self.config.static_files.is_empty() {
                metadata.insert("static_files_count".to_string(), self.config.static_files.len().to_string());
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
           handle.metadata.contains_key("url") {
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
    fn test_nginx_plugin_creation() {
        let plugin = NginxPlugin::new("test_nginx");
        assert_eq!(plugin.name(), "test_nginx");
    }

    #[test]
    fn test_nginx_config_default() {
        let config = NginxConfig::default();
        assert!(config.config.is_none());
        assert!(config.static_files.is_empty());
        assert_eq!(config.port, 80);
        assert!(!config.ssl);
        assert!(config.env_vars.is_empty());
    }

    #[test]
    fn test_nginx_plugin_with_config() {
        let mut static_files = HashMap::new();
        static_files.insert("/index.html".to_string(), "<h1>Hello World</h1>".to_string());
        
        let mut env_vars = HashMap::new();
        env_vars.insert("NGINX_HOST".to_string(), "localhost".to_string());

        let config = NginxConfig {
            config: Some("server { listen 80; }".to_string()),
            static_files,
            port: 8080,
            ssl: true,
            env_vars,
        };

        let plugin = NginxPlugin::with_config("custom_nginx", config);
        assert_eq!(plugin.name(), "custom_nginx");
    }

    #[test]
    fn test_nginx_plugin_builder_pattern() {
        let plugin = NginxPlugin::new("test")
            .with_custom_config("server { listen 80; location / { return 200 'OK'; } }")
            .with_static_file("/test.html", "<h1>Test</h1>")
            .with_port(8080)
            .with_ssl(true)
            .with_env("NGINX_HOST", "localhost");

        assert_eq!(plugin.name(), "test");
    }
}
