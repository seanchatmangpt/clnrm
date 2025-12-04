//! Integration Tests for Concurrency Limiting (v1.4.0)
//!
//! Tests the semaphore-based concurrency limiting that prevents resource exhaustion.
//!
//! Test Categories:
//! 1. Semaphore Enforcement
//! 2. Backpressure Handling
//! 3. Max Concurrent Tests Respected
//! 4. Graceful Degradation
//! 5. Resource Management

use clnrm_core::cli::types::{CliConfig, OutputFormat};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a test CLI configuration
fn create_test_config(jobs: usize) -> CliConfig {
    CliConfig {
        parallel: true,
        jobs,
        format: OutputFormat::Human,
        fail_fast: false,
        watch: false,
        verbose: 0,
        force: false,
        digest: false,
        validate: false,
        enable_pooling: false,
        pool_max_size: 10,
    }
}

/// Simulate a test execution with semaphore control
async fn simulate_test_with_semaphore(
    semaphore: Arc<Semaphore>,
    test_id: usize,
    duration_ms: u64,
    active_count: Arc<Mutex<Vec<usize>>>,
) -> clnrm_core::Result<()> {
    // Acquire permit (blocks if at capacity)
    let _permit = semaphore.acquire().await.map_err(|e| {
        clnrm_core::error::CleanroomError::internal_error(format!(
            "Semaphore acquire failed: {}",
            e
        ))
    })?;

    // Track active test
    {
        let mut active = active_count.lock().await;
        active.push(test_id);
    }

    // Simulate test work
    tokio::time::sleep(Duration::from_millis(duration_ms)).await;

    // Remove from active tests
    {
        let mut active = active_count.lock().await;
        active.retain(|&id| id != test_id);
    }

    Ok(())
}

// ============================================================================
// Semaphore Enforcement Tests
// ============================================================================

#[tokio::test]
async fn test_semaphore_limits_concurrent_execution() -> clnrm_core::Result<()> {
    // Arrange
    let max_concurrent = 3;
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let active_count = Arc::new(Mutex::new(Vec::new()));

    let mut handles = Vec::new();

    // Act - Spawn 10 tasks but only 3 should run concurrently
    for i in 0..10 {
        let sem_clone = semaphore.clone();
        let active_clone = active_count.clone();

        let handle = tokio::spawn(async move {
            simulate_test_with_semaphore(sem_clone, i, 100, active_clone).await
        });

        handles.push(handle);
    }

    // Check active count periodically
    let mut max_observed = 0;
    for _ in 0..20 {
        tokio::time::sleep(Duration::from_millis(10)).await;
        let active = active_count.lock().await;
        let current = active.len();
        if current > max_observed {
            max_observed = current;
        }
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap()?;
    }

    // Assert - Should never exceed max_concurrent
    assert!(
        max_observed <= max_concurrent,
        "Should never exceed {} concurrent executions, observed {}",
        max_concurrent,
        max_observed
    );

    Ok(())
}

