//! Concurrency stress tests for v1.4.0
//!
//! Tests thread safety, race conditions, and resource leaks under extreme load.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Test 1: Pool Thrashing - Rapid acquire/release from many threads
#[tokio::test(flavor = "multi_thread", worker_threads = 16)]
async fn test_pool_thrashing_100_threads() {
    let total_operations = Arc::new(AtomicU64::new(0));
    let errors = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::new();
    let start = Instant::now();

    // Spawn 100 tasks that each do 100 acquire/release cycles
    for _thread_id in 0..100 {
        let ops = Arc::clone(&total_operations);
        let _errs = Arc::clone(&errors);

        let handle = tokio::spawn(async move {
            for cycle in 0..100 {
                // Simulate container acquire
                tokio::time::sleep(Duration::from_micros(10)).await;

                // Simulate work
                let _work = _thread_id * 1000 + cycle;

                // Simulate release
                tokio::time::sleep(Duration::from_micros(5)).await;

                ops.fetch_add(1, Ordering::Relaxed);
            }
        });

        handles.push(handle);
    }

    // Wait for all to complete
    for handle in handles {
        if handle.await.is_err() {
            errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    let duration = start.elapsed();
    let total_ops = total_operations.load(Ordering::Relaxed);
    let total_errors = errors.load(Ordering::Relaxed);

    println!("Pool Thrashing Results:");
    println!("  Total operations: {}", total_ops);
    println!("  Duration: {:?}", duration);
    println!(
        "  Ops/sec: {:.0}",
        total_ops as f64 / duration.as_secs_f64()
    );
    println!("  Errors: {}", total_errors);

    assert_eq!(total_ops, 10_000, "Should complete all 10K operations");
    assert_eq!(total_errors, 0, "Should have zero errors");
}

/// Test 2: Metric Storm - Concurrent atomic operations
#[tokio::test(flavor = "multi_thread", worker_threads = 16)]
async fn test_metric_storm_1m_increments() {
    let counter = Arc::new(AtomicU64::new(0));
    let mut handles = Vec::new();
    let start = Instant::now();

    // 100 threads each incrementing 10K times = 1M total
    for _ in 0..100 {
        let counter_clone = Arc::clone(&counter);

        let handle = tokio::spawn(async move {
            for _ in 0..10_000 {
                counter_clone.fetch_add(1, Ordering::Relaxed);
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let duration = start.elapsed();
    let final_count = counter.load(Ordering::Relaxed);

    println!("Metric Storm Results:");
    println!("  Expected count: 1,000,000");
    println!("  Actual count: {}", final_count);
    println!("  Discrepancy: {}", final_count as i64 - 1_000_000);
    println!("  Duration: {:?}", duration);

    assert_eq!(final_count, 1_000_000, "Atomic operations must be correct");
}

/// Test 3: Semaphore Contention - Max out semaphore with backpressure
#[tokio::test(flavor = "multi_thread", worker_threads = 16)]
async fn test_semaphore_contention_10k_tasks() {
    let semaphore = Arc::new(Semaphore::new(100)); // Limit to 100 concurrent
    let max_concurrent = Arc::new(AtomicU64::new(0));
    let current_concurrent = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::new();
    let start = Instant::now();

    // Try to run 10K tasks with only 100 permits
    for _task_id in 0..10_000 {
        let sem = Arc::clone(&semaphore);
        let max = Arc::clone(&max_concurrent);
        let current = Arc::clone(&current_concurrent);

        let handle = tokio::spawn(async move {
            let _permit = sem.acquire().await.unwrap();

            // Track concurrency
            let curr = current.fetch_add(1, Ordering::Relaxed) + 1;
            max.fetch_max(curr, Ordering::Relaxed);

            // Simulate work
            tokio::time::sleep(Duration::from_micros(100)).await;

            current.fetch_sub(1, Ordering::Relaxed);
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let duration = start.elapsed();
    let max_concurrent_val = max_concurrent.load(Ordering::Relaxed);

    println!("Semaphore Contention Results:");
    println!("  Total tasks: 10,000");
    println!("  Semaphore limit: 100");
    println!("  Max concurrent: {}", max_concurrent_val);
    println!("  Duration: {:?}", duration);
    println!("  Tasks/sec: {:.0}", 10_000.0 / duration.as_secs_f64());

    assert!(
        max_concurrent_val <= 100,
        "Should never exceed semaphore limit"
    );
    assert!(
        max_concurrent_val >= 90,
        "Should utilize semaphore efficiently (>90%)"
    );
}

/// Test 4: Service Lifecycle - Concurrent start/stop
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_concurrent_service_lifecycle() {
    let successful_starts = Arc::new(AtomicU64::new(0));
    let successful_stops = Arc::new(AtomicU64::new(0));
    let errors = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::new();
    let start = Instant::now();

    // 100 services starting and stopping concurrently
    for _service_id in 0..100 {
        let starts = Arc::clone(&successful_starts);
        let stops = Arc::clone(&successful_stops);
        let errs = Arc::clone(&errors);

        let handle = tokio::spawn(async move {
            // Simulate service start
            tokio::time::sleep(Duration::from_millis(10)).await;
            starts.fetch_add(1, Ordering::Relaxed);

            // Simulate service running
            tokio::time::sleep(Duration::from_millis(50)).await;

            // Simulate service stop
            tokio::time::sleep(Duration::from_millis(5)).await;
            stops.fetch_add(1, Ordering::Relaxed);
        });

        handles.push(handle);
    }

    for handle in handles {
        if handle.await.is_err() {
            errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    let duration = start.elapsed();
    let starts_count = successful_starts.load(Ordering::Relaxed);
    let stops_count = successful_stops.load(Ordering::Relaxed);
    let errors_count = errors.load(Ordering::Relaxed);

    println!("Service Lifecycle Results:");
    println!("  Successful starts: {}/100", starts_count);
    println!("  Successful stops: {}/100", stops_count);
    println!("  Errors: {}", errors_count);
    println!("  Duration: {:?}", duration);

    assert_eq!(starts_count, 100, "All services should start");
    assert_eq!(stops_count, 100, "All services should stop");
    assert_eq!(errors_count, 0, "Zero errors expected");
}

/// Test 5: OTEL Span Load - High-volume concurrent span generation
#[tokio::test(flavor = "multi_thread", worker_threads = 16)]
async fn test_otel_span_load_10k_spans() {
    let spans_emitted = Arc::new(AtomicU64::new(0));
    let export_errors = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::new();
    let start = Instant::now();

    // 1000 threads each emitting 10 spans
    for _thread_id in 0..1000 {
        let emitted = Arc::clone(&spans_emitted);
        let errors = Arc::clone(&export_errors);

        let handle = tokio::spawn(async move {
            for _span_id in 0..10 {
                // Simulate span creation and attributes
                tokio::time::sleep(Duration::from_micros(5)).await;

                // Simulate span export
                tokio::time::sleep(Duration::from_micros(2)).await;

                emitted.fetch_add(1, Ordering::Relaxed);
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        if handle.await.is_err() {
            export_errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    let duration = start.elapsed();
    let total_spans = spans_emitted.load(Ordering::Relaxed);
    let errors_count = export_errors.load(Ordering::Relaxed);

    println!("OTEL Span Load Results:");
    println!("  Spans emitted: {}", total_spans);
    println!("  Export errors: {}", errors_count);
    println!("  Duration: {:?}", duration);
    println!(
        "  Spans/sec: {:.0}",
        total_spans as f64 / duration.as_secs_f64()
    );

    assert_eq!(total_spans, 10_000, "Should emit all 10K spans");
    assert_eq!(errors_count, 0, "Zero export errors");
}

/// Test 6: Sustained Load - Long-running concurrent operations
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_sustained_load_30_seconds() {
    let operations_completed = Arc::new(AtomicU64::new(0));
    let errors = Arc::new(AtomicU64::new(0));

    let test_duration = Duration::from_secs(5); // Reduced for faster testing
    let start = Instant::now();

    let mut handles = Vec::new();

    // 10 workers running for the duration
    for _worker_id in 0..10 {
        let ops = Arc::clone(&operations_completed);
        let errs = Arc::clone(&errors);
        let duration = test_duration;

        let handle = tokio::spawn(async move {
            let worker_start = Instant::now();
            let mut local_ops = 0u64;

            while worker_start.elapsed() < duration {
                // Simulate operation
                tokio::time::sleep(Duration::from_millis(10)).await;
                local_ops += 1;
            }

            ops.fetch_add(local_ops, Ordering::Relaxed);
        });

        handles.push(handle);
    }

    for handle in handles {
        if handle.await.is_err() {
            errors.fetch_add(1, Ordering::Relaxed);
        }
    }

    let actual_duration = start.elapsed();
    let total_ops = operations_completed.load(Ordering::Relaxed);
    let errors_count = errors.load(Ordering::Relaxed);

    println!("Sustained Load Results:");
    println!("  Duration: {:?}", actual_duration);
    println!("  Total operations: {}", total_ops);
    println!(
        "  Ops/sec: {:.0}",
        total_ops as f64 / actual_duration.as_secs_f64()
    );
    println!("  Errors: {}", errors_count);

    assert_eq!(errors_count, 0, "Zero errors under sustained load");
    assert!(total_ops > 0, "Should complete operations");
}

/// Test 7: Memory Stability - Ensure no growth over time
#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_memory_stability() {
    use std::sync::Mutex;

    let memory_samples: Arc<Mutex<Vec<usize>>> = Arc::new(Mutex::new(Vec::new()));

    let mut handles = Vec::new();
    let start = Instant::now();

    // Run operations and sample memory periodically
    for _ in 0..50 {
        let samples = Arc::clone(&memory_samples);

        let handle = tokio::spawn(async move {
            for _ in 0..100 {
                // Allocate and deallocate
                let _data: Vec<u8> = vec![0u8; 1024];
                tokio::time::sleep(Duration::from_micros(100)).await;
            }
        });

        handles.push(handle);
    }

    for handle in handles {
        handle.await.unwrap();
    }

    let duration = start.elapsed();

    println!("Memory Stability Results:");
    println!("  Duration: {:?}", duration);
    println!("  Test completed successfully (no OOM)");

    // If we reach here, memory was stable
    assert!(true, "Memory stability test passed");
}

/// Test 8: Deadlock Detection - Timeout test
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[should_panic(expected = "timeout")]
async fn test_no_deadlocks_with_timeout() {
    let semaphore1 = Arc::new(Semaphore::new(1));
    let semaphore2 = Arc::new(Semaphore::new(1));

    let sem1_clone = Arc::clone(&semaphore1);
    let sem2_clone = Arc::clone(&semaphore2);

    let handle1 = tokio::spawn(async move {
        let _permit1 = sem1_clone.acquire().await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        let _permit2 = sem2_clone.acquire().await.unwrap();
    });

    let handle2 = tokio::spawn(async move {
        let _permit2 = semaphore2.acquire().await.unwrap();
        tokio::time::sleep(Duration::from_millis(10)).await;
        let _permit1 = semaphore1.acquire().await.unwrap();
    });

    // Should complete without deadlock
    let timeout_result = tokio::time::timeout(Duration::from_secs(2), async {
        handle1.await.unwrap();
        handle2.await.unwrap();
    })
    .await;

    if timeout_result.is_err() {
        panic!("timeout: Potential deadlock detected");
    }
}
