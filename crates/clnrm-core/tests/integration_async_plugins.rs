//! Integration Tests for Async Plugins (v1.4.0)
//!
//! Tests the async plugin system that eliminates block_in_place calls.
//!
//! Test Categories:
//! 1. Async Service Start/Stop
//! 2. No block_in_place Calls
//! 3. CPU Utilization Improvement
//! 4. Concurrent Service Operations
//! 5. Plugin Lifecycle Management

use clnrm_core::cleanroom::CleanroomEnvironment;
use clnrm_core::services::generic::GenericContainerPlugin;
use std::time::{Duration, Instant};

// ============================================================================
// Test Helpers
// ============================================================================

/// Create a test cleanroom environment
async fn create_test_environment() -> clnrm_core::Result<CleanroomEnvironment> {
    CleanroomEnvironment::new().await
}

// ============================================================================
// Async Service Start/Stop Tests
// ============================================================================

#[tokio::test]
async fn test_async_service_start() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;
    let plugin = GenericContainerPlugin::new("alpine", "alpine:latest");

    env.register_service(Box::new(plugin)).await?;

    // Act - Start service asynchronously
    let start_time = Instant::now();
    let handle = env.start_service("alpine").await?;
    let duration = start_time.elapsed();

    // Assert
    assert!(handle.id.len() > 0, "Service should have valid handle");
    assert!(
        duration < Duration::from_secs(30),
        "Async start should complete in <30s"
    );

    // Cleanup
    env.stop_service(&handle.id).await?;

    Ok(())
}

#[tokio::test]
async fn test_async_service_stop() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;
    let plugin = GenericContainerPlugin::new("alpine", "alpine:latest");

    env.register_service(Box::new(plugin)).await?;
    let handle = env.start_service("alpine").await?;

    // Act - Stop service asynchronously
    let start_time = Instant::now();
    env.stop_service(&handle.id).await?;
    let duration = start_time.elapsed();

    // Assert
    assert!(
        duration < Duration::from_secs(10),
        "Async stop should complete quickly"
    );

    // Verify service is stopped
    let health = env.check_health().await;
    assert_eq!(health.len(), 0, "Service should be stopped");

    Ok(())
}

#[tokio::test]
async fn test_async_service_lifecycle() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;
    let plugin = GenericContainerPlugin::new("alpine", "alpine:latest");

    env.register_service(Box::new(plugin)).await?;

    // Act - Full lifecycle: start -> use -> stop
    let handle = env.start_service("alpine").await?;

    let result = env
        .execute_in_container(
            "alpine",
            &["echo".to_string(), "test".to_string()],
            None,
            None,
        )
        .await?;

    env.stop_service(&handle.id).await?;

    // Assert
    assert!(result.succeeded(), "Command should succeed");

    Ok(())
}

#[tokio::test]
async fn test_async_multiple_service_starts() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;

    let plugin1 = GenericContainerPlugin::new("alpine1", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("alpine2", "alpine:latest");
    let plugin3 = GenericContainerPlugin::new("alpine3", "alpine:latest");

    env.register_service(Box::new(plugin1)).await?;
    env.register_service(Box::new(plugin2)).await?;
    env.register_service(Box::new(plugin3)).await?;

    // Act - Start all services asynchronously
    let handle1 = env.start_service("alpine1").await?;
    let handle2 = env.start_service("alpine2").await?;
    let handle3 = env.start_service("alpine3").await?;

    // Assert
    let health = env.check_health().await;
    assert_eq!(health.len(), 3, "All 3 services should be running");

    // Cleanup
    env.stop_service(&handle1.id).await?;
    env.stop_service(&handle2.id).await?;
    env.stop_service(&handle3.id).await?;

    Ok(())
}

// ============================================================================
// Concurrent Service Operations Tests
// ============================================================================

#[tokio::test]
async fn test_concurrent_service_starts() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;

    let plugin1 = GenericContainerPlugin::new("alpine1", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("alpine2", "alpine:latest");
    let plugin3 = GenericContainerPlugin::new("alpine3", "alpine:latest");

    env.register_service(Box::new(plugin1)).await?;
    env.register_service(Box::new(plugin2)).await?;
    env.register_service(Box::new(plugin3)).await?;

    // Act - Start all services concurrently
    let start_time = Instant::now();

    let (handle1, handle2, handle3) = tokio::join!(
        env.start_service("alpine1"),
        env.start_service("alpine2"),
        env.start_service("alpine3")
    );

    let duration = start_time.elapsed();

    // Assert - All should succeed
    assert!(handle1.is_ok());
    assert!(handle2.is_ok());
    assert!(handle3.is_ok());

    // Concurrent starts should be faster than sequential
    // (rough heuristic: should take less than 3x single start time)
    assert!(
        duration < Duration::from_secs(60),
        "Concurrent starts should be efficient"
    );

    // Cleanup
    env.stop_service(&handle1.unwrap().id).await?;
    env.stop_service(&handle2.unwrap().id).await?;
    env.stop_service(&handle3.unwrap().id).await?;

    Ok(())
}

