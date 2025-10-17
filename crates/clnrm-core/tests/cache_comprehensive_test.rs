//! Comprehensive test suite for cache subsystem
//!
//! Test Coverage:
//! - Unit tests for CacheManager
//! - Unit tests for hash module
//! - Edge cases and error scenarios
//! - Thread safety and concurrency
//! - Performance validation
//! - Cache persistence and recovery
//!
//! Core Team Compliance:
//! - ✅ AAA pattern (Arrange, Act, Assert)
//! - ✅ Descriptive test names
//! - ✅ No unwrap/expect in production code (test code allowed)
//! - ✅ Proper error path testing
//! - ✅ Mock external dependencies

#![allow(clippy::unwrap_used, clippy::expect_used)]

use clnrm_core::cache::hash;
use clnrm_core::cache::{CacheFile, CacheManager};
use clnrm_core::error::{CleanroomError, Result};
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Barrier};
use std::thread;
use tempfile::TempDir;

// ============================================================================
// CacheFile Unit Tests
// ============================================================================

#[test]
fn test_cache_file_new_creates_valid_structure() -> Result<()> {
    // Arrange & Act
    let cache = CacheFile::new();

    // Assert
    assert_eq!(cache.version, "1.0.0");
    assert!(cache.hashes.is_empty());
    assert!(cache.last_updated.timestamp() > 0);

    Ok(())
}

#[test]
fn test_cache_file_default_matches_new() -> Result<()> {
    // Arrange & Act
    let cache1 = CacheFile::new();
    let cache2 = CacheFile::default();

    // Assert
    assert_eq!(cache1.version, cache2.version);
    assert_eq!(cache1.hashes.len(), cache2.hashes.len());

    Ok(())
}

#[test]
fn test_cache_file_is_compatible_with_current_version() -> Result<()> {
    // Arrange
    let cache = CacheFile::new();

    // Act
    let compatible = cache.is_compatible();

    // Assert
    assert!(compatible, "Current version should be compatible");

    Ok(())
}

#[test]
fn test_cache_file_is_incompatible_with_old_version() -> Result<()> {
    // Arrange
    let mut cache = CacheFile::new();
    cache.version = "0.9.0".to_string();

    // Act
    let compatible = cache.is_compatible();

    // Assert
    assert!(!compatible, "Old version should not be compatible");

    Ok(())
}

#[test]
fn test_cache_file_is_incompatible_with_future_version() -> Result<()> {
    // Arrange
    let mut cache = CacheFile::new();
    cache.version = "2.0.0".to_string();

    // Act
    let compatible = cache.is_compatible();

    // Assert
    assert!(!compatible, "Future version should not be compatible");

    Ok(())
}

// ============================================================================
// CacheManager Creation and Initialization Tests
// ============================================================================

#[test]
fn test_cache_manager_with_path_creates_directory() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache_dir").join("hashes.json");

    // Act
    let _manager = CacheManager::with_path(cache_path.clone())?;

    // Assert
    assert!(
        cache_path.parent().unwrap().exists(),
        "Cache directory should be created"
    );

    Ok(())
}

#[test]
fn test_cache_manager_with_path_handles_existing_directory() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_dir = temp_dir.path().join("cache");
    fs::create_dir_all(&cache_dir)
        .map_err(|e| CleanroomError::io_error(format!("Failed to create cache dir: {}", e)))?;
    let cache_path = cache_dir.join("hashes.json");

    // Act
    let result = CacheManager::with_path(cache_path);

    // Assert
    assert!(result.is_ok(), "Should handle existing directory");

    Ok(())
}

#[test]
fn test_cache_manager_loads_existing_cache_file() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("hashes.json");

    // Create and save initial cache
    {
        let manager = CacheManager::with_path(cache_path.clone())?;
        manager.update(&PathBuf::from("/test/file.toml"), "test content")?;
        manager.save()?;
    }

    // Act - Load in new manager
    let manager = CacheManager::with_path(cache_path)?;
    let changed = manager.has_changed(&PathBuf::from("/test/file.toml"), "test content")?;

    // Assert
    assert!(
        !changed,
        "Loaded cache should recognize unchanged content"
    );

    Ok(())
}

#[test]
fn test_cache_manager_handles_corrupted_cache_file() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("hashes.json");

    // Write invalid JSON
    fs::write(&cache_path, "{ invalid json }")
        .map_err(|e| CleanroomError::io_error(format!("Failed to write file: {}", e)))?;

    // Act - Should create new cache instead of failing
    let result = CacheManager::with_path(cache_path);

    // Assert
    assert!(result.is_ok(), "Should recover from corrupted cache");
    let stats = result?.stats()?;
    assert_eq!(stats.total_files, 0, "Should start with empty cache");

    Ok(())
}

