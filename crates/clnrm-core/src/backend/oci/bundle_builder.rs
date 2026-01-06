//! OCI bundle builder for runsc execution

use super::{ConfigParser, LayerManager, OciImage, RuntimeConfig};
use crate::backend::Cmd;
use crate::error::Result;
use std::path::PathBuf;
use tracing::info;

/// OCI bundle ready for runsc
#[derive(Debug, Clone)]
pub struct OciBundle {
    pub id: String,
    pub path: PathBuf,
    pub rootfs: PathBuf,
    pub config: RuntimeConfig,
}

/// OCI bundle builder for runsc
pub struct OciBundleBuilder {
    pub layer_manager: LayerManager,
    pub config_parser: ConfigParser,
    pub bundle_dir: PathBuf,
}

impl OciBundleBuilder {
    /// Create new bundle builder
    pub fn new() -> Result<Self> {
        let bundle_dir = dirs::cache_dir()
            .ok_or_else(|| {
                crate::error::CleanroomError::runtime_error("Failed to get cache directory")
            })?
            .join("clnrm")
            .join("bundles");

        std::fs::create_dir_all(&bundle_dir)?;

        Ok(Self {
            layer_manager: LayerManager::new()?,
            config_parser: ConfigParser,
            bundle_dir,
        })
    }

    /// Create OCI bundle from image
    pub async fn create_bundle(
        &self,
        image: &OciImage,
        cmd: Option<&Cmd>,
    ) -> Result<OciBundle> {
        // Create unique bundle directory
        let bundle_id = uuid::Uuid::new_v4().to_string();
        let bundle_path = self.bundle_dir.join(&bundle_id);
        tokio::fs::create_dir_all(&bundle_path).await?;

        info!("Creating OCI bundle at: {}", bundle_path.display());

        // 1. Extract rootfs
        let rootfs_path = self
            .layer_manager
            .extract_rootfs(&image.layers, &bundle_path)
            .await?;

        info!("Rootfs extracted to: {}", rootfs_path.display());

        // 2. Generate runtime config.json
        let runtime_config = self.config_parser.to_runtime_config(&image.config, cmd)?;

        let config_path = bundle_path.join("config.json");
        let config_json = serde_json::to_string_pretty(&runtime_config)?;
        tokio::fs::write(&config_path, config_json).await?;

        info!("Runtime config written to: {}", config_path.display());

        Ok(OciBundle {
            id: bundle_id,
            path: bundle_path,
            rootfs: rootfs_path,
            config: runtime_config,
        })
    }

    /// Clean up bundle directory
    pub async fn cleanup_bundle(&self, bundle: &OciBundle) -> Result<()> {
        info!("Cleaning up bundle: {}", bundle.path.display());
        tokio::fs::remove_dir_all(&bundle.path).await?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bundle_builder_creation() {
        let builder = OciBundleBuilder::new().unwrap();
        assert!(builder.bundle_dir.exists());
    }
}