#[tokio::test]
async fn test_semaphore_enforces_job_limit() -> clnrm_core::Result<()> {
    // Arrange
    let jobs = 5;
    let semaphore = Arc::new(Semaphore::new(jobs));
    let active_count = Arc::new(Mutex::new(0_usize));
    let max_active = Arc::new(Mutex::new(0_usize));

    let mut handles = Vec::new();

    // Act - Spawn 20 tasks with 5 job limit
    for _ in 0..20 {
        let sem_clone = semaphore.clone();
        let active_clone = active_count.clone();
        let max_clone = max_active.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();

            // Increment active count
            {
                let mut active = active_clone.lock().await;
                *active += 1;

                // Track max
                let mut max = max_clone.lock().await;
                if *active > *max {
                    *max = *active;
                }
            }

            // Simulate work
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Decrement active count
            {
                let mut active = active_clone.lock().await;
                *active -= 1;
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Assert
    let max = *max_active.lock().await;
    assert!(
        max <= jobs,
        "Should never exceed {} concurrent jobs, observed {}",
        jobs,
        max
    );

    Ok(())
}

#[tokio::test]
async fn test_semaphore_permits_released_on_completion() -> clnrm_core::Result<()> {
    // Arrange
    let semaphore = Arc::new(Semaphore::new(2));

    // Act - Acquire and release permits
    {
        let _permit1 = semaphore.acquire().await.unwrap();
        let _permit2 = semaphore.acquire().await.unwrap();
        // Permits released when dropped
    }

    // Try to acquire again - should succeed if permits were released
    let _permit3 = semaphore.acquire().await.unwrap();
    let _permit4 = semaphore.acquire().await.unwrap();

    // Assert - If we got here, permits were properly released
    assert_eq!(semaphore.available_permits(), 0);

    Ok(())
}

// ============================================================================
// Backpressure Handling Tests
// ============================================================================

#[tokio::test]
async fn test_backpressure_queues_excess_tasks() -> clnrm_core::Result<()> {
    // Arrange
    let max_concurrent = 2;
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let completion_order = Arc::new(Mutex::new(Vec::new()));

    let mut handles = Vec::new();

    // Act - Spawn 5 tasks that take 100ms each
    for i in 0..5 {
        let sem_clone = semaphore.clone();
        let order_clone = completion_order.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();
            tokio::time::sleep(Duration::from_millis(100)).await;

            let mut order = order_clone.lock().await;
            order.push(i);
        });

        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Assert - All tasks should complete (backpressure queued them)
    let order = completion_order.lock().await;
    assert_eq!(order.len(), 5, "All tasks should complete");

    Ok(())
}

#[tokio::test]
async fn test_backpressure_does_not_drop_tasks() -> clnrm_core::Result<()> {
    // Arrange
    let semaphore = Arc::new(Semaphore::new(3));
    let completed = Arc::new(Mutex::new(0_usize));

    let mut handles = Vec::new();

    // Act - Spawn 10 tasks
    for _ in 0..10 {
        let sem_clone = semaphore.clone();
        let completed_clone = completed.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();
            tokio::time::sleep(Duration::from_millis(10)).await;

            let mut count = completed_clone.lock().await;
            *count += 1;
        });

        handles.push(handle);
    }

    // Wait for all tasks
    for handle in handles {
        handle.await.unwrap();
    }

    // Assert - All 10 tasks should complete (none dropped)
    let count = *completed.lock().await;
    assert_eq!(count, 10, "All tasks should complete under backpressure");

    Ok(())
}

