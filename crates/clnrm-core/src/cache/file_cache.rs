//! File-based cache implementation with persistent storage
//!
//! Implements the Cache trait with JSON file persistence.
//! Thread-safe with Arc<Mutex<>> for concurrent access.

use super::cache_trait::{Cache, CacheStats};
use super::hash;
use crate::error::{CleanroomError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use tracing::{debug, info, warn};

/// Cache format version for invalidation when structure changes
const CACHE_VERSION: &str = "1.0.0";

/// Default cache directory under user home
fn default_cache_dir() -> Result<PathBuf> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| CleanroomError::configuration_error("Cannot determine home directory"))?;

    Ok(PathBuf::from(home).join(".clnrm").join("cache"))
}

/// Cache file structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheFile {
    /// Cache format version
    pub version: String,
    /// File path to hash mapping
    pub hashes: HashMap<String, String>,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
}

impl CacheFile {
    /// Create a new empty cache file
    pub fn new() -> Self {
        Self {
            version: CACHE_VERSION.to_string(),
            hashes: HashMap::new(),
            last_updated: Utc::now(),
        }
    }

    /// Check if cache file version is compatible
    pub fn is_compatible(&self) -> bool {
        self.version == CACHE_VERSION
    }
}

impl Default for CacheFile {
    fn default() -> Self {
        Self::new()
    }
}

/// File-based cache manager for test result caching
///
/// London School TDD Design:
/// - Implements Cache trait for collaboration contract
/// - Thread-safe with Arc<Mutex<>> for concurrent access
/// - Proper error handling with Result<T, CleanroomError>
/// - No unwrap() or expect() calls
///
/// # Example
/// ```no_run
/// use clnrm_core::cache::{FileCache, Cache};
/// use std::path::Path;
///
/// # fn main() -> clnrm_core::Result<()> {
/// let cache = FileCache::new()?;
/// let file_path = Path::new("tests/api.clnrm.toml");
/// let content = "rendered content";
///
/// if cache.has_changed(file_path, content)? {
///     // Run test
///     cache.update(file_path, content)?;
///     cache.save()?;
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct FileCache {
    /// Path to cache file
    cache_path: PathBuf,
    /// In-memory cache data (thread-safe)
    cache: Arc<Mutex<CacheFile>>,
}

impl FileCache {
    /// Create a new cache manager with default cache directory
    pub fn new() -> Result<Self> {
        let cache_dir = default_cache_dir()?;
        let cache_path = cache_dir.join("hashes.json");
        Self::with_path(cache_path)
    }

    /// Create a cache manager with custom cache file path
    pub fn with_path(cache_path: PathBuf) -> Result<Self> {
        // Ensure cache directory exists
        if let Some(parent) = cache_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent).map_err(|e| {
                    CleanroomError::io_error(format!(
                        "Failed to create cache directory '{}': {}",
                        parent.display(),
                        e
                    ))
                })?;
                info!("Created cache directory: {}", parent.display());
            }
        }

        // Load existing cache or create new one
        let cache = if cache_path.exists() {
            match Self::load_cache_file(&cache_path) {
                Ok(mut cache_file) => {
                    // Validate cache version
                    if !cache_file.is_compatible() {
                        warn!(
                            "Cache version mismatch (expected {}, got {}). Creating new cache.",
                            CACHE_VERSION, cache_file.version
                        );
                        cache_file = CacheFile::new();
                    }
                    cache_file
                }
                Err(e) => {
                    warn!("Failed to load cache file: {}. Creating new cache.", e);
                    CacheFile::new()
                }
            }
        } else {
            debug!("Cache file not found. Creating new cache.");
            CacheFile::new()
        };

        Ok(Self {
            cache_path,
            cache: Arc::new(Mutex::new(cache)),
        })
    }

    /// Load cache file from disk
    fn load_cache_file(path: &Path) -> Result<CacheFile> {
        let content = fs::read_to_string(path).map_err(|e| {
            CleanroomError::io_error(format!(
                "Failed to read cache file '{}': {}",
                path.display(),
                e
            ))
        })?;

        serde_json::from_str(&content).map_err(|e| {
            CleanroomError::serialization_error(format!(
                "Failed to parse cache file '{}': {}",
                path.display(),
                e
            ))
        })
    }

    /// Get the cache file path
    pub fn cache_path(&self) -> &Path {
        &self.cache_path
    }
}

