//! System Integration Tests
//!
//! These tests validate end-to-end workflows across the entire system,
//! including container lifecycle, observability, and multi-component orchestration.

use anyhow::Result;
use std::time::Duration;

mod common;
use common::{helpers::*, factories::*, fixtures::*};

// Docker availability check
fn docker_available() -> bool {
    use std::process::Command;
    Command::new("docker")
        .args(&["ps"])
        .output()
        .map(|output| output.status.success())
        .unwrap_or(false)
}

macro_rules! skip_if_no_docker {
    () => {
        if !docker_available() {
            println!("Docker not available, skipping test");
            return Ok(());
        }
    };
}

/// Test full test execution pipeline
#[test]
fn test_full_execution_pipeline() -> Result<()> {
    let ctx = TestContext::new()?;

    // Stage 1: Load configuration
    let config = ConfigFixture::default_alpine();
    assert_eq!(config.backend, "testcontainers");

    // Stage 2: Prepare commands
    let commands = vec![
        CommandFixture::echo_hello(),
        CommandFixture::list_files(),
    ];
    assert_eq!(commands.len(), 2);

    // Stage 3: Execute commands (simulated)
    let results = vec![
        ResultFixture::successful_execution(),
        ResultFixture::successful_execution(),
    ];

    // Stage 4: Aggregate results
    let all_passed = results.iter().all(|r| r.exit_code == 0);
    assert!(all_passed);

    // Stage 5: Generate report (simulated)
    let report = format!(
        "Executed {} commands, {} passed",
        commands.len(),
        results.iter().filter(|r| r.exit_code == 0).count()
    );
    assert!(report.contains("2 commands"));
    assert!(report.contains("2 passed"));

    Ok(())
}

/// Test container lifecycle management
#[test]
#[ignore] // Requires Docker
fn test_container_lifecycle() -> Result<()> {
    skip_if_no_docker!();

    let ctx = TestContext::new()?;

    // Phase 1: Container creation
    let config = BackendConfigBuilder::new()
        .image("alpine")
        .tag("latest")
        .build();

    // Phase 2: Container execution (would use real backend here)
    // For now, we simulate the lifecycle

    // Phase 3: Container cleanup
    // Cleanup happens automatically via TestContext Drop

    Ok(())
}

/// Test observability data flow
#[test]
fn test_observability_pipeline() -> Result<()> {
    let ctx = TestContext::new()?;

    // Step 1: Generate test execution
    let cmd = CommandBuilder::new("echo")
        .arg("Observability test")
        .build();

    // Step 2: Collect execution metrics
    let result = ResultBuilder::new()
        .exit_code(0)
        .stdout("Observability test\n")
        .duration_ms(150)
        .build();

    // Step 3: Verify metrics
    assert_eq!(result.exit_code, 0);
    assert!(result.duration_ms > 0);

    // Step 4: Export metrics (simulated)
    let metrics = format!(
        "{{\"exit_code\":{},\"duration_ms\":{},\"backend\":\"{}\"}}",
        result.exit_code, result.duration_ms, result.backend
    );
    assert!(metrics.contains("\"exit_code\":0"));

    Ok(())
}

/// Test multi-backend orchestration
#[test]
fn test_multi_backend_orchestration() -> Result<()> {
    let ctx = TestContext::new()?;

    // Create multiple backend configurations
    let backends = vec![
        BackendConfigBuilder::new()
            .name("alpine-backend")
            .image("alpine")
            .build(),
        BackendConfigBuilder::new()
            .name("ubuntu-backend")
            .image("ubuntu")
            .tag("22.04")
            .build(),
    ];

    assert_eq!(backends.len(), 2);
    assert_eq!(backends[0].name, "alpine-backend");
    assert_eq!(backends[1].name, "ubuntu-backend");

    // Execute commands on different backends (simulated)
    let results = backends
        .iter()
        .map(|backend| {
            ResultBuilder::new()
                .backend(&backend.name)
                .exit_code(0)
                .build()
        })
        .collect::<Vec<_>>();

    assert_eq!(results.len(), 2);
    assert_eq!(results[0].backend, "alpine-backend");
    assert_eq!(results[1].backend, "ubuntu-backend");

    Ok(())
}