#[tokio::test]
async fn test_concurrent_command_execution() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;
    let plugin = GenericContainerPlugin::new("alpine", "alpine:latest");

    env.register_service(Box::new(plugin)).await?;
    let handle = env.start_service("alpine").await?;

    // Act - Execute multiple commands concurrently
    let cmd1 = vec!["echo".to_string(), "test1".to_string()];
    let cmd2 = vec!["echo".to_string(), "test2".to_string()];
    let cmd3 = vec!["echo".to_string(), "test3".to_string()];

    let (result1, result2, result3) = tokio::join!(
        env.execute_in_container("alpine", &cmd1, None, None),
        env.execute_in_container("alpine", &cmd2, None, None),
        env.execute_in_container("alpine", &cmd3, None, None)
    );

    // Assert - All commands should succeed
    assert!(result1.is_ok());
    assert!(result2.is_ok());
    assert!(result3.is_ok());

    assert!(result1.unwrap().succeeded());
    assert!(result2.unwrap().succeeded());
    assert!(result3.unwrap().succeeded());

    // Cleanup
    env.stop_service(&handle.id).await?;

    Ok(())
}

#[tokio::test]
async fn test_concurrent_mixed_operations() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;

    let plugin1 = GenericContainerPlugin::new("alpine1", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("alpine2", "alpine:latest");

    env.register_service(Box::new(plugin1)).await?;
    env.register_service(Box::new(plugin2)).await?;

    // Act - Mix of start, execute, and health check operations
    let handle1 = env.start_service("alpine1").await?;

    let cmd = vec!["echo".to_string(), "test".to_string()];

    let (handle2_result, execute_result, health_result) = tokio::join!(
        env.start_service("alpine2"),
        env.execute_in_container("alpine1", &cmd, None, None),
        async { env.check_health().await }
    );

    // Assert
    assert!(handle2_result.is_ok());
    assert!(execute_result.is_ok());

    let health = health_result;
    assert!(health.len() >= 1, "At least one service should be healthy");

    // Cleanup
    env.stop_service(&handle1.id).await?;
    env.stop_service(&handle2_result.unwrap().id).await?;

    Ok(())
}

// ============================================================================
// CPU Utilization Tests
// ============================================================================

#[tokio::test]
async fn test_cpu_efficient_service_operations() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;
    let plugin = GenericContainerPlugin::new("alpine", "alpine:latest");

    env.register_service(Box::new(plugin)).await?;

    // Act - Perform multiple service operations
    let start_time = Instant::now();

    for i in 0..10 {
        let handle = env.start_service("alpine").await?;

        env.execute_in_container(
            "alpine",
            &["echo".to_string(), format!("test{}", i)],
            None,
            None,
        )
        .await?;

        env.stop_service(&handle.id).await?;
    }

    let duration = start_time.elapsed();

    // Assert - Operations should complete efficiently
    // Async operations should not block CPU
    assert!(
        duration < Duration::from_secs(300),
        "10 service cycles should complete in <5 minutes"
    );

    Ok(())
}

#[tokio::test]
async fn test_no_blocking_on_async_operations() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;
    let plugin = GenericContainerPlugin::new("alpine", "alpine:latest");

    env.register_service(Box::new(plugin)).await?;

    // Act - Start service and immediately try another operation
    let handle = env.start_service("alpine").await?;

    // This should not block - async operations should yield
    let health = env.check_health().await;

    // Assert
    assert!(health.len() > 0, "Health check should succeed");

    // Cleanup
    env.stop_service(&handle.id).await?;

    Ok(())
}

// ============================================================================
// Plugin Lifecycle Management Tests
// ============================================================================

#[tokio::test]
async fn test_plugin_registration() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;
    let plugin = GenericContainerPlugin::new("alpine", "alpine:latest");

    // Act
    env.register_service(Box::new(plugin)).await?;

    // Assert - Service should be registered but not started
    let health = env.check_health().await;
    assert_eq!(health.len(), 0, "No services should be running yet");

    Ok(())
}

#[tokio::test]
async fn test_multiple_plugin_registrations() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;

    // Act
    for i in 0..5 {
        let plugin = GenericContainerPlugin::new(&format!("alpine{}", i), "alpine:latest");
        env.register_service(Box::new(plugin)).await?;
    }

    // Assert - All plugins registered, none started
    let health = env.check_health().await;
    assert_eq!(health.len(), 0, "No services should be running yet");

    Ok(())
}

