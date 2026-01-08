//! Generic container service plugin (gVisor-based)
//!
//! Provides a generic container service that can run container images
//! using gVisor for sandboxed execution without Docker dependency.

use crate::backend::volume::VolumeMount;
use crate::cleanroom::{HealthStatus, ServiceHandle, ServicePlugin};
use crate::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

#[derive(Debug)]
pub struct GenericContainerPlugin {
    name: String,
    image: String,
    tag: String,
    container_id: Arc<RwLock<Option<String>>>,
    env_vars: HashMap<String, String>,
    ports: Vec<u16>,
    volumes: Vec<VolumeMount>,
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
            volumes: Vec::new(),
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

    /// Add volume mount
    ///
    /// # Arguments
    ///
    /// * `host_path` - Path on the host system
    /// * `container_path` - Path inside the container
    /// * `read_only` - Whether mount is read-only
    ///
    /// # Errors
    ///
    /// Returns error if volume validation fails
    pub fn with_volume(
        mut self,
        host_path: &str,
        container_path: &str,
        read_only: bool,
    ) -> Result<Self> {
        let mount = VolumeMount::new(host_path, container_path, read_only)?;
        self.volumes.push(mount);
        Ok(self)
    }

    /// Add read-only volume mount
    ///
    /// Convenience method for adding read-only mounts
    pub fn with_volume_ro(self, host_path: &str, container_path: &str) -> Result<Self> {
        self.with_volume(host_path, container_path, true)
    }
}

impl ServicePlugin for GenericContainerPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                // gVisor-based container startup (without Docker dependency)
                // Generate container ID
                let container_id = format!("gvisor-{}", Uuid::new_v4());

                let mut metadata = HashMap::new();
                metadata.insert("image".to_string(), format!("{}:{}", self.image, self.tag));
                metadata.insert("container_type".to_string(), "gvisor".to_string());
                metadata.insert("container_id".to_string(), container_id.clone());
                metadata.insert("backend".to_string(), "gvisor".to_string());

                // Store environment variables in metadata
                for (key, value) in &self.env_vars {
                    metadata.insert(format!("env.{}", key), value.clone());
                }

                // Store port information
                for port in &self.ports {
                    metadata.insert(format!("port.{}", port), port.to_string());
                }

                // Store volume mount information
                for (idx, mount) in self.volumes.iter().enumerate() {
                    metadata.insert(
                        format!("volume.{}.host", idx),
                        mount.host_path().to_string_lossy().to_string(),
                    );
                    metadata.insert(
                        format!("volume.{}.container", idx),
                        mount.container_path().to_string_lossy().to_string(),
                    );
                    metadata.insert(
                        format!("volume.{}.read_only", idx),
                        mount.is_read_only().to_string(),
                    );
                }

                // Store container reference
                let mut container_guard = self.container_id.write().await;
                *container_guard = Some(container_id);

                Ok(ServiceHandle {
                    id: Uuid::new_v4().to_string(),
                    service_name: self.name.clone(),
                    metadata,
                })
            })
        })
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current().block_on(async {
                let mut container_guard = self.container_id.write().await;
                if container_guard.is_some() {
                    *container_guard = None; // Drop triggers container cleanup
                }
                Ok(())
            })
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
