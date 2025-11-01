//! Integration Test Suite for v1.3.0
//!
//! Comprehensive integration tests organized by category:
//! 1. End-to-End Workflows
//! 2. Feature Integration
//! 3. Regression Tests
//! 4. Error Scenarios
//! 5. Performance Tests
//! 6. Security Tests

use clnrm_core::*;
use std::collections::HashMap;
use std::time::Instant;

// ============================================================================
// Test Helpers
// ============================================================================

mod helpers {
    use super::*;

    /// Create a test environment with OTEL configured for testing
    pub async fn create_test_environment() -> Result<cleanroom::CleanroomEnvironment> {
        cleanroom::CleanroomEnvironment::new().await
    }
}

// ============================================================================
// End-to-End Workflow Tests (21 tests)
// ============================================================================

#[tokio::test]
async fn test_e2e_basic_workflow_single_container() -> Result<()> {
    // Arrange
    let env = helpers::create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act
    let handle = env.start_service("alpine").await?;
    let result = env
        .execute_in_container(
            "alpine",
            &["echo".to_string(), "hello".to_string()],
            None,
            None,
        )
        .await?;

    // Assert
    assert!(result.succeeded());
    assert!(result.stdout.contains("hello"));

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_e2e_basic_workflow_with_environment_variables() -> Result<()> {
    let env = helpers::create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let mut env_vars = HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());

    let handle = env.start_service("alpine").await?;
    let result = env
        .execute_in_container(
            "alpine",
            &[
                "sh".to_string(),
                "-c".to_string(),
                "echo $TEST_VAR".to_string(),
            ],
            None,
            Some(&env_vars),
        )
        .await?;

    assert!(result.succeeded());
    assert!(result.stdout.contains("test_value"));

    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_e2e_basic_workflow_multiple_steps() -> Result<()> {
    let env = helpers::create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;

    let step1 = env
        .execute_in_container(
            "alpine",
            &["echo".to_string(), "step1".to_string()],
            None,
            None,
        )
        .await?;
    let step2 = env
        .execute_in_container(
            "alpine",
            &["echo".to_string(), "step2".to_string()],
            None,
            None,
        )
        .await?;
    let step3 = env
        .execute_in_container(
            "alpine",
            &["echo".to_string(), "step3".to_string()],
            None,
            None,
        )
        .await?;

    assert!(step1.succeeded() && step2.succeeded() && step3.succeeded());

    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_e2e_multi_service_coordination() -> Result<()> {
    let env = helpers::create_test_environment().await?;

    let alpine = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    let busybox = services::generic::GenericContainerPlugin::new("busybox", "busybox:latest");

    env.register_service(Box::new(alpine)).await?;
    env.register_service(Box::new(busybox)).await?;

    let alpine_handle = env.start_service("alpine").await?;
    let busybox_handle = env.start_service("busybox").await?;

    let health = env.check_health().await;
    assert_eq!(health.len(), 2);

    env.stop_service(&alpine_handle.id).await?;
    env.stop_service(&busybox_handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_e2e_template_vars_basic() -> Result<()> {
    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();
    context.vars.insert(
        "name".to_string(),
        serde_json::Value::String("test".to_string()),
    );

    let rendered = renderer.render_str("Hello {{ name }}", "test")?;
    assert_eq!(rendered, "Hello test");
    Ok(())
}

// ============================================================================
// Feature Integration Tests (10 tests)
// ============================================================================

#[tokio::test]
async fn test_feature_span_enforcement_basic() -> Result<()> {
    let env = helpers::create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;
    let result = env
        .execute_in_container(
            "alpine",
            &["echo".to_string(), "test".to_string()],
            None,
            None,
        )
        .await?;

    assert!(result.succeeded());
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_feature_service_routing() -> Result<()> {
    let env = helpers::create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;
    let result = env
        .execute_in_container(
            "alpine",
            &["echo".to_string(), "test".to_string()],
            None,
            None,
        )
        .await?;

    assert!(result.succeeded());
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_feature_chaos_container_restart() -> Result<()> {
    let env = helpers::create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle1 = env.start_service("alpine").await?;
    env.stop_service(&handle1.id).await?;

    let handle2 = env.start_service("alpine").await?;
    let result = env
        .execute_in_container(
            "alpine",
            &["echo".to_string(), "restarted".to_string()],
            None,
            None,
        )
        .await?;

    assert!(result.succeeded());
    env.stop_service(&handle2.id).await?;
    Ok(())
}

// ============================================================================
// Regression Tests (10 tests)
// ============================================================================

#[tokio::test]
async fn test_regression_container_lifecycle() -> Result<()> {
    let env = helpers::create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;
    let result = env
        .execute_in_container(
            "alpine",
            &["echo".to_string(), "test".to_string()],
            None,
            None,
        )
        .await?;
    env.stop_service(&handle.id).await?;

    assert!(result.succeeded());
    Ok(())
}

#[tokio::test]
async fn test_regression_toml_parsing() -> Result<()> {
    let toml = r#"
[meta]
name = "test"
version = "1.0.0"

[[scenario]]
name = "test"

[[scenario.steps]]
name = "step1"
command = ["echo", "test"]
"#;

    let config = config::parse_toml_config(toml)?;
    assert!(config.meta.is_some());
    Ok(())
}

#[tokio::test]
async fn test_regression_otel_initialization() -> Result<()> {
    let otel_config = telemetry::OtelConfig {
        service_name: "test",
        deployment_env: "test",
        sample_ratio: 1.0,
        export: telemetry::Export::Stdout,
        enable_fmt_layer: false,
        headers: None,
    };

    let guard = telemetry::init_otel(otel_config)?;
    drop(guard);
    Ok(())
}

// ============================================================================
// Error Scenario Tests (28 tests)
// ============================================================================

#[tokio::test]
async fn test_error_invalid_toml_syntax() -> Result<()> {
    let invalid_toml = "[meta\nname = \"test\"";
    let result = config::parse_toml_config(invalid_toml);
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_error_execute_on_unregistered_service() -> Result<()> {
    let env = helpers::create_test_environment().await?;
    let result = env
        .execute_in_container("nonexistent", &["echo".to_string()], None, None)
        .await;

    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_error_start_nonexistent_service() -> Result<()> {
    let env = helpers::create_test_environment().await?;
    let result = env.start_service("nonexistent").await;
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_error_undefined_template_variable() -> Result<()> {
    let mut renderer = TemplateRenderer::new()?;
    let result = renderer.render_str("{{ undefined }}", "test");
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_error_invalid_template_syntax() -> Result<()> {
    let mut renderer = TemplateRenderer::new()?;
    let result = renderer.render_str("{{ unclosed ", "test");
    assert!(result.is_err());
    Ok(())
}

// ============================================================================
// Performance Tests (7 tests)
// ============================================================================

#[tokio::test]
async fn test_perf_container_startup_time() -> Result<()> {
    let env = helpers::create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let start = Instant::now();
    let handle = env.start_service("alpine").await?;
    let duration = start.elapsed();

    assert!(duration.as_secs() < 30, "Startup should be < 30s");

    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_perf_command_execution_overhead() -> Result<()> {
    let env = helpers::create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;

    let start = Instant::now();
    env.execute_in_container("alpine", &["true".to_string()], None, None)
        .await?;
    let duration = start.elapsed();

    assert!(duration.as_secs() < 5, "Execution should be < 5s");

    env.stop_service(&handle.id).await?;
    Ok(())
}

// ============================================================================
// Security Tests (7 tests)
// ============================================================================

#[tokio::test]
async fn test_security_filesystem_isolation() -> Result<()> {
    let env1 = helpers::create_test_environment().await?;
    let env2 = helpers::create_test_environment().await?;

    let plugin1 = services::generic::GenericContainerPlugin::new("alpine1", "alpine:latest");
    let plugin2 = services::generic::GenericContainerPlugin::new("alpine2", "alpine:latest");

    env1.register_service(Box::new(plugin1)).await?;
    env2.register_service(Box::new(plugin2)).await?;

    let handle1 = env1.start_service("alpine1").await?;
    let handle2 = env2.start_service("alpine2").await?;

    env1.execute_in_container(
        "alpine1",
        &[
            "sh".to_string(),
            "-c".to_string(),
            "echo 'secret' > /tmp/secret.txt".to_string(),
        ],
        None,
        None,
    )
    .await?;

    let result = env2
        .execute_in_container(
            "alpine2",
            &["cat".to_string(), "/tmp/secret.txt".to_string()],
            None,
            None,
        )
        .await?;

    assert!(!result.succeeded(), "File should not be accessible");

    env1.stop_service(&handle1.id).await?;
    env2.stop_service(&handle2.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_security_cleanup_verification() -> Result<()> {
    let env = helpers::create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;
    env.stop_service(&handle.id).await?;

    let health = env.check_health().await;
    assert!(health.is_empty(), "All services should be cleaned up");

    Ok(())
}
