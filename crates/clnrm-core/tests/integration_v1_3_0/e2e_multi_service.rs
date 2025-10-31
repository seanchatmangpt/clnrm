//! End-to-End Multi-Service Integration Tests
//!
//! Tests coordination and communication between multiple services.

use super::common::*;
use clnrm_core::*;

#[tokio::test]
async fn test_multi_service_startup_coordination() -> Result<()> {
    // Arrange: Create environment with multiple services
    let env = create_test_environment().await?;

    let alpine_plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    let busybox_plugin = services::generic::GenericContainerPlugin::new("busybox", "busybox:latest");

    env.register_service(Box::new(alpine_plugin)).await?;
    env.register_service(Box::new(busybox_plugin)).await?;

    // Act: Start both services
    let alpine_handle = env.start_service("alpine").await?;
    let busybox_handle = env.start_service("busybox").await?;

    // Assert: Both services running
    let health = env.check_health().await;
    assert_eq!(health.len(), 2, "Both services should be running");

    // Cleanup
    env.stop_service(&alpine_handle.id).await?;
    env.stop_service(&busybox_handle.id).await?;

    Ok(())
}

#[tokio::test]
async fn test_multi_service_independent_execution() -> Result<()> {
    // Arrange: Create environment with multiple services
    let env = create_test_environment().await?;

    let alpine_plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    let busybox_plugin = services::generic::GenericContainerPlugin::new("busybox", "busybox:latest");

    env.register_service(Box::new(alpine_plugin)).await?;
    env.register_service(Box::new(busybox_plugin)).await?;

    let alpine_handle = env.start_service("alpine").await?;
    let busybox_handle = env.start_service("busybox").await?;

    // Act: Execute commands independently in each service
    let alpine_result = env
        .execute_in_container("alpine", &["echo".to_string(), "alpine_test".to_string()], None, None)
        .await?;

    let busybox_result = env
        .execute_in_container("busybox", &["echo".to_string(), "busybox_test".to_string()], None, None)
        .await?;

    // Assert: Both executions succeeded independently
    assert!(alpine_result.succeeded(), "Alpine command should succeed");
    assert!(busybox_result.succeeded(), "Busybox command should succeed");
    assert!(alpine_result.stdout.contains("alpine_test"));
    assert!(busybox_result.stdout.contains("busybox_test"));

    // Cleanup
    env.stop_service(&alpine_handle.id).await?;
    env.stop_service(&busybox_handle.id).await?;

    Ok(())
}

#[tokio::test]
async fn test_multi_service_sequential_operations() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;

    let alpine_plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    let busybox_plugin = services::generic::GenericContainerPlugin::new("busybox", "busybox:latest");

    env.register_service(Box::new(alpine_plugin)).await?;
    env.register_service(Box::new(busybox_plugin)).await?;

    let alpine_handle = env.start_service("alpine").await?;
    let busybox_handle = env.start_service("busybox").await?;

    // Act: Perform operations sequentially
    let result1 = env
        .execute_in_container("alpine", &["echo".to_string(), "step1".to_string()], None, None)
        .await?;

    let result2 = env
        .execute_in_container("busybox", &["echo".to_string(), "step2".to_string()], None, None)
        .await?;

    let result3 = env
        .execute_in_container("alpine", &["echo".to_string(), "step3".to_string()], None, None)
        .await?;

    // Assert: All operations completed in order
    assert!(result1.succeeded() && result2.succeeded() && result3.succeeded());

    // Cleanup
    env.stop_service(&alpine_handle.id).await?;
    env.stop_service(&busybox_handle.id).await?;

    Ok(())
}

#[tokio::test]
async fn test_multi_service_cleanup_order() -> Result<()> {
    // Arrange: Create environment with multiple services
    let env = create_test_environment().await?;

    let alpine_plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    let busybox_plugin = services::generic::GenericContainerPlugin::new("busybox", "busybox:latest");

    env.register_service(Box::new(alpine_plugin)).await?;
    env.register_service(Box::new(busybox_plugin)).await?;

    let alpine_handle = env.start_service("alpine").await?;
    let busybox_handle = env.start_service("busybox").await?;

    // Act: Stop services in specific order
    env.stop_service(&alpine_handle.id).await?;

    // Assert: First service stopped, second still running
    let health = env.check_health().await;
    assert_eq!(health.len(), 1, "One service should still be running");

    env.stop_service(&busybox_handle.id).await?;

    // Assert: All services stopped
    let health = env.check_health().await;
    assert!(health.is_empty(), "All services should be stopped");

    Ok(())
}

#[tokio::test]
async fn test_multi_service_isolation() -> Result<()> {
    // Arrange: Create environment with isolated services
    let env = create_test_environment().await?;

    let alpine_plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    let busybox_plugin = services::generic::GenericContainerPlugin::new("busybox", "busybox:latest");

    env.register_service(Box::new(alpine_plugin)).await?;
    env.register_service(Box::new(busybox_plugin)).await?;

    let alpine_handle = env.start_service("alpine").await?;
    let busybox_handle = env.start_service("busybox").await?;

    // Act: Create file in alpine, verify not visible in busybox
    let _ = env
        .execute_in_container(
            "alpine",
            &[
                "sh".to_string(),
                "-c".to_string(),
                "echo 'test' > /tmp/alpine_file".to_string(),
            ],
            None,
            None,
        )
        .await?;

    let check_result = env
        .execute_in_container(
            "busybox",
            &["cat".to_string(), "/tmp/alpine_file".to_string()],
            None,
            None,
        )
        .await?;

    // Assert: File should not be accessible from busybox
    assert!(!check_result.succeeded(), "File should not be accessible across containers");

    // Cleanup
    env.stop_service(&alpine_handle.id).await?;
    env.stop_service(&busybox_handle.id).await?;

    Ok(())
}
