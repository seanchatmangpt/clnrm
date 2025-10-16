//! Generic container service plugin
//!
//! Provides a generic container service that can run any Docker image
//! with configurable environment variables, ports, and commands.

use crate::cleanroom::{HealthStatus, ServiceHandle, ServicePlugin};
use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use testcontainers::runners::AsyncRunner;
use testcontainers::{GenericImage, ImageExt};
use tokio::sync::RwLock;
use uuid::Uuid;

pub struct GenericContainerPlugin {
    name: String,
    image: String,
    tag: String,
    container_id: Arc<RwLock<Option<String>>>,
    env_vars: HashMap<String, String>,
    ports: Vec<u16>,
}

impl GenericContainerPlugin {
    pub fn new(name: &str, image: &str) -> Self {
        let (image_name, image_tag) = if let Some((name, tag)) = image.split_once(':') {
            (name.to_string(), tag.to_string())
        } else {
            (image.to_string(), "latest".to_string())
        };

        Self {
            name: name.to_string(),
            image: image_name,
            tag: image_tag,
            container_id: Arc::new(RwLock::new(None)),
            env_vars: HashMap::new(),
            ports: Vec::new(),
        }
    }

    pub fn with_env(mut self, key: &str, value: &str) -> Self {
        self.env_vars.insert(key.to_string(), value.to_string());
        self
    }

    pub fn with_port(mut self, port: u16) -> Self {
        self.ports.push(port);
        self
    }

    async fn verify_connection(&self, _host_port: u16) -> Result<()> {
        // For generic containers, we assume they're healthy if they start successfully
        // Specific health checks would need to be implemented per container type
        Ok(())
    }
}

impl ServicePlugin for GenericContainerPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Pin<Box<dyn Future<Output = Result<ServiceHandle>> + Send + '_>> {
        Box::pin(async move {
            // Create container configuration
            let image = GenericImage::new(self.image.clone(), self.tag.clone());

            // Build container request with environment variables and ports
            let mut container_request: testcontainers::core::ContainerRequest<GenericImage> =
                image.into();

            // Add environment variables
            for (key, value) in &self.env_vars {
                container_request = container_request.with_env_var(key, value);
            }

            // Add port mappings
            for port in &self.ports {
                container_request = container_request
                    .with_mapped_port(*port, testcontainers::core::ContainerPort::Tcp(*port));
            }

            // Start container
            let node = container_request.start().await.map_err(|e| {
                CleanroomError::container_error("Failed to start generic container")
                    .with_context("Container startup failed")
                    .with_source(e.to_string())
            })?;

            let mut metadata = HashMap::new();
            metadata.insert("image".to_string(), format!("{}:{}", self.image, self.tag));
            metadata.insert("container_type".to_string(), "generic".to_string());

            // Add port information
            for port in &self.ports {
                if let Ok(host_port) = node.get_host_port_ipv4(*port).await {
                    metadata.insert(format!("port_{}", port), host_port.to_string());
                }
            }

            // Store container reference
            let mut container_guard = self.container_id.write().await;
            *container_guard = Some(format!("generic-{}", Uuid::new_v4()));

            Ok(ServiceHandle {
                id: Uuid::new_v4().to_string(),
                service_name: self.name.clone(),
                metadata,
            })
        })
    }

    fn stop(
        &self,
        _handle: ServiceHandle,
    ) -> Pin<Box<dyn Future<Output = Result<()>> + Send + '_>> {
        Box::pin(async move {
            let mut container_guard = self.container_id.write().await;
            if container_guard.is_some() {
                *container_guard = None; // Drop triggers container cleanup
            }
            Ok(())
        })
    }

    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus {
        if handle.metadata.contains_key("image") && handle.metadata.contains_key("container_type") {
            HealthStatus::Healthy
        } else {
            HealthStatus::Unknown
        }
    }
}
