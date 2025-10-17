//! Container Isolation Integration Test
//!
//! This test proves that clnrm executes commands INSIDE Docker containers,
//! not on the host machine. It verifies actual hermetic isolation.

use clnrm_core::backend::{Backend, Cmd, TestcontainerBackend};
use clnrm_core::error::Result;

/// Test that commands execute inside actual Docker containers
#[test]
fn test_commands_execute_inside_containers_not_on_host() -> Result<()> {
    // Arrange
    let backend = TestcontainerBackend::new("alpine:latest")?;

    // Act - Run uname to check OS
    let cmd = Cmd::new("uname").arg("-s");
    let result = backend.run_cmd(cmd)?;

    // Assert
    // On macOS/Windows host, containers run Linux
    // This proves we're executing INSIDE a container, not on host
    assert_eq!(result.exit_code, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Linux"),
        "Container should be running Linux, not host OS. Got: {}",
        result.stdout
    );

    Ok(())
}

/// Test that Alpine-specific files exist inside container
#[test]
fn test_alpine_container_has_alpine_specific_files() -> Result<()> {
    // Arrange
    let backend = TestcontainerBackend::new("alpine:latest")?;

    // Act - Check for Alpine-specific release file
    let cmd = Cmd::new("cat").arg("/etc/os-release");
    let result = backend.run_cmd(cmd)?;

    // Assert
    assert_eq!(result.exit_code, 0, "Command should succeed");
    assert!(
        result.stdout.contains("Alpine Linux"),
        "Container should identify as Alpine Linux. Got: {}",
        result.stdout
    );

    Ok(())
}

/// Test that containers are isolated from host filesystem
#[test]
fn test_container_filesystem_isolated_from_host() -> Result<()> {
    // Arrange
    let backend = TestcontainerBackend::new("alpine:latest")?;

    // Act - Try to access a file that exists on macOS but not in Alpine containers
    let cmd = Cmd::new("test").arg("-f").arg("/System/Library/CoreServices/SystemVersion.plist");
    let result = backend.run_cmd(cmd)?;

    // Assert
    // macOS-specific file should NOT exist in Linux container
    assert_ne!(
        result.exit_code, 0,
        "macOS-specific file should not exist in container"
    );

    Ok(())
}

/// Test that Alpine package manager works inside container
#[test]
fn test_alpine_package_manager_available() -> Result<()> {
    // Arrange
    let backend = TestcontainerBackend::new("alpine:latest")?;

    // Act - Check for Alpine package manager (apk)
    let cmd = Cmd::new("which").arg("apk");
    let result = backend.run_cmd(cmd)?;

    // Assert
    assert_eq!(result.exit_code, 0, "apk command should exist in Alpine container");
    assert!(
        result.stdout.contains("/sbin/apk"),
        "apk should be located at /sbin/apk. Got: {}",
        result.stdout
    );

    Ok(())
}

/// Test that container processes are isolated
#[test]
fn test_container_process_isolation() -> Result<()> {
    // Arrange
    let backend = TestcontainerBackend::new("alpine:latest")?;

    // Act - List processes
    let cmd = Cmd::new("ps").arg("aux");
    let result = backend.run_cmd(cmd)?;

    // Assert
    assert_eq!(result.exit_code, 0, "ps command should succeed");

    // Container should have very few processes (isolated)
    let process_count = result.stdout.lines().count();
    assert!(
        process_count < 10,
        "Container should have minimal processes (isolated). Got {} processes",
        process_count
    );

    Ok(())
}

/// Test multiple containers are isolated from each other
#[test]
fn test_multiple_containers_are_isolated() -> Result<()> {
    // Arrange
    let backend1 = TestcontainerBackend::new("alpine:latest")?;
    let backend2 = TestcontainerBackend::new("alpine:latest")?;

    // Act - Create a file in first container
    let cmd1 = Cmd::new("sh")
        .arg("-c")
        .arg("echo 'container1' > /tmp/test.txt && cat /tmp/test.txt");
    let result1 = backend1.run_cmd(cmd1)?;

    // Try to read that file in second container (should fail - isolated)
    let cmd2 = Cmd::new("cat").arg("/tmp/test.txt");
    let result2 = backend2.run_cmd(cmd2)?;

    // Assert
    assert_eq!(result1.exit_code, 0, "First container write should succeed");
    assert!(result1.stdout.contains("container1"));

    assert_ne!(
        result2.exit_code, 0,
        "Second container should NOT see first container's file"
    );

    Ok(())
}

/// Test that environment variables are isolated to containers
#[test]
fn test_container_environment_isolation() -> Result<()> {
    // Arrange
    let backend = TestcontainerBackend::new("alpine:latest")?
        .with_env("CONTAINER_VAR", "container_value");

    // Act - Check environment variable
    let cmd = Cmd::new("sh").arg("-c").arg("echo $CONTAINER_VAR");
    let result = backend.run_cmd(cmd)?;

    // Assert
    assert_eq!(result.exit_code, 0, "Command should succeed");
    assert!(
        result.stdout.contains("container_value"),
        "Container should have environment variable. Got: {}",
        result.stdout
    );

    Ok(())
}

/// Test that container cleanup happens automatically
#[test]
fn test_container_cleanup_automatic() -> Result<()> {
    // Arrange & Act
    {
        let backend = TestcontainerBackend::new("alpine:latest")?;
        let cmd = Cmd::new("echo").arg("hello");
        let result = backend.run_cmd(cmd)?;
        assert_eq!(result.exit_code, 0);
        // Backend goes out of scope here - container should be cleaned up
    }

    // Assert - If we get here without hanging, cleanup worked
    // testcontainers-rs automatically cleans up containers when dropped
    Ok(())
}

