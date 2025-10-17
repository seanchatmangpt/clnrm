//! Comprehensive unit tests for cache trait implementations
//!
//! Tests follow TDD London School methodology:
//! - Mock-driven development for cache backends
//! - Contract testing through Cache trait
//! - Thread-safety verification
//! - Behavior-focused assertions

use clnrm_core::cache::{Cache, MemoryCache};
use clnrm_core::Result;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;

// ============================================================================
// Cache Trait Contract Tests (London School)
// ============================================================================

#[test]
fn test_cache_trait_allows_trait_object_creation() -> Result<()> {
    // Arrange & Act - verify Cache trait is object-safe
    let cache: Box<dyn Cache> = Box::new(MemoryCache::new());
    let test_path = PathBuf::from("/test/file.toml");

    // Assert - can use through trait interface
    let result = cache.has_changed(&test_path, "content");
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_cache_trait_supports_send_sync_for_multithreading() {
    // Arrange
    fn assert_send_sync<T: Send + Sync>() {}

    // Act & Assert - verify trait bounds
    assert_send_sync::<Box<dyn Cache>>();
    assert_send_sync::<MemoryCache>();
}

// ============================================================================
// MemoryCache Basic Operations Tests
// ============================================================================

#[test]
fn test_memory_cache_new_creates_empty_cache() {
    // Arrange & Act
    let cache = MemoryCache::new();

    // Assert
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
}

#[test]
fn test_memory_cache_has_changed_returns_true_for_new_file() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file_path = PathBuf::from("/test/new_file.toml");
    let content = "test content";

    // Act
    let has_changed = cache.has_changed(&file_path, content)?;

    // Assert
    assert!(has_changed, "New file should be marked as changed");
    Ok(())
}

#[test]
fn test_memory_cache_has_changed_returns_false_for_unchanged_file() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file_path = PathBuf::from("/test/file.toml");
    let content = "test content";

    // Act
    cache.update(&file_path, content)?;
    let has_changed = cache.has_changed(&file_path, content)?;

    // Assert
    assert!(
        !has_changed,
        "Unchanged file should not be marked as changed"
    );
    Ok(())
}

#[test]
fn test_memory_cache_has_changed_returns_true_for_modified_file() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file_path = PathBuf::from("/test/file.toml");
    let original_content = "original content";
    let modified_content = "modified content";

    // Act
    cache.update(&file_path, original_content)?;
    let has_changed = cache.has_changed(&file_path, modified_content)?;

    // Assert
    assert!(has_changed, "Modified file should be marked as changed");
    Ok(())
}

#[test]
fn test_memory_cache_update_stores_file_hash() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file_path = PathBuf::from("/test/file.toml");
    let content = "test content";

    // Act
    cache.update(&file_path, content)?;

    // Assert
    assert_eq!(cache.len(), 1);
    assert!(!cache.is_empty());
    Ok(())
}

#[test]
fn test_memory_cache_update_overwrites_existing_entry() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file_path = PathBuf::from("/test/file.toml");
    let content1 = "content version 1";
    let content2 = "content version 2";

    // Act
    cache.update(&file_path, content1)?;
    cache.update(&file_path, content2)?;

    // Assert - still only one entry
    assert_eq!(cache.len(), 1);
    // Verify updated hash is different
    assert!(!cache.has_changed(&file_path, content2)?);
    assert!(cache.has_changed(&file_path, content1)?);
    Ok(())
}

// ============================================================================
// MemoryCache Remove Operations Tests
// ============================================================================

#[test]
fn test_memory_cache_remove_deletes_file_from_cache() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file_path = PathBuf::from("/test/file.toml");
    let content = "test content";

    // Act
    cache.update(&file_path, content)?;
    cache.remove(&file_path)?;

    // Assert
    assert!(cache.is_empty());
    assert!(cache.has_changed(&file_path, content)?);
    Ok(())
}

#[test]
fn test_memory_cache_remove_on_nonexistent_file_succeeds() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file_path = PathBuf::from("/test/nonexistent.toml");

    // Act
    let result = cache.remove(&file_path);

    // Assert - should succeed (idempotent operation)
    assert!(result.is_ok());
    Ok(())
}

#[test]
fn test_memory_cache_remove_does_not_affect_other_files() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file1 = PathBuf::from("/test/file1.toml");
    let file2 = PathBuf::from("/test/file2.toml");
    let content = "content";

    // Act
    cache.update(&file1, content)?;
    cache.update(&file2, content)?;
    cache.remove(&file1)?;

    // Assert
    assert_eq!(cache.len(), 1);
    assert!(cache.has_changed(&file1, content)?);
    assert!(!cache.has_changed(&file2, content)?);
    Ok(())
}

// ============================================================================
// MemoryCache Clear Operations Tests
// ============================================================================

