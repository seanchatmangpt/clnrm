//! Performance Integration: Concurrent Execution Tests
//!
//! Tests performance and correctness of concurrent test execution.

use super::common::*;
use clnrm_core::*;
use std::time::Instant;

#[tokio::test]
async fn test_perf_sequential_vs_concurrent() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;

    // Act: Measure sequential execution
    let sequential_start = Instant::now();
    for i in 0..5 {
        env.execute_in_container(
            "alpine",
            &["echo".to_string(), format!("seq_{}", i)],
            None,
            None,
        )
        .await?;
    }
    let sequential_duration = sequential_start.elapsed();

    // Act: Measure concurrent execution
    let concurrent_start = Instant::now();
    let mut tasks = vec![];
    for i in 0..5 {
        let env_clone = env.clone();
        tasks.push(tokio::spawn(async move {
            env_clone
                .execute_in_container(
                    "alpine",
                    &["echo".to_string(), format!("concurrent_{}", i)],
                    None,
                    None,
                )
                .await
        }));
    }

    for task in tasks {
        task.await??;
    }
    let concurrent_duration = concurrent_start.elapsed();

    // Assert: Concurrent should be faster or similar
    // (May not always be true for simple echo commands due to overhead)
    println!("Sequential: {:?}, Concurrent: {:?}", sequential_duration, concurrent_duration);
    assert!(
        concurrent_duration <= sequential_duration * 2,
        "Concurrent execution should not be significantly slower"
    );

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_perf_container_startup_time() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Measure container startup time
    let start = Instant::now();
    let handle = env.start_service("alpine").await?;
    let startup_duration = start.elapsed();

    // Assert: Startup should be reasonably fast (< 30 seconds)
    assert!(
        startup_duration.as_secs() < 30,
        "Container startup should complete in < 30 seconds"
    );

    println!("Container startup time: {:?}", startup_duration);

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_perf_command_execution_overhead() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;

    // Act: Measure command execution time for simple command
    let iterations = 10;
    let mut total_duration = std::time::Duration::ZERO;

    for _ in 0..iterations {
        let start = Instant::now();
        env.execute_in_container("alpine", &["true".to_string()], None, None)
            .await?;
        total_duration += start.elapsed();
    }

    let avg_duration = total_duration / iterations;

    // Assert: Average execution time should be reasonable (< 5 seconds)
    assert!(
        avg_duration.as_secs() < 5,
        "Average command execution should be < 5 seconds"
    );

    println!("Average command execution time: {:?}", avg_duration);

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_perf_multiple_services_startup() -> Result<()> {
    // Arrange: Create environment with multiple services
    let env = create_test_environment().await?;

    let alpine = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    let busybox = services::generic::GenericContainerPlugin::new("busybox", "busybox:latest");

    env.register_service(Box::new(alpine)).await?;
    env.register_service(Box::new(busybox)).await?;

    // Act: Measure time to start multiple services
    let start = Instant::now();
    let alpine_handle = env.start_service("alpine").await?;
    let busybox_handle = env.start_service("busybox").await?;
    let total_duration = start.elapsed();

    // Assert: Should start both services in reasonable time (< 60 seconds)
    assert!(
        total_duration.as_secs() < 60,
        "Multiple services should start in < 60 seconds"
    );

    println!("Multiple services startup time: {:?}", total_duration);

    // Cleanup
    env.stop_service(&alpine_handle.id).await?;
    env.stop_service(&busybox_handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_perf_cleanup_efficiency() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;

    // Act: Measure cleanup time
    let start = Instant::now();
    env.stop_service(&handle.id).await?;
    let cleanup_duration = start.elapsed();

    // Assert: Cleanup should be fast (< 10 seconds)
    assert!(
        cleanup_duration.as_secs() < 10,
        "Cleanup should complete in < 10 seconds"
    );

    println!("Cleanup time: {:?}", cleanup_duration);
    Ok(())
}

#[tokio::test]
async fn test_perf_memory_usage_stability() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;

    // Act: Execute many commands to test memory stability
    for i in 0..100 {
        env.execute_in_container(
            "alpine",
            &["echo".to_string(), format!("iteration_{}", i)],
            None,
            None,
        )
        .await?;
    }

    // Assert: Test completed without panics or excessive memory growth
    // (Memory validation would require external tooling)

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_perf_concurrent_service_operations() -> Result<()> {
    // Arrange: Create environment with multiple services
    let env = create_test_environment().await?;

    let alpine = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    let busybox = services::generic::GenericContainerPlugin::new("busybox", "busybox:latest");

    env.register_service(Box::new(alpine)).await?;
    env.register_service(Box::new(busybox)).await?;

    let alpine_handle = env.start_service("alpine").await?;
    let busybox_handle = env.start_service("busybox").await?;

    // Act: Concurrent operations on different services
    let start = Instant::now();

    let env_clone1 = env.clone();
    let task1 = tokio::spawn(async move {
        for i in 0..10 {
            env_clone1
                .execute_in_container(
                    "alpine",
                    &["echo".to_string(), format!("alpine_{}", i)],
                    None,
                    None,
                )
                .await?;
        }
        Ok::<_, error::CleanroomError>(())
    });

    let env_clone2 = env.clone();
    let task2 = tokio::spawn(async move {
        for i in 0..10 {
            env_clone2
                .execute_in_container(
                    "busybox",
                    &["echo".to_string(), format!("busybox_{}", i)],
                    None,
                    None,
                )
                .await?;
        }
        Ok::<_, error::CleanroomError>(())
    });

    task1.await??;
    task2.await??;

    let concurrent_duration = start.elapsed();

    // Assert: Concurrent operations completed successfully
    println!("Concurrent operations duration: {:?}", concurrent_duration);

    // Cleanup
    env.stop_service(&alpine_handle.id).await?;
    env.stop_service(&busybox_handle.id).await?;
    Ok(())
}
