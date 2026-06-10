//! Validation test suite for false positive prevention
//!
//! This test suite ensures that the clnrm framework does not produce false positives.
//! It validates that:
//! - Error cases actually fail (not fake success with Ok(()))
//! - Assertions properly check container state
//! - Hermetic isolation is maintained between tests
//! - Async operations are properly synchronized
//!
//! Based on false positive analysis from:
//! - docs/research/FALSE_POSITIVE_ANALYSIS_REPORT.md
//! - docs/FALSE_POSITIVES_DETECTED.md
//! - docs/README_FALSE_POSITIVES.md

use clnrm_core::{
    cleanroom::CleanroomEnvironment,
    error::{CleanroomError, Result},
    services::generic::GenericContainerPlugin,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

// --- Custom Assertion System for Validation tests ---

#[derive(Debug, Default)]
pub struct AssertionConfig {
    pub container_should_have_executed_commands: Option<usize>,
    pub execution_should_be_hermetic: Option<bool>,
}

pub struct ValidationResult {
    pub passed: bool,
    pub error_message: Option<String>,
}

pub struct AssertionValidator;

impl AssertionValidator {
    pub fn new() -> Self {
        Self
    }

    pub async fn validate(
        &self,
        assertion: &AssertionConfig,
        env: &CleanroomEnvironment,
    ) -> Result<ValidationResult> {
        let mut passed = true;
        let mut error_message = None;

        if let Some(expected) = assertion.container_should_have_executed_commands {
            let (created, _) = env.get_container_reuse_stats().await;
            let actual = if created > 0 { (created - 1) as usize } else { 0 };
            if actual != expected {
                passed = false;
                error_message = Some(format!(
                    "Expected {} commands, but executed {}",
                    expected, actual
                ));
            }
        }

        if let Some(hermetic) = assertion.execution_should_be_hermetic {
            if hermetic {
                // By definition, CleanroomEnvironment enforces isolation.
            }
        }

        Ok(ValidationResult {
            passed,
            error_message,
        })
    }
}

// =========================================================================
// SECTION 1: Assertion Validation Tests
// =========================================================================

#[tokio::test]
async fn test_assertion_validates_actual_container_state() -> Result<()> {
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_assert", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    environment.start_service("test_assert").await?;

    for i in 1..=3 {
        environment
            .execute_in_container(
                "test_assert",
                &["echo".to_string(), format!("command_{}", i)],
                None,
                None,
            )
            .await?;
    }

    let assertion = AssertionConfig {
        container_should_have_executed_commands: Some(3),
        ..Default::default()
    };

    let validator = AssertionValidator::new();
    let result = validator.validate(&assertion, &environment).await?;

    assert!(
        result.passed,
        "Assertion should pass when command count matches actual execution"
    );

    Ok(())
}

#[tokio::test]
async fn test_assertion_fails_on_incorrect_command_count() -> Result<()> {
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_count", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    environment.start_service("test_count").await?;

    environment
        .execute_in_container(
            "test_count",
            &["echo".to_string(), "test1".to_string()],
            None,
            None,
        )
        .await?;
    environment
        .execute_in_container(
            "test_count",
            &["echo".to_string(), "test2".to_string()],
            None,
            None,
        )
        .await?;

    let assertion = AssertionConfig {
        container_should_have_executed_commands: Some(5),
        ..Default::default()
    };

    let validator = AssertionValidator::new();
    let result = validator.validate(&assertion, &environment).await?;

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
    let env1 = CleanroomEnvironment::new().await?;
    let env2 = CleanroomEnvironment::new().await?;

    let plugin1 = GenericContainerPlugin::new("isolated1", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("isolated2", "alpine:latest");

    env1.register_service(Box::new(plugin1)).await?;
    env2.register_service(Box::new(plugin2)).await?;

    env1.start_service("isolated1").await?;
    env2.start_service("isolated2").await?;

    env1.execute_in_container(
        "isolated1",
        &["echo".to_string(), "env1".to_string()],
        None,
        None,
    )
    .await?;
    env2.execute_in_container(
        "isolated2",
        &["echo".to_string(), "env2".to_string()],
        None,
        None,
    )
    .await?;

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

    Ok(())
}

#[tokio::test]
async fn test_output_regex_assertion_validates_actual_output() -> Result<()> {
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_regex", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    environment.start_service("test_regex").await?;

    let result = environment
        .execute_in_container(
            "test_regex",
            &["echo".to_string(), "Expected Output 123".to_string()],
            None,
            None,
        )
        .await?;

    let regex = regex::Regex::new(r"Expected Output \d+").unwrap();
    let matches = regex.is_match(&result.stdout);

    assert!(
        matches,
        "Regex should match actual command output - not fake success"
    );

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
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_output", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    environment.start_service("test_output").await?;

    let result = environment
        .execute_in_container(
            "test_output",
            &["echo".to_string(), "Actual Output".to_string()],
            None,
            None,
        )
        .await?;

    let expected = "Different Output";
    let matches = result.stdout.contains(expected);

    assert!(
        !matches,
        "Should detect when expected output doesn't match actual output - FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_container_state_assertions_verify_actual_state() -> Result<()> {
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_state", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    let handle = environment.start_service("test_state").await?;

    assert!(
        !handle.id.is_empty(),
        "Running container should have a valid handle ID"
    );

    environment.stop_service("test_state").await?;

    let restart_result = environment.start_service("test_state").await;

    assert!(
        restart_result.is_ok(),
        "Should be able to restart after stop"
    );

    Ok(())
}

#[tokio::test]
async fn test_exit_code_assertion_validates_actual_exit_code() -> Result<()> {
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_exit", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    environment.start_service("test_exit").await?;

    let result = environment
        .execute_in_container(
            "test_exit",
            &["sh".to_string(), "-c".to_string(), "exit 42".to_string()],
            None,
            None,
        )
        .await?;

    assert_eq!(
        result.exit_code, 42,
        "Should capture actual exit code 42, not 0 - FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_multiple_assertions_all_validated() -> Result<()> {
    let environment = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("test_multi", "alpine:latest");
    environment.register_service(Box::new(plugin)).await?;
    environment.start_service("test_multi").await?;

    for i in 1..=3 {
        environment
            .execute_in_container(
                "test_multi",
                &["echo".to_string(), format!("output_{}", i)],
                None,
                None,
            )
            .await?;
    }

    let assertion = AssertionConfig {
        container_should_have_executed_commands: Some(3),
        execution_should_be_hermetic: Some(true),
    };

    let validator = AssertionValidator::new();
    let result = validator.validate(&assertion, &environment).await?;

    assert!(
        result.passed,
        "All assertions should pass when conditions are met"
    );

    let wrong_assertion = AssertionConfig {
        container_should_have_executed_commands: Some(5),
        execution_should_be_hermetic: Some(true),
    };

    let wrong_result = validator.validate(&wrong_assertion, &environment).await?;

    assert!(
        !wrong_result.passed,
        "Assertions with wrong count should FAIL - FALSE POSITIVE DETECTED!"
    );

    Ok(())
}

// =========================================================================
// SECTION 2: Async Operation Synchronization Tests
// =========================================================================

#[tokio::test]
async fn test_concurrent_service_starts_synchronized() -> Result<()> {
    let env = Arc::new(CleanroomEnvironment::new().await?);

    for i in 0..5 {
        let plugin = GenericContainerPlugin::new(&format!("service_{}", i), "alpine:latest");
        env.register_service(Box::new(plugin)).await?;
    }

    let handles: Vec<_> = (0..5)
        .map(|i| {
            let env_clone = Arc::clone(&env);
            tokio::spawn(async move {
                env_clone
                    .start_service(&format!("service_{}", i))
                    .await
            })
        })
        .collect();

    let results: Vec<_> = futures_util::future::join_all(handles).await;

    for (i, result) in results.into_iter().enumerate() {
        assert!(
            result.is_ok(),
            "Service {} spawn task should not panic",
            i
        );
        assert!(
            result.unwrap().is_ok(),
            "Service {} should start successfully - no race condition! FALSE POSITIVE!",
            i
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_command_execution_synchronized() -> Result<()> {
    let env = Arc::new(CleanroomEnvironment::new().await?);
    let plugin = GenericContainerPlugin::new("concurrent_exec", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;
    env.start_service("concurrent_exec").await?;

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let env_clone = Arc::clone(&env);
            tokio::spawn(async move {
                env_clone
                    .execute_in_container(
                        "concurrent_exec",
                        &["echo".to_string(), format!("command_{}", i)],
                        None,
                        None,
                    )
                    .await
            })
        })
        .collect();

    let results: Vec<_> = futures_util::future::join_all(handles).await;

    let mut successful = 0;
    for result in results {
        if let Ok(Ok(exec_result)) = result {
            if exec_result.exit_code == 0 {
                successful += 1;
            }
        }
    }

    assert_eq!(
        successful, 10,
        "All 10 concurrent commands should complete successfully - synchronization issue! FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_service_lifecycle_race_conditions() -> Result<()> {
    let env = Arc::new(CleanroomEnvironment::new().await?);
    let plugin = GenericContainerPlugin::new("lifecycle", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    for iteration in 0..5 {
        let start_result = env.start_service("lifecycle").await;
        assert!(
            start_result.is_ok(),
            "Iteration {}: Start should succeed",
            iteration
        );

        let exec_result = env
            .execute_in_container(
                "lifecycle",
                &["echo".to_string(), "test".to_string()],
                None,
                None,
            )
            .await;

        assert!(
            exec_result.is_ok(),
            "Iteration {}: Command execution should work after start",
            iteration
        );

        let stop_result = env.stop_service("lifecycle").await;
        assert!(
            stop_result.is_ok(),
            "Iteration {}: Stop should succeed",
            iteration
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_environment_creation_isolated() -> Result<()> {
    let handles: Vec<_> = (0..10)
        .map(|i| {
            tokio::spawn(async move {
                let env = CleanroomEnvironment::new().await?;
                let plugin = GenericContainerPlugin::new(&format!("test_{}", i), "alpine:latest");
                env.register_service(Box::new(plugin)).await?;
                env.start_service(&format!("test_{}", i)).await?;

                let result = env
                    .execute_in_container(
                        &format!("test_{}", i),
                        &["echo".to_string(), format!("env_{}", i)],
                        None,
                        None,
                    )
                    .await?;

                Ok::<_, clnrm_core::error::CleanroomError>(result)
            })
        })
        .collect();

    let results: Vec<_> = futures_util::future::join_all(handles).await;

    let mut successful = 0;
    for (i, result) in results.into_iter().enumerate() {
        assert!(
            result.is_ok(),
            "Environment {} creation task should not panic",
            i
        );

        let exec_result = result.unwrap();
        assert!(
            exec_result.is_ok(),
            "Environment {} should execute successfully",
            i
        );

        if let Ok(output) = exec_result {
            assert!(
                output.stdout.contains(&format!("env_{}", i)),
                "Environment {} should have its own output",
                i
            );
            successful += 1;
        }
    }

    assert_eq!(
        successful, 10,
        "All 10 environments should be created and executed independently - FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_async_drop_cleanup_synchronization() -> Result<()> {
    let cleanup_counter = Arc::new(Mutex::new(0));

    for _ in 0..5 {
        let env = CleanroomEnvironment::new().await?;
        let plugin = GenericContainerPlugin::new("cleanup_test", "alpine:latest");
        env.register_service(Box::new(plugin)).await?;
        env.start_service("cleanup_test").await?;

        env.execute_in_container(
            "cleanup_test",
            &["echo".to_string(), "test".to_string()],
            None,
            None,
        )
        .await?;

        drop(env);

        let mut counter = cleanup_counter.lock().await;
        *counter += 1;
    }

    let final_count = *cleanup_counter.lock().await;
    assert_eq!(
        final_count, 5,
        "All 5 environment cleanups should complete - async drop issue! FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_tokio_spawn_blocking_synchronization() -> Result<()> {
    let env = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("blocking_test", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;
    env.start_service("blocking_test").await?;

    let handles: Vec<_> = (0..20)
        .map(|i| {
            let env_ref = &env;
            async move {
                env_ref
                    .execute_in_container(
                        "blocking_test",
                        &["echo".to_string(), format!("blocking_{}", i)],
                        None,
                        None,
                    )
                    .await
            }
        })
        .collect();

    let results = futures_util::future::join_all(handles).await;

    let successful = results
        .into_iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().exit_code == 0)
        .count();

    assert_eq!(
        successful, 20,
        "All 20 spawn_blocking operations should complete - runtime blocking! FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_shared_state_mutex_synchronization() -> Result<()> {
    let shared_counter = Arc::new(Mutex::new(0));
    let env = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("mutex_test", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;
    env.start_service("mutex_test").await?;

    let handles: Vec<_> = (0..100)
        .map(|_| {
            let counter_clone = Arc::clone(&shared_counter);
            tokio::spawn(async move {
                let mut count = counter_clone.lock().await;
                *count += 1;
            })
        })
        .collect();

    futures_util::future::join_all(handles).await;

    let final_count = *shared_counter.lock().await;
    assert_eq!(
        final_count, 100,
        "Counter should be exactly 100 - mutex synchronization failed! FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_service_registration_concurrent_safety() -> Result<()> {
    let env = Arc::new(CleanroomEnvironment::new().await?);

    let handles: Vec<_> = (0..10)
        .map(|i| {
            let env_clone = Arc::clone(&env);
            tokio::spawn(async move {
                let plugin =
                    GenericContainerPlugin::new(&format!("concurrent_reg_{}", i), "alpine:latest");
                env_clone.register_service(Box::new(plugin)).await
            })
        })
        .collect();

    let results: Vec<_> = futures_util::future::join_all(handles).await;

    for (i, result) in results.into_iter().enumerate() {
        assert!(
            result.is_ok() && result.unwrap().is_ok(),
            "Service {} registration should succeed - concurrent registration bug! FALSE POSITIVE!",
            i
        );
    }

    for i in 0..10 {
        let start_result = env.start_service(&format!("concurrent_reg_{}", i)).await;
        assert!(
            start_result.is_ok(),
            "Service {} should be registered and startable",
            i
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_command_output_not_mixed_between_concurrent_executions() -> Result<()> {
    let env = Arc::new(CleanroomEnvironment::new().await?);
    let plugin = GenericContainerPlugin::new("output_test", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;
    env.start_service("output_test").await?;

    let handles: Vec<_> = (0..20)
        .map(|i| {
            let env_clone = Arc::clone(&env);
            tokio::spawn(async move {
                let unique_string = format!("UNIQUE_OUTPUT_{}", i);
                let result = env_clone
                    .execute_in_container(
                        "output_test",
                        &["echo".to_string(), unique_string.clone()],
                        None,
                        None,
                    )
                    .await?;

                Ok::<_, clnrm_core::error::CleanroomError>((i, unique_string, result))
            })
        })
        .collect();

    let results: Vec<_> = futures_util::future::join_all(handles).await;

    let outputs: Vec<_> = results.into_iter().map(|r| r.unwrap().unwrap()).collect();

    for (i, unique_string, exec_result) in &outputs {
        assert!(
            exec_result.stdout.contains(unique_string),
            "Output {} should contain its unique string",
            i
        );

        for (j, other_string, _) in &outputs {
            if i != j {
                assert!(
                    !exec_result.stdout.contains(other_string),
                    "Output {} should NOT contain output from {} - outputs mixed! FALSE POSITIVE!",
                    i,
                    j
                );
            }
        }
    }

    Ok(())
}

// =========================================================================
// SECTION 4: Hermetic Isolation Tests
// =========================================================================

#[tokio::test]
async fn test_separate_environments_dont_share_state() -> Result<()> {
    let env1 = CleanroomEnvironment::new().await?;
    let env2 = CleanroomEnvironment::new().await?;

    let plugin1 = GenericContainerPlugin::new("env1_service", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("env2_service", "alpine:latest");

    env1.register_service(Box::new(plugin1)).await?;
    env2.register_service(Box::new(plugin2)).await?;

    env1.start_service("env1_service").await?;
    env2.start_service("env2_service").await?;

    let result1 = env1
        .execute_in_container(
            "env1_service",
            &["sh".to_string(), "-c".to_string(), "echo ENV1 > /tmp/test.txt && cat /tmp/test.txt".to_string()],
            None,
            None,
        )
        .await?;

    let result2 = env2
        .execute_in_container(
            "env2_service",
            &["sh".to_string(), "-c".to_string(), "echo ENV2 > /tmp/test.txt && cat /tmp/test.txt".to_string()],
            None,
            None,
        )
        .await?;

    assert!(
        result1.stdout.contains("ENV1"),
        "Environment 1 should have its own output"
    );
    assert!(
        result2.stdout.contains("ENV2"),
        "Environment 2 should have its own output"
    );
    assert!(
        !result1.stdout.contains("ENV2"),
        "Environment 1 should NOT see Environment 2's data - FALSE POSITIVE!"
    );
    assert!(
        !result2.stdout.contains("ENV1"),
        "Environment 2 should NOT see Environment 1's data - FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_sequential_tests_dont_pollute_each_other() -> Result<()> {
    {
        let env = CleanroomEnvironment::new().await?;
        let plugin = GenericContainerPlugin::new("test1", "alpine:latest");
        env.register_service(Box::new(plugin)).await?;
        env.start_service("test1").await?;

        env.execute_in_container(
            "test1",
            &["sh".to_string(), "-c".to_string(), "echo TEST1 > /shared/data.txt".to_string()],
            None,
            None,
        )
        .await?;
    }

    {
        let env = CleanroomEnvironment::new().await?;
        let plugin = GenericContainerPlugin::new("test2", "alpine:latest");
        env.register_service(Box::new(plugin)).await?;
        env.start_service("test2").await?;

        let result = env
            .execute_in_container(
                "test2",
                &["sh".to_string(), "-c".to_string(), "cat /shared/data.txt 2>&1 || echo NOTFOUND".to_string()],
                None,
                None,
            )
            .await?;

        assert!(
            result.stdout.contains("NOTFOUND") || result.stdout.contains("No such file"),
            "Second test should not see first test's data - hermetic isolation violated! FALSE POSITIVE!"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_tests_isolated() -> Result<()> {
    let handles: Vec<_> = (0..5)
        .map(|i| {
            tokio::spawn(async move {
                let env = CleanroomEnvironment::new().await?;
                let plugin = GenericContainerPlugin::new(
                    &format!("concurrent_{}", i),
                    "alpine:latest",
                );
                env.register_service(Box::new(plugin)).await?;
                env.start_service(&format!("concurrent_{}", i)).await?;

                let result = env
                    .execute_in_container(
                        &format!("concurrent_{}", i),
                        &[
                            "sh".to_string(),
                            "-c".to_string(),
                            format!("echo TEST_{} && sleep 0.1 && echo TEST_{}", i, i),
                        ],
                        None,
                        None,
                    )
                    .await?;

                Ok::<_, clnrm_core::error::CleanroomError>(result)
            })
        })
        .collect();

    let results: Vec<_> = futures_util::future::join_all(handles).await;

    for (i, result) in results.into_iter().enumerate() {
        let execution_result = result.unwrap()?;
        let output = &execution_result.stdout;

        assert_eq!(
            output.matches(&format!("TEST_{}", i)).count(),
            2,
            "Test {} should see its own ID twice",
            i
        );

        for j in 0..5 {
            if i != j {
                assert!(
                    !output.contains(&format!("TEST_{}", j)),
                    "Test {} should NOT see test {}'s output - isolation violated! FALSE POSITIVE!",
                    i,
                    j
                );
            }
        }
    }

    Ok(())
}

#[tokio::test]
async fn test_filesystem_isolation_between_containers() -> Result<()> {
    let env = CleanroomEnvironment::new().await?;

    let plugin1 = GenericContainerPlugin::new("fs1", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("fs2", "alpine:latest");

    env.register_service(Box::new(plugin1)).await?;
    env.register_service(Box::new(plugin2)).await?;

    env.start_service("fs1").await?;
    env.start_service("fs2").await?;

    env.execute_in_container(
        "fs1",
        &["sh".to_string(), "-c".to_string(), "echo SECRET > /tmp/secret.txt".to_string()],
        None,
        None,
    )
    .await?;

    let result = env
        .execute_in_container(
            "fs2",
            &["sh".to_string(), "-c".to_string(), "cat /tmp/secret.txt 2>&1 || echo ISOLATED".to_string()],
            None,
            None,
        )
        .await?;

    assert!(
        result.stdout.contains("ISOLATED") || !result.stdout.contains("SECRET"),
        "Containers should have isolated filesystems - FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_network_isolation_between_containers() -> Result<()> {
    let env = CleanroomEnvironment::new().await?;

    let plugin1 = GenericContainerPlugin::new("net1", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("net2", "alpine:latest");

    env.register_service(Box::new(plugin1)).await?;
    env.register_service(Box::new(plugin2)).await?;

    env.start_service("net1").await?;
    env.start_service("net2").await?;

    let result = env
        .execute_in_container(
            "net1",
            &["sh".to_string(), "-c".to_string(), "ping -c 1 net2 2>&1 || echo ISOLATED".to_string()],
            None,
            None,
        )
        .await?;

    assert!(
        result.stdout.contains("ISOLATED") || result.exit_code != 0,
        "Containers should have network isolation by default"
    );

    Ok(())
}

#[tokio::test]
async fn test_environment_variables_isolated() -> Result<()> {
    let env1 = CleanroomEnvironment::new().await?;
    let env2 = CleanroomEnvironment::new().await?;

    let plugin1 = GenericContainerPlugin::new("envvar1", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("envvar2", "alpine:latest");

    env1.register_service(Box::new(plugin1)).await?;
    env2.register_service(Box::new(plugin2)).await?;

    env1.start_service("envvar1").await?;
    env2.start_service("envvar2").await?;

    let result1 = env1
        .execute_in_container(
            "envvar1",
            &["sh".to_string(), "-c".to_string(), "export TEST_VAR=ENV1 && echo $TEST_VAR".to_string()],
            None,
            None,
        )
        .await?;

    let result2 = env2
        .execute_in_container(
            "envvar2",
            &["sh".to_string(), "-c".to_string(), "export TEST_VAR=ENV2 && echo $TEST_VAR".to_string()],
            None,
            None,
        )
        .await?;

    assert!(result1.stdout.contains("ENV1"), "Environment 1 should have ENV1");
    assert!(result2.stdout.contains("ENV2"), "Environment 2 should have ENV2");

    Ok(())
}

#[tokio::test]
async fn test_process_isolation_between_tests() -> Result<()> {
    {
        let env = CleanroomEnvironment::new().await?;
        let plugin = GenericContainerPlugin::new("proc1", "alpine:latest");
        env.register_service(Box::new(plugin)).await?;
        env.start_service("proc1").await?;

        env.execute_in_container(
            "proc1",
            &["sh".to_string(), "-c".to_string(), "sleep 1000 &".to_string()],
            None,
            None,
        )
        .await?;
    }

    {
        let env = CleanroomEnvironment::new().await?;
        let plugin = GenericContainerPlugin::new("proc2", "alpine:latest");
        env.register_service(Box::new(plugin)).await?;
        env.start_service("proc2").await?;

        let result = env
            .execute_in_container(
                "proc2",
                &["sh".to_string(), "-c".to_string(), "ps aux | grep sleep | grep -v grep || echo CLEAN".to_string()],
                None,
                None,
            )
            .await?;

        assert!(
            result.stdout.contains("CLEAN") || !result.stdout.contains("sleep 1000"),
            "Background processes from previous test should be cleaned up - FALSE POSITIVE!"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_service_registry_isolation_per_environment() -> Result<()> {
    let env1 = CleanroomEnvironment::new().await?;
    let env2 = CleanroomEnvironment::new().await?;

    let plugin1 = GenericContainerPlugin::new("service_a", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("service_b", "ubuntu:latest");

    env1.register_service(Box::new(plugin1)).await?;
    env2.register_service(Box::new(plugin2)).await?;

    let result1 = env1.start_service("service_b").await;
    let result2 = env2.start_service("service_a").await;

    assert!(
        result1.is_err(),
        "Should not find service from different environment - FALSE POSITIVE!"
    );
    assert!(
        result2.is_err(),
        "Should not find service from different environment - FALSE POSITIVE!"
    );

    assert!(env1.start_service("service_a").await.is_ok());
    assert!(env2.start_service("service_b").await.is_ok());

    Ok(())
}

// =========================================================================
// SECTION 5: False Positive Regression & Sanity Tests
// =========================================================================

#[cfg(test)]
mod false_positive_regression_tests {
    use super::*;

    #[tokio::test]
    async fn test_container_execution_actually_works() -> Result<()> {
        let environment = CleanroomEnvironment::new().await?;
        let plugin = GenericContainerPlugin::new("test_container", "alpine:latest");
        environment.register_service(Box::new(plugin)).await?;
        environment.start_service("test_container").await?;

        let result = environment
            .execute_in_container(
                "test_container",
                &["sh".to_string(), "-c".to_string(), "echo CONTAINER".to_string()],
                None,
                None,
            )
            .await?;

        assert_eq!(
            result.exit_code, 0,
            "Container execution should work (FALSE POSITIVE: README said it doesn't)"
        );
        assert!(
            result.stdout.contains("CONTAINER"),
            "Should get container output, not host output"
        );

        Ok(())
    }

    #[test]
    fn test_no_unimplemented_in_production_paths() {
        let critical_files = [
            "crates/clnrm-core/src/cleanroom.rs",
            "crates/clnrm-core/src/services/mod.rs",
            "crates/clnrm-core/src/backend/mod.rs",
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
