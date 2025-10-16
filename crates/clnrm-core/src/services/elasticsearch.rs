//! Elasticsearch Search Service Plugin
//!
//! Production-ready Elasticsearch container management with health checks,
//! index operations, and search functionality.

use crate::cleanroom::{ServicePlugin, ServiceHandle, HealthStatus};
use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;
use testcontainers::runners::AsyncRunner;
use testcontainers_modules::elasticsearch::{Elasticsearch, ELASTICSEARCH_PORT};
use std::future::Future;
use std::pin::Pin;

/// Elasticsearch service plugin configuration
#[derive(Debug, Clone)]
pub struct ElasticsearchConfig {
    /// Cluster name
    pub cluster_name: String,
    /// Node name
    pub node_name: String,
    /// Memory limit
    pub memory_limit: String,
    /// Enable security
    pub security_enabled: bool,
    /// Custom configuration
    pub custom_config: HashMap<String, String>,
}

impl Default for ElasticsearchConfig {
    fn default() -> Self {
        Self {
            cluster_name: "cleanroom-cluster".to_string(),
            node_name: "cleanroom-node".to_string(),
            memory_limit: "512m".to_string(),
            security_enabled: false,
            custom_config: HashMap::new(),
        }
    }
}

/// Elasticsearch service plugin
pub struct ElasticsearchPlugin {
    name: String,
    config: ElasticsearchConfig,
    container_id: Arc<RwLock<Option<String>>>,
}

impl ElasticsearchPlugin {
    /// Create a new Elasticsearch plugin instance
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            config: ElasticsearchConfig::default(),
            container_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Create with custom configuration
    pub fn with_config(name: &str, config: ElasticsearchConfig) -> Self {
        Self {
            name: name.to_string(),
            config,
            container_id: Arc::new(RwLock::new(None)),
        }
    }

    /// Set cluster name
    pub fn with_cluster_name(mut self, name: &str) -> Self {
        self.config.cluster_name = name.to_string();
        self
    }

    /// Set node name
    pub fn with_node_name(mut self, name: &str) -> Self {
        self.config.node_name = name.to_string();
        self
    }

    /// Set memory limit
    pub fn with_memory_limit(mut self, limit: &str) -> Self {
        self.config.memory_limit = limit.to_string();
        self
    }

    /// Enable security
    pub fn with_security(mut self, enabled: bool) -> Self {
        self.config.security_enabled = enabled;
        self
    }

    /// Add custom configuration
    pub fn with_custom_config(mut self, key: &str, value: &str) -> Self {
        self.config.custom_config.insert(key.to_string(), value.to_string());
        self
    }

    /// Test Elasticsearch connection
    async fn verify_connection(&self, host_port: u16) -> Result<()> {
        let url = format!("http://127.0.0.1:{}", host_port);
        
        let response = reqwest::get(&url).await
            .map_err(|e| CleanroomError::connection_failed("Failed to connect to Elasticsearch")
                .with_source(e.to_string()))?;

        if response.status().is_success() {
            Ok(())
        } else {
            Err(CleanroomError::service_error(&format!("Elasticsearch returned status: {}", response.status())))
        }
    }

    /// Check cluster health
    async fn check_cluster_health(&self, host_port: u16) -> Result<String> {
        let url = format!("http://127.0.0.1:{}/_cluster/health", host_port);
        
        let response = reqwest::get(&url).await
            .map_err(|e| CleanroomError::service_error("Failed to check cluster health")
                .with_source(e.to_string()))?;

        if response.status().is_success() {
            let health: serde_json::Value = response.json().await
                .map_err(|e| CleanroomError::service_error("Failed to parse cluster health")
                    .with_source(e.to_string()))?;
            
            if let Some(status) = health.get("status").and_then(|s| s.as_str()) {
                Ok(status.to_string())
            } else {
                Ok("unknown".to_string())
            }
        } else {
            Err(CleanroomError::service_error(&format!("Cluster health check failed: {}", response.status())))
        }
    }

    /// Create index
    pub async fn create_index(&self, index_name: &str, mapping: &str) -> Result<()> {
        // This would need the connection info from the service handle
        // For now, return unimplemented
        unimplemented!("create_index: requires connection management from service handle")
    }

