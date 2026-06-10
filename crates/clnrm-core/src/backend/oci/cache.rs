//! Image cache with LRU eviction

use super::{OciImage, OciLayer};
use crate::error::{CleanroomError, Result};
use serde::{Deserialize, Serialize};
use sha2::Digest;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::time::SystemTime;
use tokio::sync::RwLock;
use tracing::info;

/// Image cache with LRU eviction
#[derive(Debug)]
pub struct ImageCache {
    cache_dir: PathBuf,
    max_size_gb: u64,
    index: RwLock<CacheIndex>,
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheIndex {
    entries: BTreeMap<String, CacheEntry>,
    total_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct CacheEntry {
    image_ref: String,
    layers: Vec<LayerEntry>,
    config_digest: String,
    #[serde(with = "systemtime_serde")]
    last_accessed: SystemTime,
    total_size: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LayerEntry {
    digest: String,
    size: u64,
    path: PathBuf,
}

// Custom serialization for SystemTime
mod systemtime_serde {
    use serde::{Deserialize, Deserializer, Serialize, Serializer};
    use std::time::{Duration, SystemTime, UNIX_EPOCH};

    pub fn serialize<S>(time: &SystemTime, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let duration = time
            .duration_since(UNIX_EPOCH)
            .unwrap_or(Duration::from_secs(0));
        duration.as_secs().serialize(serializer)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<SystemTime, D::Error>
    where
        D: Deserializer<'de>,
    {
        let secs = u64::deserialize(deserializer)?;
        Ok(UNIX_EPOCH + Duration::from_secs(secs))
    }
}

impl ImageCache {
    /// Create new cache
    pub fn new(max_size_gb: u64) -> Result<Self> {
        let cache_dir = dirs::cache_dir()
            .ok_or_else(|| CleanroomError::runtime_error("Failed to get cache directory"))?
            .join("clnrm")
            .join("oci");

        std::fs::create_dir_all(&cache_dir)?;

        // Load existing index
        let index = Self::load_index(&cache_dir)?;

        info!(
            "Image cache initialized at {} (max size: {}GB, current: {}MB)",
            cache_dir.display(),
            max_size_gb,
            index.total_size / 1024 / 1024
        );

        Ok(Self {
            cache_dir,
            max_size_gb,
            index: RwLock::new(index),
        })
    }

    /// Get cached image
    pub async fn get(&self, image_ref: &str) -> Result<Option<OciImage>> {
        let load_result = {
            let mut index = self.index.write().await;

            if let Some(entry) = index.entries.get_mut(image_ref) {
                info!("Cache hit for image: {}", image_ref);

                // Update last accessed time
                entry.last_accessed = SystemTime::now();

                // Load image from cache
                Some(self.load_from_cache(entry).await)
            } else {
                info!("Cache miss for image: {}", image_ref);
                None
            }
        };

        if let Some(result) = load_result {
            match result {
                Ok(image) => Ok(Some(image)),
                Err(e) => {
                    tracing::warn!("Corrupted cache entry detected: {}. Purging cache.", e);
                    let _ = self.clear().await;
                    Err(e)
                }
            }
        } else {
            Ok(None)
        }
    }

    /// Store image in cache
    pub async fn store(&self, image_ref: &str, image: &OciImage) -> Result<()> {
        let mut index = self.index.write().await;

        // Calculate total size
        let total_size: u64 = image.layers.iter().map(|l| l.size).sum();

        info!(
            "Storing image {} in cache ({} MB)",
            image_ref,
            total_size / 1024 / 1024
        );

        // Check if we need to evict
        while index.total_size + total_size > self.max_size_gb * 1024 * 1024 * 1024 {
            self.evict_lru(&mut index).await?;
        }

        // Store layers
        let mut layer_entries = Vec::new();
        for layer in &image.layers {
            let layer_path = self
                .cache_dir
                .join("layers")
                .join(&layer.digest.replace(':', "_"));

            tokio::fs::create_dir_all(layer_path.parent().unwrap()).await?;
            tokio::fs::write(&layer_path, &layer.data).await?;

            layer_entries.push(LayerEntry {
                digest: layer.digest.clone(),
                size: layer.size,
                path: layer_path,
            });
        }

        // Store config
        let config_digest = format!(
            "sha256:{}",
            hex::encode(sha2::Sha256::digest(&image.config_bytes))
        );
        let config_path = self
            .cache_dir
            .join("configs")
            .join(&config_digest.replace(':', "_"));

        tokio::fs::create_dir_all(config_path.parent().unwrap()).await?;
        tokio::fs::write(&config_path, &image.config_bytes).await?;

        // Add to index
        let entry = CacheEntry {
            image_ref: image_ref.to_string(),
            layers: layer_entries,
            config_digest,
            last_accessed: SystemTime::now(),
            total_size,
        };

        index.entries.insert(image_ref.to_string(), entry);
        index.total_size += total_size;

        // Save index
        self.save_index(&index).await?;

        info!(
            "Image {} cached successfully (total cache: {} MB)",
            image_ref,
            index.total_size / 1024 / 1024
        );

        Ok(())
    }

    /// Evict least recently used image
    async fn evict_lru(&self, index: &mut CacheIndex) -> Result<()> {
        // Find LRU entry
        let lru_ref = index
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.last_accessed)
            .map(|(k, _)| k.clone());

        if let Some(ref_to_remove) = lru_ref {
            if let Some(entry) = index.entries.remove(&ref_to_remove) {
                info!(
                    "Evicting cached image: {} ({} MB)",
                    ref_to_remove,
                    entry.total_size / 1024 / 1024
                );

                // Remove layer files
                for layer in &entry.layers {
                    let _ = tokio::fs::remove_file(&layer.path).await;
                }

                // Remove config file
                let config_path = self
                    .cache_dir
                    .join("configs")
                    .join(&entry.config_digest.replace(':', "_"));
                let _ = tokio::fs::remove_file(&config_path).await;

                // Update total size
                index.total_size -= entry.total_size;
            }
        }

        Ok(())
    }

    /// Load image from cache
    async fn load_from_cache(&self, entry: &CacheEntry) -> Result<OciImage> {
        // Load layers
        let mut layers = Vec::new();
        for layer_entry in &entry.layers {
            let data = tokio::fs::read(&layer_entry.path).await?;

            if let Some(expected_hex) = layer_entry.digest.strip_prefix("sha256:") {
                let actual_digest = sha2::Sha256::digest(&data);
                let actual_hex = hex::encode(actual_digest);
                if actual_hex != expected_hex {
                    return Err(CleanroomError::oci_error(format!(
                        "Layer integrity check failed: expected {}, got {}",
                        expected_hex, actual_hex
                    )));
                }
            }

            layers.push(OciLayer {
                digest: layer_entry.digest.clone(),
                media_type: "application/vnd.docker.image.rootfs.diff.tar.gzip".to_string(),
                size: layer_entry.size,
                data,
            });
        }

        // Load config
        let config_path = self
            .cache_dir
            .join("configs")
            .join(&entry.config_digest.replace(':', "_"));
        let config_bytes = tokio::fs::read(&config_path).await?;
        let config = serde_json::from_slice(&config_bytes)?;

        Ok(OciImage {
            manifest: super::OciManifest::default(), // Reconstruct if needed
            config,
            layers,
            config_bytes,
        })
    }

    /// Load cache index from disk
    fn load_index(cache_dir: &Path) -> Result<CacheIndex> {
        let index_path = cache_dir.join("index.json");
        if index_path.exists() {
            let data = std::fs::read(&index_path)?;
            Ok(serde_json::from_slice(&data)?)
        } else {
            Ok(CacheIndex {
                entries: BTreeMap::new(),
                total_size: 0,
            })
        }
    }

    /// Save cache index to disk
    async fn save_index(&self, index: &CacheIndex) -> Result<()> {
        let index_path = self.cache_dir.join("index.json");
        let data = serde_json::to_vec_pretty(index)?;
        tokio::fs::write(&index_path, data).await?;
        Ok(())
    }

    /// Clear all cached images
    pub async fn clear(&self) -> Result<()> {
        let mut index = self.index.write().await;

        info!("Clearing image cache");

        // Remove all layer and config files
        for entry in index.entries.values() {
            for layer in &entry.layers {
                let _ = tokio::fs::remove_file(&layer.path).await;
            }

            let config_path = self
                .cache_dir
                .join("configs")
                .join(&entry.config_digest.replace(':', "_"));
            let _ = tokio::fs::remove_file(&config_path).await;
        }

        // Clear index
        index.entries.clear();
        index.total_size = 0;

        // Save empty index
        self.save_index(&index).await?;

        info!("Image cache cleared");

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_cache_creation() {
        let cache = ImageCache::new(1).unwrap();
        assert!(cache.cache_dir.exists());
    }

    #[tokio::test]
    async fn test_cache_miss() {
        let cache = ImageCache::new(1).unwrap();
        let result = cache.get("nonexistent:latest").await.unwrap();
        assert!(result.is_none());
    }
}
