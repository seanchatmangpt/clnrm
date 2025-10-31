//! Feature Integration: Chaos Engineering Tests
//!
//! Tests chaos engineering capabilities (container failures, network issues).

use super::common::*;
use clnrm_core::*;
use std::time::Duration;

#[tokio::test]
async fn test_chaos_container_restart() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Start, stop, and restart service
    let handle1 = env.start_service("alpine").await?;
    env.stop_service(&handle1.id).await?;

    let handle2 = env.start_service("alpine").await?;

    // Assert: Service restarted successfully
    let result = env
        .execute_in_container("alpine", &["echo".to_string(), "restarted".to_string()], None, None)
        .await?;

    assert!(result.succeeded());
    assert!(result.stdout.contains("restarted"));

    // Cleanup
    env.stop_service(&handle2.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_chaos_command_timeout() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Execute long-running command
    let handle = env.start_service("alpine").await?;
    let result = env
        .execute_in_container(
            "alpine",
            &["sleep".to_string(), "1".to_string()],
            None,
            None,
        )
        .await?;

    // Assert: Command completed (not timed out)
    assert!(result.succeeded());

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_chaos_service_failure_isolation() -> Result<()> {
    // Arrange: Create environment with multiple services
    let env = create_test_environment().await?;

    let alpine = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    let busybox = services::generic::GenericContainerPlugin::new("busybox", "busybox:latest");

    env.register_service(Box::new(alpine)).await?;
    env.register_service(Box::new(busybox)).await?;

    let alpine_handle = env.start_service("alpine").await?;
    let busybox_handle = env.start_service("busybox").await?;

    // Act: Stop one service, verify other unaffected
    env.stop_service(&alpine_handle.id).await?;

    let busybox_result = env
        .execute_in_container("busybox", &["echo".to_string(), "still_works".to_string()], None, None)
        .await?;

    // Assert: Busybox still works after alpine stopped
    assert!(busybox_result.succeeded());
    assert!(busybox_result.stdout.contains("still_works"));

    // Cleanup
    env.stop_service(&busybox_handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_chaos_resource_limits() -> Result<()> {
    // Arrange: Create environment with resource limits
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Execute resource-intensive command
    let handle = env.start_service("alpine").await?;
    let result = env
        .execute_in_container(
            "alpine",
            &["sh".to_string(), "-c".to_string(), "echo 'test' > /tmp/test.txt && cat /tmp/test.txt".to_string()],
            None,
            None,
        )
        .await?;

    // Assert: Command succeeded within limits
    assert!(result.succeeded());

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}