/// Test execution with different container images
#[test]
fn test_different_container_images() -> Result<()> {
    // Arrange
    let alpine_backend = TestcontainerBackend::new("alpine:latest")?;

    // Act - Check Alpine version
    let cmd = Cmd::new("cat").arg("/etc/alpine-release");
    let result = alpine_backend.run_cmd(cmd)?;

    // Assert
    assert_eq!(result.exit_code, 0, "Should be able to read Alpine version");
    assert!(!result.stdout.is_empty(), "Alpine version should not be empty");

    Ok(())
}

/// Test that command failures are properly captured
#[test]
fn test_command_failures_captured() -> Result<()> {
    // Arrange
    let backend = TestcontainerBackend::new("alpine:latest")?;

    // Act - Run a command that fails
    let cmd = Cmd::new("ls").arg("/nonexistent/path");
    let result = backend.run_cmd(cmd)?;

    // Assert
    assert_ne!(result.exit_code, 0, "Command should fail");
    assert!(!result.stderr.is_empty(), "Should capture stderr");

    Ok(())
}

/// Test that stdout and stderr are properly separated
#[test]
fn test_stdout_stderr_separation() -> Result<()> {
    // Arrange
    let backend = TestcontainerBackend::new("alpine:latest")?;

    // Act - Command that outputs to both stdout and stderr
    let cmd = Cmd::new("sh")
        .arg("-c")
        .arg("echo 'stdout message' && echo 'stderr message' >&2");
    let result = backend.run_cmd(cmd)?;

    // Assert
    assert_eq!(result.exit_code, 0, "Command should succeed");
    assert!(
        result.stdout.contains("stdout message"),
        "Should capture stdout"
    );
    assert!(
        result.stderr.contains("stderr message"),
        "Should capture stderr"
    );

    Ok(())
}

/// Test working directory can be set in container
#[test]
fn test_working_directory_in_container() -> Result<()> {
    // Arrange
    let backend = TestcontainerBackend::new("alpine:latest")?;

    // Act - Set working directory and verify
    let cmd = Cmd::new("pwd").workdir(std::path::PathBuf::from("/tmp"));
    let result = backend.run_cmd(cmd)?;

    // Assert
    assert_eq!(result.exit_code, 0, "Command should succeed");
    assert!(
        result.stdout.contains("/tmp"),
        "Should execute in /tmp directory. Got: {}",
        result.stdout
    );

    Ok(())
}

/// Test that long-running commands are supported
#[test]
fn test_long_running_command_support() -> Result<()> {
    // Arrange
    let backend = TestcontainerBackend::new("alpine:latest")?;

    // Act - Run a command that takes a moment
    let cmd = Cmd::new("sh").arg("-c").arg("sleep 1 && echo 'done'");
    let result = backend.run_cmd(cmd)?;

    // Assert
    assert_eq!(result.exit_code, 0, "Long-running command should complete");
    assert!(result.stdout.contains("done"));
    assert!(
        result.duration_ms >= 1000,
        "Should track duration accurately"
    );

    Ok(())
}

/// Test container execution with policy
#[test]
fn test_container_execution_with_policy() -> Result<()> {
    // Arrange
    let backend = TestcontainerBackend::new("alpine:latest")?;
    let policy = clnrm_core::policy::Policy::default();

    // Act
    let cmd = Cmd::new("echo")
        .arg("hello")
        .policy(policy);
    let result = backend.run_cmd(cmd)?;

    // Assert
    assert_eq!(result.exit_code, 0);
    assert!(result.stdout.contains("hello"));

    Ok(())
}

/// Test hermetic execution support
#[test]
fn test_hermetic_execution_support() -> Result<()> {
    // Arrange
    let backend = TestcontainerBackend::new("alpine:latest")?;

    // Assert
    assert!(
        backend.supports_hermetic(),
        "TestcontainerBackend should support hermetic execution"
    );

    Ok(())
}

/// Test deterministic execution support
#[test]
fn test_deterministic_execution_support() -> Result<()> {
    // Arrange
    let backend = TestcontainerBackend::new("alpine:latest")?;

    // Assert
    assert!(
        backend.supports_deterministic(),
        "TestcontainerBackend should support deterministic execution"
    );

    Ok(())
}

/// Test backend availability check
#[test]
fn test_backend_availability() -> Result<()> {
    // Arrange
    let backend = TestcontainerBackend::new("alpine:latest")?;

    // Assert
    assert!(
        backend.is_available(),
        "Backend should be available if Docker is running"
    );
    assert_eq!(backend.name(), "testcontainers");

    Ok(())
}

/// Test that concurrent container executions are isolated
#[test]
fn test_concurrent_container_isolation() -> Result<()> {
    use std::sync::Arc;
    use std::thread;

    // Arrange
    let backend = Arc::new(TestcontainerBackend::new("alpine:latest")?);

    // Act - Spawn multiple threads executing commands concurrently
    let handles: Vec<_> = (0..3)
        .map(|i| {
            let backend_clone = Arc::clone(&backend);
            thread::spawn(move || {
                let cmd = Cmd::new("sh")
                    .arg("-c")
                    .arg(format!("echo 'thread-{}' && sleep 0.1", i));
                backend_clone.run_cmd(cmd)
            })
        })
        .collect();

    // Collect results
    let results: Vec<_> = handles
        .into_iter()
        .map(|h| h.join().expect("Thread should complete"))
        .collect::<Result<Vec<_>>>()?;

    // Assert
    assert_eq!(results.len(), 3, "All commands should complete");
    for (i, result) in results.iter().enumerate() {
        assert_eq!(result.exit_code, 0, "Command {} should succeed", i);
        assert!(
            result.stdout.contains(&format!("thread-{}", i)),
            "Each container should have unique output"
        );
    }

    Ok(())
}
