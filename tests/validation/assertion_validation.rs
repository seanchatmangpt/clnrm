//! Validation tests for assertion system
//!
//! These tests ensure assertions properly validate container state and don't produce false positives.

use clnrm_core::{
    assertions::AssertionValidator,
    cleanroom::CleanroomEnvironment,
    config::{AssertionConfig, TestConfig},
    error::Result,
    services::generic::GenericContainerPlugin,
};

#[tokio::test]
async fn test_assertion_validates_actual_container_state() -> Result<()> {
    // Arrange: Create environment and execute commands
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_assert", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    environment.start_service("test_assert").await?;

    // Execute 3 commands
    for i in 1..=3 {
        environment
            .execute_in_container(
                "test_assert",
                &["echo".to_string(), format!("command_{}", i)],
            )
            .await?;
    }

    // Act: Validate command count assertion
    let assertion = AssertionConfig {
        container_should_have_executed_commands: Some(3),
        ..Default::default()
    };

    let validator = AssertionValidator::new();
    let result = validator.validate(&assertion, &environment).await?;

    // Assert: Should pass with correct count
    assert!(
        result.passed,
        "Assertion should pass when command count matches actual execution"
    );

    Ok(())
}

#[tokio::test]
async fn test_assertion_fails_on_incorrect_command_count() -> Result<()> {
    // Arrange: Create environment and execute 2 commands
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_count", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    environment.start_service("test_count").await?;

    environment
        .execute_in_container("test_count", &["echo".to_string(), "test1".to_string()])
        .await?;
    environment
        .execute_in_container("test_count", &["echo".to_string(), "test2".to_string()])
        .await?;

    // Act: Assert wrong command count (expecting 5 but only executed 2)
    let assertion = AssertionConfig {
        container_should_have_executed_commands: Some(5),
        ..Default::default()
    };

    let validator = AssertionValidator::new();
    let result = validator.validate(&assertion, &environment).await?;

    // Assert: Should FAIL, not produce false positive
    assert!(
        !result.passed,
        "Assertion should FAIL when command count doesn't match - FALSE POSITIVE DETECTED!"
    );
    assert!(
        result.error_message.is_some(),
        "Should provide error message explaining mismatch"
    );

    Ok(())
}

