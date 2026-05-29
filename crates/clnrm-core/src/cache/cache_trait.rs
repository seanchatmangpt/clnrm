//! Cache trait abstraction for pluggable backends
//!
//! Provides trait-based interface following Chicago School TDD principles:
//! - Clear contract definition through trait methods
//! - Mockable interface for testing
//! - Support for multiple backend implementations

use crate::error::Result;
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};

/// Statistics about cache state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CacheStats {
    /// Total number of files in cache
    pub total_files: usize,
    /// Last update timestamp
    pub last_updated: DateTime<Utc>,
    /// Cache file path (if applicable)
    pub cache_path: Option<PathBuf>,
}

/// Cache trait defining the contract for cache backends
///
/// Chicago School TDD:
/// - This trait defines the collaboration contract
/// - Implementations can be mocked for testing
/// - Focuses on behavior verification over state
///
/// # Design Principles
/// - All methods return Result for proper error handling
/// - Thread-safe implementations required (Clone + Send + Sync)
/// - No async to maintain dyn compatibility
pub trait Cache: Send + Sync {
    /// Check if a file has changed since last cache update
    ///
    /// # Arguments
    /// * `file_path` - Path to the file being checked
    /// * `rendered_content` - Current content to compare against cache
    ///
    /// # Returns
    /// - Ok(true) if file changed or not in cache
    /// - Ok(false) if file unchanged
    /// - Err if operation fails
    fn has_changed(&self, file_path: &Path, rendered_content: &str) -> Result<bool>;

    /// Update cache with new file hash
    ///
    /// # Arguments
    /// * `file_path` - Path to the file being updated
    /// * `rendered_content` - Content to hash and store
    fn update(&self, file_path: &Path, rendered_content: &str) -> Result<()>;

    /// Remove a file from cache
    ///
    /// # Arguments
    /// * `file_path` - Path to the file being removed
    fn remove(&self, file_path: &Path) -> Result<()>;

    /// Save cache to persistent storage (if applicable)
    ///
    /// For in-memory caches, this is a no-op
    fn save(&self) -> Result<()>;

    /// Get cache statistics
    fn stats(&self) -> Result<CacheStats>;

    /// Clear all cache entries
    fn clear(&self) -> Result<()>;
}

/// Type alias for boxed cache trait object
pub type BoxedCache = Box<dyn Cache>;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    // Mock cache implementation for testing
    struct MockCache {
        should_change: bool,
        update_calls: std::sync::Mutex<Vec<PathBuf>>,
    }

    impl MockCache {
        fn new(should_change: bool) -> Self {
            Self {
                should_change,
                update_calls: std::sync::Mutex::new(Vec::new()),
            }
        }

        fn update_call_count(&self) -> usize {
            self.update_calls.lock().unwrap().len()
        }

        fn get_update_calls(&self) -> Vec<PathBuf> {
            self.update_calls.lock().unwrap().clone()
        }
    }

    impl Cache for MockCache {
        fn has_changed(&self, _file_path: &Path, _rendered_content: &str) -> Result<bool> {
            Ok(self.should_change)
        }

        fn update(&self, file_path: &Path, _rendered_content: &str) -> Result<()> {
            self.update_calls
                .lock()
                .unwrap()
                .push(file_path.to_path_buf());
            Ok(())
        }

        fn remove(&self, _file_path: &Path) -> Result<()> {
            Ok(())
        }

        fn save(&self) -> Result<()> {
            Ok(())
        }

        fn clear(&self) -> Result<()> {
            Ok(())
        }

        fn stats(&self) -> Result<CacheStats> {
            Ok(CacheStats {
                total_files: 42,
                last_updated: Utc::now(),
                cache_path: Some(PathBuf::from("/tmp/mock_cache")),
            })
        }
    }

    #[test]
    fn test_cache_stats_creation() {
        // Test CacheStats struct creation
        let stats = CacheStats {
            total_files: 10,
            last_updated: Utc::now(),
            cache_path: Some(PathBuf::from("/tmp/test")),
        };

        assert_eq!(stats.total_files, 10);
        assert!(stats.cache_path.is_some());
        assert_eq!(
            stats.cache_path.as_ref().unwrap(),
            &PathBuf::from("/tmp/test")
        );
    }

    #[test]
    fn test_cache_stats_equality() {
        // Test CacheStats equality
        let time = Utc::now();
        let stats1 = CacheStats {
            total_files: 5,
            last_updated: time,
            cache_path: None,
        };

        let stats2 = CacheStats {
            total_files: 5,
            last_updated: time,
            cache_path: None,
        };

        assert_eq!(stats1, stats2);
    }

    #[test]
    fn test_mock_cache_has_changed() {
        // Test mock cache has_changed method
        let cache_always_changed = MockCache::new(true);
        let cache_never_changed = MockCache::new(false);

        let path = Path::new("test.txt");

        assert!(cache_always_changed.has_changed(path, "content").unwrap());
        assert!(!cache_never_changed.has_changed(path, "content").unwrap());
    }

    #[test]
    fn test_mock_cache_update() {
        // Test mock cache update method
        let cache = MockCache::new(true);
        let path = Path::new("test.txt");

        assert_eq!(cache.update_call_count(), 0);

        cache.update(path, "content").unwrap();

        assert_eq!(cache.update_call_count(), 1);
        let calls = cache.get_update_calls();
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0], path);
    }

    #[test]
    fn test_mock_cache_stats() {
        // Test mock cache stats method
        let cache = MockCache::new(true);
        let stats = cache.stats().unwrap();

        assert_eq!(stats.total_files, 42);
        assert!(stats.cache_path.is_some());
        assert!(stats.last_updated <= Utc::now());
    }

    #[test]
    fn test_mock_cache_remove() {
        // Test mock cache remove method
        let cache = MockCache::new(true);
        let path = Path::new("test.txt");

        // Should not panic
        cache.remove(path).unwrap();
    }

    #[test]
    fn test_mock_cache_clear() {
        // Test mock cache clear method
        let cache = MockCache::new(true);

        // Should not panic
        cache.clear().unwrap();
    }

    #[test]
    fn test_boxed_cache_trait() {
        // Test that BoxedCache type alias works
        let cache = MockCache::new(false);
        let boxed: BoxedCache = Box::new(cache);

        // Test that boxed cache works
        let result = boxed.has_changed(Path::new("test"), "content");
        assert!(result.is_ok());
        assert!(!result.unwrap());
    }
}
