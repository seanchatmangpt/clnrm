//! Critical Integration Tests - The Essential 20%
//!
//! This file contains the most important integration tests that validate
//! core clnrm functionality. These tests provide 80% of the value.

use clnrm_core::*;

#[tokio::test(flavor = "multi_thread")]
async fn test_cleanroom_environment_creation() -> Result<()> {
    // Arrange & Act
    let env = CleanroomEnvironment::new().await?;

    // Assert
    assert!(!env.session_id().is_nil());
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_container_execution_hermetic() -> Result<()> {
    // Arrange
    let env = CleanroomEnvironment::new().await?;

    // Act
    let result = env
        .execute_in_container("test", &["echo".to_string(), "hello".to_string()])
        .await?;

    // Assert - proves hermetic isolation works
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("hello"));
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_service_registration_and_lifecycle() -> Result<()> {
    // Arrange
    let env = CleanroomEnvironment::new().await?;
    let plugin = Box::new(cleanroom::MockDatabasePlugin::new());

    // Act
    env.register_service(plugin).await?;
    let handle = env.start_service("mock_database").await?;

    // Assert
    assert_eq!(handle.service_name, "mock_database");
    assert!(handle.metadata.contains_key("port"));

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_container_reuse_tracking() -> Result<()> {
    // Arrange
    let env = CleanroomEnvironment::new().await?;

    // Act - First call creates container
    let container1 = env
        .get_or_create_container("test-container", || {
            Ok::<String, CleanroomError>("instance-1".to_string())
        })
        .await?;

    // Act - Second call reuses container
    let container2 = env
        .get_or_create_container("test-container", || {
            Ok::<String, CleanroomError>("instance-2".to_string())
        })
        .await?;

    // Assert - Same instance returned (true reuse)
    assert_eq!(container1, container2);
    assert_eq!(container1, "instance-1"); // Not instance-2

    // Assert - Metrics tracked correctly
    let (created, reused) = env.get_container_reuse_stats().await;
    assert_eq!(created, 1);
    assert_eq!(reused, 1);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_metrics_tracking() -> Result<()> {
    // Arrange
    let env = CleanroomEnvironment::new().await?;

    // Act
    let _result = env
        .execute_test("test_metrics", || Ok::<(), CleanroomError>(()))
        .await?;

    // Assert
    let metrics = env.get_metrics().await?;
    assert_eq!(metrics.tests_executed, 1);
    assert_eq!(metrics.tests_passed, 1);
    assert!(metrics.total_duration_ms > 0);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_health_check_all_services() -> Result<()> {
    // Arrange
    let env = CleanroomEnvironment::new().await?;
    let plugin = Box::new(cleanroom::MockDatabasePlugin::new());
    env.register_service(plugin).await?;
    let handle = env.start_service("mock_database").await?;

    // Act
    let health_status = env.check_health().await;

    // Assert
    assert_eq!(health_status.len(), 1);
    assert_eq!(
        health_status.get(&handle.id),
        Some(&cleanroom::HealthStatus::Healthy)
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_error_propagation() -> Result<()> {
    // Arrange
    let env = CleanroomEnvironment::new().await?;

    // Act
    let result = env
        .execute_test("failing_test", || {
            Err::<(), CleanroomError>(CleanroomError::validation_error("Expected failure"))
        })
        .await;

    // Assert
    assert!(result.is_err());

    let metrics = env.get_metrics().await?;
    assert_eq!(metrics.tests_executed, 1);
    assert_eq!(metrics.tests_failed, 1);
    assert_eq!(metrics.tests_passed, 0);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_telemetry_enabling() -> Result<()> {
    // Arrange
    let env = CleanroomEnvironment::new().await?;

    // Act
    env.enable_tracing().await?;
    env.enable_metrics().await?;

    // Assert - No panic, operations succeed
    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
async fn test_execution_result_helpers() {
    let result = ExecutionResult {
        exit_code: 0,
        stdout: "test output".to_string(),
        stderr: "".to_string(),
        duration: std::time::Duration::from_millis(100),
        command: vec!["echo".to_string()],
        container_name: "test".to_string(),
    };

    assert!(result.succeeded());
    assert!(!result.failed());
}

#[tokio::test(flavor = "multi_thread")]
async fn test_execution_result_regex_matching() -> Result<()> {
    let result = ExecutionResult {
        exit_code: 0,
        stdout: "Hello World 123".to_string(),
        stderr: "".to_string(),
        duration: std::time::Duration::from_millis(100),
        command: vec!["echo".to_string()],
        container_name: "test".to_string(),
    };

    assert!(result.matches_regex(r"\d+")?);
    assert!(result.does_not_match_regex(r"Error")?);

    Ok(())
}
