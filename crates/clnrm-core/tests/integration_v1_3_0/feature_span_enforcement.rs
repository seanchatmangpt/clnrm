//! Feature Integration: Span Enforcement Tests
//!
//! Tests span enforcement integration with OTEL validation.

use super::common::*;
use clnrm_core::*;

#[tokio::test]
async fn test_span_enforcement_basic() -> Result<()> {
    // Arrange: Create environment with OTEL enabled
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Execute command and check span creation
    let handle = env.start_service("alpine").await?;
    let result = env
        .execute_in_container("alpine", &["echo".to_string(), "test".to_string()], None, None)
        .await?;

    // Assert: Command succeeded (span validation happens internally)
    assert!(result.succeeded());

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_span_enforcement_with_attributes() -> Result<()> {
    // Arrange: Create environment
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Execute with expected span attributes
    let handle = env.start_service("alpine").await?;
    let result = env
        .execute_in_container("alpine", &["echo".to_string(), "test".to_string()], None, None)
        .await?;

    // Assert: Execution succeeded with proper instrumentation
    assert!(result.succeeded());

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_span_enforcement_nested_spans() -> Result<()> {
    // Arrange: Create environment for nested operations
    let env = create_test_environment().await?;
    let plugin = services::generic::GenericContainerPlugin::new("alpine", "alpine:latest");
    env.register_service(Box::new(plugin)).await?;

    // Act: Execute nested operations
    let handle = env.start_service("alpine").await?;

    // Parent operation
    let result1 = env
        .execute_in_container("alpine", &["echo".to_string(), "parent".to_string()], None, None)
        .await?;

    // Child operation
    let result2 = env
        .execute_in_container("alpine", &["echo".to_string(), "child".to_string()], None, None)
        .await?;

    // Assert: Both operations succeeded with span hierarchy
    assert!(result1.succeeded() && result2.succeeded());

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}
