//! OCI image and bundle management
//!
//! Handles OCI image pulling, extraction, and bundle creation for gVisor.

use crate::error::{CleanroomError, Result};
use crate::service::definition::ImageRef;
use std::path::PathBuf;
use tracing::info;

/// OCI image manager
pub struct OciImageManager {
    /// Cache directory for OCI images
    cache_dir: PathBuf,
    /// Real OCI image loader backend
    image_loader: crate::backend::OciImageLoader,
}

impl OciImageManager {
    /// Create new OCI image manager
    pub fn new() -> Result<Self> {
        let cache_dir = std::env::temp_dir().join("clnrm-oci-cache");
        std::fs::create_dir_all(&cache_dir).map_err(|e| {
            CleanroomError::container_error(format!("Failed to create OCI cache directory: {}", e))
        })?;

        let image_loader = crate::backend::OciImageLoader::new()?;

        Ok(Self {
            cache_dir,
            image_loader,
        })
    }

    /// Create with custom cache directory
    pub fn with_cache_dir(cache_dir: PathBuf) -> Result<Self> {
        std::fs::create_dir_all(&cache_dir).map_err(|e| {
            CleanroomError::container_error(format!("Failed to create OCI cache directory: {}", e))
        })?;

        let image_loader = crate::backend::OciImageLoader::new()?;

        Ok(Self {
            cache_dir,
            image_loader,
        })
    }

    /// Pull OCI image
    pub async fn pull_image(&self, image: &ImageRef) -> Result<PathBuf> {
        let image_str = image.to_string();
        info!("Pulling OCI image: {}", image_str);

        // Create image-specific directory
        let image_dir = self.cache_dir.join(format!(
            "{}-{}",
            image.repository.replace('/', "-"),
            image.tag
        ));

        // Check if image already cached
        if image_dir.exists() {
            info!("Image already cached: {}", image_str);
            return Ok(image_dir);
        }

        let registry = image
            .registry
            .clone()
            .unwrap_or_else(|| "registry-1.docker.io".to_string());
        let source = crate::backend::ImageSource::Registry {
            registry,
            repository: image.repository.clone(),
            tag: image.tag.clone(),
        };

        // Real image pull
        let oci_image = self.image_loader.load_image(source).await?;

        std::fs::create_dir_all(&image_dir).map_err(|e| {
            CleanroomError::container_error(format!("Failed to create image directory: {}", e))
        })?;

        // Write configuration json
        let config_path = image_dir.join("config.json");
        let config_str = serde_json::to_string_pretty(&oci_image.config).map_err(|e| {
            CleanroomError::container_error(format!("Failed to serialize OCI config: {}", e))
        })?;
        std::fs::write(config_path, config_str).map_err(|e| {
            CleanroomError::container_error(format!("Failed to write OCI config file: {}", e))
        })?;

        std::fs::create_dir_all(image_dir.join("rootfs")).map_err(|e| {
            CleanroomError::container_error(format!("Failed to create rootfs directory: {}", e))
        })?;

        Ok(image_dir)
    }

    /// Create OCI bundle for container
    pub async fn create_bundle(&self, image: &ImageRef, bundle_dir: PathBuf) -> Result<PathBuf> {
        info!("Creating OCI bundle for image: {}", image.to_string());

        // Pull image first
        let image_dir = self.pull_image(image).await?;

        // Create bundle directory structure
        std::fs::create_dir_all(&bundle_dir).map_err(|e| {
            CleanroomError::container_error(format!("Failed to create bundle directory: {}", e))
        })?;

        let rootfs_dir = bundle_dir.join("rootfs");
        std::fs::create_dir_all(&rootfs_dir).map_err(|e| {
            CleanroomError::container_error(format!("Failed to create rootfs directory: {}", e))
        })?;

        // Copy config from image directory to bundle directory
        let config_src = image_dir.join("config.json");
        let config_dest = bundle_dir.join("config.json");
        if config_src.exists() {
            tokio::fs::copy(&config_src, &config_dest)
                .await
                .map_err(|e| {
                    CleanroomError::container_error(format!(
                        "Failed to copy config.json to bundle: {}",
                        e
                    ))
                })?;
        }

        Ok(bundle_dir)
    }

    /// Get cached image directory
    pub fn get_cached_image(&self, image: &ImageRef) -> Option<PathBuf> {
        let image_dir = self.cache_dir.join(format!(
            "{}-{}",
            image.repository.replace('/', "-"),
            image.tag
        ));

        if image_dir.exists() {
            Some(image_dir)
        } else {
            None
        }
    }

    /// Clear image cache
    pub fn clear_cache(&self) -> Result<()> {
        std::fs::remove_dir_all(&self.cache_dir).map_err(|e| {
            CleanroomError::container_error(format!("Failed to clear image cache: {}", e))
        })?;

        std::fs::create_dir_all(&self.cache_dir).map_err(|e| {
            CleanroomError::container_error(format!("Failed to recreate cache directory: {}", e))
        })?;

        Ok(())
    }
}

impl Default for OciImageManager {
    fn default() -> Self {
        Self::new().unwrap()
    }
}

/// OCI bundle representation
pub struct OciBundle {
    /// Bundle directory
    pub path: PathBuf,
    /// Rootfs directory
    pub rootfs: PathBuf,
    /// Config file path
    pub config: PathBuf,
}

impl OciBundle {
    /// Create new OCI bundle
    pub fn new(bundle_dir: PathBuf) -> Self {
        let rootfs = bundle_dir.join("rootfs");
        let config = bundle_dir.join("config.json");

        Self {
            path: bundle_dir,
            rootfs,
            config,
        }
    }

    /// Validate bundle structure
    pub fn validate(&self) -> Result<()> {
        if !self.path.exists() {
            return Err(CleanroomError::validation_error(format!(
                "Bundle directory does not exist: {}",
                self.path.display()
            )));
        }

        if !self.rootfs.exists() {
            return Err(CleanroomError::validation_error(format!(
                "Rootfs directory does not exist: {}",
                self.rootfs.display()
            )));
        }

        if !self.config.exists() {
            return Err(CleanroomError::validation_error(format!(
                "Config file does not exist: {}",
                self.config.display()
            )));
        }

        Ok(())
    }

    /// Get bundle path
    pub fn path(&self) -> &PathBuf {
        &self.path
    }

    /// Get rootfs path
    pub fn rootfs(&self) -> &PathBuf {
        &self.rootfs
    }

    /// Get config path
    pub fn config(&self) -> &PathBuf {
        &self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_oci_image_manager_creation() {
        let manager = OciImageManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_oci_bundle_creation() {
        let bundle_dir = std::env::temp_dir().join("test-bundle");
        let bundle = OciBundle::new(bundle_dir);

        assert!(bundle.path.ends_with("test-bundle"));
        assert!(bundle.rootfs.ends_with("rootfs"));
        assert!(bundle.config.ends_with("config.json"));
    }
}