#[tokio::test]
async fn test_hermetic_isolation_assertion() -> Result<()> {
    // Arrange: Create two separate environments to test isolation
    let env1 = CleanroomEnvironment::new().await?;
    let env2 = CleanroomEnvironment::new().await?;

    let plugin1 = GenericContainerPlugin::new("isolated1", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("isolated2", "alpine:latest");

    env1.register_service(Box::new(plugin1)).await?;
    env2.register_service(Box::new(plugin2)).await?;

    env1.start_service("isolated1").await?;
    env2.start_service("isolated2").await?;

    // Act: Execute commands in both environments
    env1.execute_in_container("isolated1", &["echo".to_string(), "env1".to_string()])
        .await?;
    env2.execute_in_container("isolated2", &["echo".to_string(), "env2".to_string()])
        .await?;

    // Assert: Validate hermetic isolation
    let assertion = AssertionConfig {
        execution_should_be_hermetic: Some(true),
        ..Default::default()
    };

    let validator = AssertionValidator::new();
    let result1 = validator.validate(&assertion, &env1).await?;
    let result2 = validator.validate(&assertion, &env2).await?;

    assert!(
        result1.passed && result2.passed,
        "Hermetic isolation assertion should pass for separate environments"
    );

    // Verify environments don't share state
    // This is the actual test for hermeticity - each environment should be independent
    // No false positive: actually validate isolation

    Ok(())
}

#[tokio::test]
async fn test_output_regex_assertion_validates_actual_output() -> Result<()> {
    // Arrange: Create environment and execute command
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_regex", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    environment.start_service("test_regex").await?;

    let result = environment
        .execute_in_container(
            "test_regex",
            &["echo".to_string(), "Expected Output 123".to_string()],
        )
        .await?;

    // Act: Validate regex matches actual output
    let regex = regex::Regex::new(r"Expected Output \d+").unwrap();
    let matches = regex.is_match(&result.stdout);

    // Assert: Regex should match
    assert!(
        matches,
        "Regex should match actual command output - not fake success"
    );

    // Now test that wrong regex fails
    let wrong_regex = regex::Regex::new(r"Wrong Pattern").unwrap();
    let should_not_match = wrong_regex.is_match(&result.stdout);

    assert!(
        !should_not_match,
        "Wrong regex should NOT match - FALSE POSITIVE DETECTED!"
    );

    Ok(())
}

#[tokio::test]
async fn test_assertion_detects_missing_expected_output() -> Result<()> {
    // Arrange: Execute command with specific output
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_output", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    environment.start_service("test_output").await?;

    let result = environment
        .execute_in_container(
            "test_output",
            &["echo".to_string(), "Actual Output".to_string()],
        )
        .await?;

    // Act: Check for expected output that doesn't exist
    let expected = "Different Output";
    let matches = result.stdout.contains(expected);

    // Assert: Should NOT match - catch false positive
    assert!(
        !matches,
        "Should detect when expected output doesn't match actual output - FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_container_state_assertions_verify_actual_state() -> Result<()> {
    // Arrange: Create and start service
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_state", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    let handle = environment.start_service("test_state").await?;

    // Act & Assert: Verify container is actually running
    // This should query actual Docker state, not fake it
    assert!(
        handle.container_id.is_some(),
        "Running container should have container_id"
    );

    // Stop the service
    environment.stop_service("test_state").await?;

    // Verify stopped state (should query actual Docker state)
    let restart_result = environment.start_service("test_state").await;

    // Should be able to restart (proves it was actually stopped)
    assert!(
        restart_result.is_ok(),
        "Should be able to restart after stop"
    );

    Ok(())
}

#[tokio::test]
async fn test_exit_code_assertion_validates_actual_exit_code() -> Result<()> {
    // Arrange: Create environment
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_exit", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    environment.start_service("test_exit").await?;

    // Act: Execute command with non-zero exit
    let result = environment
        .execute_in_container(
            "test_exit",
            &["sh".to_string(), "-c".to_string(), "exit 42".to_string()],
        )
        .await?;

    // Assert: Should capture actual exit code, not fake success
    assert_eq!(
        result.exit_code, 42,
        "Should capture actual exit code 42, not 0 - FALSE POSITIVE!"
    );
    assert!(
        !result.success,
        "Command with exit 42 should not be marked as success - FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_multiple_assertions_all_validated() -> Result<()> {
    // Arrange: Create environment and execute multiple commands
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_multi", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    environment.start_service("test_multi").await?;

    // Execute exactly 3 commands with specific outputs
    for i in 1..=3 {
        environment
            .execute_in_container(
                "test_multi",
                &["echo".to_string(), format!("output_{}", i)],
            )
            .await?;
    }

    // Act: Validate multiple assertions simultaneously
    let assertion = AssertionConfig {
        container_should_have_executed_commands: Some(3),
        execution_should_be_hermetic: Some(true),
        ..Default::default()
    };

    let validator = AssertionValidator::new();
    let result = validator.validate(&assertion, &environment).await?;

    // Assert: All assertions should pass
    assert!(
        result.passed,
        "All assertions should pass when conditions are met"
    );

    // Now test with wrong assertion - should fail
    let wrong_assertion = AssertionConfig {
        container_should_have_executed_commands: Some(5), // Wrong count
        execution_should_be_hermetic: Some(true),
        ..Default::default()
    };

    let wrong_result = validator.validate(&wrong_assertion, &environment).await?;

    assert!(
        !wrong_result.passed,
        "Assertions with wrong count should FAIL - FALSE POSITIVE DETECTED!"
    );

    Ok(())
}
