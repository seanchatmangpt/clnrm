//! Comprehensive tests for atomic port allocator
//!
//! Tests verify:
//! - Port allocation success
//! - Lock release on drop (RAII)
//! - Parallel allocation with zero conflicts
//! - Health check timeout behavior
//! - Custom port ranges
//! - Port exhaustion handling

use clnrm_core::error::Result;
use clnrm_core::telemetry::live_check::{wait_for_service_ready, PortAllocator, PortRange};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::Barrier;

#[tokio::test]
async fn test_port_allocation_success() -> Result<()> {
    // ARRANGE
    let allocator = PortAllocator::new()?;

    // ACT
    let lock = allocator.allocate_port().await?;

    // ASSERT
    assert!(
        lock.port() >= 4317,
        "Port should be in primary range or higher"
    );
    println!("✓ Allocated port: {}", lock.port());

    Ok(())
}

#[tokio::test]
async fn test_port_lock_released_on_drop() -> Result<()> {
    // ARRANGE
    let allocator = PortAllocator::new()?;

    // ACT
    let port1 = {
        let lock = allocator.allocate_port().await?;
        lock.port()
    }; // lock dropped here

    // Small delay to ensure OS releases lock
    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

    // Should be able to allocate same port again
    let lock2 = allocator.allocate_port().await?;

    // ASSERT
    assert_eq!(
        port1,
        lock2.port(),
        "Should be able to reuse port after lock released"
    );
    println!("✓ Port {} reused after lock release", port1);

    Ok(())
}

#[tokio::test]
async fn test_parallel_allocation_no_conflicts() -> Result<()> {
    // ARRANGE
    let allocator = Arc::new(PortAllocator::new()?);
    let barrier = Arc::new(Barrier::new(10));
    let mut handles = vec![];

    // ACT - Spawn 10 parallel tasks allocating ports
    for i in 0..10 {
        let allocator = Arc::clone(&allocator);
        let barrier = Arc::clone(&barrier);

        handles.push(tokio::spawn(async move {
            // Wait for all tasks to be ready (maximizes parallelism)
            barrier.wait().await;

            let lock = allocator.allocate_port().await?;
            println!("Task {} allocated port {}", i, lock.port());

            // Hold lock for a bit to simulate real usage
            tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;

            Ok::<_, clnrm_core::error::CleanroomError>(lock.port())
        }));
    }

    let results: Vec<_> = futures::future::join_all(handles).await;

    // ASSERT - All should succeed
    let ports: Vec<u16> = results
        .into_iter()
        .map(|r| r.expect("Task panicked").expect("Allocation failed"))
        .collect();

    assert_eq!(ports.len(), 10, "All 10 tasks should succeed");

    // No duplicate ports
    let unique_ports: HashSet<_> = ports.iter().copied().collect();
    assert_eq!(
        unique_ports.len(),
        10,
        "All ports should be unique: {:?}",
        ports
    );

    println!("✓ 10 parallel allocations succeeded with unique ports");
    println!("  Allocated ports: {:?}", ports);

    Ok(())
}

#[tokio::test]
async fn test_parallel_allocation_stress_test() -> Result<()> {
    // ARRANGE - Test with 20 parallel allocations (more than default range)
    let allocator = Arc::new(PortAllocator::new()?);
    let barrier = Arc::new(Barrier::new(20));
    let mut handles = vec![];

    // ACT
    for i in 0..20 {
        let allocator = Arc::clone(&allocator);
        let barrier = Arc::clone(&barrier);

        handles.push(tokio::spawn(async move {
            barrier.wait().await;

            let lock = allocator.allocate_port().await?;
            println!("Stress task {} allocated port {}", i, lock.port());

            tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

            Ok::<_, clnrm_core::error::CleanroomError>(lock.port())
        }));
    }

    let results: Vec<_> = futures::future::join_all(handles).await;

    // ASSERT - All should succeed (using primary + fallback ranges)
    let ports: Vec<u16> = results
        .into_iter()
        .map(|r| r.expect("Task panicked").expect("Allocation failed"))
        .collect();

    assert_eq!(ports.len(), 20, "All 20 stress tasks should succeed");

    let unique_ports: HashSet<_> = ports.iter().copied().collect();
    assert_eq!(
        unique_ports.len(),
        20,
        "All ports should be unique in stress test"
    );

    println!("✓ 20 parallel allocations succeeded (stress test)");

    Ok(())
}

#[tokio::test]
async fn test_custom_port_ranges() -> Result<()> {
    // ARRANGE
    let allocator = PortAllocator::with_ranges(
        PortRange::new(6000, 6005),
        PortRange::new(7000, 7005),
        PortRange::new(8000, 8005),
    )?;

    // ACT
    let lock = allocator.allocate_port().await?;

    // ASSERT
    let port = lock.port();
    assert!(
        (6000..=6005).contains(&port)
            || (7000..=7005).contains(&port)
            || (8000..=8005).contains(&port),
        "Port {} should be in custom ranges",
        port
    );

    println!("✓ Allocated port {} from custom range", port);

    Ok(())
}

