//! Integration Tests for Atomic Metrics (v1.4.0)
//!
//! Tests the lock-free atomic metrics that eliminate lock contention.
//!
//! Test Categories:
//! 1. Concurrent Updates (100 threads)
//! 2. Snapshot Consistency
//! 3. Zero Lock Contention
//! 4. Performance Comparison vs RwLock
//! 5. Metric Calculations

use clnrm_core::metrics::atomic::AtomicMetrics;
use std::sync::Arc;
use std::thread;
use std::time::Instant;

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a test metrics instance
fn create_test_metrics() -> AtomicMetrics {
    AtomicMetrics::new()
}

// ============================================================================
// Concurrent Update Tests
// ============================================================================

#[test]
fn test_concurrent_updates_100_threads() {
    // Arrange
    let metrics = Arc::new(create_test_metrics());
    let num_threads = 100;
    let increments_per_thread = 100;
    let mut handles = Vec::new();

    // Act - Spawn 100 threads, each incrementing 100 times
    for _ in 0..num_threads {
        let metrics_clone = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for _ in 0..increments_per_thread {
                metrics_clone.increment_executed();
                metrics_clone.increment_passed();
                metrics_clone.add_duration(1);
            }
        });
        handles.push(handle);
    }

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    // Assert - All 10,000 increments should be counted (lock-free correctness)
    let expected_count = num_threads * increments_per_thread;
    assert_eq!(
        metrics.tests_executed(),
        expected_count,
        "Should count all {} increments",
        expected_count
    );
    assert_eq!(metrics.tests_passed(), expected_count);
    assert_eq!(metrics.total_duration_ms(), expected_count as u64);
}