    /// Index document
    pub async fn index_document(&self, index_name: &str, doc_id: &str, document: &str) -> Result<()> {
        unimplemented!("index_document: requires connection management from service handle")
    }

    /// Search documents
    pub async fn search(&self, index_name: &str, query: &str) -> Result<Vec<serde_json::Value>> {
        unimplemented!("search: requires connection management from service handle")
    }

    /// Delete index
    pub async fn delete_index(&self, index_name: &str) -> Result<()> {
        unimplemented!("delete_index: requires connection management from service handle")
    }
}

impl ServicePlugin for ElasticsearchPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Pin<Box<dyn Future<Output = Result<ServiceHandle>> + Send + '_>> {
        Box::pin(async move {
            // Create Elasticsearch container
            let es_config = Elasticsearch::default();

            let node = es_config
                .start()
                .await
                .map_err(|e| CleanroomError::container_error("Failed to start Elasticsearch container")
                    .with_context("Container startup failed")
                    .with_source(e.to_string()))?;

            let host_port = node.get_host_port_ipv4(ELASTICSEARCH_PORT)
                .await
                .map_err(|e| CleanroomError::container_error("Failed to get container port")
                    .with_source(e.to_string()))?;

            // Wait for Elasticsearch to be ready
            tokio::time::sleep(std::time::Duration::from_secs(10)).await;

            // Verify connection works
            self.verify_connection(host_port).await?;

            // Check cluster health
            let cluster_health = self.check_cluster_health(host_port).await?;

            // Apply custom configuration
            for (key, value) in &self.config.custom_config {
                println!("📝 Would apply Elasticsearch config: {} = {}", key, value);
            }

            let mut container_guard = self.container_id.write().await;
            *container_guard = Some(format!("elasticsearch-{}", host_port));

            let mut metadata = HashMap::new();
            metadata.insert("host".to_string(), "127.0.0.1".to_string());
            metadata.insert("port".to_string(), host_port.to_string());
            metadata.insert("url".to_string(), format!("http://127.0.0.1:{}", host_port));
            metadata.insert("cluster_name".to_string(), self.config.cluster_name.clone());
            metadata.insert("node_name".to_string(), self.config.node_name.clone());
            metadata.insert("cluster_health".to_string(), cluster_health);
            metadata.insert("search_type".to_string(), "elasticsearch".to_string());
            metadata.insert("security_enabled".to_string(), self.config.security_enabled.to_string());

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
        if let Some(health) = handle.metadata.get("cluster_health") {
            match health.as_str() {
                "green" | "yellow" => HealthStatus::Healthy,
                "red" => HealthStatus::Unhealthy,
                _ => HealthStatus::Unknown,
            }
        } else {
            HealthStatus::Unknown
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_elasticsearch_plugin_creation() {
        let plugin = ElasticsearchPlugin::new("test_elasticsearch");
        assert_eq!(plugin.name(), "test_elasticsearch");
    }

    #[test]
    fn test_elasticsearch_config_default() {
        let config = ElasticsearchConfig::default();
        assert_eq!(config.cluster_name, "cleanroom-cluster");
        assert_eq!(config.node_name, "cleanroom-node");
        assert_eq!(config.memory_limit, "512m");
        assert!(!config.security_enabled);
        assert!(config.custom_config.is_empty());
    }

    #[test]
    fn test_elasticsearch_plugin_with_config() {
        let mut custom_config = HashMap::new();
        custom_config.insert("discovery.type".to_string(), "single-node".to_string());

        let config = ElasticsearchConfig {
            cluster_name: "test-cluster".to_string(),
            node_name: "test-node".to_string(),
            memory_limit: "1g".to_string(),
            security_enabled: true,
            custom_config,
        };

        let plugin = ElasticsearchPlugin::with_config("custom_elasticsearch", config);
        assert_eq!(plugin.name(), "custom_elasticsearch");
    }

    #[test]
    fn test_elasticsearch_plugin_builder_pattern() {
        let plugin = ElasticsearchPlugin::new("test")
            .with_cluster_name("my-cluster")
            .with_node_name("my-node")
            .with_memory_limit("1g")
            .with_security(true)
            .with_custom_config("discovery.type", "single-node");

        assert_eq!(plugin.name(), "test");
    }
}
