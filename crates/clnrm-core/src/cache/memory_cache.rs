//! In-memory cache implementation for testing
//!
//! Provides a fast, thread-safe cache that doesn't persist to disk.
//! Ideal for unit tests and development workflows.

use super::cache_trait::{Cache, CacheStats};
use super::hash;
use crate::error::{CleanroomError, Result};
use chrono::Utc;
use std::collections::HashMap;
use std::path::Path;
use std::sync::{Arc, Mutex};
use tracing::debug;

/// In-memory cache for testing and development
///
/// London School TDD Design:
/// - Implements Cache trait for collaboration contract
/// - Thread-safe with Arc<Mutex<>> for concurrent access
/// - No persistence - perfect for testing
/// - Fast operations without disk I/O
///
/// # Example
/// ```
/// use clnrm_core::cache::{MemoryCache, Cache};
/// use std::path::Path;
///
/// # fn main() -> clnrm_core::Result<()> {
/// let cache = MemoryCache::new();
/// let file_path = Path::new("tests/api.clnrm.toml");
/// let content = "rendered content";
///
/// if cache.has_changed(file_path, content)? {
///     // Run test
///     cache.update(file_path, content)?;
/// }
/// # Ok(())
/// # }
/// ```
#[derive(Debug, Clone)]
pub struct MemoryCache {
    /// In-memory hash storage (thread-safe)
    hashes: Arc<Mutex<HashMap<String, String>>>,
}

impl MemoryCache {
    /// Create a new in-memory cache
    pub fn new() -> Self {
        Self {
            hashes: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Get the number of entries in cache (for testing)
    ///
    /// Returns 0 if lock acquisition fails (defensive fallback for testing utility)
    pub fn len(&self) -> usize {
        self.hashes.lock().map(|h| h.len()).unwrap_or_else(|e| {
            // This should never happen in practice, but we provide a safe fallback
            // rather than panicking. Log at debug level for visibility in tests.
            debug!("Failed to acquire cache lock in len(): {}", e);
            0
        })
    }

    /// Check if cache is empty (for testing)
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl Cache for MemoryCache {
    fn has_changed(&self, file_path: &Path, rendered_content: &str) -> Result<bool> {
        let file_key = file_path
            .to_str()
            .ok_or_else(|| CleanroomError::validation_error("Invalid file path encoding"))?
            .to_string();

        // Calculate current hash
        let current_hash = hash::hash_content(rendered_content)?;

        // Check against cached hash
        let hashes = self.hashes.lock().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to acquire cache lock: {}", e))
        })?;

        match hashes.get(&file_key) {
            Some(cached_hash) if cached_hash == &current_hash => {
                debug!("Memory cache hit: {} (unchanged)", file_key);
                Ok(false)
            }
            Some(_) => {
                debug!("Memory cache miss: {} (changed)", file_key);
                Ok(true)
            }
            None => {
                debug!("Memory cache miss: {} (new file)", file_key);
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

        let mut hashes = self.hashes.lock().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to acquire cache lock: {}", e))
        })?;

        hashes.insert(file_key.clone(), hash);
        debug!("Memory cache updated: {}", file_key);

        Ok(())
    }

    fn remove(&self, file_path: &Path) -> Result<()> {
        let file_key = file_path
            .to_str()
            .ok_or_else(|| CleanroomError::validation_error("Invalid file path encoding"))?
            .to_string();

        let mut hashes = self.hashes.lock().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to acquire cache lock: {}", e))
        })?;

        if hashes.remove(&file_key).is_some() {
            debug!("Removed from memory cache: {}", file_key);
        }

        Ok(())
    }

    fn save(&self) -> Result<()> {
        // No-op for memory cache
        Ok(())
    }

    fn stats(&self) -> Result<CacheStats> {
        let hashes = self.hashes.lock().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to acquire cache lock: {}", e))
        })?;

        Ok(CacheStats {
            total_files: hashes.len(),
            last_updated: Utc::now(),
            cache_path: None,
        })
    }

    fn clear(&self) -> Result<()> {
        let mut hashes = self.hashes.lock().map_err(|e| {
            CleanroomError::internal_error(format!("Failed to acquire cache lock: {}", e))
        })?;

        hashes.clear();
        debug!("Memory cache cleared");

        Ok(())
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}
