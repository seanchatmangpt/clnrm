//! Regression Tests for v1.2.2 Features
//!
//! Ensures all v1.2.2 features continue to work in v1.3.0.

use super::common::*;
use clnrm_core::*;

#[tokio::test]
async fn test_regression_container_lifecycle() -> Result<()> {
    // v1.2.2 Feature: Basic container lifecycle
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Full lifecycle
    let handle = env.start_service("alpine").await?;
    let result = env
        .execute_in_container("alpine", &["echo".to_string(), "test".to_string()], None, None)
        .await?;
    env.stop_service(&handle.id).await?;

    // Assert: Lifecycle works
    assert!(result.succeeded());
    Ok(())
}

#[tokio::test]
async fn test_regression_toml_parsing() -> Result<()> {
    // v1.2.2 Feature: TOML configuration parsing
    // Arrange: Valid v1.2.2 TOML
    let toml = r#"
[meta]
name = "regression_test"
version = "1.0.0"

[[scenario]]
name = "test"

[[scenario.steps]]
name = "step1"
command = ["echo", "test"]
"#;

    // Act: Parse TOML
    let config = config::parse_toml_config(toml)?;

    // Assert: Parsing succeeded
    assert!(config.meta.is_some());
    assert_eq!(config.meta.unwrap().name, "regression_test");
    Ok(())
}

#[tokio::test]
async fn test_regression_otel_initialization() -> Result<()> {
    // v1.2.2 Feature: OpenTelemetry initialization
    // Arrange: OTEL config
    let otel_config = telemetry::OtelConfig {
        service_name: "regression_test",
        deployment_env: "test",
        sample_ratio: 1.0,
        export: telemetry::Export::Stdout,
        enable_fmt_layer: false,
        headers: None,
    };

    // Act: Initialize OTEL
    let guard = telemetry::init_otel(otel_config)?;

    // Assert: Initialization succeeded
    drop(guard); // Cleanup
    Ok(())
}

#[tokio::test]
async fn test_regression_service_registration() -> Result<()> {
    // v1.2.2 Feature: Service plugin registration
    // Arrange: Create environment
    let env = create_test_environment().await?;

    // Act: Register multiple services
    let alpine = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    let busybox = services::generic::GenericContainerPlugin::new("busybox", "busybox:latest");

    env.register_service(Box::new(alpine)).await?;
    env.register_service(Box::new(busybox)).await?;

    // Assert: Services registered
    Ok(())
}

#[tokio::test]
async fn test_regression_command_execution() -> Result<()> {
    // v1.2.2 Feature: Command execution in containers
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let handle = env.start_service("alpine").await?;

    // Act: Execute multiple commands
    let result1 = env
        .execute_in_container("alpine", &["echo".to_string(), "cmd1".to_string()], None, None)
        .await?;

    let result2 = env
        .execute_in_container("alpine", &["echo".to_string(), "cmd2".to_string()], None, None)
        .await?;

    // Assert: Both commands executed
    assert!(result1.succeeded() && result2.succeeded());

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_regression_environment_variables() -> Result<()> {
    // v1.2.2 Feature: Environment variable support
    // Arrange: Create environment with env vars
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    let mut env_vars = std::collections::HashMap::new();
    env_vars.insert("TEST_VAR".to_string(), "test_value".to_string());

    // Act: Execute with env vars
    let handle = env.start_service("alpine").await?;
    let result = env
        .execute_in_container(
            "alpine",
            &["sh".to_string(), "-c".to_string(), "echo $TEST_VAR".to_string()],
            Some(&env_vars),
            None,
        )
        .await?;

    // Assert: Env var was set
    assert!(result.stdout.contains("test_value"));

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_regression_error_handling() -> Result<()> {
    // v1.2.2 Feature: Comprehensive error handling
    // Arrange: Create error scenario
    let env = create_test_environment().await?;

    // Act: Try to start nonexistent service
    let result = env.start_service("nonexistent").await;

    // Assert: Error handled properly
    assert!(result.is_err());
    Ok(())
}

#[tokio::test]
async fn test_regression_health_checks() -> Result<()> {
    // v1.2.2 Feature: Service health checks
    // Arrange: Create environment with service
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Check health before and after starting service
    let health_before = env.check_health().await;
    let handle = env.start_service("alpine").await?;
    let health_after = env.check_health().await;

    // Assert: Health check works
    assert!(health_before.is_empty());
    assert!(!health_after.is_empty());

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_regression_template_rendering() -> Result<()> {
    // v1.2.2 Feature: Template rendering
    // Arrange: Create renderer with context
    let mut renderer = TemplateRenderer::new()?;
    let mut context = TemplateContext::new();
    context.vars.insert(
        "name".to_string(),
        serde_json::Value::String("test".to_string()),
    );

    // Act: Render template
    let rendered = renderer.render_str("Hello {{ name }}", "test")?;

    // Assert: Template rendered correctly
    assert_eq!(rendered, "Hello test");
    Ok(())
}

#[tokio::test]
async fn test_regression_validation() -> Result<()> {
    // v1.2.2 Feature: Configuration validation
    // Arrange: Create validator and valid config file
    use tempfile::TempDir;
    let temp_dir = TempDir::new()?;
    let config_path = temp_dir.path().join("test.toml");

    let config = r#"
[meta]
name = "validation_test"
version = "1.0.0"

[[scenario]]
name = "test"

[[scenario.steps]]
name = "step1"
command = ["echo", "test"]
"#;

    std::fs::write(&config_path, config)?;

    // Act: Validate configuration
    let mut validator = validation::shape::ShapeValidator::new();
    let result = validator.validate_file(&config_path)?;

    // Assert: Validation passed
    assert!(result.passed, "Configuration should be valid");
    Ok(())
}
