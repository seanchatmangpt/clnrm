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
    pub fn len(&self) -> usize {
        self.hashes
            .lock()
            .map(|h| h.len())
            .unwrap_or(0)
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

#[cfg(test)]
mod tests {
    #![allow(
        clippy::unwrap_used,
        clippy::expect_used,
        clippy::indexing_slicing,
        clippy::panic
    )]

    use super::*;
    use std::path::PathBuf;

    #[test]
    fn test_memory_cache_implements_trait() -> Result<()> {
        // Arrange & Act - create as trait object
        let cache: Box<dyn Cache> = Box::new(MemoryCache::new());
        let test_path = PathBuf::from("/test/file.toml");
        let content = "test content";

        // Assert - can use through trait interface
        let changed = cache.has_changed(&test_path, content)?;
        assert!(changed);

        Ok(())
    }

    #[test]
    fn test_memory_cache_has_changed_new_file() -> Result<()> {
        // Arrange
        let cache = MemoryCache::new();
        let test_path = PathBuf::from("/test/file.toml");
        let content = "test content";

        // Act
        let changed = cache.has_changed(&test_path, content)?;

        // Assert
        assert!(changed, "New file should be marked as changed");

        Ok(())
    }

    #[test]
    fn test_memory_cache_update_and_check() -> Result<()> {
        // Arrange
        let cache = MemoryCache::new();
        let test_path = PathBuf::from("/test/file.toml");
        let content = "test content";

        // Act
        cache.update(&test_path, content)?;
        let changed = cache.has_changed(&test_path, content)?;

        // Assert - verify interaction pattern
        assert!(!changed, "Unchanged file should not be marked as changed");

        Ok(())
    }

    #[test]
    fn test_memory_cache_detects_changes() -> Result<()> {
        // Arrange
        let cache = MemoryCache::new();
        let test_path = PathBuf::from("/test/file.toml");
        let content1 = "test content 1";
        let content2 = "test content 2";

        // Act
        cache.update(&test_path, content1)?;
        let changed = cache.has_changed(&test_path, content2)?;

        // Assert
        assert!(changed, "Changed file should be marked as changed");

        Ok(())
    }

    #[test]
    fn test_memory_cache_thread_safety() -> Result<()> {
        use std::thread;

        // Arrange
        let cache = MemoryCache::new();

        // Act - spawn multiple threads updating cache
        let mut handles = vec![];
        for i in 0..10 {
            let cache_clone = cache.clone();
            let handle = thread::spawn(move || {
                let path = PathBuf::from(format!("/test/file{}.toml", i));
                let content = format!("content {}", i);
                cache_clone.update(&path, &content).unwrap();
            });
            handles.push(handle);
        }

        // Wait for all threads
        for handle in handles {
            handle.join().unwrap();
        }

        // Assert
        let stats = cache.stats()?;
        assert_eq!(stats.total_files, 10);

        Ok(())
    }

    #[test]
    fn test_memory_cache_remove() -> Result<()> {
        // Arrange
        let cache = MemoryCache::new();
        let test_path = PathBuf::from("/test/file.toml");
        let content = "test content";

        // Act
        cache.update(&test_path, content)?;
        cache.remove(&test_path)?;
        let changed = cache.has_changed(&test_path, content)?;

        // Assert
        assert!(changed, "Removed file should be marked as changed");

        Ok(())
    }

    #[test]
    fn test_memory_cache_clear() -> Result<()> {
        // Arrange
        let cache = MemoryCache::new();
        cache.update(&PathBuf::from("/test/file1.toml"), "content1")?;
        cache.update(&PathBuf::from("/test/file2.toml"), "content2")?;

        // Act
        cache.clear()?;
        let stats = cache.stats()?;

        // Assert
        assert_eq!(stats.total_files, 0);
        assert!(cache.is_empty());

        Ok(())
    }

    #[test]
    fn test_memory_cache_save_noop() -> Result<()> {
        // Arrange
        let cache = MemoryCache::new();

        // Act & Assert - save should succeed but do nothing
        cache.save()?;

        Ok(())
    }

    #[test]
    fn test_memory_cache_stats_no_path() -> Result<()> {
        // Arrange
        let cache = MemoryCache::new();
        cache.update(&PathBuf::from("/test/file.toml"), "content")?;

        // Act
        let stats = cache.stats()?;

        // Assert
        assert_eq!(stats.total_files, 1);
        assert!(stats.cache_path.is_none());

        Ok(())
    }

    #[test]
    fn test_memory_cache_collaboration_workflow() -> Result<()> {
        // Arrange
        let cache = MemoryCache::new();
        let file1 = PathBuf::from("/test/file1.toml");
        let file2 = PathBuf::from("/test/file2.toml");
        let content = "shared content";

        // Act - simulate typical test runner workflow
        // First run: both files are new
        assert!(cache.has_changed(&file1, content)?);
        assert!(cache.has_changed(&file2, content)?);

        cache.update(&file1, content)?;
        cache.update(&file2, content)?;

        // Second run: both files unchanged
        assert!(!cache.has_changed(&file1, content)?);
        assert!(!cache.has_changed(&file2, content)?);

        // Third run: file1 changes
        let new_content = "new content";
        assert!(cache.has_changed(&file1, new_content)?);
        assert!(!cache.has_changed(&file2, content)?);

        // Assert final state
        let stats = cache.stats()?;
        assert_eq!(stats.total_files, 2);

        Ok(())
    }
}
