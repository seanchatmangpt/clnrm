//! Phase 1: RED - Lock-free queue performance tests
//!
//! These tests validate that switching from Mutex<VecDeque> to SegQueue
//! provides 50% latency reduction on acquire/release operations.
//!
//! Target: <500ms for 1000 acquire/release cycles (0.5ms per cycle)

use clnrm_core::backend::{ContainerPool, PoolConfig};
use std::time::{Duration, Instant};

#[tokio::test]
#[ignore = "Requires container runtime (Docker or gVisor)"]
async fn test_lock_free_queue_performance() {
    let config = PoolConfig {
        max_size: 100,
        min_idle: 50,
        health_check_interval: Duration::from_secs(3600), // Disable during test
        ..Default::default()
    };

    let pool = ContainerPool::new(config)
        .await
        .expect("Failed to create pool");

    // Wait for pre-warming to complete
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Stress test: 1000 acquire/release cycles
    let start = Instant::now();
    for _ in 0..1000 {
        let container = pool.acquire().await.expect("Failed to acquire");
        pool.release(container).await.expect("Failed to release");
    }
    let duration = start.elapsed();

    println!(
        "1000 acquire/release cycles took {}ms ({}μs per cycle)",
        duration.as_millis(),
        duration.as_micros() / 1000
    );

    // Should be faster with lock-free SegQueue
    // Target: <500ms for 1000 cycles (0.5ms per cycle)
    assert!(
        duration.as_millis() < 500,
        "1000 acquire/release took {}ms, expected <500ms (lock-free target)",
        duration.as_millis()
    );

    let stats = pool.stats();
    println!(
        "Pool stats - hits: {}, misses: {}, hit_rate: {:.2}%",
        stats.hits,
        stats.misses,
        stats.hit_rate() * 100.0
    );

    pool.shutdown().await.expect("Failed to shutdown pool");
}

#[tokio::test]
#[ignore = "Requires container runtime (Docker or gVisor)"]
async fn test_lock_free_concurrent_acquire_release() {
    let config = PoolConfig {
        max_size: 50,
        min_idle: 25,
        health_check_interval: Duration::from_secs(3600),
        ..Default::default()
    };

    let pool = ContainerPool::new(config)
        .await
        .expect("Failed to create pool");

    // Spawn 16 concurrent tasks
    let mut handles = vec![];
    for task_id in 0..16 {
        let pool_clone = pool.clone();
        let handle = tokio::spawn(async move {
            for i in 0..50 {
                let container = pool_clone.acquire().await.unwrap_or_else(|_| {
                    panic!("Task {} failed acquire at iteration {}", task_id, i)
                });
                pool_clone.release(container).await.unwrap_or_else(|_| {
                    panic!("Task {} failed release at iteration {}", task_id, i)
                });
            }
        });
        handles.push(handle);
    }

    // Wait for all tasks to complete
    for handle in handles {
        handle.await.expect("Task panicked");
    }

    let stats = pool.stats();
    println!(
        "Lock-free concurrent test - Total operations: {}, Hit rate: {:.2}%",
        stats.hits + stats.misses,
        stats.hit_rate() * 100.0
    );

    // Should complete without errors
    assert!(stats.hits + stats.misses > 0);

    pool.shutdown().await.expect("Failed to shutdown pool");
}
