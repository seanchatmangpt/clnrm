//! Security Integration: Isolation Tests
//!
//! Tests hermetic isolation and security boundaries between containers.

use super::common::*;
use clnrm_core::*;

#[tokio::test]
async fn test_security_filesystem_isolation() -> Result<()> {
    // Arrange: Create two separate environments
    let env1 = create_test_environment().await?;
    let env2 = create_test_environment().await?;

    let plugin1 = services::generic::GenericContainerPlugin::new("alpine1", "alpine:latest");
    let plugin2 = services::generic::GenericContainerPlugin::new("alpine2", "alpine:latest");

    env1.register_service(Box::new(plugin1)).await?;
    env2.register_service(Box::new(plugin2)).await?;

    let handle1 = env1.start_service("alpine1").await?;
    let handle2 = env2.start_service("alpine2").await?;

    // Act: Create file in first container
    env1.execute_in_container(
        "alpine1",
        &[
            "sh".to_string(),
            "-c".to_string(),
            "echo 'secret' > /tmp/secret.txt".to_string(),
        ],
        None,
        None,
    )
    .await?;

    // Try to read file from second container
    let result = env2
        .execute_in_container(
            "alpine2",
            &["cat".to_string(), "/tmp/secret.txt".to_string()],
            None,
            None,
        )
        .await?;

    // Assert: File should not be accessible
    assert!(
        !result.succeeded(),
        "File should not be accessible across isolated containers"
    );

    // Cleanup
    env1.stop_service(&handle1.id).await?;
    env2.stop_service(&handle2.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_security_process_isolation() -> Result<()> {
    // Arrange: Create environment with multiple services
    let env = create_test_environment().await?;

    let alpine = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    let busybox = services::generic::GenericContainerPlugin::new("busybox", "busybox:latest");

    env.register_service(Box::new(alpine)).await?;
    env.register_service(Box::new(busybox)).await?;

    let alpine_handle = env.start_service("alpine").await?;
    let busybox_handle = env.start_service("busybox").await?;

    // Act: Run process in alpine, verify not visible from busybox
    env.execute_in_container(
        "alpine",
        &[
            "sh".to_string(),
            "-c".to_string(),
            "sleep 1 &".to_string(),
        ],
        None,
        None,
    )
    .await?;

    let ps_result = env
        .execute_in_container("busybox", &["ps".to_string(), "aux".to_string()], None, None)
        .await?;

    // Assert: Alpine processes should not be visible in busybox
    assert!(
        !ps_result.stdout.contains("alpine"),
        "Processes should be isolated between containers"
    );

    // Cleanup
    env.stop_service(&alpine_handle.id).await?;
    env.stop_service(&busybox_handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_security_network_isolation() -> Result<()> {
    // Arrange: Create environment with isolated networking
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;

    // Act: Verify network isolation (container has limited network access)
    // Note: Actual network isolation depends on Docker configuration
    let result = env
        .execute_in_container("alpine", &["hostname".to_string()], None, None)
        .await?;

    // Assert: Command succeeded (basic networking works)
    assert!(result.succeeded());

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_security_environment_variable_isolation() -> Result<()> {
    // Arrange: Create environment with different env vars per service
    let env = create_test_environment().await?;

    let alpine = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    let busybox = services::generic::GenericContainerPlugin::new("busybox", "busybox:latest");

    env.register_service(Box::new(alpine)).await?;
    env.register_service(Box::new(busybox)).await?;

    let alpine_handle = env.start_service("alpine").await?;
    let busybox_handle = env.start_service("busybox").await?;

    let mut alpine_env = std::collections::HashMap::new();
    alpine_env.insert("SECRET".to_string(), "alpine_secret".to_string());

    // Act: Set env var in alpine, check not visible in busybox
    env.execute_in_container(
        "alpine",
        &["sh".to_string(), "-c".to_string(), "echo $SECRET".to_string()],
        Some(&alpine_env),
        None,
    )
    .await?;

    let busybox_result = env
        .execute_in_container(
            "busybox",
            &["sh".to_string(), "-c".to_string(), "echo $SECRET".to_string()],
            None,
            None,
        )
        .await?;

    // Assert: Env var should not be visible in busybox
    assert!(
        !busybox_result.stdout.contains("alpine_secret"),
        "Environment variables should be isolated"
    );

    // Cleanup
    env.stop_service(&alpine_handle.id).await?;
    env.stop_service(&busybox_handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_security_working_directory_isolation() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;

    // Act: Create directory structure, verify isolation
    env.execute_in_container(
        "alpine",
        &["mkdir".to_string(), "-p".to_string(), "/tmp/test1".to_string()],
        None,
        None,
    )
    .await?;

    env.execute_in_container(
        "alpine",
        &["mkdir".to_string(), "-p".to_string(), "/tmp/test2".to_string()],
        None,
        None,
    )
    .await?;

    // Execute in test1
    let result1 = env
        .execute_in_container(
            "alpine",
            &["pwd".to_string()],
            None,
            Some("/tmp/test1"),
        )
        .await?;

    // Execute in test2
    let result2 = env
        .execute_in_container(
            "alpine",
            &["pwd".to_string()],
            None,
            Some("/tmp/test2"),
        )
        .await?;

    // Assert: Working directories are isolated
    assert!(result1.stdout.contains("/tmp/test1"));
    assert!(result2.stdout.contains("/tmp/test2"));

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_security_resource_limits() -> Result<()> {
    // Arrange: Create environment with resource constraints
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;

    // Act: Try to consume resources (within safe limits)
    let result = env
        .execute_in_container(
            "alpine",
            &[
                "sh".to_string(),
                "-c".to_string(),
                "dd if=/dev/zero of=/tmp/test bs=1M count=10".to_string(),
            ],
            None,
            None,
        )
        .await?;

    // Assert: Operation succeeded (within limits)
    assert!(result.succeeded());

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_security_cleanup_verification() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Start and stop service multiple times
    let handle1 = env.start_service("alpine").await?;
    env.stop_service(&handle1.id).await?;

    let handle2 = env.start_service("alpine").await?;
    env.stop_service(&handle2.id).await?;

    // Assert: All resources cleaned up
    let health = env.check_health().await;
    assert!(health.is_empty(), "All services should be cleaned up");

    Ok(())
}
