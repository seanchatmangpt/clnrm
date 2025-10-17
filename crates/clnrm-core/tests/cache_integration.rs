//! Integration tests for cache subsystem
//!
//! Demonstrates London School TDD principles:
//! - Testing through trait interface
//! - Behavior verification over state testing
//! - Collaboration between components
//! - Real-world usage patterns

use clnrm_core::cache::{Cache, FileCache, MemoryCache};
use clnrm_core::Result;
use std::path::PathBuf;
use tempfile::TempDir;

/// Test using cache through trait interface (polymorphism)
#[test]
fn test_cache_polymorphism() -> Result<()> {
    // Arrange - can use different backends through same interface
    let file_cache: Box<dyn Cache> = Box::new(
        FileCache::with_path(TempDir::new()?.path().join("cache.json"))?,
    );
    let memory_cache: Box<dyn Cache> = Box::new(MemoryCache::new());

    let caches: Vec<Box<dyn Cache>> = vec![file_cache, memory_cache];

    let test_path = PathBuf::from("/test/file.toml");
    let content = "test content";

    // Act & Assert - both backends behave identically through trait
    for cache in caches {
        assert!(cache.has_changed(&test_path, content)?);
        cache.update(&test_path, content)?;
        assert!(!cache.has_changed(&test_path, content)?);
    }

    Ok(())
}

/// Test typical test runner workflow
#[test]
fn test_runner_workflow_simulation() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let test_files = vec![
        (PathBuf::from("/tests/api.clnrm.toml"), "api test config"),
        (PathBuf::from("/tests/db.clnrm.toml"), "db test config"),
        (PathBuf::from("/tests/auth.clnrm.toml"), "auth test config"),
    ];

    // Act - First run: all tests should execute
    let mut tests_run_first = 0;
    for (path, content) in &test_files {
        if cache.has_changed(path, content)? {
            // Simulate running test
            tests_run_first += 1;
            cache.update(path, content)?;
        }
    }

    // Second run: no tests should execute (nothing changed)
    let mut tests_run_second = 0;
    for (path, content) in &test_files {
        if cache.has_changed(path, content)? {
            tests_run_second += 1;
            cache.update(path, content)?;
        }
    }

    // Third run: one test changes
    let mut tests_run_third = 0;
    let updated_test_files = vec![
        (
            PathBuf::from("/tests/api.clnrm.toml"),
            "api test config UPDATED",
        ), // Changed
        (PathBuf::from("/tests/db.clnrm.toml"), "db test config"),
        (PathBuf::from("/tests/auth.clnrm.toml"), "auth test config"),
    ];

    for (path, content) in &updated_test_files {
        if cache.has_changed(path, content)? {
            tests_run_third += 1;
            cache.update(path, content)?;
        }
    }

    // Assert - verify 10x performance improvement pattern
    assert_eq!(tests_run_first, 3, "First run: all 3 tests should execute");
    assert_eq!(tests_run_second, 0, "Second run: 0 tests should execute");
    assert_eq!(tests_run_third, 1, "Third run: only 1 changed test should execute");

    Ok(())
}

/// Test cache persistence across process restarts
#[test]
fn test_cache_persistence() -> Result<()> {
    // Arrange
    let temp_dir = TempDir::new()?;
    let cache_path = temp_dir.path().join("cache.json");

    let test_path = PathBuf::from("/test/persistent.toml");
    let content = "persistent content";

    // Act - Session 1: create cache and update
    {
        let cache = FileCache::with_path(cache_path.clone())?;
        cache.update(&test_path, content)?;
        cache.save()?;
    }

    // Session 2: load cache and verify
    {
        let cache = FileCache::with_path(cache_path)?;
        let changed = cache.has_changed(&test_path, content)?;
        assert!(
            !changed,
            "Cache should persist data across process restarts"
        );
    }

    Ok(())
}

/// Test concurrent cache access (thread safety)
#[test]
fn test_concurrent_cache_access() -> Result<()> {
    use std::sync::Arc;
    use std::thread;

    // Arrange
    let cache = Arc::new(MemoryCache::new());

    // Act - spawn 20 threads, each updating 10 files
    let mut handles = vec![];
    for thread_id in 0..20 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for file_id in 0..10 {
                let path = PathBuf::from(format!("/test/thread{}_file{}.toml", thread_id, file_id));
                let content = format!("thread {} file {}", thread_id, file_id);

                cache_clone.update(&path, &content).unwrap();
                cache_clone.has_changed(&path, &content).unwrap();
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Assert
    let stats = cache.stats()?;
    assert_eq!(
        stats.total_files, 200,
        "All concurrent updates should succeed"
    );

    Ok(())
}

/// Test cache invalidation strategies
#[test]
fn test_cache_invalidation_strategies() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let test_path = PathBuf::from("/test/file.toml");

    // Strategy 1: Content-based invalidation
    cache.update(&test_path, "version 1")?;
    assert!(
        !cache.has_changed(&test_path, "version 1")?,
        "Same content should not trigger invalidation"
    );
    assert!(
        cache.has_changed(&test_path, "version 2")?,
        "Different content should trigger invalidation"
    );

    // Strategy 2: Explicit invalidation via remove
    cache.update(&test_path, "version 2")?;
    cache.remove(&test_path)?;
    assert!(
        cache.has_changed(&test_path, "version 2")?,
        "Removed file should be invalidated"
    );

    // Strategy 3: Bulk invalidation via clear
    cache.update(&PathBuf::from("/test/file1.toml"), "content1")?;
    cache.update(&PathBuf::from("/test/file2.toml"), "content2")?;
    cache.clear()?;
    assert!(
        cache.has_changed(&PathBuf::from("/test/file1.toml"), "content1")?,
        "Cleared cache should invalidate all entries"
    );

    Ok(())
}

