//! Test suite for lock-free cache implementation
//!
//! Verifies that the cache handles concurrent access without lock poisoning.

#[allow(unused_imports)]
use clnrm_template::cache::{CacheStats, TemplateCache};
#[allow(unused_imports)]
use std::path::Path;
use std::sync::Arc;
use std::thread;
use std::time::Duration;

#[test]
fn test_cache_concurrent_reads_no_panic() {
    let cache = Arc::new(TemplateCache::with_defaults());

    // Populate cache
    cache.get_or_compile("test1", "content1", None).unwrap();
    cache.get_or_compile("test2", "content2", None).unwrap();

    // Spawn multiple readers
    let mut handles = vec![];
    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let key = format!("test{}", (i % 2) + 1);
                let result = cache_clone.get_or_compile(&key, "content", None);
                assert!(result.is_ok(), "Cache read should never panic");
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let stats = cache.stats();
    assert!(stats.hits > 0, "Should have cache hits");
}

#[test]
fn test_cache_concurrent_writes_no_panic() {
    let cache = Arc::new(TemplateCache::with_defaults());

    // Spawn multiple writers
    let mut handles = vec![];
    for i in 0..10 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for j in 0..100 {
                let key = format!("test_{}", i * 100 + j);
                let content = format!("content_{}", j);
                let result = cache_clone.get_or_compile(&key, &content, None);
                assert!(result.is_ok(), "Cache write should never panic");
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let stats = cache.stats();
    assert_eq!(stats.template_count, 1000, "All templates should be cached");
}

#[test]
fn test_cache_concurrent_mixed_operations_no_panic() {
    let cache = Arc::new(TemplateCache::with_defaults());

    // Pre-populate some templates
    for i in 0..10 {
        cache
            .get_or_compile(&format!("template{}", i), "content", None)
            .unwrap();
    }

    let mut handles = vec![];

    // Spawn readers
    for _ in 0..5 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let key = format!("template{}", i % 10);
                cache_clone.get_or_compile(&key, "content", None).unwrap();
            }
        });
        handles.push(handle);
    }

    // Spawn writers
    for thread_id in 0..5 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for i in 0..100 {
                let key = format!("new_template_{}_{}", thread_id, i);
                cache_clone
                    .get_or_compile(&key, "new content", None)
                    .unwrap();
            }
        });
        handles.push(handle);
    }

    // Spawn stats readers
    for _ in 0..3 {
        let cache_clone = Arc::clone(&cache);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                let _stats = cache_clone.stats();
            }
        });
        handles.push(handle);
    }

    // Spawn clear operations
    let cache_clone = Arc::clone(&cache);
    let clear_handle = thread::spawn(move || {
        thread::sleep(Duration::from_millis(10));
        cache_clone.clear();
    });
    handles.push(clear_handle);

    for handle in handles {
        handle.join().expect("Thread should not panic");
    }
}

#[test]
fn test_cache_stats_always_succeeds() {
    let cache = TemplateCache::with_defaults();

    // Stats should work even on empty cache
    let stats = cache.stats();
    assert_eq!(stats.hits, 0);
    assert_eq!(stats.misses, 0);

    // Add some data
    cache.get_or_compile("test", "content", None).unwrap();

    // Stats should work after operations
    let stats = cache.stats();
    assert_eq!(stats.misses, 1);
    assert_eq!(stats.template_count, 1);
}

#[test]
fn test_cache_clear_no_panic() {
    let cache = Arc::new(TemplateCache::with_defaults());

    // Populate cache
    for i in 0..100 {
        cache
            .get_or_compile(&format!("test{}", i), "content", None)
            .unwrap();
    }

    // Clear should work
    cache.clear();

    let stats = cache.stats();
    assert_eq!(stats.template_count, 0);
    assert_eq!(stats.total_size, 0);
}

#[test]
fn test_cache_eviction_no_panic() {
    let cache = TemplateCache::new(false, Duration::from_millis(1));

    // Add templates
    for i in 0..10 {
        cache
            .get_or_compile(&format!("test{}", i), "content", None)
            .unwrap();
    }

    // Wait for expiration
    thread::sleep(Duration::from_millis(10));

    // Eviction should work
    let evicted = cache.evict_expired();
    assert_eq!(evicted, 10, "All templates should be evicted");

    let stats = cache.stats();
    assert_eq!(stats.template_count, 0);
}

#[test]
fn test_cache_hot_reload_no_panic() {
    use std::io::Write;
    use tempfile::NamedTempFile;

    let cache = TemplateCache::new(true, Duration::from_secs(3600));

    // Create temp file
    let mut temp_file = NamedTempFile::new().unwrap();
    write!(temp_file, "original content").unwrap();
    let path = temp_file.path();

    // First compilation
    let result1 = cache
        .get_or_compile("test", "original content", Some(path))
        .unwrap();
    assert_eq!(result1, "original content");

    let stats = cache.stats();
    assert_eq!(stats.misses, 1);

    // Second access should hit cache
    let result2 = cache
        .get_or_compile("test", "original content", Some(path))
        .unwrap();
    assert_eq!(result2, "original content");

    let stats = cache.stats();
    assert_eq!(stats.hits, 1);
}