#[test]
fn test_cache_manager_handles_incompatible_version() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("hashes.json");

    // Write cache with old version
    let old_cache = r#"{
        "version": "0.5.0",
        "hashes": {"test": "abc123"},
        "last_updated": "2024-01-01T00:00:00Z"
    }"#;
    fs::write(&cache_path, old_cache)
        .map_err(|e| CleanroomError::io_error(format!("Failed to write file: {}", e)))?;

    // Act - Should create new cache due to version mismatch
    let manager = CacheManager::with_path(cache_path)?;
    let stats = manager.stats()?;

    // Assert
    assert_eq!(
        stats.total_files, 0,
        "Should start fresh due to version mismatch"
    );

    Ok(())
}

// ============================================================================
// Cache Change Detection Tests
// ============================================================================

#[test]
fn test_has_changed_returns_true_for_new_file() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;
    let file_path = PathBuf::from("/test/new_file.toml");

    // Act
    let changed = manager.has_changed(&file_path, "new content")?;

    // Assert
    assert!(changed, "New file should be detected as changed");

    Ok(())
}

#[test]
fn test_has_changed_returns_false_for_unchanged_file() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;
    let file_path = PathBuf::from("/test/file.toml");
    let content = "unchanged content";

    manager.update(&file_path, content)?;

    // Act
    let changed = manager.has_changed(&file_path, content)?;

    // Assert
    assert!(!changed, "Unchanged file should not be detected as changed");

    Ok(())
}

#[test]
fn test_has_changed_returns_true_for_modified_file() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;
    let file_path = PathBuf::from("/test/file.toml");

    manager.update(&file_path, "original content")?;

    // Act
    let changed = manager.has_changed(&file_path, "modified content")?;

    // Assert
    assert!(changed, "Modified file should be detected as changed");

    Ok(())
}

#[test]
fn test_has_changed_detects_whitespace_differences() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;
    let file_path = PathBuf::from("/test/file.toml");

    manager.update(&file_path, "content")?;

    // Act
    let changed = manager.has_changed(&file_path, "content ")?; // Added trailing space

    // Assert
    assert!(
        changed,
        "Whitespace differences should be detected as changes"
    );

    Ok(())
}

#[test]
fn test_has_changed_handles_unicode_content() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;
    let file_path = PathBuf::from("/test/unicode.toml");
    let content = "Hello 世界 🚀";

    manager.update(&file_path, content)?;

    // Act
    let changed = manager.has_changed(&file_path, content)?;

    // Assert
    assert!(!changed, "Unicode content should be handled correctly");

    Ok(())
}

// ============================================================================
// Cache Update and Remove Tests
// ============================================================================

#[test]
fn test_update_adds_file_to_cache() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;
    let file_path = PathBuf::from("/test/file.toml");

    // Act
    manager.update(&file_path, "content")?;
    let stats = manager.stats()?;

    // Assert
    assert_eq!(stats.total_files, 1, "File should be added to cache");

    Ok(())
}

#[test]
fn test_update_replaces_existing_hash() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;
    let file_path = PathBuf::from("/test/file.toml");

    manager.update(&file_path, "original")?;
    manager.update(&file_path, "modified")?;

    // Act
    let stats = manager.stats()?;

    // Assert
    assert_eq!(
        stats.total_files, 1,
        "Should still have only one entry after update"
    );
    assert!(
        !manager.has_changed(&file_path, "modified")?,
        "Should recognize updated content"
    );

    Ok(())
}

#[test]
fn test_remove_deletes_file_from_cache() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;
    let file_path = PathBuf::from("/test/file.toml");

    manager.update(&file_path, "content")?;

    // Act
    manager.remove(&file_path)?;
    let stats = manager.stats()?;

    // Assert
    assert_eq!(stats.total_files, 0, "File should be removed from cache");

    Ok(())
}

#[test]
fn test_remove_nonexistent_file_succeeds() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;
    let file_path = PathBuf::from("/test/nonexistent.toml");

    // Act
    let result = manager.remove(&file_path);

    // Assert
    assert!(
        result.is_ok(),
        "Removing nonexistent file should not error"
    );

    Ok(())
}

// ============================================================================
// Cache Persistence Tests
// ============================================================================

