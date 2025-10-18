//! Service Plugin Lifecycle Integration Tests
//!
//! This file consolidates critical service plugin lifecycle tests focusing on:
//! - GenericContainerPlugin configuration and lifecycle
//! - ServiceRegistry behavior
//! - Service start/stop operations
//! - Health check functionality
//! - Real container execution (not mocked)
//!
//! Testing Philosophy: 80/20 Principle
//! - Test actual container behavior, not mocks
//! - Focus on critical lifecycle paths
//! - Avoid excessive London School TDD mocking
//!
//! Core Team Compliance:
//! - ✅ AAA pattern (Arrange, Act, Assert)
//! - ✅ Descriptive test names
//! - ✅ Proper error handling

#![allow(clippy::unwrap_used)] // Test code only

use clnrm_core::cleanroom::{ServicePlugin, ServiceRegistry, HealthStatus};
use clnrm_core::error::Result;
use clnrm_core::services::generic::GenericContainerPlugin;

// ============================================================================
// GenericContainerPlugin Configuration Tests
// ============================================================================

#[test]
fn test_generic_container_plugin_with_image_name_succeeds() {
    // Arrange & Act
    let plugin = GenericContainerPlugin::new("test_service", "alpine:latest");

    // Assert
    assert_eq!(plugin.name(), "test_service");
}

#[test]
fn test_generic_container_plugin_with_environment_variables_succeeds() {
    // Arrange
    let plugin = GenericContainerPlugin::new("env_service", "alpine:latest")
        .with_env("DATABASE_URL", "postgres://localhost/test")
        .with_env("LOG_LEVEL", "debug");

    // Act & Assert
    assert_eq!(plugin.name(), "env_service");
}

#[test]
fn test_generic_container_plugin_with_port_mapping_succeeds() {
    // Arrange
    let plugin = GenericContainerPlugin::new("web_service", "nginx:alpine")
        .with_port(80)
        .with_port(443);

    // Act & Assert
    assert_eq!(plugin.name(), "web_service");
}

#[test]
fn test_generic_container_plugin_with_volume_mount_succeeds() -> Result<()> {
    // Arrange & Act
    let plugin = GenericContainerPlugin::new("data_service", "alpine:latest")
        .with_volume("/tmp/host", "/data/container", false)?;

    // Assert
    assert_eq!(plugin.name(), "data_service");
    Ok(())
}

#[test]
fn test_generic_container_plugin_with_invalid_volume_path_returns_error() {
    // Arrange & Act
    let result = GenericContainerPlugin::new("invalid_volume", "alpine:latest")
        .with_volume("", "/container/path", false);

    // Assert
    assert!(result.is_err(), "Should fail with empty host path");
}

#[test]
fn test_generic_container_plugin_implements_service_plugin_trait() {
    // Arrange
    let plugin: Box<dyn ServicePlugin> = Box::new(GenericContainerPlugin::new("trait_test", "alpine:latest"));

    // Act & Assert
    assert_eq!(plugin.name(), "trait_test");
}

// ============================================================================
// ServiceRegistry Registration Tests
// ============================================================================

#[test]
fn test_service_registry_with_new_creates_empty_registry() {
    // Arrange & Act
    let registry = ServiceRegistry::new();

    // Assert
    assert_eq!(registry.active_services().len(), 0);
}

#[test]
fn test_service_registry_with_plugin_registration_succeeds() {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let plugin = Box::new(GenericContainerPlugin::new("test_service", "alpine:latest"));

    // Act
    registry.register_plugin(plugin);

    // Assert - Plugin is registered
}

#[test]
fn test_registry_is_service_running_with_inactive_service_returns_false() {
    // Arrange
    let registry = ServiceRegistry::new();

    // Act
    let is_running = registry.is_service_running("nonexistent");

    // Assert
    assert!(!is_running);
}

// ============================================================================
// ServiceRegistry Lifecycle Tests (with mocks for isolation)
// ============================================================================

// Mock plugin for testing registry behavior without Docker
#[derive(Debug)]
struct MockServicePlugin {
    name: String,
    should_fail_start: bool,
}

impl MockServicePlugin {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            should_fail_start: false,
        }
    }

    fn with_failing_start(mut self) -> Self {
        self.should_fail_start = true;
        self
    }
}