impl Cache for FileCache {
    fn has_changed(&self, file_path: &Path, rendered_content: &str) -> Result<bool> {
        let file_key = file_path
            .to_str()
            .ok_or_else(|| CleanroomError::validation_error("Invalid file path encoding"))?
            .to_string();

        // Calculate current hash
        let current_hash = hash::hash_content(rendered_content)?;

        // Check against cached hash
        let cache = self.cache.lock().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to acquire cache lock: {}", e))
        })?;

        match cache.hashes.get(&file_key) {
            Some(cached_hash) if cached_hash == &current_hash => {
                debug!("Cache hit: {} (unchanged)", file_key);
                Ok(false)
            }
            Some(_) => {
                debug!("Cache miss: {} (changed)", file_key);
                Ok(true)
            }
            None => {
                debug!("Cache miss: {} (new file)", file_key);
                Ok(true)
            }
        }
    }

    fn update(&self, file_path: &Path, rendered_content: &str) -> Result<()> {
        let file_key = file_path
            .to_str()
            .ok_or_else(|| CleanroomError::validation_error("Invalid file path encoding"))?
            .to_string();

        let hash = hash::hash_content(rendered_content)?;

        let mut cache = self.cache.lock().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to acquire cache lock: {}", e))
        })?;

        cache.hashes.insert(file_key.clone(), hash);
        debug!("Cache updated: {}", file_key);

        Ok(())
    }

    fn remove(&self, file_path: &Path) -> Result<()> {
        let file_key = file_path
            .to_str()
            .ok_or_else(|| CleanroomError::validation_error("Invalid file path encoding"))?
            .to_string();

        let mut cache = self.cache.lock().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to acquire cache lock: {}", e))
        })?;

        if cache.hashes.remove(&file_key).is_some() {
            debug!("Removed from cache: {}", file_key);
        }

        Ok(())
    }

    fn save(&self) -> Result<()> {
        let cache = self.cache.lock().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to acquire cache lock: {}", e))
        })?;

        // Update timestamp
        let mut cache_to_save = cache.clone();
        cache_to_save.last_updated = Utc::now();

        let content = serde_json::to_string_pretty(&cache_to_save).map_err(|e| {
            CleanroomError::serialization_error(format!("Failed to serialize cache: {}", e))
        })?;

        fs::write(&self.cache_path, content).map_err(|e| {
            CleanroomError::io_error(format!(
                "Failed to write cache file '{}': {}",
                self.cache_path.display(),
                e
            ))
        })?;

        debug!("Cache saved to: {}", self.cache_path.display());
        Ok(())
    }

    fn stats(&self) -> Result<CacheStats> {
        let cache = self.cache.lock().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to acquire cache lock: {}", e))
        })?;

        Ok(CacheStats {
            total_files: cache.hashes.len(),
            last_updated: cache.last_updated,
            cache_path: Some(self.cache_path.clone()),
        })
    }

    fn clear(&self) -> Result<()> {
        let mut cache = self.cache.lock().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to acquire cache lock: {}", e))
        })?;

        let count = cache.hashes.len();
        cache.hashes.clear();
        cache.last_updated = Utc::now();

        info!("Cleared {} entries from cache", count);
        Ok(())
    }
}

// Note: Default implementation removed to avoid panic risk.
// FileCache creation is fallible and MUST return Result.
// Use FileCache::new() or FileCache::with_path() instead.
//
// Reasoning:
// - Cache creation can fail due to filesystem permissions
// - Default trait cannot return Result, forcing unwrap/panic
// - Core team standard: No unwrap/expect in production code
// - Explicit Result handling provides better error messages
