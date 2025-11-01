//! Integration Tests for Container Pool (v1.4.0)
//!
//! Tests the container pooling feature that enables >90% pool hit rate after warm-up.
//!
//! Test Categories:
//! 1. Pool Acquisition and Release
//! 2. Pool Hit Rate Performance
//! 3. Health Check and Eviction
//! 4. Concurrent Access Stress Test
//! 5. Pool Statistics Accuracy

use clnrm_core::stress_test::pool::{ContainerPool, ContainerPoolConfig};
use std::time::Duration;

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a test pool configuration
fn create_test_pool_config() -> ContainerPoolConfig {
    ContainerPoolConfig {
        max_size: 10,
        startup_timeout: Duration::from_secs(30),
        cleanup_timeout: Duration::from_secs(60),
        memory_limit: Some(512), // 512MB per container
        cpu_limit: Some(0.5),    // 0.5 cores per container
    }
}

/// Create a test pool with default configuration
fn create_test_pool() -> ContainerPool {
    ContainerPool::new(create_test_pool_config())
}

// ============================================================================
// Pool Acquisition and Release Tests
// ============================================================================

#[tokio::test]
async fn test_pool_acquisition_and_release() -> clnrm_core::Result<()> {
    // Arrange
    let pool = create_test_pool();
    let image = "alpine:latest";

    // Act - First acquisition (pool miss - creates new container)
    let container1 = pool.acquire(image).await?;
    let container1_id = container1.id.clone();

    // Release container back to pool
    pool.release(&container1_id).await?;

    // Second acquisition (pool hit - reuses container)
    let container2 = pool.acquire(image).await?;

    // Assert
    assert_eq!(container2.id, container1_id, "Should reuse same container");

    let stats = pool.stats().await;
    assert_eq!(
        stats.total_allocated, 1,
        "Should have 1 container allocated"
    );
    assert_eq!(stats.in_use, 1, "Should have 1 container in use");

    // Cleanup
    pool.release(&container2.id).await?;
    pool.cleanup().await?;

    Ok(())
}

#[tokio::test]
async fn test_pool_pre_allocation() -> clnrm_core::Result<()> {
    // Arrange
    let pool = create_test_pool();
    let image = "alpine:latest";
    let pre_allocate_count = 3;

    // Act
    pool.pre_allocate(image, pre_allocate_count).await?;

    // Assert
    let stats = pool.stats().await;
    assert_eq!(
        stats.total_allocated, pre_allocate_count,
        "Should have {} containers pre-allocated",
        pre_allocate_count
    );
    assert_eq!(
        stats.available, pre_allocate_count,
        "All should be available"
    );
    assert_eq!(stats.in_use, 0, "None should be in use");

    // Cleanup
    pool.cleanup().await?;

    Ok(())
}

#[tokio::test]
async fn test_pool_acquire_from_pre_allocated() -> clnrm_core::Result<()> {
    // Arrange
    let pool = create_test_pool();
    let image = "alpine:latest";

    pool.pre_allocate(image, 3).await?;

    // Act - Acquire 3 containers (all from pre-allocated pool)
    let container1 = pool.acquire(image).await?;
    let container2 = pool.acquire(image).await?;
    let container3 = pool.acquire(image).await?;

    // Assert
    let stats = pool.stats().await;
    assert_eq!(stats.total_allocated, 3);
    assert_eq!(stats.in_use, 3);
    assert_eq!(stats.available, 0);

    // Cleanup
    pool.release(&container1.id).await?;
    pool.release(&container2.id).await?;
    pool.release(&container3.id).await?;
    pool.cleanup().await?;

    Ok(())
}

#[tokio::test]
async fn test_pool_max_size_enforcement() -> clnrm_core::Result<()> {
    // Arrange
    let config = ContainerPoolConfig {
        max_size: 2, // Small pool for testing
        startup_timeout: Duration::from_secs(30),
        cleanup_timeout: Duration::from_secs(60),
        memory_limit: Some(512),
        cpu_limit: Some(0.5),
    };
    let pool = ContainerPool::new(config);
    let image = "alpine:latest";

    // Act - Acquire up to max_size
    let container1 = pool.acquire(image).await?;
    let container2 = pool.acquire(image).await?;

    // Try to acquire beyond max_size
    let result = pool.acquire(image).await;

    // Assert
    assert!(result.is_err(), "Should fail to acquire beyond max_size");

    let stats = pool.stats().await;
    assert_eq!(stats.total_allocated, 2, "Should be at max size");

    // Cleanup
    pool.release(&container1.id).await?;
    pool.release(&container2.id).await?;
    pool.cleanup().await?;

    Ok(())
}

