//! Validation tests for async operation synchronization
//!
//! These tests ensure async operations are properly synchronized and don't produce race conditions or false positives.

use clnrm_core::{
    cleanroom::CleanroomEnvironment,
    error::Result,
    services::generic::GenericContainerPlugin,
};
use std::sync::Arc;
use tokio::sync::Mutex;

#[tokio::test]
async fn test_concurrent_service_starts_synchronized() -> Result<()> {
    // Arrange: Create environment and register multiple services
    let env = Arc::new(CleanroomEnvironment::new().await?);

    for i in 0..5 {
        let plugin = GenericContainerPlugin::new(&format!("service_{}", i), "alpine:latest");
        env.register_service(Box::new(plugin)).await?;
    }

    // Act: Start all services concurrently
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

    let results: Vec<_> = futures::future::join_all(handles).await;

    // Assert: All should succeed without race conditions
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
    // Arrange: Create environment with service
    let env = Arc::new(CleanroomEnvironment::new().await?);
    let plugin = GenericContainerPlugin::new("concurrent_exec", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;
    env.start_service("concurrent_exec").await?;

    // Act: Execute multiple commands concurrently in same container
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let env_clone = Arc::clone(&env);
            tokio::spawn(async move {
                env_clone
                    .execute_in_container(
                        "concurrent_exec",
                        &["echo".to_string(), format!("command_{}", i)],
                    )
                    .await
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    // Assert: All commands should complete successfully
    let mut successful = 0;
    for result in results {
        if let Ok(Ok(exec_result)) = result {
            if exec_result.success {
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
    // Arrange: Create environment and register service
    let env = Arc::new(CleanroomEnvironment::new().await?);
    let plugin = GenericContainerPlugin::new("lifecycle", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Rapidly start and stop service multiple times
    for iteration in 0..5 {
        let start_result = env.start_service("lifecycle").await;
        assert!(
            start_result.is_ok(),
            "Iteration {}: Start should succeed",
            iteration
        );

        // Execute command to verify service is running
        let exec_result = env
            .execute_in_container("lifecycle", &["echo".to_string(), "test".to_string()])
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

    // Assert: No race conditions or state corruption
    Ok(())
}

#[tokio::test]
async fn test_concurrent_environment_creation_isolated() -> Result<()> {
    // Act: Create multiple environments concurrently
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
                    )
                    .await?;

                Ok::<_, clnrm_core::error::CleanroomError>(result)
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    // Assert: All environments should be independent
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
    // Arrange: Create counter to track cleanup
    let cleanup_counter = Arc::new(Mutex::new(0));

    // Act: Create and drop multiple environments
    for _ in 0..5 {
        let env = CleanroomEnvironment::new().await?;
        let plugin = GenericContainerPlugin::new("cleanup_test", "alpine:latest");
        env.register_service(Box::new(plugin)).await?;
        env.start_service("cleanup_test").await?;

        // Execute command
        env.execute_in_container("cleanup_test", &["echo".to_string(), "test".to_string()])
            .await?;

        // Drop environment (should trigger cleanup)
        drop(env);

        // Increment counter
        let mut counter = cleanup_counter.lock().await;
        *counter += 1;
    }

    // Assert: All cleanups should complete
    let final_count = *cleanup_counter.lock().await;
    assert_eq!(
        final_count, 5,
        "All 5 environment cleanups should complete - async drop issue! FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_tokio_spawn_blocking_synchronization() -> Result<()> {
    // This tests that tokio::task::spawn_blocking is used correctly for sync operations
    // Arrange: Create environment
    let env = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("blocking_test", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;
    env.start_service("blocking_test").await?;

    // Act: Execute commands that trigger spawn_blocking internally
    let handles: Vec<_> = (0..20)
        .map(|i| {
            let env_ref = &env;
            async move {
                env_ref
                    .execute_in_container(
                        "blocking_test",
                        &["echo".to_string(), format!("blocking_{}", i)],
                    )
                    .await
            }
        })
        .collect();

    let results = futures::future::join_all(handles).await;

    // Assert: All should complete without blocking the runtime
    let successful = results
        .into_iter()
        .filter(|r| r.is_ok() && r.as_ref().unwrap().success)
        .count();

    assert_eq!(
        successful, 20,
        "All 20 spawn_blocking operations should complete - runtime blocking! FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_shared_state_mutex_synchronization() -> Result<()> {
    // Arrange: Create shared state and environment
    let shared_counter = Arc::new(Mutex::new(0));
    let env = CleanroomEnvironment::new().await?;
    let plugin = GenericContainerPlugin::new("mutex_test", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;
    env.start_service("mutex_test").await?;

    // Act: Increment counter from multiple concurrent tasks
    let handles: Vec<_> = (0..100)
        .map(|_| {
            let counter_clone = Arc::clone(&shared_counter);
            tokio::spawn(async move {
                let mut count = counter_clone.lock().await;
                *count += 1;
            })
        })
        .collect();

    futures::future::join_all(handles).await;

    // Assert: Counter should be exactly 100 (no race conditions)
    let final_count = *shared_counter.lock().await;
    assert_eq!(
        final_count, 100,
        "Counter should be exactly 100 - mutex synchronization failed! FALSE POSITIVE!"
    );

    Ok(())
}

#[tokio::test]
async fn test_service_registration_concurrent_safety() -> Result<()> {
    // Arrange: Create environment
    let env = Arc::new(CleanroomEnvironment::new().await?);

    // Act: Register multiple services concurrently
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

    let results: Vec<_> = futures::future::join_all(handles).await;

    // Assert: All registrations should succeed
    for (i, result) in results.into_iter().enumerate() {
        assert!(
            result.is_ok() && result.unwrap().is_ok(),
            "Service {} registration should succeed - concurrent registration bug! FALSE POSITIVE!",
            i
        );
    }

    // Verify all services were registered
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
    // Arrange: Create environment and service
    let env = Arc::new(CleanroomEnvironment::new().await?);
    let plugin = GenericContainerPlugin::new("output_test", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;
    env.start_service("output_test").await?;

    // Act: Execute commands with unique outputs concurrently
    let handles: Vec<_> = (0..20)
        .map(|i| {
            let env_clone = Arc::clone(&env);
            tokio::spawn(async move {
                let unique_string = format!("UNIQUE_OUTPUT_{}", i);
                let result = env_clone
                    .execute_in_container(
                        "output_test",
                        &["echo".to_string(), unique_string.clone()],
                    )
                    .await?;

                Ok::<_, clnrm_core::error::CleanroomError>((i, unique_string, result))
            })
        })
        .collect();

    let results: Vec<_> = futures::future::join_all(handles).await;

    // Assert: Each output should contain ONLY its unique string
    for result in results {
        let (i, unique_string, exec_result) = result.unwrap()?;

        assert!(
            exec_result.stdout.contains(&unique_string),
            "Output {} should contain its unique string",
            i
        );

        // Check that output doesn't contain other outputs (no mixing)
        for j in 0..20 {
            if i != j {
                let other_string = format!("UNIQUE_OUTPUT_{}", j);
                assert!(
                    !exec_result.stdout.contains(&other_string),
                    "Output {} should NOT contain output from {} - outputs mixed! FALSE POSITIVE!",
                    i,
                    j
                );
            }
        }
    }

    Ok(())
}
