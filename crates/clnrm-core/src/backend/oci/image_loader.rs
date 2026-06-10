//! OCI image loader supporting multiple sources

use super::{ImageCache, OciImage, RegistryClient};
use crate::error::{CleanroomError, Result};
use std::path::PathBuf;
use std::sync::Arc;
use tracing::info;

/// Source for loading OCI images
#[derive(Debug, Clone)]
pub enum ImageSource {
    /// Docker registry (registry.hub.docker.com/library/alpine:latest)
    Registry {
        registry: String,
        repository: String,
        tag: String,
    },
    /// Local OCI directory layout
    Local { path: PathBuf },
    /// Embedded tarball in binary
    Embedded { data: &'static [u8] },
}

/// OCI image loader from multiple sources
#[derive(Debug)]
pub struct OciImageLoader {
    pub cache: Arc<ImageCache>,
    pub registry_client: RegistryClient,
    pub local_store: LocalImageStore,
}

impl OciImageLoader {
    /// Create new image loader
    pub fn new() -> Result<Self> {
        let cache = Arc::new(ImageCache::new(10)?); // 10GB cache
        let registry_client = RegistryClient::new()?;
        let local_store = LocalImageStore::new()?;

        Ok(Self {
            cache,
            registry_client,
            local_store,
        })
    }

    /// Load image from any source
    pub async fn load_image(&self, source: ImageSource) -> Result<OciImage> {
        match source {
            ImageSource::Registry {
                registry,
                repository,
                tag,
            } => {
                // Check cache first
                let image_ref = format!("{}/{}:{}", registry, repository, tag);
                info!("Loading image from registry: {}", image_ref);

                if let Some(cached) = self.cache.get(&image_ref).await? {
                    info!("Image found in cache: {}", image_ref);
                    return Ok(cached);
                }

                // Pull from registry
                info!("Pulling image from registry: {}", image_ref);
                let image = self
                    .registry_client
                    .pull_image(&registry, &repository, &tag)
                    .await?;

                // Cache for future use
                info!("Caching image: {}", image_ref);
                self.cache.store(&image_ref, &image).await?;

                Ok(image)
            }
            ImageSource::Local { path } => {
                info!("Loading image from local path: {}", path.display());
                self.local_store.load_from_path(path).await
            }
            ImageSource::Embedded { data } => {
                info!("Loading image from embedded data ({} bytes)", data.len());
                self.local_store.load_from_tarball(data).await
            }
        }
    }
}

/// Local OCI image store
#[derive(Debug)]
pub struct LocalImageStore {
    temp_dir: PathBuf,
}

impl LocalImageStore {
    /// Create new local store
    pub fn new() -> Result<Self> {
        let temp_dir = std::env::temp_dir().join("clnrm-oci");
        std::fs::create_dir_all(&temp_dir)?;

        Ok(Self { temp_dir })
    }

    /// Load image from local OCI directory
    pub async fn load_from_path(&self, path: PathBuf) -> Result<OciImage> {
        use serde::Deserialize;

        // 1. Read index.json
        let index_path = path.join("index.json");
        let index_content = std::fs::read_to_string(&index_path)
            .map_err(|e| CleanroomError::io_error(format!("Failed to read index.json: {}", e)))?;

        // Deserializing index.json
        #[derive(Deserialize)]
        struct OciIndex {
            manifests: Vec<crate::backend::oci::OciDescriptor>,
        }

        let index: OciIndex = serde_json::from_str(&index_content).map_err(|e| {
            CleanroomError::serialization_error(format!("Failed to parse index.json: {}", e))
        })?;

        let manifest_desc = index.manifests.first().ok_or_else(|| {
            CleanroomError::validation_error("index.json does not contain any manifests")
        })?;

        // 2. Read manifest from blobs/sha256/<hash>
        let manifest_hash = manifest_desc
            .digest
            .strip_prefix("sha256:")
            .ok_or_else(|| {
                CleanroomError::validation_error("Unsupported digest format in index")
            })?;

        let blobs_dir = path.join("blobs").join("sha256");
        let manifest_path = blobs_dir.join(manifest_hash);
        let manifest_content = std::fs::read(&manifest_path).map_err(|e| {
            CleanroomError::io_error(format!("Failed to read manifest file: {}", e))
        })?;

        let manifest: crate::backend::oci::OciManifest = serde_json::from_slice(&manifest_content)
            .map_err(|e| {
                CleanroomError::serialization_error(format!("Failed to parse manifest: {}", e))
            })?;

        // 3. Read config from blobs/sha256/<hash>
        let config_hash = manifest
            .config
            .digest
            .strip_prefix("sha256:")
            .ok_or_else(|| CleanroomError::validation_error("Unsupported config digest format"))?;
        let config_path = blobs_dir.join(config_hash);
        let config_content = std::fs::read(&config_path)
            .map_err(|e| CleanroomError::io_error(format!("Failed to read config file: {}", e)))?;

        let config: crate::backend::oci::OciImageConfig = serde_json::from_slice(&config_content)
            .map_err(|e| {
            CleanroomError::serialization_error(format!("Failed to parse config: {}", e))
        })?;

        // 4. Read and verify layers
        let mut layers = Vec::new();
        for layer_desc in &manifest.layers {
            let layer_hash = layer_desc.digest.strip_prefix("sha256:").ok_or_else(|| {
                CleanroomError::validation_error("Unsupported layer digest format")
            })?;
            let layer_path = blobs_dir.join(layer_hash);
            let layer_content = std::fs::read(&layer_path).map_err(|e| {
                CleanroomError::io_error(format!("Failed to read layer file: {}", e))
            })?;

            // Verify checksum/hash
            use sha2::{Digest, Sha256};
            let mut hasher = Sha256::new();
            hasher.update(&layer_content);
            let computed_hash = hex::encode(hasher.finalize());
            if computed_hash != layer_hash {
                return Err(CleanroomError::validation_error(format!(
                    "layer digest mismatch: expected sha256:{}, got sha256:{}",
                    layer_hash, computed_hash
                )));
            }

            layers.push(crate::backend::oci::OciLayer {
                digest: layer_desc.digest.clone(),
                media_type: layer_desc.media_type.clone(),
                size: layer_desc.size,
                data: layer_content,
            });
        }

        Ok(OciImage {
            manifest,
            config,
            layers,
            config_bytes: config_content,
        })
    }

    /// Load image from tarball
    pub async fn load_from_tarball(&self, data: &[u8]) -> Result<OciImage> {
        let unique_dir = self.temp_dir.join(uuid::Uuid::new_v4().to_string());
        std::fs::create_dir_all(&unique_dir)?;

        let mut archive = tar::Archive::new(data);
        archive
            .unpack(&unique_dir)
            .map_err(|e| CleanroomError::io_error(format!("Failed to unpack tarball: {}", e)))?;

        self.load_from_path(unique_dir).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_source_creation() {
        let source = ImageSource::Registry {
            registry: "registry-1.docker.io".to_string(),
            repository: "library/alpine".to_string(),
            tag: "latest".to_string(),
        };

        match source {
            ImageSource::Registry {
                registry,
                repository,
                tag,
            } => {
                assert_eq!(registry, "registry-1.docker.io");
                assert_eq!(repository, "library/alpine");
                assert_eq!(tag, "latest");
            }
            _ => panic!("Expected registry source"),
        }
    }
}