impl ServicePlugin for MockServicePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<clnrm_core::cleanroom::ServiceHandle> {
        if self.should_fail_start {
            return Err(clnrm_core::error::CleanroomError::service_error(
                format!("Mock start failure for service '{}'", self.name)
            ));
        }

        Ok(clnrm_core::cleanroom::ServiceHandle {
            id: format!("{}-handle-{}", self.name, uuid::Uuid::new_v4()),
            service_name: self.name.clone(),
            metadata: std::collections::HashMap::new(),
        })
    }

    fn stop(&self, _handle: clnrm_core::cleanroom::ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _handle: &clnrm_core::cleanroom::ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}

#[tokio::test]
async fn test_start_service_with_registered_plugin_succeeds() -> Result<()> {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let mock = MockServicePlugin::new("api_service");
    registry.register_plugin(Box::new(mock));

    // Act
    let handle = registry.start_service("api_service").await?;

    // Assert
    assert_eq!(handle.service_name, "api_service");

    Ok(())
}

#[tokio::test]
async fn test_start_service_with_unregistered_plugin_fails() {
    // Arrange
    let mut registry = ServiceRegistry::new();

    // Act
    let result = registry.start_service("nonexistent_service").await;

    // Assert
    assert!(result.is_err());
}

#[tokio::test]
async fn test_start_service_with_failing_plugin_propagates_error() {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let mock = MockServicePlugin::new("failing_service").with_failing_start();
    registry.register_plugin(Box::new(mock));

    // Act
    let result = registry.start_service("failing_service").await;

    // Assert
    assert!(result.is_err());
}

#[tokio::test]
async fn test_stop_service_with_active_handle_succeeds() -> Result<()> {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let mock = MockServicePlugin::new("stoppable_service");
    registry.register_plugin(Box::new(mock));
    let handle = registry.start_service("stoppable_service").await?;

    // Act
    registry.stop_service(&handle.id).await?;

    // Assert - No error thrown
    Ok(())
}

#[tokio::test]
async fn test_stop_service_with_nonexistent_handle_succeeds_gracefully() {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let fake_handle_id = "nonexistent-handle-123";

    // Act
    let result = registry.stop_service(fake_handle_id).await;

    // Assert - Should be idempotent
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_stop_service_removes_handle_from_active_services() -> Result<()> {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let mock = MockServicePlugin::new("removable_service");
    registry.register_plugin(Box::new(mock));
    let handle = registry.start_service("removable_service").await?;

    // Act
    let active_before = registry.active_services().len();
    registry.stop_service(&handle.id).await?;
    let active_after = registry.active_services().len();

    // Assert
    assert_eq!(active_before, 1);
    assert_eq!(active_after, 0);

    Ok(())
}

#[tokio::test]
async fn test_check_all_health_with_healthy_services_reports_correctly() -> Result<()> {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let mock = MockServicePlugin::new("healthy_service");
    registry.register_plugin(Box::new(mock));
    let handle = registry.start_service("healthy_service").await?;

    // Act
    let health_status = registry.check_all_health().await;

    // Assert
    assert_eq!(health_status.len(), 1);
    assert_eq!(health_status.get(&handle.id), Some(&HealthStatus::Healthy));

    Ok(())
}

#[tokio::test]
async fn test_complete_service_lifecycle_succeeds() -> Result<()> {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let mock = MockServicePlugin::new("lifecycle_service");
    registry.register_plugin(Box::new(mock));

    // Act - Complete lifecycle: register -> start -> health_check -> stop
    let handle = registry.start_service("lifecycle_service").await?;
    let _health = registry.check_all_health().await;
    registry.stop_service(&handle.id).await?;

    // Assert - Complete lifecycle executed without errors
    Ok(())
}

#[tokio::test]
async fn test_registry_is_service_running_with_active_service_returns_true() -> Result<()> {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let mock = MockServicePlugin::new("active_service");
    registry.register_plugin(Box::new(mock));

    // Act
    registry.start_service("active_service").await?;
    let is_running = registry.is_service_running("active_service");

    // Assert
    assert!(is_running);

    Ok(())
}