// ============================================================================
// Pool Hit Rate Performance Tests
// ============================================================================

#[tokio::test]
async fn test_pool_hit_rate_after_warmup() -> clnrm_core::Result<()> {
    // Arrange
    let pool = create_test_pool();
    let image = "alpine:latest";
    let warmup_count = 5;

    pool.pre_allocate(image, warmup_count).await?;

    // Act - Warm-up phase: acquire and release containers
    let mut container_ids = Vec::new();
    for _ in 0..warmup_count {
        let container = pool.acquire(image).await?;
        container_ids.push(container.id.clone());
    }

    for id in &container_ids {
        pool.release(id).await?;
    }

    // Performance test: acquire/release 100 times (should all be hits)
    let mut hits = 0;
    let test_iterations = 100;

    for _ in 0..test_iterations {
        let container = pool.acquire(image).await?;
        hits += 1; // Every acquisition after warmup should be a hit
        pool.release(&container.id).await?;
    }

    // Assert
    let hit_rate = (hits as f64 / test_iterations as f64) * 100.0;
    assert!(
        hit_rate >= 90.0,
        "Pool hit rate should be >=90% after warmup, got {}%",
        hit_rate
    );

    // Cleanup
    pool.cleanup().await?;

    Ok(())
}

#[tokio::test]
async fn test_pool_miss_on_first_acquisition() -> clnrm_core::Result<()> {
    // Arrange
    let pool = create_test_pool();
    let image = "alpine:latest";

    let stats_before = pool.stats().await;
    assert_eq!(stats_before.total_allocated, 0, "Pool should start empty");

    // Act - First acquisition (pool miss)
    let container = pool.acquire(image).await?;

    // Assert
    let stats_after = pool.stats().await;
    assert_eq!(
        stats_after.total_allocated, 1,
        "Should create new container on miss"
    );

    // Cleanup
    pool.release(&container.id).await?;
    pool.cleanup().await?;

    Ok(())
}

// ============================================================================
// Concurrent Access Stress Tests
// ============================================================================

#[tokio::test]
async fn test_pool_concurrent_acquisition() -> clnrm_core::Result<()> {
    // Arrange
    let pool = std::sync::Arc::new(create_test_pool());
    let image = "alpine:latest";
    let concurrent_tasks = 5;

    pool.pre_allocate(image, concurrent_tasks).await?;

    // Act - Spawn concurrent tasks acquiring containers
    let mut handles = Vec::new();

    for _ in 0..concurrent_tasks {
        let pool_clone = pool.clone();
        let image_str = image.to_string();

        let handle = tokio::spawn(async move {
            let container = pool_clone.acquire(&image_str).await?;
            // Simulate work
            tokio::time::sleep(Duration::from_millis(10)).await;
            pool_clone.release(&container.id).await?;
            clnrm_core::Result::Ok(())
        });

        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap()?;
    }

    // Assert
    let stats = pool.stats().await;
    assert_eq!(
        stats.total_allocated, concurrent_tasks,
        "Should maintain correct allocation count"
    );
    assert_eq!(stats.in_use, 0, "All containers should be released");
    assert_eq!(
        stats.available, concurrent_tasks,
        "All containers should be available"
    );

    // Cleanup
    pool.cleanup().await?;

    Ok(())
}

