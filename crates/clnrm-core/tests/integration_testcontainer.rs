//! Integration tests for testcontainer backend
//!
//! These tests verify that the testcontainer backend actually works end-to-end
//! by running real commands in containers.
//!
//! Note: These tests require Docker to be running. If Docker is not available,
//! the tests will be skipped automatically.

use clnrm_core::backend::{Backend, Cmd, TestcontainerBackend};
use clnrm_core::policy::{Policy, SecurityLevel};

/// Helper function to check if Docker is available
fn docker_available() -> bool {
    use std::process::Command;
    
    // Try a quick docker ps command
    let output = Command::new("docker")
        .args(&["ps", "--format", "table {{.Names}}\t{{.Status}}"])
        .output();
    
    match output {
        Ok(result) => result.status.success(),
        Err(_) => false,
    }
}

/// Macro to skip test if Docker is not available
macro_rules! skip_if_no_docker {
    () => {
        if !docker_available() {
            println!("Docker not available, skipping test. To run Docker tests:");
            println!("  1. Start Docker Desktop or Docker daemon");
            println!("  2. Run: docker ps (to verify Docker is working)");
            println!("  3. Re-run the tests");
            return;
        }
    };
}

/// Comprehensive testcontainer integration test (80/20 rule)
#[test]
fn test_testcontainer_comprehensive() {
    skip_if_no_docker!();

    // Test 1: Basic functionality and backend capabilities
    let alpine_backend = TestcontainerBackend::new("alpine:latest")
        .expect("Failed to create testcontainer backend - check Docker is running and try: docker pull alpine:latest");

    // Verify backend capabilities
    assert!(alpine_backend.is_available(), "Testcontainer backend should be available");
    assert!(alpine_backend.supports_hermetic(), "Testcontainer should support hermetic execution");
    assert!(alpine_backend.supports_deterministic(), "Testcontainer should support deterministic execution");
    assert_eq!(alpine_backend.name(), "testcontainers", "Backend name should be correct");

    // Test 2: Basic command execution
    let basic_cmd = Cmd::new("echo")
        .arg("Hello, World!")
        .policy(Policy::default());

    let basic_result = alpine_backend.run_cmd(basic_cmd)
        .expect("Command execution should succeed - check container logs if this fails");

    assert_eq!(basic_result.exit_code, 0, "Command should succeed");
    assert!(basic_result.stdout.contains("Hello, World!"), "Output should contain expected text");
    assert!(basic_result.duration_ms > 0, "Duration should be recorded");
    assert_eq!(basic_result.backend, "testcontainers", "Backend name should be correct");
    assert!(!basic_result.concurrent, "Single command should not be concurrent");

    // Test 3: Environment variables
    let env_cmd = Cmd::new("env")
        .env("TEST_VAR", "test_value")
        .env("ANOTHER_VAR", "another_value")
        .policy(Policy::default());

    let env_result = alpine_backend.run_cmd(env_cmd)
        .expect("Command execution should succeed");

    assert_eq!(env_result.exit_code, 0, "Command should succeed");
    assert!(env_result.stdout.contains("TEST_VAR=test_value"), "Should contain first env var");
    assert!(env_result.stdout.contains("ANOTHER_VAR=another_value"), "Should contain second env var");

    // Test 4: Security policies
    let high_security_policy = Policy::with_security_level(SecurityLevel::High);
    let security_cmd = Cmd::new("echo")
        .arg("High security test")
        .policy(high_security_policy);

    let security_result = alpine_backend.run_cmd(security_cmd)
        .expect("Command execution should succeed even with high security");

    assert_eq!(security_result.exit_code, 0, "Command should succeed");
    assert!(security_result.stdout.contains("High security test"), "Output should contain expected text");

    // Test 5: Different container images
    let ubuntu_backend = TestcontainerBackend::new("ubuntu:20.04")
        .expect("Failed to create Ubuntu backend");

    let ubuntu_cmd = Cmd::new("cat")
        .arg("/etc/os-release")
        .policy(Policy::default());

    let ubuntu_result = ubuntu_backend.run_cmd(ubuntu_cmd)
        .expect("Ubuntu command should succeed");

    assert_eq!(ubuntu_result.exit_code, 0, "Ubuntu command should succeed");
    assert!(ubuntu_result.stdout.contains("Ubuntu"), "Should show Ubuntu OS info");

    // Test 6: Command failure handling
    let failure_cmd = Cmd::new("false")  // false command always returns non-zero exit code
        .policy(Policy::default());

    let failure_result = alpine_backend.run_cmd(failure_cmd)
        .expect("Command execution should complete even if command fails");

    assert_ne!(failure_result.exit_code, 0, "Command should fail");
    assert!(failure_result.duration_ms > 0, "Duration should still be recorded");

    // This comprehensive test validates the 80% most important aspects:
    // - Backend capabilities and availability
    // - Basic command execution with proper result validation
    // - Environment variable handling
    // - Security policy integration
    // - Multi-image support
    // - Error handling
}