/// Test cache statistics and monitoring
#[test]
fn test_cache_statistics() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();

    // Initially empty
    let stats = cache.stats()?;
    assert_eq!(stats.total_files, 0);

    // Add files
    cache.update(&PathBuf::from("/test/file1.toml"), "content1")?;
    cache.update(&PathBuf::from("/test/file2.toml"), "content2")?;
    cache.update(&PathBuf::from("/test/file3.toml"), "content3")?;

    // Check stats
    let stats = cache.stats()?;
    assert_eq!(stats.total_files, 3);
    assert!(stats.cache_path.is_none()); // MemoryCache has no path

    // FileCache should have path
    let temp_dir = TempDir::new()?;
    let file_cache = FileCache::with_path(temp_dir.path().join("cache.json"))?;
    file_cache.update(&PathBuf::from("/test/file.toml"), "content")?;

    let file_stats = file_cache.stats()?;
    assert!(file_stats.cache_path.is_some());

    Ok(())
}

/// Test cache behavior with template rendering workflow
#[test]
fn test_cache_with_template_rendering() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();

    // Simulate Tera template rendering workflow
    let template_path = PathBuf::from("/tests/api_test.clnrm.toml");

    let rendered_v1 = r#"
[test.metadata]
name = "api_test"
environment = "dev"

[[steps]]
command = ["echo", "hello"]
"#;

    let rendered_v2 = r#"
[test.metadata]
name = "api_test"
environment = "prod"  # Environment changed

[[steps]]
command = ["echo", "hello"]
"#;

    // Act - First render
    assert!(cache.has_changed(&template_path, rendered_v1)?);
    cache.update(&template_path, rendered_v1)?;

    // Second render with same output
    assert!(!cache.has_changed(&template_path, rendered_v1)?);

    // Third render with different output (template variable changed)
    assert!(cache.has_changed(&template_path, rendered_v2)?);

    Ok(())
}

/// Test error handling in cache operations
#[test]
fn test_cache_error_handling() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();

    // Invalid path encoding should be handled gracefully
    // (This is hard to test on modern systems, but the error path exists)

    // Normal operations should succeed
    let path = PathBuf::from("/test/file.toml");
    assert!(cache.update(&path, "content").is_ok());
    assert!(cache.has_changed(&path, "content").is_ok());
    assert!(cache.remove(&path).is_ok());
    assert!(cache.save().is_ok());
    assert!(cache.clear().is_ok());
    assert!(cache.stats().is_ok());

    Ok(())
}

/// Test cache collaboration with file watcher
#[test]
fn test_cache_file_watcher_collaboration() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();

    // Simulate file watcher detecting changes
    let watch_files = vec![
        PathBuf::from("/tests/api.clnrm.toml"),
        PathBuf::from("/tests/db.clnrm.toml"),
    ];

    // Initial state
    for file in &watch_files {
        cache.update(file, "initial content")?;
    }

    // Act - File watcher detects change in one file
    let changed_file = &watch_files[0];
    let new_content = "modified content";

    // Check which tests need to run
    let needs_rerun_api = cache.has_changed(changed_file, new_content)?;
    let needs_rerun_db = cache.has_changed(&watch_files[1], "initial content")?;

    // Assert - verify selective re-execution
    assert!(needs_rerun_api, "Changed file should trigger re-run");
    assert!(!needs_rerun_db, "Unchanged file should not trigger re-run");

    Ok(())
}

/// Test cache behavior with large number of files
#[test]
fn test_cache_scalability() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();
    let num_files = 1000;

    // Act - add many files
    for i in 0..num_files {
        let path = PathBuf::from(format!("/tests/test_{}.clnrm.toml", i));
        let content = format!("test content {}", i);
        cache.update(&path, &content)?;
    }

    // Assert
    let stats = cache.stats()?;
    assert_eq!(
        stats.total_files, num_files,
        "Cache should handle large number of files"
    );

    // Verify random access is fast
    let test_path = PathBuf::from("/tests/test_500.clnrm.toml");
    let changed = cache.has_changed(&test_path, "test content 500")?;
    assert!(!changed);

    Ok(())
}

/// Test cache with different content types
#[test]
fn test_cache_content_types() -> Result<()> {
    // Arrange
    let cache = MemoryCache::new();

    // Different content types that might appear in rendered templates
    let test_cases = vec![
        ("empty", ""),
        ("simple", "key = value"),
        ("multiline", "key1 = value1\nkey2 = value2\nkey3 = value3"),
        ("unicode", "message = \"Hello 世界 🌍\""),
        ("json-like", r#"{"key": "value", "nested": {"a": 1}}"#),
        ("toml", r#"[section]
key = "value"
array = [1, 2, 3]"#),
    ];

    // Act & Assert
    for (name, content) in test_cases {
        let path = PathBuf::from(format!("/test/{}.toml", name));

        // First check should be changed
        assert!(
            cache.has_changed(&path, content)?,
            "New file {} should be changed",
            name
        );

        // Update cache
        cache.update(&path, content)?;

        // Second check should be unchanged
        assert!(
            !cache.has_changed(&path, content)?,
            "Cached file {} should be unchanged",
            name
        );
    }

    Ok(())
}