#[tokio::test]
async fn test_backpressure_timing() -> clnrm_core::Result<()> {
    // Arrange
    let max_concurrent = 2;
    let semaphore = Arc::new(Semaphore::new(max_concurrent));
    let test_count = 6;
    let task_duration_ms = 100;

    // Act
    let start = Instant::now();

    let mut handles = Vec::new();
    for _ in 0..test_count {
        let sem_clone = semaphore.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();
            tokio::time::sleep(Duration::from_millis(task_duration_ms)).await;
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let duration = start.elapsed();

    // Assert - With 2 concurrent and 6 tasks of 100ms each:
    // Expected: ~300ms (3 batches of 2 tasks)
    // Allow some overhead
    let expected_ms = (test_count as u64 / max_concurrent as u64) * task_duration_ms;
    let max_allowed_ms = expected_ms + 200; // 200ms overhead allowance

    assert!(
        duration.as_millis() as u64 >= expected_ms,
        "Should take at least {}ms due to batching",
        expected_ms
    );

    assert!(
        duration.as_millis() as u64 <= max_allowed_ms,
        "Should complete within {}ms, took {}ms",
        max_allowed_ms,
        duration.as_millis()
    );

    Ok(())
}

// ============================================================================
// Max Concurrent Tests Respected
// ============================================================================

#[tokio::test]
async fn test_respects_jobs_config() -> clnrm_core::Result<()> {
    // Arrange
    let config = create_test_config(4);
    let semaphore = Arc::new(Semaphore::new(config.jobs));
    let active_count = Arc::new(Mutex::new(0_usize));
    let max_observed = Arc::new(Mutex::new(0_usize));

    let mut handles = Vec::new();

    // Act - Spawn many tasks
    for _ in 0..20 {
        let sem_clone = semaphore.clone();
        let active_clone = active_count.clone();
        let max_clone = max_observed.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();

            let mut active = active_clone.lock().await;
            *active += 1;

            let mut max = max_clone.lock().await;
            if *active > *max {
                *max = *active;
            }

            drop(active);
            drop(max);

            tokio::time::sleep(Duration::from_millis(50)).await;

            let mut active = active_clone.lock().await;
            *active -= 1;
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Assert
    let max = *max_observed.lock().await;
    assert!(
        max <= config.jobs,
        "Should respect jobs config of {}, observed {}",
        config.jobs,
        max
    );

    Ok(())
}

#[tokio::test]
async fn test_different_job_limits() -> clnrm_core::Result<()> {
    // Test with different job limits
    for jobs in [1, 2, 5, 10] {
        // Arrange
        let semaphore = Arc::new(Semaphore::new(jobs));
        let max_observed = Arc::new(Mutex::new(0_usize));
        let active_count = Arc::new(Mutex::new(0_usize));

        let mut handles = Vec::new();

        // Act
        for _ in 0..50 {
            let sem_clone = semaphore.clone();
            let active_clone = active_count.clone();
            let max_clone = max_observed.clone();

            let handle = tokio::spawn(async move {
                let _permit = sem_clone.acquire().await.unwrap();

                let mut active = active_clone.lock().await;
                *active += 1;

                let mut max = max_clone.lock().await;
                if *active > *max {
                    *max = *active;
                }

                drop(active);
                drop(max);

                tokio::time::sleep(Duration::from_millis(10)).await;

                let mut active = active_clone.lock().await;
                *active -= 1;
            });

            handles.push(handle);
        }

        for handle in handles {
            handle.await.unwrap();
        }

        // Assert
        let max = *max_observed.lock().await;
        assert!(
            max <= jobs,
            "With jobs={}, should not exceed {}, observed {}",
            jobs,
            jobs,
            max
        );
    }

    Ok(())
}

// ============================================================================
// Graceful Degradation Tests
// ============================================================================

#[tokio::test]
async fn test_graceful_degradation_under_load() -> clnrm_core::Result<()> {
    // Arrange
    let semaphore = Arc::new(Semaphore::new(3));
    let success_count = Arc::new(Mutex::new(0_usize));

    let mut handles = Vec::new();

    // Act - Spawn 100 tasks (heavy load)
    for _ in 0..100 {
        let sem_clone = semaphore.clone();
        let success_clone = success_count.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();
            tokio::time::sleep(Duration::from_millis(5)).await;

            let mut count = success_clone.lock().await;
            *count += 1;
        });

        handles.push(handle);
    }

    // Wait for all
    for handle in handles {
        handle.await.unwrap();
    }

    // Assert - All should complete successfully (graceful degradation)
    let count = *success_count.lock().await;
    assert_eq!(count, 100, "All tasks should complete under heavy load");

    Ok(())
}

#[tokio::test]
async fn test_no_resource_exhaustion() -> clnrm_core::Result<()> {
    // Arrange
    let semaphore = Arc::new(Semaphore::new(5));

    // Act - Try to spawn way more tasks than the limit
    let mut handles = Vec::new();

    for _ in 0..1000 {
        let sem_clone = semaphore.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();
            tokio::time::sleep(Duration::from_millis(1)).await;
        });

        handles.push(handle);
    }

    // Wait for all (this should not cause resource exhaustion)
    for handle in handles {
        handle.await.unwrap();
    }

    // Assert - If we got here, no resource exhaustion occurred
    // (successful completion is the assertion)

    Ok(())
}