#[test]
fn test_memory_cache_clear_removes_all_entries() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    cache.update(&PathBuf::from("/test/file1.toml"), "content1")?;
    cache.update(&PathBuf::from("/test/file2.toml"), "content2")?;
    cache.update(&PathBuf::from("/test/file3.toml"), "content3")?;

    // Act
    cache.clear()?;

    // Assert
    assert!(cache.is_empty());
    assert_eq!(cache.len(), 0);
    Ok(())
}

#[test]
fn test_memory_cache_clear_on_empty_cache_succeeds() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();

    // Act
    let result = cache.clear();

    // Assert
    assert!(result.is_ok());
    assert!(cache.is_empty());
    Ok(())
}

// ============================================================================
// MemoryCache Save Operations Tests
// ============================================================================

#[test]
fn test_memory_cache_save_is_noop_operation() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    cache.update(&PathBuf::from("/test/file.toml"), "content")?;

    // Act
    let result = cache.save();

    // Assert - save should succeed but do nothing
    assert!(result.is_ok());
    assert_eq!(cache.len(), 1);
    Ok(())
}

// ============================================================================
// MemoryCache Stats Tests
// ============================================================================

#[test]
fn test_memory_cache_stats_returns_correct_file_count() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    cache.update(&PathBuf::from("/test/file1.toml"), "content1")?;
    cache.update(&PathBuf::from("/test/file2.toml"), "content2")?;
    cache.update(&PathBuf::from("/test/file3.toml"), "content3")?;

    // Act
    let stats = cache.stats()?;

    // Assert
    assert_eq!(stats.total_files, 3);
    Ok(())
}

#[test]
fn test_memory_cache_stats_has_no_cache_path() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    cache.update(&PathBuf::from("/test/file.toml"), "content")?;

    // Act
    let stats = cache.stats()?;

    // Assert
    assert!(stats.cache_path.is_none());
    Ok(())
}

#[test]
fn test_memory_cache_stats_timestamp_is_recent() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let before = chrono::Utc::now();

    // Act
    let stats = cache.stats()?;
    let after = chrono::Utc::now();

    // Assert
    assert!(stats.last_updated >= before);
    assert!(stats.last_updated <= after);
    Ok(())
}

// ============================================================================
// Thread Safety Tests (London School: Collaboration Patterns)
// ============================================================================

#[test]
fn test_memory_cache_concurrent_updates_do_not_lose_data() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let cache_arc = Arc::new(cache);
    let num_threads = 10;

    // Act - spawn multiple threads updating cache
    let mut handles = vec![];
    for i in 0..num_threads {
        let cache_clone = Arc::clone(&cache_arc);
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
    let stats = cache_arc.stats()?;
    assert_eq!(stats.total_files, num_threads);
    Ok(())
}

#[test]
fn test_memory_cache_concurrent_reads_are_thread_safe() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file_path = PathBuf::from("/test/shared.toml");
    let content = "shared content";
    cache.update(&file_path, content)?;

    let cache_arc = Arc::new(cache);
    let num_threads = 10;

    // Act - spawn multiple threads reading cache
    let mut handles = vec![];
    for _ in 0..num_threads {
        let cache_clone = Arc::clone(&cache_arc);
        let path_clone = file_path.clone();
        let content_clone = content.to_string();
        let handle = thread::spawn(move || {
            cache_clone
                .has_changed(&path_clone, &content_clone)
                .unwrap()
        });
        handles.push(handle);
    }

    // Collect results
    let results: Vec<bool> = handles.into_iter().map(|h| h.join().unwrap()).collect();

    // Assert - all reads should return false (unchanged)
    assert!(results.iter().all(|&changed| !changed));
    Ok(())
}

#[test]
fn test_memory_cache_concurrent_mixed_operations_maintain_consistency() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let cache_arc = Arc::new(cache);

    // Act - spawn threads performing different operations
    let mut handles = vec![];

    // Updaters
    for i in 0..5 {
        let cache_clone = Arc::clone(&cache_arc);
        let handle = thread::spawn(move || {
            let path = PathBuf::from(format!("/test/file{}.toml", i));
            cache_clone
                .update(&path, &format!("content {}", i))
                .unwrap();
        });
        handles.push(handle);
    }

    // Readers
    for i in 0..5 {
        let cache_clone = Arc::clone(&cache_arc);
        let handle = thread::spawn(move || {
            let path = PathBuf::from(format!("/test/file{}.toml", i));
            let _ = cache_clone.has_changed(&path, "test");
        });
        handles.push(handle);
    }

    // Removers
    for i in 0..3 {
        let cache_clone = Arc::clone(&cache_arc);
        let handle = thread::spawn(move || {
            let path = PathBuf::from(format!("/test/file{}.toml", i));
            cache_clone.remove(&path).unwrap();
        });
        handles.push(handle);
    }

    // Wait for all operations
    for handle in handles {
        handle.join().unwrap();
    }

    // Assert - cache should be in valid state
    let stats = cache_arc.stats()?;
    assert!(stats.total_files <= 5);
    Ok(())
}

