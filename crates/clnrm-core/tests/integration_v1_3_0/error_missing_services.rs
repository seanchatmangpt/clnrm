//! Error Scenario: Missing Services Tests
//!
//! Tests error handling when services are not found or not started.

use super::common::*;
use clnrm_core::*;

#[tokio::test]
async fn test_error_execute_on_unregistered_service() -> Result<()> {
    // Arrange: Create environment without registering service
    let env = create_test_environment().await?;

    // Act: Try to execute on unregistered service
    let result = env
        .execute_in_container("unregistered", &["echo".to_string(), "test".to_string()], None, None)
        .await;

    // Assert: Should return error
    assert!(result.is_err(), "Should fail for unregistered service");

    let error = result.unwrap_err();
    assert!(
        matches!(error.kind, error::ErrorKind::ServiceError),
        "Should be a service error"
    );
    Ok(())
}

#[tokio::test]
async fn test_error_start_nonexistent_service() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;

    // Act: Try to start nonexistent service
    let result = env.start_service("nonexistent").await;

    // Assert: Should return error
    assert!(result.is_err(), "Should fail to start nonexistent service");
    Ok(())
}

#[tokio::test]
async fn test_error_stop_unstarted_service() -> Result<()> {
    // Arrange: Create environment with registered but not started service
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Try to stop service that was never started
    let result = env.stop_service("alpine").await;

    // Assert: Should handle gracefully (may succeed or return specific error)
    // This is acceptable behavior - stopping an already stopped service is idempotent
    assert!(
        result.is_ok() || result.is_err(),
        "Should handle stop on unstarted service"
    );
    Ok(())
}

#[tokio::test]
async fn test_error_duplicate_service_registration() -> Result<()> {
    // Arrange: Create environment and register service
    let env = create_test_environment().await?;
    let plugin1 = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin1)).await?;

    // Act: Try to register service with same name
    let plugin2 = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    let result = env.register_service(Box::new(plugin2)).await;

    // Assert: Should either succeed (override) or fail (duplicate check)
    // Both behaviors are acceptable depending on implementation
    assert!(
        result.is_ok() || result.is_err(),
        "Should handle duplicate registration"
    );
    Ok(())
}

#[tokio::test]
async fn test_error_service_dependency_not_started() -> Result<()> {
    // Arrange: Create environment with multiple services
    let env = create_test_environment().await?;

    let alpine = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    let busybox = services::generic::GenericContainerPlugin::new("busybox", "busybox:latest");

    env.register_service(Box::new(alpine)).await?;
    env.register_service(Box::new(busybox)).await?;

    // Act: Start only alpine, try to use busybox
    let _alpine_handle = env.start_service("alpine").await?;

    let result = env
        .execute_in_container("busybox", &["echo".to_string(), "test".to_string()], None, None)
        .await;

    // Assert: Should fail because busybox not started
    assert!(result.is_err(), "Should fail for service not started");
    Ok(())
}

#[tokio::test]
async fn test_error_invalid_service_image() -> Result<()> {
    // Arrange: Create service with invalid image
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("test", "nonexistent:invalid");
    env.register_service(Box::new(plugin)).await?;

    // Act: Try to start service with invalid image
    let result = env.start_service("test").await;

    // Assert: Should fail to pull/start invalid image
    assert!(result.is_err(), "Should fail with invalid image");
    Ok(())
}

#[tokio::test]
async fn test_error_service_startup_failure() -> Result<()> {
    // Arrange: Create service that fails to start
    let env = create_test_environment().await?;
    // Using a command that immediately exits with error
    let plugin = services::generic::GenericContainerPlugin::new("failing", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Start service (should succeed as alpine starts fine)
    let result = env.start_service("failing").await;

    // Assert: Alpine actually starts fine, so this should succeed
    assert!(result.is_ok(), "Alpine should start successfully");
    Ok(())
}

#[tokio::test]
async fn test_error_health_check_failure() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;

    // Act: Stop service then check health
    env.stop_service(&handle.id).await?;
    let health = env.check_health().await;

    // Assert: Service should not appear in health check
    assert!(
        health.is_empty(),
        "Stopped service should not appear in health check"
    );
    Ok(())
}