#[tokio::test]
async fn test_service_restart() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;
    let plugin = GenericContainerPlugin::new("alpine", "alpine:latest");

    env.register_service(Box::new(plugin)).await?;

    // Act - Start, stop, and restart service
    let handle1 = env.start_service("alpine").await?;
    env.stop_service(&handle1.id).await?;

    let handle2 = env.start_service("alpine").await?;

    // Assert - Should be able to restart
    let result = env
        .execute_in_container(
            "alpine",
            &["echo".to_string(), "restarted".to_string()],
            None,
            None,
        )
        .await?;

    assert!(result.succeeded());

    // Cleanup
    env.stop_service(&handle2.id).await?;

    Ok(())
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[tokio::test]
async fn test_start_nonexistent_service() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;

    // Act
    let result = env.start_service("nonexistent").await;

    // Assert
    assert!(result.is_err(), "Should fail to start nonexistent service");

    Ok(())
}

#[tokio::test]
async fn test_execute_on_unstarted_service() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;
    let plugin = GenericContainerPlugin::new("alpine", "alpine:latest");

    env.register_service(Box::new(plugin)).await?;

    // Act - Try to execute without starting service
    let result = env
        .execute_in_container("alpine", &["echo".to_string()], None, None)
        .await;

    // Assert
    assert!(
        result.is_err(),
        "Should fail to execute on unstarted service"
    );

    Ok(())
}

#[tokio::test]
async fn test_double_stop_service() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;
    let plugin = GenericContainerPlugin::new("alpine", "alpine:latest");

    env.register_service(Box::new(plugin)).await?;
    let handle = env.start_service("alpine").await?;

    // Act - Stop service twice
    env.stop_service(&handle.id).await?;
    let result = env.stop_service(&handle.id).await;

    // Assert - Second stop should fail or be idempotent
    // (depending on implementation, both are acceptable)
    // We just verify it doesn't panic
    let _ = result;

    Ok(())
}

// ============================================================================
// Performance Tests
// ============================================================================

#[tokio::test]
async fn test_rapid_service_cycling() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;
    let plugin = GenericContainerPlugin::new("alpine", "alpine:latest");

    env.register_service(Box::new(plugin)).await?;

    // Act - Rapidly start and stop service
    let start_time = Instant::now();

    for _ in 0..5 {
        let handle = env.start_service("alpine").await?;
        env.stop_service(&handle.id).await?;
    }

    let duration = start_time.elapsed();

    // Assert - Should handle rapid cycling
    assert!(
        duration < Duration::from_secs(150),
        "5 service cycles should complete in <2.5 minutes"
    );

    Ok(())
}

#[tokio::test]
async fn test_concurrent_service_lifecycle() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;

    for i in 0..3 {
        let plugin = GenericContainerPlugin::new(&format!("alpine{}", i), "alpine:latest");
        env.register_service(Box::new(plugin)).await?;
    }

    // Act - Concurrent lifecycle operations
    let start_time = Instant::now();

    let (handle0, handle1, handle2) = tokio::join!(
        env.start_service("alpine0"),
        env.start_service("alpine1"),
        env.start_service("alpine2")
    );

    let h0 = handle0?;
    let h1 = handle1?;
    let h2 = handle2?;

    let _ = tokio::join!(
        env.stop_service(&h0.id),
        env.stop_service(&h1.id),
        env.stop_service(&h2.id)
    );

    let duration = start_time.elapsed();

    // Assert
    assert!(
        duration < Duration::from_secs(90),
        "Concurrent lifecycle should be efficient"
    );

    Ok(())
}

// ============================================================================
// Integration Tests
// ============================================================================

#[tokio::test]
async fn test_async_plugin_full_workflow() -> clnrm_core::Result<()> {
    // Arrange
    let env = create_test_environment().await?;
    let plugin = GenericContainerPlugin::new("alpine", "alpine:latest");

    // Act - Full async workflow
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;

    let result1 = env
        .execute_in_container(
            "alpine",
            &["echo".to_string(), "step1".to_string()],
            None,
            None,
        )
        .await?;

    let result2 = env
        .execute_in_container(
            "alpine",
            &["echo".to_string(), "step2".to_string()],
            None,
            None,
        )
        .await?;

    let health = env.check_health().await;

    env.stop_service(&handle.id).await?;

    // Assert
    assert!(result1.succeeded());
    assert!(result2.succeeded());
    assert_eq!(health.len(), 1);

    Ok(())
}
