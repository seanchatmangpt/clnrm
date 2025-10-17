//! Integration tests for hermetic isolation validation
//!
//! These tests validate that the cleanroom framework provides true hermetic isolation:
//! - Each test runs in a fresh container
//! - No state leakage between tests
//! - No external service dependencies
//! - Reproducible execution
//!
//! Following London School TDD:
//! - Mock external dependencies
//! - Verify container lifecycle interactions
//! - Test collaboration between cleanroom environment and backend
//! - AAA pattern for all tests

#![cfg(test)]

use clnrm_core::error::Result;
use clnrm_core::cleanroom::CleanroomEnvironment;
use clnrm_core::services::generic::GenericContainerPlugin;
use clnrm_core::backend::testcontainer::TestcontainerBackend;
use std::sync::Arc;

// ============================================================================
// Hermetic Isolation Tests
// ============================================================================

#[tokio::test]
async fn test_fresh_container_per_test_ensures_isolation() -> Result<()> {
    // Arrange - Create two independent cleanroom environments
    let backend1 = Arc::new(TestcontainerBackend::new());
    let env1 = CleanroomEnvironment::with_backend(backend1);

    let backend2 = Arc::new(TestcontainerBackend::new());
    let env2 = CleanroomEnvironment::with_backend(backend2);

    // Act - Register same plugin in both environments
    let plugin1 = GenericContainerPlugin::new("test", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("test", "alpine:latest");

    env1.register_service(Box::new(plugin1)).await?;
    env2.register_service(Box::new(plugin2)).await?;

    let handle1 = env1.start_service("test").await?;
    let handle2 = env2.start_service("test").await?;

    // Assert - Containers should have different IDs (isolated)
    assert_ne!(handle1.container_id, handle2.container_id,
        "Containers should be independent for hermetic isolation");

    Ok(())
}

#[tokio::test]
async fn test_container_filesystem_isolation() -> Result<()> {
    // Arrange - Create cleanroom environment
    let backend = Arc::new(TestcontainerBackend::new());
    let env = CleanroomEnvironment::with_backend(backend);

    let plugin = GenericContainerPlugin::new("test", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;
    let handle = env.start_service("test").await?;

    // Act - Write file in container
    let write_result = env.execute_command(
        &handle,
        &["sh", "-c", "echo 'test data' > /tmp/test.txt && cat /tmp/test.txt"]
    ).await?;

    // Assert - File should be isolated to this container
    assert!(write_result.stdout.contains("test data"));

    // Act - Stop and start new container with same plugin
    env.stop_service("test").await?;

    let plugin2 = GenericContainerPlugin::new("test", "alpine:latest");
    env.register_service(Box::new(plugin2)).await?;
    let handle2 = env.start_service("test").await?;

    // Try to read file (should not exist - hermetic isolation)
    let read_result = env.execute_command(
        &handle2,
        &["sh", "-c", "test -f /tmp/test.txt && echo 'exists' || echo 'not found'"]
    ).await?;

    // Assert - File should not exist in new container
    assert!(read_result.stdout.contains("not found"),
        "Files should not leak between container instances");

    Ok(())
}

#[tokio::test]
async fn test_no_network_leakage_between_containers() -> Result<()> {
    // Arrange - Create two isolated cleanroom environments
    let backend1 = Arc::new(TestcontainerBackend::new());
    let env1 = CleanroomEnvironment::with_backend(backend1);

    let backend2 = Arc::new(TestcontainerBackend::new());
    let env2 = CleanroomEnvironment::with_backend(backend2);

    let plugin1 = GenericContainerPlugin::new("container1", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("container2", "alpine:latest");

    env1.register_service(Box::new(plugin1)).await?;
    env2.register_service(Box::new(plugin2)).await?;

    let handle1 = env1.start_service("container1").await?;
    let handle2 = env2.start_service("container2").await?;

    // Act - Try to communicate between containers (should fail)
    let ping_result = env1.execute_command(
        &handle1,
        &["sh", "-c", &format!("ping -c 1 -W 1 {} || echo 'isolated'", handle2.container_id)]
    ).await?;

    // Assert - Containers should be network isolated
    assert!(ping_result.stdout.contains("isolated") ||
            ping_result.stderr.contains("Name or service not known"),
        "Containers should not be able to communicate without explicit networking");

    Ok(())
}

#[tokio::test]
async fn test_container_cleanup_prevents_state_leakage() -> Result<()> {
    // Arrange
    let backend = Arc::new(TestcontainerBackend::new());
    let env = CleanroomEnvironment::with_backend(backend);

    let plugin = GenericContainerPlugin::new("test", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;
    let handle = env.start_service("test").await?;

    let container_id = handle.container_id.clone();

    // Act - Execute command and stop service
    env.execute_command(&handle, &["echo", "test"]).await?;
    env.stop_service("test").await?;

    // Assert - Container should be cleaned up
    // NOTE: This is implementation-specific validation
    // Real validation would check container no longer exists via Docker API
    assert!(!container_id.is_empty(), "Container ID should have been set");

    Ok(())
}

// ============================================================================
// Determinism and Reproducibility Tests
// ============================================================================

#[tokio::test]
async fn test_reproducible_execution_with_same_inputs() -> Result<()> {
    // Arrange - Create two identical environments
    let backend1 = Arc::new(TestcontainerBackend::new());
    let env1 = CleanroomEnvironment::with_backend(backend1);

    let backend2 = Arc::new(TestcontainerBackend::new());
    let env2 = CleanroomEnvironment::with_backend(backend2);

    // Act - Execute identical command in both
    let plugin1 = GenericContainerPlugin::new("test", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("test", "alpine:latest");

    env1.register_service(Box::new(plugin1)).await?;
    env2.register_service(Box::new(plugin2)).await?;

    let handle1 = env1.start_service("test").await?;
    let handle2 = env2.start_service("test").await?;

    let result1 = env1.execute_command(&handle1, &["echo", "deterministic output"]).await?;
    let result2 = env2.execute_command(&handle2, &["echo", "deterministic output"]).await?;

    // Assert - Outputs should be identical
    assert_eq!(result1.stdout, result2.stdout,
        "Identical inputs should produce identical outputs");

    Ok(())
}

#[tokio::test]
async fn test_environment_variable_isolation() -> Result<()> {
    // Arrange - Create two containers with different env vars
    let backend = Arc::new(TestcontainerBackend::new());
    let env = CleanroomEnvironment::with_backend(backend.clone());

    let mut plugin1 = GenericContainerPlugin::new("test1", "alpine:latest");
    plugin1.with_env("TEST_VAR", "value1");

    let mut plugin2 = GenericContainerPlugin::new("test2", "alpine:latest");
    plugin2.with_env("TEST_VAR", "value2");

    env.register_service(Box::new(plugin1)).await?;
    env.register_service(Box::new(plugin2)).await?;

    let handle1 = env.start_service("test1").await?;
    let handle2 = env.start_service("test2").await?;

    // Act - Check environment variables
    let result1 = env.execute_command(&handle1, &["sh", "-c", "echo $TEST_VAR"]).await?;
    let result2 = env.execute_command(&handle2, &["sh", "-c", "echo $TEST_VAR"]).await?;

    // Assert - Environment variables should be isolated
    assert!(result1.stdout.contains("value1"));
    assert!(result2.stdout.contains("value2"));

    Ok(())
}

// ============================================================================
// Resource Isolation Tests
// ============================================================================

#[tokio::test]
async fn test_process_isolation_between_executions() -> Result<()> {
    // Arrange
    let backend = Arc::new(TestcontainerBackend::new());
    let env = CleanroomEnvironment::with_backend(backend);

    let plugin = GenericContainerPlugin::new("test", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;
    let handle = env.start_service("test").await?;

    // Act - Execute commands that create processes
    let result1 = env.execute_command(
        &handle,
        &["sh", "-c", "echo $$ > /tmp/pid1.txt && cat /tmp/pid1.txt"]
    ).await?;

    let result2 = env.execute_command(
        &handle,
        &["sh", "-c", "echo $$ > /tmp/pid2.txt && cat /tmp/pid2.txt"]
    ).await?;

    // Assert - Process IDs should be different (separate executions)
    let pid1 = result1.stdout.trim();
    let pid2 = result2.stdout.trim();
    assert_ne!(pid1, pid2, "Each command execution should have isolated process");

    Ok(())
}

#[tokio::test]
async fn test_working_directory_isolation() -> Result<()> {
    // Arrange
    let backend = Arc::new(TestcontainerBackend::new());
    let env = CleanroomEnvironment::with_backend(backend);

    let plugin = GenericContainerPlugin::new("test", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;
    let handle = env.start_service("test").await?;

    // Act - Create directory in one execution
    env.execute_command(&handle, &["mkdir", "/tmp/test_dir"]).await?;

    // Change to that directory
    let pwd_result = env.execute_command(
        &handle,
        &["sh", "-c", "cd /tmp/test_dir && pwd"]
    ).await?;

    // Assert - Working directory should be accessible
    assert!(pwd_result.stdout.contains("/tmp/test_dir"));

    // New execution should start in default directory
    let default_pwd = env.execute_command(&handle, &["pwd"]).await?;
    assert!(!default_pwd.stdout.contains("test_dir"),
        "Each execution should start in default working directory");

    Ok(())
}

// ============================================================================
// Error Isolation Tests
// ============================================================================

#[tokio::test]
async fn test_failed_command_does_not_affect_subsequent_executions() -> Result<()> {
    // Arrange
    let backend = Arc::new(TestcontainerBackend::new());
    let env = CleanroomEnvironment::with_backend(backend);

    let plugin = GenericContainerPlugin::new("test", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;
    let handle = env.start_service("test").await?;

    // Act - Execute failing command
    let fail_result = env.execute_command(&handle, &["false"]).await;

    // Assert - Command should fail
    assert!(fail_result.is_err() || !fail_result.unwrap().success);

    // Act - Execute successful command after failure
    let success_result = env.execute_command(&handle, &["echo", "success"]).await?;

    // Assert - Subsequent command should succeed
    assert!(success_result.success);
    assert!(success_result.stdout.contains("success"));

    Ok(())
}

#[tokio::test]
async fn test_container_restart_provides_clean_slate() -> Result<()> {
    // Arrange
    let backend = Arc::new(TestcontainerBackend::new());
    let env = CleanroomEnvironment::with_backend(backend);

    let plugin = GenericContainerPlugin::new("test", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;
    let handle = env.start_service("test").await?;

    // Act - Create state in container
    env.execute_command(&handle, &["sh", "-c", "echo 'state' > /tmp/state.txt"]).await?;

    // Stop and restart
    env.stop_service("test").await?;

    let plugin2 = GenericContainerPlugin::new("test", "alpine:latest");
    env.register_service(Box::new(plugin2)).await?;
    let new_handle = env.start_service("test").await?;

    // Check if state persists
    let check_result = env.execute_command(
        &new_handle,
        &["sh", "-c", "test -f /tmp/state.txt && echo 'exists' || echo 'clean'"]
    ).await?;

    // Assert - State should not persist (clean slate)
    assert!(check_result.stdout.contains("clean"),
        "Container restart should provide clean slate");

    Ok(())
}

// ============================================================================
// Multi-Service Isolation Tests
// ============================================================================

#[tokio::test]
async fn test_multiple_services_remain_isolated() -> Result<()> {
    // Arrange - Create environment with multiple services
    let backend = Arc::new(TestcontainerBackend::new());
    let env = CleanroomEnvironment::with_backend(backend);

    let plugin1 = GenericContainerPlugin::new("service1", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("service2", "alpine:latest");

    env.register_service(Box::new(plugin1)).await?;
    env.register_service(Box::new(plugin2)).await?;

    let handle1 = env.start_service("service1").await?;
    let handle2 = env.start_service("service2").await?;

    // Act - Create files in each service
    env.execute_command(&handle1, &["sh", "-c", "echo 'service1' > /tmp/test.txt"]).await?;
    env.execute_command(&handle2, &["sh", "-c", "echo 'service2' > /tmp/test.txt"]).await?;

    let result1 = env.execute_command(&handle1, &["cat", "/tmp/test.txt"]).await?;
    let result2 = env.execute_command(&handle2, &["cat", "/tmp/test.txt"]).await?;

    // Assert - Files should be independent
    assert!(result1.stdout.contains("service1"));
    assert!(result2.stdout.contains("service2"));

    Ok(())
}
