//! Feature Integration: Service Routing Tests
//!
//! Tests service routing and discovery mechanisms.

use super::common::*;
use clnrm_core::*;

#[tokio::test]
async fn test_service_routing_by_name() -> Result<()> {
    // Arrange: Create environment with multiple services
    let env = create_test_environment().await?;

    let alpine = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    let busybox = services::generic::GenericContainerPlugin::new("busybox", "busybox:latest");

    env.register_service(Box::new(alpine)).await?;
    env.register_service(Box::new(busybox)).await?;

    // Act: Route commands to specific services
    let alpine_handle = env.start_service("alpine").await?;
    let busybox_handle = env.start_service("busybox").await?;

    let alpine_result = env
        .execute_in_container("alpine", &["echo".to_string(), "alpine".to_string()], None, None)
        .await?;

    let busybox_result = env
        .execute_in_container("busybox", &["echo".to_string(), "busybox".to_string()], None, None)
        .await?;

    // Assert: Commands routed to correct services
    assert!(alpine_result.stdout.contains("alpine"));
    assert!(busybox_result.stdout.contains("busybox"));

    // Cleanup
    env.stop_service(&alpine_handle.id).await?;
    env.stop_service(&busybox_handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_service_routing_nonexistent_service() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;

    // Act: Try to execute on nonexistent service
    let result = env
        .execute_in_container("nonexistent", &["echo".to_string(), "test".to_string()], None, None)
        .await;

    // Assert: Should return error
    assert!(result.is_err(), "Should error for nonexistent service");
    Ok(())
}

#[tokio::test]
async fn test_service_routing_health_checks() -> Result<()> {
    // Arrange: Create environment with services
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Start service and check health
    let handle = env.start_service("alpine").await?;
    let health = env.check_health().await;

    // Assert: Service appears in health check
    assert!(!health.is_empty(), "Health check should return active services");

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}