#[test]
fn test_save_creates_cache_file() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("hashes.json");
    let manager = CacheManager::with_path(cache_path.clone())?;

    manager.update(&PathBuf::from("/test/file.toml"), "content")?;

    // Act
    manager.save()?;

    // Assert
    assert!(cache_path.exists(), "Cache file should be created");

    Ok(())
}

#[test]
fn test_save_creates_valid_json() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("hashes.json");
    let manager = CacheManager::with_path(cache_path.clone())?;

    manager.update(&PathBuf::from("/test/file.toml"), "content")?;
    manager.save()?;

    // Act
    let content = fs::read_to_string(&cache_path)
        .map_err(|e| CleanroomError::io_error(format!("Failed to read cache file: {}", e)))?;
    let parsed: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| CleanroomError::serialization_error(format!("Failed to parse JSON: {}", e)))?;

    // Assert
    assert!(parsed["version"].is_string());
    assert!(parsed["hashes"].is_object());
    assert!(parsed["last_updated"].is_string());

    Ok(())
}

#[test]
fn test_save_and_load_preserves_data() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("hashes.json");

    let files = vec![
        ("/test/file1.toml", "content1"),
        ("/test/file2.toml", "content2"),
        ("/test/file3.toml", "content3"),
    ];

    // Save cache
    {
        let manager = CacheManager::with_path(cache_path.clone())?;
        for (path, content) in &files {
            manager.update(&PathBuf::from(path), content)?;
        }
        manager.save()?;
    }

    // Act - Load and verify
    let manager = CacheManager::with_path(cache_path)?;
    let stats = manager.stats()?;

    // Assert
    assert_eq!(stats.total_files, 3, "All files should be preserved");
    for (path, content) in &files {
        assert!(
            !manager.has_changed(&PathBuf::from(path), content)?,
            "File {} should be recognized",
            path
        );
    }

    Ok(())
}

// ============================================================================
// Cache Statistics Tests
// ============================================================================

#[test]
fn test_stats_returns_correct_count() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;

    for i in 0..5 {
        manager.update(&PathBuf::from(format!("/test/file{}.toml", i)), "content")?;
    }

    // Act
    let stats = manager.stats()?;

    // Assert
    assert_eq!(stats.total_files, 5);

    Ok(())
}

#[test]
fn test_stats_includes_cache_path() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path.clone())?;

    // Act
    let stats = manager.stats()?;

    // Assert
    assert_eq!(stats.cache_path, cache_path);

    Ok(())
}

#[test]
fn test_stats_includes_last_updated() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;

    // Act
    let stats = manager.stats()?;

    // Assert
    assert!(
        stats.last_updated.timestamp() > 0,
        "Should have valid timestamp"
    );

    Ok(())
}

// ============================================================================
// Cache Clear Tests
// ============================================================================

#[test]
fn test_clear_removes_all_entries() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;

    for i in 0..10 {
        manager.update(&PathBuf::from(format!("/test/file{}.toml", i)), "content")?;
    }

    // Act
    manager.clear()?;
    let stats = manager.stats()?;

    // Assert
    assert_eq!(stats.total_files, 0, "All entries should be cleared");

    Ok(())
}

#[test]
fn test_clear_allows_new_updates() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;

    manager.update(&PathBuf::from("/test/file.toml"), "content")?;
    manager.clear()?;

    // Act
    manager.update(&PathBuf::from("/test/new_file.toml"), "new content")?;
    let stats = manager.stats()?;

    // Assert
    assert_eq!(
        stats.total_files, 1,
        "Should accept new updates after clear"
    );

    Ok(())
}

// ============================================================================
// Thread Safety Tests
// ============================================================================

#[test]
fn test_concurrent_updates_are_thread_safe() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = Arc::new(CacheManager::with_path(cache_path)?);
    let barrier = Arc::new(Barrier::new(10));

    // Act - Spawn 10 threads updating concurrently
    let mut handles = vec![];
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier_clone.wait(); // Synchronize start
            let path = PathBuf::from(format!("/test/file{}.toml", i));
            let content = format!("content {}", i);
            manager_clone.update(&path, &content).unwrap();
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert
    let stats = manager.stats()?;
    assert_eq!(stats.total_files, 10, "All updates should succeed");

    Ok(())
}