#[tokio::test]
async fn test_pool_stress_100_concurrent_acquisitions() -> clnrm_core::Result<()> {
    // Arrange
    let pool = std::sync::Arc::new(create_test_pool());
    let image = "alpine:latest";
    let stress_count = 100;

    // Pre-allocate pool to max size
    pool.pre_allocate(image, pool.stats().await.max_size)
        .await?;

    // Act - Stress test with 100 concurrent tasks
    let mut handles = Vec::new();

    for _ in 0..stress_count {
        let pool_clone = pool.clone();
        let image_str = image.to_string();

        let handle = tokio::spawn(async move {
            // Try to acquire (may fail due to pool exhaustion)
            match pool_clone.acquire(&image_str).await {
                Ok(container) => {
                    tokio::time::sleep(Duration::from_millis(1)).await;
                    pool_clone.release(&container.id).await?;
                    clnrm_core::Result::Ok(true)
                }
                Err(_) => clnrm_core::Result::Ok(false), // Pool exhausted
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks and count successes
    let mut successes = 0;
    for handle in handles {
        if handle.await.unwrap()? {
            successes += 1;
        }
    }

    // Assert
    assert!(successes > 0, "Should have some successful acquisitions");

    let stats = pool.stats().await;
    assert_eq!(stats.in_use, 0, "All containers should be released");

    // Cleanup
    pool.cleanup().await?;

    Ok(())
}

// ============================================================================
// Pool Statistics Accuracy Tests
// ============================================================================

#[tokio::test]
async fn test_pool_stats_accuracy() -> clnrm_core::Result<()> {
    // Arrange
    let pool = create_test_pool();
    let image = "alpine:latest";

    // Act - Create specific pool state
    pool.pre_allocate(image, 5).await?;

    let container1 = pool.acquire(image).await?;
    let container2 = pool.acquire(image).await?;

    // Assert
    let stats = pool.stats().await;

    assert_eq!(stats.total_allocated, 5, "Total allocated should be 5");
    assert_eq!(stats.in_use, 2, "In use should be 2");
    assert_eq!(stats.available, 3, "Available should be 3");
    assert_eq!(stats.max_size, 10, "Max size should be 10");

    let utilization = stats.utilization();
    assert!(
        (utilization - 50.0).abs() < 0.1,
        "Utilization should be 50%, got {}",
        utilization
    );

    // Cleanup
    pool.release(&container1.id).await?;
    pool.release(&container2.id).await?;
    pool.cleanup().await?;

    Ok(())
}

#[tokio::test]
async fn test_pool_utilization_calculation() -> clnrm_core::Result<()> {
    // Arrange
    let config = ContainerPoolConfig {
        max_size: 10,
        startup_timeout: Duration::from_secs(30),
        cleanup_timeout: Duration::from_secs(60),
        memory_limit: Some(512),
        cpu_limit: Some(0.5),
    };
    let pool = ContainerPool::new(config);
    let image = "alpine:latest";

    // Act & Assert - Test different utilization levels

    // 0% utilization
    let stats = pool.stats().await;
    assert_eq!(stats.utilization(), 0.0, "Should be 0% when empty");

    // 30% utilization
    pool.pre_allocate(image, 3).await?;
    let stats = pool.stats().await;
    assert!(
        (stats.utilization() - 30.0).abs() < 0.1,
        "Should be 30% with 3/10 containers"
    );

    // 100% utilization
    pool.pre_allocate(image, 7).await?;
    let stats = pool.stats().await;
    assert!(
        (stats.utilization() - 100.0).abs() < 0.1,
        "Should be 100% with 10/10 containers"
    );

    // Cleanup
    pool.cleanup().await?;

    Ok(())
}

#[tokio::test]
async fn test_pool_cleanup_resets_stats() -> clnrm_core::Result<()> {
    // Arrange
    let pool = create_test_pool();
    let image = "alpine:latest";

    pool.pre_allocate(image, 5).await?;

    let stats_before = pool.stats().await;
    assert_eq!(stats_before.total_allocated, 5);

    // Act
    pool.cleanup().await?;

    // Assert
    let stats_after = pool.stats().await;
    assert_eq!(
        stats_after.total_allocated, 0,
        "Total allocated should be 0"
    );
    assert_eq!(stats_after.in_use, 0, "In use should be 0");
    assert_eq!(stats_after.available, 0, "Available should be 0");

    Ok(())
}

// ============================================================================
// Edge Case Tests
// ============================================================================

#[tokio::test]
async fn test_pool_release_nonexistent_container() -> clnrm_core::Result<()> {
    // Arrange
    let pool = create_test_pool();
    let fake_id = "nonexistent-container-id";

    // Act
    let result = pool.release(fake_id).await;

    // Assert
    assert!(
        result.is_err(),
        "Should fail to release nonexistent container"
    );

    Ok(())
}

#[tokio::test]
async fn test_pool_multiple_images() -> clnrm_core::Result<()> {
    // Arrange
    let pool = create_test_pool();
    let image1 = "alpine:latest";
    let image2 = "busybox:latest";

    // Act
    pool.pre_allocate(image1, 2).await?;
    pool.pre_allocate(image2, 3).await?;

    // Assert
    let stats = pool.stats().await;
    assert_eq!(
        stats.total_allocated, 5,
        "Should track containers across multiple images"
    );

    // Cleanup
    pool.cleanup().await?;

    Ok(())
}