#[tokio::test]
async fn test_port_exhaustion_with_small_range() -> Result<()> {
    // ARRANGE - Very small range (2 ports)
    let allocator = PortAllocator::with_ranges(
        PortRange::new(19990, 19991),
        PortRange::new(19992, 19993),
        PortRange::new(19994, 19995),
    )?;

    // ACT - Allocate all ports
    let _lock1 = allocator.allocate_port().await?;
    let _lock2 = allocator.allocate_port().await?;
    let _lock3 = allocator.allocate_port().await?;
    let _lock4 = allocator.allocate_port().await?;
    let _lock5 = allocator.allocate_port().await?;
    let _lock6 = allocator.allocate_port().await?;

    // Try to allocate one more (should fail)
    let result = allocator.allocate_port().await;

    // ASSERT
    assert!(result.is_err(), "Should fail when all ports are exhausted");

    println!("✓ Port exhaustion correctly detected");

    Ok(())
}

#[tokio::test]
async fn test_health_check_timeout() {
    // ARRANGE - Port with no service
    let result = wait_for_service_ready(19999, 1).await;

    // ASSERT
    assert!(result.is_err(), "Should timeout when service not available");

    println!("✓ Health check timeout works correctly");
}

#[tokio::test]
async fn test_multiple_allocators_independent() -> Result<()> {
    // ARRANGE - Two separate allocators
    let allocator1 = Arc::new(PortAllocator::new()?);
    let allocator2 = Arc::new(PortAllocator::new()?);

    // ACT - Allocate from each in parallel
    let lock1 = allocator1.allocate_port().await?;
    let lock2 = allocator2.allocate_port().await?;

    // ASSERT - Both should succeed with different ports
    assert_ne!(
        lock1.port(),
        lock2.port(),
        "Locks from different allocators should use different ports"
    );

    println!(
        "✓ Independent allocators: {} and {}",
        lock1.port(),
        lock2.port()
    );

    Ok(())
}

#[tokio::test]
async fn test_lock_survives_across_await() -> Result<()> {
    // ARRANGE
    let allocator = PortAllocator::new()?;
    let lock = allocator.allocate_port().await?;
    let port = lock.port();

    // ACT - Hold lock across async operations
    tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

    // Try to allocate same port from another allocator (should fail or get different port)
    let allocator2 = PortAllocator::new()?;
    let lock2 = allocator2.allocate_port().await?;

    // ASSERT - Should get different port (first is still locked)
    assert_ne!(
        port,
        lock2.port(),
        "Lock should survive across await points"
    );

    println!("✓ Lock survived across async operations");

    Ok(())
}

#[tokio::test]
async fn test_concurrent_allocation_and_release() -> Result<()> {
    // ARRANGE
    let allocator = Arc::new(PortAllocator::new()?);
    let mut handles = vec![];

    // ACT - 5 tasks: allocate, hold briefly, release
    for i in 0..5 {
        let allocator = Arc::clone(&allocator);

        handles.push(tokio::spawn(async move {
            for iteration in 0..3 {
                let lock = allocator.allocate_port().await?;
                println!(
                    "Task {} iteration {} allocated port {}",
                    i,
                    iteration,
                    lock.port()
                );

                tokio::time::sleep(tokio::time::Duration::from_millis(20)).await;

                // Lock released here
            }
            Ok::<_, clnrm_core::error::CleanroomError>(())
        }));
    }

    let results: Vec<_> = futures::future::join_all(handles).await;

    // ASSERT - All should succeed
    for result in results {
        result.expect("Task panicked").expect("Task failed");
    }

    println!("✓ Concurrent allocation/release cycles succeeded");

    Ok(())
}

#[test]
fn test_port_range_size_calculation() {
    assert_eq!(PortRange::new(4317, 4327).size(), 11);
    assert_eq!(PortRange::new(5000, 5000).size(), 1);
    assert_eq!(PortRange::new(6000, 6009).size(), 10);
}

#[test]
fn test_port_range_contains_all_ports() {
    let range = PortRange::new(4317, 4319);
    let ports: Vec<u16> = range.iter().collect();
    assert_eq!(ports, vec![4317, 4318, 4319]);
}

#[tokio::test]
async fn test_allocation_performance_benchmark() -> Result<()> {
    // ARRANGE
    let allocator = PortAllocator::new()?;
    let iterations = 100;

    // ACT
    let start = std::time::Instant::now();

    for i in 0..iterations {
        let _lock = allocator.allocate_port().await?;
        // Immediately release
        if i % 10 == 0 {
            println!("Completed {} allocations", i);
        }
    }

    let duration = start.elapsed();

    // ASSERT - Should be fast
    let avg_ms = duration.as_millis() / iterations;
    println!(
        "✓ {} allocations in {:?} (avg: {}ms per allocation)",
        iterations, duration, avg_ms
    );

    // Performance target: <100ms per allocation at P95
    assert!(
        avg_ms < 100,
        "Average allocation time {}ms exceeds 100ms target",
        avg_ms
    );

    Ok(())
}