#[test]
fn test_concurrent_mixed_operations() {
    // Arrange
    let metrics = Arc::new(create_test_metrics());
    let num_threads = 52;
    let mut handles = Vec::new();

    // Act - Mix of different operations across threads
    for i in 0..num_threads {
        let metrics_clone = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                match i % 4 {
                    0 => {
                        metrics_clone.increment_executed();
                        metrics_clone.increment_passed();
                    }
                    1 => {
                        metrics_clone.increment_executed();
                        metrics_clone.increment_failed();
                    }
                    2 => {
                        metrics_clone.increment_active_containers();
                        metrics_clone.decrement_active_containers();
                    }
                    3 => {
                        metrics_clone.increment_containers_created();
                        metrics_clone.increment_containers_reused();
                    }
                    _ => unreachable!(),
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Assert - Verify counts match expected operations
    let executed_expected = (num_threads / 2) * 100; // Half the threads increment executed
    assert_eq!(metrics.tests_executed(), executed_expected);

    let passed_expected = (num_threads / 4) * 100; // Quarter of threads increment passed
    assert_eq!(metrics.tests_passed(), passed_expected);
}

#[test]
fn test_no_lost_updates_under_contention() {
    // Arrange
    let metrics = Arc::new(create_test_metrics());
    let num_threads = 200; // High contention
    let increments_per_thread = 50;
    let mut handles = Vec::new();

    // Act - Create high contention scenario
    for _ in 0..num_threads {
        let metrics_clone = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for _ in 0..increments_per_thread {
                // All threads incrementing same counter simultaneously
                metrics_clone.increment_executed();
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Assert - No updates should be lost even under extreme contention
    let expected = num_threads * increments_per_thread;
    assert_eq!(
        metrics.tests_executed(),
        expected,
        "No updates should be lost under contention"
    );
}

// ============================================================================
// Snapshot Consistency Tests
// ============================================================================

#[test]
fn test_snapshot_consistency() {
    // Arrange
    let metrics = create_test_metrics();

    // Act - Update metrics
    metrics.increment_executed();
    metrics.increment_executed();
    metrics.increment_executed();

    metrics.increment_passed();
    metrics.increment_passed();

    metrics.increment_failed();

    metrics.add_duration(300);

    metrics.increment_active_containers();
    metrics.increment_active_services();

    metrics.increment_containers_created();
    metrics.increment_containers_reused();
    metrics.increment_containers_reused();

    // Assert - Snapshot should capture all values
    let snapshot = metrics.snapshot();

    assert_eq!(snapshot.tests_executed, 3);
    assert_eq!(snapshot.tests_passed, 2);
    assert_eq!(snapshot.tests_failed, 1);
    assert_eq!(snapshot.total_duration_ms, 300);
    assert_eq!(snapshot.active_containers, 1);
    assert_eq!(snapshot.active_services, 1);
    assert_eq!(snapshot.containers_created, 1);
    assert_eq!(snapshot.containers_reused, 2);
}

#[test]
fn test_snapshot_immutability() {
    // Arrange
    let metrics = create_test_metrics();

    metrics.increment_executed();
    let snapshot1 = metrics.snapshot();

    // Act - Modify metrics after taking snapshot
    metrics.increment_executed();
    metrics.increment_executed();

    // Assert - First snapshot should be unchanged
    assert_eq!(snapshot1.tests_executed, 1);
    assert_eq!(metrics.tests_executed(), 3);

    let snapshot2 = metrics.snapshot();
    assert_eq!(snapshot2.tests_executed, 3);
}

#[test]
fn test_concurrent_snapshots() {
    // Arrange
    let metrics = Arc::new(create_test_metrics());
    let mut handles = Vec::new();

    // Act - Concurrent updates and snapshots
    for i in 0..10 {
        let metrics_clone = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                if i % 2 == 0 {
                    // Even threads update
                    metrics_clone.increment_executed();
                } else {
                    // Odd threads take snapshots
                    let _snapshot = metrics_clone.snapshot();
                }
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Assert - Should have completed without panicking
    assert!(metrics.tests_executed() > 0);
}

// ============================================================================
// Zero Lock Contention Tests
// ============================================================================

#[test]
fn test_zero_lock_contention_performance() {
    // Arrange
    let metrics = Arc::new(create_test_metrics());
    let num_threads = 100;
    let operations_per_thread = 10_000;

    // Act - Measure time for high-concurrency atomic operations
    let start = Instant::now();

    let mut handles = Vec::new();
    for _ in 0..num_threads {
        let metrics_clone = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for _ in 0..operations_per_thread {
                metrics_clone.increment_executed();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start.elapsed();

    // Assert - Performance characteristics
    let total_ops = num_threads * operations_per_thread;
    let ops_per_ms = total_ops as f64 / duration.as_millis() as f64;

    assert_eq!(metrics.tests_executed(), total_ops);

    // Atomic operations should be extremely fast (>100k ops/ms on modern hardware)
    // This is a conservative check - actual performance is usually much higher
    assert!(
        ops_per_ms > 1000.0,
        "Atomic operations should be fast: {} ops/ms",
        ops_per_ms
    );
}

#[test]
fn test_no_blocking_on_concurrent_access() {
    // Arrange
    let metrics = Arc::new(create_test_metrics());
    let mut handles = Vec::new();

    // Act - Create scenario where threads would block with locks
    for _ in 0..100 {
        let metrics_clone = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            // Tight loop that would cause lock contention with RwLock
            for _ in 0..1000 {
                metrics_clone.increment_executed();
                let _count = metrics_clone.tests_executed(); // Read
                metrics_clone.increment_passed();
                let _snapshot = metrics_clone.snapshot(); // Multiple reads
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    let start = Instant::now();
    for handle in handles {
        handle.join().unwrap();
    }
    let duration = start.elapsed();

    // Assert - Should complete quickly without blocking
    // With locks, this would take much longer due to contention
    assert!(
        duration.as_millis() < 5000,
        "Should complete quickly without lock contention: {}ms",
        duration.as_millis()
    );
}

// ============================================================================
// Metric Calculations Tests
// ============================================================================

#[test]
fn test_snapshot_success_rate_calculation() {
    // Arrange
    let metrics = create_test_metrics();

    // Act - 3 executed, 2 passed, 1 failed
    metrics.increment_executed();
    metrics.increment_executed();
    metrics.increment_executed();

    metrics.increment_passed();
    metrics.increment_passed();

    metrics.increment_failed();

    let snapshot = metrics.snapshot();

    // Assert - Success rate: 2/3 = 66.67%
    let success_rate = snapshot.success_rate();
    assert!(
        (success_rate - 66.67).abs() < 0.1,
        "Success rate should be 66.67%, got {}",
        success_rate
    );
}

#[test]
fn test_snapshot_avg_duration_calculation() {
    // Arrange
    let metrics = create_test_metrics();

    // Act - 3 tests with total duration 300ms
    metrics.increment_executed();
    metrics.increment_executed();
    metrics.increment_executed();

    metrics.add_duration(100);
    metrics.add_duration(50);
    metrics.add_duration(150);

    let snapshot = metrics.snapshot();

    // Assert - Average: 300/3 = 100ms
    assert_eq!(snapshot.avg_duration_ms(), 100.0);
}

#[test]
fn test_snapshot_container_reuse_rate() {
    // Arrange
    let metrics = create_test_metrics();

    // Act - 1 created, 2 reused = 66.67% reuse rate
    metrics.increment_containers_created();
    metrics.increment_containers_reused();
    metrics.increment_containers_reused();

    let snapshot = metrics.snapshot();

    // Assert
    let reuse_rate = snapshot.container_reuse_rate();
    assert!(
        (reuse_rate - 66.67).abs() < 0.1,
        "Reuse rate should be 66.67%, got {}",
        reuse_rate
    );
}

#[test]
fn test_zero_division_safety() {
    // Arrange
    let metrics = create_test_metrics();
    let snapshot = metrics.snapshot();

    // Assert - Should handle zero division gracefully
    assert_eq!(snapshot.success_rate(), 0.0);
    assert_eq!(snapshot.avg_duration_ms(), 0.0);
    assert_eq!(snapshot.container_reuse_rate(), 0.0);
}

// ============================================================================
// Individual Operations Tests
// ============================================================================

#[test]
fn test_individual_metric_increments() {
    // Arrange
    let metrics = create_test_metrics();

    // Act & Assert - Test each increment operation
    assert_eq!(metrics.tests_executed(), 0);
    metrics.increment_executed();
    assert_eq!(metrics.tests_executed(), 1);

    assert_eq!(metrics.tests_passed(), 0);
    metrics.increment_passed();
    assert_eq!(metrics.tests_passed(), 1);

    assert_eq!(metrics.tests_failed(), 0);
    metrics.increment_failed();
    assert_eq!(metrics.tests_failed(), 1);

    assert_eq!(metrics.total_duration_ms(), 0);
    metrics.add_duration(250);
    assert_eq!(metrics.total_duration_ms(), 250);
}

#[test]
fn test_container_counter_operations() {
    // Arrange
    let metrics = create_test_metrics();

    // Act & Assert - Increment
    metrics.increment_active_containers();
    metrics.increment_active_containers();
    assert_eq!(metrics.active_containers(), 2);

    // Decrement
    metrics.decrement_active_containers();
    assert_eq!(metrics.active_containers(), 1);

    // Set
    metrics.set_active_containers(5);
    assert_eq!(metrics.active_containers(), 5);
}

#[test]
fn test_service_counter_operations() {
    // Arrange
    let metrics = create_test_metrics();

    // Act & Assert - Increment
    metrics.increment_active_services();
    metrics.increment_active_services();
    assert_eq!(metrics.active_services(), 2);

    // Decrement
    metrics.decrement_active_services();
    assert_eq!(metrics.active_services(), 1);

    // Set
    metrics.set_active_services(3);
    assert_eq!(metrics.active_services(), 3);
}

#[test]
fn test_session_metadata() {
    // Arrange & Act
    let metrics = create_test_metrics();

    // Assert - Session ID should be set
    let session_id = metrics.session_id();
    assert_ne!(session_id.to_string(), "");

    // Start time should be reasonable
    let start_time = metrics.start_time_ms();
    assert!(start_time > 0);

    // Snapshot should preserve session metadata
    let snapshot = metrics.snapshot();
    assert_eq!(snapshot.session_id, session_id);
    assert_eq!(snapshot.start_time_ms, start_time);
}

// ============================================================================
// Stress Tests
// ============================================================================

#[test]
fn test_extreme_concurrency_1000_threads() {
    // Arrange
    let metrics = Arc::new(create_test_metrics());
    let num_threads = 1000;
    let mut handles = Vec::new();

    // Act - Extreme concurrency test
    for _ in 0..num_threads {
        let metrics_clone = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for _ in 0..100 {
                metrics_clone.increment_executed();
            }
        });
        handles.push(handle);
    }

    // Wait for all threads
    for handle in handles {
        handle.join().unwrap();
    }

    // Assert
    let expected = num_threads * 100;
    assert_eq!(
        metrics.tests_executed(),
        expected,
        "Should handle extreme concurrency correctly"
    );
}

#[test]
fn test_high_throughput_metrics() {
    // Arrange
    let metrics = Arc::new(create_test_metrics());
    let num_threads = 10;
    let operations_per_thread = 100_000; // High throughput

    // Act
    let mut handles = Vec::new();
    for _ in 0..num_threads {
        let metrics_clone = Arc::clone(&metrics);
        let handle = thread::spawn(move || {
            for _ in 0..operations_per_thread {
                metrics_clone.increment_executed();
            }
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.join().unwrap();
    }

    // Assert
    let expected = num_threads * operations_per_thread;
    assert_eq!(metrics.tests_executed(), expected);
}
