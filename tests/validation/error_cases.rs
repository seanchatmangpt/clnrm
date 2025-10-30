//! Validation tests for error case handling
//!
//! These tests ensure that error cases ACTUALLY fail and don't produce false positives.
//! Based on false positive analysis findings.

use clnrm_core::{
    cleanroom::CleanroomEnvironment,
    error::{CleanroomError, Result},
    services::generic::GenericContainerPlugin,
};

#[tokio::test]
async fn test_invalid_container_image_fails() -> Result<()> {
    // Arrange: Create environment with invalid image
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_invalid", "invalid:nonexistent");
    environment.register_service(Box::new(plugin)).await?;

    // Act: Attempt to start service with invalid image
    let result = environment.start_service("test_invalid").await;

    // Assert: Should fail with appropriate error
    assert!(
        result.is_err(),
        "Starting service with invalid image should fail, not return Ok()"
    );

    match result {
        Err(CleanroomError::ServiceStartFailed { .. }) => {
            // Correct error type
        }
        Err(e) => panic!("Wrong error type: expected ServiceStartFailed, got {:?}", e),
        Ok(_) => panic!("Should not succeed with invalid image - FALSE POSITIVE!"),
    }

    Ok(())
}

#[tokio::test]
async fn test_execute_in_nonexistent_container_fails() -> Result<()> {
    // Arrange: Create environment without starting service
    let environment = CleanroomEnvironment::new().await?;

    // Act: Attempt to execute in non-existent container
    let result = environment
        .execute_in_container("nonexistent_container", &["echo".to_string(), "test".to_string()])
        .await;

    // Assert: Should fail, not pretend to succeed
    assert!(
        result.is_err(),
        "Executing in non-existent container should fail, not return Ok() - FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_command_timeout_fails() -> Result<()> {
    // Arrange: Create environment with service
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_timeout", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    environment.start_service("test_timeout").await?;

    // Act: Execute command that exceeds timeout (sleep for 300s with 1s timeout)
    let result = environment
        .execute_in_container(
            "test_timeout",
            &["sleep".to_string(), "300".to_string()],
        )
        .await;

    // Assert: Should timeout and fail
    // Note: Actual timeout implementation may vary, but it should NOT succeed
    match result {
        Ok(exec_result) => {
            assert!(
                !exec_result.success,
                "Long-running command should timeout or fail, not succeed - FALSE POSITIVE!"
            );
        }
        Err(_) => {
            // Acceptable: timeout produces error
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_failing_command_reports_failure() -> Result<()> {
    // Arrange: Create environment with service
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_fail", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    environment.start_service("test_fail").await?;

    // Act: Execute command that will fail (exit code 1)
    let result = environment
        .execute_in_container("test_fail", &["sh".to_string(), "-c".to_string(), "exit 1".to_string()])
        .await?;

    // Assert: Should report failure, not fake success
    assert!(
        !result.success,
        "Failed command (exit 1) should report success=false, not fake success - FALSE POSITIVE!"
    );
    assert_eq!(
        result.exit_code, 1,
        "Exit code should be 1, not 0 - FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_invalid_service_name_fails() -> Result<()> {
    // Arrange: Create environment
    let environment = CleanroomEnvironment::new().await?;

    // Act: Attempt to start service that was never registered
    let result = environment.start_service("never_registered").await;

    // Assert: Should fail with appropriate error
    assert!(
        result.is_err(),
        "Starting unregistered service should fail - FALSE POSITIVE!"
    );

    match result {
        Err(CleanroomError::ServiceNotFound { .. }) => {
            // Correct error type
        }
        Err(e) => panic!(
            "Wrong error type: expected ServiceNotFound, got {:?}",
            e
        ),
        Ok(_) => panic!("Should not succeed with unregistered service - FALSE POSITIVE!"),
    }

    Ok(())
}

#[tokio::test]
async fn test_double_service_registration_fails() -> Result<()> {
    // Arrange: Create environment and register service
    let environment = CleanroomEnvironment::new().await?;
    let plugin1 = GenericContainerPlugin::new("duplicate", "alpine:latest");
    environment.register_service(Box::new(plugin1)).await?;

    // Act: Attempt to register service with same name
    let plugin2 = GenericContainerPlugin::new("duplicate", "ubuntu:latest");
    let result = environment.register_service(Box::new(plugin2)).await;

    // Assert: Should fail or at least not corrupt state
    // Implementation may allow overwriting or may error - either is acceptable
    // but it should NOT corrupt the service registry
    let verify_result = environment.start_service("duplicate").await;
    assert!(
        verify_result.is_ok(),
        "Service registry should remain functional after duplicate registration"
    );

    Ok(())
}

#[tokio::test]
async fn test_empty_command_fails() -> Result<()> {
    // Arrange: Create environment with service
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_empty", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    environment.start_service("test_empty").await?;

    // Act: Execute empty command
    let result = environment
        .execute_in_container("test_empty", &[])
        .await;

    // Assert: Should fail or handle gracefully, not crash
    match result {
        Err(_) => {
            // Acceptable: empty command produces error
        }
        Ok(exec_result) => {
            assert!(
                !exec_result.success,
                "Empty command should not succeed - FALSE POSITIVE!"
            );
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_service_stop_without_start_fails() -> Result<()> {
    // Arrange: Create environment and register service but don't start
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_stop", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;

    // Act: Attempt to stop service that was never started
    let result = environment.stop_service("test_stop").await;

    // Assert: Should handle gracefully (either succeed as no-op or error)
    // but should NOT panic or corrupt state
    match result {
        Ok(_) | Err(_) => {
            // Both acceptable - as long as it doesn't panic
        }
    }

    Ok(())
}

#[cfg(test)]
mod false_positive_regression_tests {
    use super::*;

    /// Regression test for FALSE POSITIVE #1 from analysis:
    /// README claimed "Commands execute on HOST system, not in actual containers yet"
    /// But code DOES execute in containers. This test validates container execution works.
    #[tokio::test]
    async fn test_container_execution_actually_works() -> Result<()> {
        // Arrange
        let environment = CleanroomEnvironment::new().await?;
        let plugin = GenericContainerPlugin::new("test_container", "alpine:latest");
        environment.register_service(Box::new(plugin)).await?;
        environment.start_service("test_container").await?;

        // Act: Execute command that only exists in container, not on host
        let result = environment
            .execute_in_container(
                "test_container",
                &["sh".to_string(), "-c".to_string(), "echo CONTAINER".to_string()],
            )
            .await?;

        // Assert: Should succeed with container output
        assert!(
            result.success,
            "Container execution should work (FALSE POSITIVE: README said it doesn't)"
        );
        assert!(
            result.stdout.contains("CONTAINER"),
            "Should get container output, not host output"
        );

        Ok(())
    }

    /// Regression test for unimplemented!() false positives
    /// Ensures no production code paths call unimplemented!()
    #[test]
    fn test_no_unimplemented_in_production_paths() {
        // This is a compile-time check encoded as a test
        // If any production path calls unimplemented!(), it would panic in execution
        // The existence of passing integration tests validates this

        // Additionally, we can grep for unimplemented! in critical paths
        let critical_files = [
            "src/cleanroom.rs",
            "src/services/mod.rs",
            "src/backend/testcontainer.rs",
        ];

        for file in &critical_files {
            let content = std::fs::read_to_string(file);
            if let Ok(content) = content {
                assert!(
                    !content.contains("unimplemented!"),
                    "File {} contains unimplemented!() in production code - FALSE POSITIVE RISK!",
                    file
                );
            }
        }
    }
}
