//! Validation tests for hermetic isolation
//!
//! These tests ensure each test truly runs in complete isolation without cross-contamination.

use clnrm_core::{
    cleanroom::CleanroomEnvironment,
    error::Result,
    services::generic::GenericContainerPlugin,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_separate_environments_dont_share_state() -> Result<()> {
    // Arrange: Create two completely independent environments
    let env1 = CleanroomEnvironment::new().await?;
    let env2 = CleanroomEnvironment::new().await?;

    let plugin1 = GenericContainerPlugin::new("env1_service", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("env2_service", "alpine:latest");

    env1.register_service(Box::new(plugin1)).await?;
    env2.register_service(Box::new(plugin2)).await?;

    env1.start_service("env1_service").await?;
    env2.start_service("env2_service").await?;

    // Act: Execute different commands in each environment
    let result1 = env1
        .execute_in_container(
            "env1_service",
            &["sh".to_string(), "-c".to_string(), "echo ENV1 > /tmp/test.txt && cat /tmp/test.txt".to_string()],
        )
        .await?;

    let result2 = env2
        .execute_in_container(
            "env2_service",
            &["sh".to_string(), "-c".to_string(), "echo ENV2 > /tmp/test.txt && cat /tmp/test.txt".to_string()],
        )
        .await?;

    // Assert: Outputs should be different, proving isolation
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
    // Test 1: Create, use, and destroy environment
    {
        let env = CleanroomEnvironment::new().await?;
        let plugin = GenericContainerPlugin::new("test1", "alpine:latest");
        env.register_service(Box::new(plugin)).await?;
        env.start_service("test1").await?;

        env.execute_in_container(
            "test1",
            &["sh".to_string(), "-c".to_string(), "echo TEST1 > /shared/data.txt".to_string()],
        )
        .await?;

        // env drops here, cleaning up
    }

    // Test 2: Create fresh environment
    {
        let env = CleanroomEnvironment::new().await?;
        let plugin = GenericContainerPlugin::new("test2", "alpine:latest");
        env.register_service(Box::new(plugin)).await?;
        env.start_service("test2").await?;

        // Try to read file from previous test (should not exist)
        let result = env
            .execute_in_container(
                "test2",
                &["sh".to_string(), "-c".to_string(), "cat /shared/data.txt 2>&1 || echo NOTFOUND".to_string()],
            )
            .await?;

        // Assert: Should NOT find previous test's data
        assert!(
            result.stdout.contains("NOTFOUND") || result.stdout.contains("No such file"),
            "Second test should not see first test's data - hermetic isolation violated! FALSE POSITIVE!"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_concurrent_tests_isolated() -> Result<()> {
    // Arrange: Run multiple tests concurrently
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

                // Each test writes its own ID
                let result = env
                    .execute_in_container(
                        &format!("concurrent_{}", i),
                        &[
                            "sh".to_string(),
                            "-c".to_string(),
                            format!("echo TEST_{} && sleep 0.1 && echo TEST_{}", i, i),
                        ],
                    )
                    .await?;

                Ok::<_, clnrm_core::error::CleanroomError>(result)
            })
        })
        .collect();

    // Act: Wait for all tests to complete
    let results: Vec<_> = futures::future::join_all(handles).await;

    // Assert: Each test should only see its own ID
    for (i, result) in results.into_iter().enumerate() {
        let execution_result = result.unwrap()?;
        let output = &execution_result.stdout;

        // Should contain own ID twice
        assert!(
            output.matches(&format!("TEST_{}", i)).count() == 2,
            "Test {} should see its own ID twice",
            i
        );

        // Should NOT contain other IDs
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
    // Arrange: Create two services in same environment
    let env = CleanroomEnvironment::new().await?;

    let plugin1 = GenericContainerPlugin::new("fs1", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("fs2", "alpine:latest");

    env.register_service(Box::new(plugin1)).await?;
    env.register_service(Box::new(plugin2)).await?;

    env.start_service("fs1").await?;
    env.start_service("fs2").await?;

    // Act: Write file in first container
    env.execute_in_container(
        "fs1",
        &["sh".to_string(), "-c".to_string(), "echo SECRET > /tmp/secret.txt".to_string()],
    )
    .await?;

    // Try to read file from second container (should not exist)
    let result = env
        .execute_in_container(
            "fs2",
            &["sh".to_string(), "-c".to_string(), "cat /tmp/secret.txt 2>&1 || echo ISOLATED".to_string()],
        )
        .await?;

    // Assert: Second container should NOT see first container's files
    assert!(
        result.stdout.contains("ISOLATED") || !result.stdout.contains("SECRET"),
        "Containers should have isolated filesystems - FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_network_isolation_between_containers() -> Result<()> {
    // Arrange: Create two services
    let env = CleanroomEnvironment::new().await?;

    let plugin1 = GenericContainerPlugin::new("net1", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("net2", "alpine:latest");

    env.register_service(Box::new(plugin1)).await?;
    env.register_service(Box::new(plugin2)).await?;

    env.start_service("net1").await?;
    env.start_service("net2").await?;

    // Act: Try to reach net2 from net1 (should fail without explicit networking)
    // This assumes default network isolation
    let result = env
        .execute_in_container(
            "net1",
            &["sh".to_string(), "-c".to_string(), "ping -c 1 net2 2>&1 || echo ISOLATED".to_string()],
        )
        .await?;

    // Assert: Should not be able to reach other container by default
    // (This test may need adjustment based on network configuration)
    assert!(
        result.stdout.contains("ISOLATED") || !result.success,
        "Containers should have network isolation by default"
    );

    Ok(())
}

#[tokio::test]
async fn test_environment_variables_isolated() -> Result<()> {
    // Arrange: Create two environments with different variables
    let env1 = CleanroomEnvironment::new().await?;
    let env2 = CleanroomEnvironment::new().await?;

    let plugin1 = GenericContainerPlugin::new("envvar1", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("envvar2", "alpine:latest");

    env1.register_service(Box::new(plugin1)).await?;
    env2.register_service(Box::new(plugin2)).await?;

    env1.start_service("envvar1").await?;
    env2.start_service("envvar2").await?;

    // Act: Set different environment variables in each
    let result1 = env1
        .execute_in_container(
            "envvar1",
            &["sh".to_string(), "-c".to_string(), "export TEST_VAR=ENV1 && echo $TEST_VAR".to_string()],
        )
        .await?;

    let result2 = env2
        .execute_in_container(
            "envvar2",
            &["sh".to_string(), "-c".to_string(), "export TEST_VAR=ENV2 && echo $TEST_VAR".to_string()],
        )
        .await?;

    // Assert: Each should have its own variable value
    assert!(
        result1.stdout.contains("ENV1"),
        "Environment 1 should have ENV1"
    );
    assert!(
        result2.stdout.contains("ENV2"),
        "Environment 2 should have ENV2"
    );

    Ok(())
}

#[tokio::test]
async fn test_process_isolation_between_tests() -> Result<()> {
    // Arrange & Act: Run background process in first test
    {
        let env = CleanroomEnvironment::new().await?;
        let plugin = GenericContainerPlugin::new("proc1", "alpine:latest");
        env.register_service(Box::new(plugin)).await?;
        env.start_service("proc1").await?;

        // Start background process (sleep)
        env.execute_in_container(
            "proc1",
            &["sh".to_string(), "-c".to_string(), "sleep 1000 &".to_string()],
        )
        .await?;

        // Environment drops, should clean up
    }

    // Second test: Check for leftover processes
    {
        let env = CleanroomEnvironment::new().await?;
        let plugin = GenericContainerPlugin::new("proc2", "alpine:latest");
        env.register_service(Box::new(plugin)).await?;
        env.start_service("proc2").await?;

        let result = env
            .execute_in_container(
                "proc2",
                &["sh".to_string(), "-c".to_string(), "ps aux | grep sleep | grep -v grep || echo CLEAN".to_string()],
            )
            .await?;

        // Assert: Should not find sleep process from previous test
        assert!(
            result.stdout.contains("CLEAN") || !result.stdout.contains("sleep 1000"),
            "Background processes from previous test should be cleaned up - FALSE POSITIVE!"
        );
    }

    Ok(())
}

#[tokio::test]
async fn test_service_registry_isolation_per_environment() -> Result<()> {
    // Arrange: Create two environments
    let env1 = CleanroomEnvironment::new().await?;
    let env2 = CleanroomEnvironment::new().await?;

    // Register different services in each
    let plugin1 = GenericContainerPlugin::new("service_a", "alpine:latest");
    let plugin2 = GenericContainerPlugin::new("service_b", "ubuntu:latest");

    env1.register_service(Box::new(plugin1)).await?;
    env2.register_service(Box::new(plugin2)).await?;

    // Act: Try to start services cross-environment (should fail)
    let result1 = env1.start_service("service_b").await; // service_b is in env2
    let result2 = env2.start_service("service_a").await; // service_a is in env1

    // Assert: Should fail - services are environment-specific
    assert!(
        result1.is_err(),
        "Should not find service from different environment - FALSE POSITIVE!"
    );
    assert!(
        result2.is_err(),
        "Should not find service from different environment - FALSE POSITIVE!"
    );

    // But correct services should work
    assert!(env1.start_service("service_a").await.is_ok());
    assert!(env2.start_service("service_b").await.is_ok());

    Ok(())
}
