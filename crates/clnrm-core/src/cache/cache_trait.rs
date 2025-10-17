//! Cache trait abstraction for pluggable backends
//!
//! Provides trait-based interface following London School TDD principles:
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
/// London School TDD:
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