#[test]
fn test_concurrent_read_write_operations() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = Arc::new(CacheManager::with_path(cache_path)?);

    // Pre-populate cache
    for i in 0..5 {
        manager.update(&PathBuf::from(format!("/test/file{}.toml", i)), "content")?;
    }

    let barrier = Arc::new(Barrier::new(20));

    // Act - Mix of readers and writers
    let mut handles = vec![];

    // 10 readers
    for i in 0..10 {
        let manager_clone = Arc::clone(&manager);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier_clone.wait();
            let path = PathBuf::from(format!("/test/file{}.toml", i % 5));
            let _ = manager_clone.has_changed(&path, "content");
        });
        handles.push(handle);
    }

    // 10 writers
    for i in 10..20 {
        let manager_clone = Arc::clone(&manager);
        let barrier_clone = Arc::clone(&barrier);

        let handle = thread::spawn(move || {
            barrier_clone.wait();
            let path = PathBuf::from(format!("/test/new{}.toml", i));
            let _ = manager_clone.update(&path, "new content");
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert - No crashes, all operations completed
    let stats = manager.stats()?;
    assert!(
        stats.total_files >= 5,
        "Should have at least original files"
    );

    Ok(())
}

// ============================================================================
// Edge Cases and Error Handling Tests
// ============================================================================

#[test]
fn test_empty_content_is_handled() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;
    let file_path = PathBuf::from("/test/empty.toml");

    // Act
    manager.update(&file_path, "")?;
    let changed = manager.has_changed(&file_path, "")?;

    // Assert
    assert!(!changed, "Empty content should be handled correctly");

    Ok(())
}

#[test]
fn test_large_content_is_handled() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;
    let file_path = PathBuf::from("/test/large.toml");
    let large_content = "x".repeat(1_000_000); // 1MB content

    // Act
    manager.update(&file_path, &large_content)?;
    let changed = manager.has_changed(&file_path, &large_content)?;

    // Assert
    assert!(!changed, "Large content should be handled correctly");

    Ok(())
}

#[test]
fn test_many_files_in_cache() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;

    // Act - Add 1000 files
    for i in 0..1000 {
        manager.update(&PathBuf::from(format!("/test/file{}.toml", i)), "content")?;
    }

    let stats = manager.stats()?;

    // Assert
    assert_eq!(stats.total_files, 1000, "Should handle many files");

    Ok(())
}

#[test]
fn test_special_characters_in_path() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;
    let file_path = PathBuf::from("/test/file with spaces & symbols!.toml");

    // Act
    manager.update(&file_path, "content")?;
    let changed = manager.has_changed(&file_path, "content")?;

    // Assert
    assert!(
        !changed,
        "Special characters in path should be handled"
    );

    Ok(())
}

// ============================================================================
// Integration with Hash Module Tests
// ============================================================================

#[test]
fn test_cache_uses_hash_module_correctly() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;
    let content = "test content";
    let expected_hash = hash::hash_content(content)?;

    // Act
    manager.update(&PathBuf::from("/test/file.toml"), content)?;
    manager.save()?;

    // Verify hash is used
    let cache_content = fs::read_to_string(temp_dir.path().join("cache.json"))
        .map_err(|e| CleanroomError::io_error(format!("Failed to read cache: {}", e)))?;

    // Assert
    assert!(
        cache_content.contains(&expected_hash),
        "Cache should contain computed hash"
    );

    Ok(())
}

// ============================================================================
// Performance Tests
// ============================================================================

#[test]
fn test_cache_update_performance() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;
    let content = "test content for performance";

    // Act
    let start = std::time::Instant::now();
    for i in 0..100 {
        manager.update(&PathBuf::from(format!("/test/file{}.toml", i)), content)?;
    }
    let duration = start.elapsed();

    // Assert
    assert!(
        duration.as_millis() < 1000,
        "100 updates should complete in <1s, took {}ms",
        duration.as_millis()
    );

    Ok(())
}

#[test]
fn test_cache_has_changed_performance() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new().map_err(|e| {
        CleanroomError::internal_error(format!("Failed to create temp dir: {}", e))
    })?;
    let cache_path = temp_dir.path().join("cache.json");
    let manager = CacheManager::with_path(cache_path)?;
    let content = "test content";

    // Pre-populate
    for i in 0..100 {
        manager.update(&PathBuf::from(format!("/test/file{}.toml", i)), content)?;
    }

    // Act
    let start = std::time::Instant::now();
    for i in 0..100 {
        let _ = manager.has_changed(&PathBuf::from(format!("/test/file{}.toml", i)), content)?;
    }
    let duration = start.elapsed();

    // Assert
    assert!(
        duration.as_millis() < 500,
        "100 checks should complete in <500ms, took {}ms",
        duration.as_millis()
    );

    Ok(())
}
