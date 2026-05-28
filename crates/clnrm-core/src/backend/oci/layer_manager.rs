//! OCI layer extraction and rootfs merging

use super::OciLayer;
use crate::error::{CleanroomError, Result};
use flate2::read::GzDecoder;
use std::path::{Path, PathBuf};
use tar::Archive;
use tracing::{info, warn};

/// Manages OCI layer extraction and merging
#[derive(Debug)]
pub struct LayerManager {
    cache_dir: PathBuf,
    temp_dir: PathBuf,
}

impl LayerManager {
    /// Create new layer manager
    pub fn new() -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| CleanroomError::runtime_error("Failed to get cache directory"))?
            .join("clnrm")
            .join("oci")
            .join("layers");

        let temp_dir = std::env::temp_dir().join("clnrm-layers");

        std::fs::create_dir_all(&cache_dir)?;
        std::fs::create_dir_all(&temp_dir)?;

        Ok(Self {
            cache_dir,
            temp_dir,
        })
    }

    /// Extract all layers to create merged rootfs
    pub async fn extract_rootfs(
        &self,
        layers: &[OciLayer],
        target_dir: &Path,
    ) -> Result<PathBuf> {
        let rootfs_path = target_dir.join("rootfs");
        tokio::fs::create_dir_all(&rootfs_path).await?;

        info!("Extracting {} layers to rootfs", layers.len());

        // Extract layers in order (base to top)
        for (idx, layer) in layers.iter().enumerate() {
            info!(
                "Extracting layer {}/{}: {} ({} bytes)",
                idx + 1,
                layers.len(),
                layer.digest,
                layer.size
            );

            match layer.media_type.as_str() {
                "application/vnd.docker.image.rootfs.diff.tar.gzip"
                | "application/vnd.oci.image.layer.v1.tar+gzip" => {
                    self.extract_gzip_layer(layer, &rootfs_path).await?;
                }
                "application/vnd.docker.image.rootfs.diff.tar"
                | "application/vnd.oci.image.layer.v1.tar" => {
                    self.extract_tar_layer(layer, &rootfs_path).await?;
                }
                _ => {
                    warn!("Unsupported layer media type: {}", layer.media_type);
                    return Err(CleanroomError::oci_error(format!(
                        "Unsupported layer media type: {}",
                        layer.media_type
                    )));
                }
            }
        }

        info!("Rootfs extraction complete: {}", rootfs_path.display());

        Ok(rootfs_path)
    }

    /// Extract gzipped tar layer
    async fn extract_gzip_layer(&self, layer: &OciLayer, target: &Path) -> Result<()> {
        // Run blocking I/O in separate task
        let target = target.to_path_buf();
        let data = layer.data.clone();

        tokio::task::spawn_blocking(move || {
            let decoder = GzDecoder::new(&data[..]);
            let mut archive = Archive::new(decoder);

            // Extract with whiteout handling (Docker layer spec)
            for entry in archive.entries()? {
                let mut entry = entry?;
                let path = entry.path()?;

                // Handle whiteout files (.wh.* files delete files)
                if let Some(name) = path.file_name() {
                    let name_str = name.to_string_lossy();
                    if name_str.starts_with(".wh.") {
                        // Special whiteout: .wh..wh..opq means delete entire directory contents
                        if name_str == ".wh..wh..opq" {
                            let dir_path = target.join(path.parent().unwrap_or(Path::new("")));
                            if dir_path.exists() {
                                std::fs::remove_dir_all(&dir_path)?;
                                std::fs::create_dir_all(&dir_path)?;
                            }
                            continue;
                        }

                        // Regular whiteout: delete specific file
                        let whiteout_target =
                            path.with_file_name(name_str.strip_prefix(".wh.").unwrap());
                        let full_path = target.join(&whiteout_target);
                        if full_path.exists() {
                            if full_path.is_dir() {
                                std::fs::remove_dir_all(&full_path)?;
                            } else {
                                std::fs::remove_file(&full_path)?;
                            }
                        }
                        continue;
                    }
                }

                // Extract normally
                let full_path = target.join(&path);
                if let Some(parent) = full_path.parent() {
                    std::fs::create_dir_all(parent)?;
                }

                // Handle file type
                let entry_type = entry.header().entry_type();
                match entry_type {
                    tar::EntryType::Regular => {
                        entry.unpack(&full_path)?;
                    }
                    tar::EntryType::Directory => {
                        std::fs::create_dir_all(&full_path)?;
                    }
                    tar::EntryType::Symlink | tar::EntryType::Link => {
                        // Handle symlinks
                        if let Ok(link_name) = entry.link_name() {
                            if let Some(link_path) = link_name {
                                #[cfg(unix)]
                                {
                                    std::os::unix::fs::symlink(link_path, &full_path)?;
                                }
                                #[cfg(windows)]
                                {
                                    // Windows requires different handling
                                    std::os::windows::fs::symlink_file(link_path, &full_path)?;
                                }
                            }
                        }
                    }
                    _ => {
                        // Ignore other types (char devices, block devices, etc.)
                        warn!("Skipping unsupported entry type: {:?}", entry_type);
                    }
                }
            }

            Ok::<(), CleanroomError>(())
        })
        .await
        .map_err(|e| CleanroomError::runtime_error(format!("Layer extraction failed: {}", e)))??;

        Ok(())
    }

    /// Extract plain tar layer
    async fn extract_tar_layer(&self, layer: &OciLayer, target: &Path) -> Result<()> {
        let target = target.to_path_buf();
        let data = layer.data.clone();

        tokio::task::spawn_blocking(move || {
            let mut archive = Archive::new(&data[..]);
            archive.unpack(target)?;
            Ok::<(), CleanroomError>(())
        })
        .await
        .map_err(|e| CleanroomError::runtime_error(format!("Layer extraction failed: {}", e)))??;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_manager_creation() {
        let manager = LayerManager::new().unwrap();
        assert!(manager.cache_dir.exists());
    }
}
