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
        // TODO: Implement OCI directory layout parsing
        // See: https://github.com/opencontainers/image-spec/blob/main/image-layout.md
        Err(CleanroomError::not_implemented(
            "Local OCI directory loading not yet implemented",
        ))
    }

    /// Load image from embedded tarball
    pub async fn load_from_tarball(&self, data: &[u8]) -> Result<OciImage> {
        // TODO: Implement tarball extraction and parsing
        Err(CleanroomError::not_implemented(
            "Embedded tarball loading not yet implemented",
        ))
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