// ============================================================================
// Cache Workflow Integration Tests (London School)
// ============================================================================

#[test]
fn test_cache_typical_test_runner_workflow() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file1 = PathBuf::from("/tests/api_test.toml");
    let file2 = PathBuf::from("/tests/db_test.toml");
    let file3 = PathBuf::from("/tests/ui_test.toml");
    let content1 = "api test content";
    let content2 = "db test content";
    let content3 = "ui test content";

    // Act & Assert - First test run: all files are new
    assert!(cache.has_changed(&file1, content1)?);
    assert!(cache.has_changed(&file2, content2)?);
    assert!(cache.has_changed(&file3, content3)?);

    // Update cache after running tests
    cache.update(&file1, content1)?;
    cache.update(&file2, content2)?;
    cache.update(&file3, content3)?;

    // Second test run: no files changed
    assert!(!cache.has_changed(&file1, content1)?);
    assert!(!cache.has_changed(&file2, content2)?);
    assert!(!cache.has_changed(&file3, content3)?);

    // Third test run: file1 modified
    let new_content1 = "modified api test content";
    assert!(cache.has_changed(&file1, new_content1)?);
    assert!(!cache.has_changed(&file2, content2)?);
    assert!(!cache.has_changed(&file3, content3)?);

    // Update only changed file
    cache.update(&file1, new_content1)?;

    // Fourth test run: all unchanged again
    assert!(!cache.has_changed(&file1, new_content1)?);
    assert!(!cache.has_changed(&file2, content2)?);
    assert!(!cache.has_changed(&file3, content3)?);

    // Assert final state
    let stats = cache.stats()?;
    assert_eq!(stats.total_files, 3);
    Ok(())
}

#[test]
fn test_cache_file_deletion_workflow() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file_path = PathBuf::from("/tests/deleted_test.toml");
    let content = "test content";

    // Act
    // File exists and is cached
    cache.update(&file_path, content)?;
    assert!(!cache.has_changed(&file_path, content)?);

    // File is deleted from filesystem (simulated by removal from cache)
    cache.remove(&file_path)?;

    // Assert - cache should treat it as new file again
    assert!(cache.has_changed(&file_path, content)?);
    Ok(())
}

#[test]
fn test_cache_handles_identical_content_in_different_files() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file1 = PathBuf::from("/test/file1.toml");
    let file2 = PathBuf::from("/test/file2.toml");
    let shared_content = "identical content";

    // Act
    cache.update(&file1, shared_content)?;
    cache.update(&file2, shared_content)?;

    // Assert - both files should be tracked separately
    assert_eq!(cache.len(), 2);
    assert!(!cache.has_changed(&file1, shared_content)?);
    assert!(!cache.has_changed(&file2, shared_content)?);

    // Removing one should not affect the other
    cache.remove(&file1)?;
    assert!(cache.has_changed(&file1, shared_content)?);
    assert!(!cache.has_changed(&file2, shared_content)?);
    Ok(())
}

// ============================================================================
// Edge Cases and Error Handling
// ============================================================================

#[test]
fn test_cache_handles_empty_content() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file_path = PathBuf::from("/test/empty.toml");
    let empty_content = "";

    // Act
    cache.update(&file_path, empty_content)?;
    let has_changed = cache.has_changed(&file_path, empty_content)?;

    // Assert
    assert!(!has_changed);
    Ok(())
}

#[test]
fn test_cache_handles_large_content() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file_path = PathBuf::from("/test/large.toml");
    let large_content = "x".repeat(1_000_000); // 1MB of content

    // Act
    cache.update(&file_path, &large_content)?;
    let has_changed = cache.has_changed(&file_path, &large_content)?;

    // Assert
    assert!(!has_changed);
    Ok(())
}

#[test]
fn test_cache_handles_unicode_content() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file_path = PathBuf::from("/test/unicode.toml");
    let unicode_content = "Hello 世界 🚀 Привет مرحبا";

    // Act
    cache.update(&file_path, unicode_content)?;
    let has_changed = cache.has_changed(&file_path, unicode_content)?;

    // Assert
    assert!(!has_changed);
    Ok(())
}

#[test]
fn test_cache_handles_special_characters_in_path() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file_path = PathBuf::from("/test/special-chars_[123].toml");
    let content = "test content";

    // Act
    cache.update(&file_path, content)?;
    let has_changed = cache.has_changed(&file_path, content)?;

    // Assert
    assert!(!has_changed);
    Ok(())
}

#[test]
fn test_cache_detects_whitespace_only_changes() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let file_path = PathBuf::from("/test/whitespace.toml");
    let content1 = "line1\nline2\nline3";
    let content2 = "line1\n\nline2\nline3"; // Extra newline

    // Act
    cache.update(&file_path, content1)?;
    let has_changed = cache.has_changed(&file_path, content2)?;

    // Assert - should detect whitespace change
    assert!(has_changed);
    Ok(())
}