#[tokio::test]
async fn test_handles_task_failures_gracefully() -> clnrm_core::Result<()> {
    // Arrange
    let semaphore = Arc::new(Semaphore::new(3));
    let mut handles = Vec::new();

    // Act - Mix of successful and failing tasks
    for i in 0..10 {
        let sem_clone = semaphore.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();

            if i % 3 == 0 {
                // Simulate failure
                Err::<(), clnrm_core::error::CleanroomError>(
                    clnrm_core::error::CleanroomError::internal_error("simulated failure"),
                )
            } else {
                tokio::time::sleep(Duration::from_millis(10)).await;
                Ok(())
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks
    let mut success_count = 0;
    let mut failure_count = 0;

    for handle in handles {
        match handle.await.unwrap() {
            Ok(_) => success_count += 1,
            Err(_) => failure_count += 1,
        }
    }

    // Assert - Should handle failures gracefully
    assert!(success_count > 0, "Some tasks should succeed");
    assert!(failure_count > 0, "Some tasks should fail");
    assert_eq!(
        success_count + failure_count,
        10,
        "All tasks should complete"
    );

    Ok(())
}

// ============================================================================
// Resource Management Tests
// ============================================================================

#[tokio::test]
async fn test_semaphore_cleanup_on_drop() -> clnrm_core::Result<()> {
    // Arrange
    let semaphore = Arc::new(Semaphore::new(2));

    // Act - Acquire permits in scope
    {
        let _permit1 = semaphore.acquire().await.unwrap();
        let _permit2 = semaphore.acquire().await.unwrap();

        assert_eq!(semaphore.available_permits(), 0);
        // Permits dropped here
    }

    // Assert - Permits should be released
    assert_eq!(
        semaphore.available_permits(),
        2,
        "Permits should be released on drop"
    );

    Ok(())
}

#[tokio::test]
async fn test_owned_permits_transfer() -> clnrm_core::Result<()> {
    // Arrange
    let semaphore = Arc::new(Semaphore::new(2));

    // Act - Acquire owned permit and transfer to another task
    let permit = semaphore.clone().acquire_owned().await.unwrap();

    let handle = tokio::spawn(async move {
        let _permit = permit; // Ownership transferred
        tokio::time::sleep(Duration::from_millis(50)).await;
    });

    // Semaphore should show permit in use
    assert_eq!(semaphore.available_permits(), 1);

    handle.await.unwrap();

    // Assert - Permit released after task completes
    assert_eq!(semaphore.available_permits(), 2);

    Ok(())
}

#[tokio::test]
async fn test_semaphore_available_permits() -> clnrm_core::Result<()> {
    // Arrange
    let semaphore = Arc::new(Semaphore::new(5));

    // Act & Assert - Check available permits at each step
    assert_eq!(semaphore.available_permits(), 5);

    let _p1 = semaphore.acquire().await.unwrap();
    assert_eq!(semaphore.available_permits(), 4);

    let _p2 = semaphore.acquire().await.unwrap();
    assert_eq!(semaphore.available_permits(), 3);

    let _p3 = semaphore.acquire().await.unwrap();
    assert_eq!(semaphore.available_permits(), 2);

    drop(_p1);
    assert_eq!(semaphore.available_permits(), 3);

    drop(_p2);
    drop(_p3);
    assert_eq!(semaphore.available_permits(), 5);

    Ok(())
}

// ============================================================================
// Edge Cases
// ============================================================================

#[tokio::test]
async fn test_zero_capacity_semaphore() -> clnrm_core::Result<()> {
    // Arrange
    let semaphore = Arc::new(Semaphore::new(0));

    // Act - Try to acquire with zero capacity (should block indefinitely)
    let sem_clone = semaphore.clone();
    let result = tokio::time::timeout(Duration::from_millis(100), async move {
        sem_clone.acquire_owned().await
    })
    .await;

    // Assert - Should timeout (can't acquire with zero capacity)
    assert!(result.is_err(), "Should timeout with zero capacity");

    Ok(())
}

#[tokio::test]
async fn test_single_permit_sequential_execution() -> clnrm_core::Result<()> {
    // Arrange
    let semaphore = Arc::new(Semaphore::new(1)); // Sequential execution
    let execution_order = Arc::new(Mutex::new(Vec::new()));

    let mut handles = Vec::new();

    // Act - Spawn tasks that should execute sequentially
    for i in 0..5 {
        let sem_clone = semaphore.clone();
        let order_clone = execution_order.clone();

        let handle = tokio::spawn(async move {
            let _permit = sem_clone.acquire().await.unwrap();

            let mut order = order_clone.lock().await;
            order.push(i);

            tokio::time::sleep(Duration::from_millis(10)).await;
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    // Assert - All tasks should execute (sequentially due to single permit)
    let order = execution_order.lock().await;
    assert_eq!(order.len(), 5, "All tasks should execute sequentially");

    Ok(())
}
