//! End-to-End Basic Workflow Integration Tests
//!
//! Tests the complete workflow from configuration loading to test execution
//! with single service container.

use super::common::*;
use clnrm_core::*;

#[tokio::test]
async fn test_basic_workflow_single_container() -> Result<()> {
    // Arrange: Create environment and configuration
    let env = create_test_environment().await?;
    let config = test_config_with_service("basic_test", "alpine", "alpine:latest");

    // Register service
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Start service and execute command
    let handle = env.start_service("alpine").await?;
    let result = env
        .execute_in_container("alpine", &["echo".to_string(), "hello".to_string()], None, None)
        .await?;

    // Assert: Verify execution succeeded
    assert!(result.succeeded(), "Command execution should succeed");
    assert!(
        result.stdout.contains("hello"),
        "Output should contain 'hello'"
    );

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_basic_workflow_with_environment_variables() -> Result<()> {
    // Arrange: Create environment with env vars
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let mut env_vars = std::collections::HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());

    // Act: Start service and execute with env vars
    let handle = env.start_service("alpine").await?;
    let result = env
        .execute_in_container(
            "alpine",
            &["sh".to_string(), "-c".to_string(), "echo $TEST_VAR".to_string()],
            Some(&env_vars),
            None,
        )
        .await?;

    // Assert: Verify env var was set
    assert!(result.succeeded(), "Command should succeed");
    assert!(
        result.stdout.contains("test_value"),
        "Output should contain env var value"
    );

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_basic_workflow_with_multiple_steps() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Execute multiple sequential steps
    let handle = env.start_service("alpine").await?;

    let step1 = env
        .execute_in_container("alpine", &["echo".to_string(), "step1".to_string()], None, None)
        .await?;

    let step2 = env
        .execute_in_container("alpine", &["echo".to_string(), "step2".to_string()], None, None)
        .await?;

    let step3 = env
        .execute_in_container("alpine", &["echo".to_string(), "step3".to_string()], None, None)
        .await?;

    // Assert: All steps succeeded
    assert!(step1.succeeded(), "Step 1 should succeed");
    assert!(step2.succeeded(), "Step 2 should succeed");
    assert!(step3.succeeded(), "Step 3 should succeed");

    assert!(step1.stdout.contains("step1"));
    assert!(step2.stdout.contains("step2"));
    assert!(step3.stdout.contains("step3"));

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_basic_workflow_command_failure_handling() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Execute command that fails
    let handle = env.start_service("alpine").await?;
    let result = env
        .execute_in_container(
            "alpine",
            &["sh".to_string(), "-c".to_string(), "exit 1".to_string()],
            None,
            None,
        )
        .await?;

    // Assert: Command should fail with exit code 1
    assert!(!result.succeeded(), "Command should fail");
    assert_eq!(result.exit_code, 1, "Exit code should be 1");

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_basic_workflow_with_workdir() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Execute command in specific workdir
    let handle = env.start_service("alpine").await?;

    // Create directory first
    let _ = env
        .execute_in_container(
            "alpine",
            &["mkdir".to_string(), "-p".to_string(), "/tmp/test".to_string()],
            None,
            None,
        )
        .await?;

    // Execute in workdir
    let result = env
        .execute_in_container(
            "alpine",
            &["pwd".to_string()],
            None,
            Some("/tmp/test"),
        )
        .await?;

    // Assert: Verify workdir was set
    assert!(result.succeeded(), "Command should succeed");
    assert!(
        result.stdout.contains("/tmp/test"),
        "Should be in correct workdir"
    );

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_basic_workflow_cleanup_on_error() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Start and stop service
    let handle = env.start_service("alpine").await?;
    env.stop_service(&handle.id).await?;

    // Assert: Service should be cleaned up
    let health = env.check_health().await;
    assert!(health.is_empty(), "All services should be cleaned up");

    Ok(())
}

#[tokio::test]
async fn test_basic_workflow_stdout_stderr_capture() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Execute commands writing to stdout and stderr
    let handle = env.start_service("alpine").await?;

    let stdout_result = env
        .execute_in_container(
            "alpine",
            &["sh".to_string(), "-c".to_string(), "echo 'to stdout'".to_string()],
            None,
            None,
        )
        .await?;

    let stderr_result = env
        .execute_in_container(
            "alpine",
            &[
                "sh".to_string(),
                "-c".to_string(),
                "echo 'to stderr' >&2".to_string(),
            ],
            None,
            None,
        )
        .await?;

    // Assert: Both stdout and stderr captured correctly
    assert!(
        stdout_result.stdout.contains("to stdout"),
        "Stdout should be captured"
    );
    assert!(
        stderr_result.stderr.contains("to stderr"),
        "Stderr should be captured"
    );

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}