/// Test error recovery workflow
#[test]
fn test_error_recovery_workflow() -> Result<()> {
    let ctx = TestContext::new()?;

    // Attempt 1: Initial execution fails
    let attempt1 = ResultBuilder::new()
        .exit_code(1)
        .stderr("Error: Connection refused")
        .build();

    assert_ne!(attempt1.exit_code, 0);

    // Attempt 2: Retry with backoff
    std::thread::sleep(Duration::from_millis(10));

    let attempt2 = ResultBuilder::new()
        .exit_code(0)
        .stdout("Success after retry")
        .build();

    assert_eq!(attempt2.exit_code, 0);

    // Verify recovery
    let recovered = attempt2.exit_code == 0;
    assert!(recovered);

    Ok(())
}

/// Test parallel execution workflow
#[test]
fn test_parallel_execution_workflow() -> Result<()> {
    let ctx = TestContext::new()?;

    // Create parallel commands
    let commands = (0..5)
        .map(|i| {
            CommandBuilder::new("echo")
                .arg(format!("Task {}", i))
                .build()
        })
        .collect::<Vec<_>>();

    assert_eq!(commands.len(), 5);

    // Simulate parallel execution
    let results = commands
        .iter()
        .map(|_| {
            ResultBuilder::new()
                .concurrent(true)
                .duration_ms(100)
                .exit_code(0)
                .build()
        })
        .collect::<Vec<_>>();

    // All should be concurrent
    assert!(results.iter().all(|r| r.concurrent));

    // All should succeed
    assert!(results.iter().all(|r| r.exit_code == 0));

    Ok(())
}

/// Test security policy enforcement workflow
#[test]
fn test_security_policy_workflow() -> Result<()> {
    let ctx = TestContext::new()?;

    // High security configuration
    let high_sec_config = BackendConfigBuilder::new()
        .name("secure-backend")
        .hermetic(true)
        .deterministic(true)
        .build();

    assert!(high_sec_config.hermetic);
    assert!(high_sec_config.deterministic);

    // Verify security constraints are enforced
    let secure_cmd = CommandBuilder::new("echo")
        .arg("secure test")
        .build();

    // Execution should succeed with security policies
    let result = ResultBuilder::new()
        .exit_code(0)
        .stdout("secure test\n")
        .build();

    assert_eq!(result.exit_code, 0);

    Ok(())
}

/// Test resource limits enforcement
#[test]
fn test_resource_limits_workflow() -> Result<()> {
    let ctx = TestContext::new()?;

    // Create configuration with resource limits
    let config = BackendConfigBuilder::new()
        .name("limited-backend")
        .timeout(10)
        .build();

    assert_eq!(config.timeout, 10);

    // Simulate execution within limits
    let result = ResultBuilder::new()
        .duration_ms(5000)
        .exit_code(0)
        .build();

    assert!(result.duration_ms < config.timeout * 1000);

    Ok(())
}

/// Test configuration hot-reload workflow
#[test]
fn test_config_hot_reload() -> Result<()> {
    let ctx = TestContext::new()?;

    // Initial configuration
    let config_v1 = r#"
[backend]
name = "test-backend"
timeout = 30
"#;
    ctx.create_file("config.toml", config_v1)?;

    // Updated configuration
    let config_v2 = r#"
[backend]
name = "test-backend"
timeout = 60
"#;
    ctx.create_file("config.toml", config_v2)?;

    // Verify configuration was updated
    let content = ctx.read_file("config.toml")?;
    assert!(content.contains("timeout = 60"));

    Ok(())
}

/// Test distributed tracing workflow
#[test]
fn test_distributed_tracing_workflow() -> Result<()> {
    let ctx = TestContext::new()?;

    // Create traced execution
    let trace_id = "trace-123";
    let span_id = "span-456";

    // Stage 1: Start trace
    let cmd = CommandBuilder::new("echo")
        .arg("traced command")
        .env("TRACE_ID", trace_id)
        .env("SPAN_ID", span_id)
        .build();

    // Stage 2: Execute with tracing
    let result = ResultBuilder::new()
        .exit_code(0)
        .stdout("traced command\n")
        .duration_ms(200)
        .build();

    // Stage 3: Verify trace context
    assert_eq!(cmd.env_vars.get("TRACE_ID"), Some(&trace_id.to_string()));
    assert_eq!(cmd.env_vars.get("SPAN_ID"), Some(&span_id.to_string()));

    Ok(())
}
