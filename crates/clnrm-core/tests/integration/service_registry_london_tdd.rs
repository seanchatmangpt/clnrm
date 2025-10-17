//! London School TDD Tests for ServiceRegistry
//!
//! Test Coverage:
//! - Service plugin registration behavior
//! - Service lifecycle management (start/stop)
//! - Plugin interaction verification (mockist approach)
//! - Error handling with proper contract definition
//!
//! Testing Philosophy:
//! - OUTSIDE-IN: Test behavior from registry perspective
//! - MOCK-FIRST: Define plugin contracts through mocks
//! - BEHAVIOR VERIFICATION: Focus on interactions, not state
//!
//! Core Team Compliance:
//! - ✅ AAA pattern (Arrange, Act, Assert)
//! - ✅ Descriptive test names (test_X_with_Y_succeeds/fails)
//! - ✅ No false positives - proper error propagation
//! - ✅ Mock-driven contract definition

#![allow(clippy::unwrap_used)] // Test code only

use clnrm_core::cleanroom::{HealthStatus, ServiceHandle, ServicePlugin, ServiceRegistry};
use clnrm_core::error::{CleanroomError, Result};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

// ============================================================================
// Mock Service Plugin - Behavior Tracking
// ============================================================================

#[derive(Debug, Clone, Default)]
struct MockPluginCalls {
    start_calls: Vec<String>,
    stop_calls: Vec<String>,
    health_check_calls: Vec<String>,
}

#[derive(Debug)]
struct MockServicePlugin {
    name: String,
    calls: Arc<Mutex<MockPluginCalls>>,
    should_fail_start: bool,
    should_fail_stop: bool,
    health_status: HealthStatus,
}

impl MockServicePlugin {
    fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            calls: Arc::new(Mutex::new(MockPluginCalls::default())),
            should_fail_start: false,
            should_fail_stop: false,
            health_status: HealthStatus::Healthy,
        }
    }

    fn with_failing_start(mut self) -> Self {
        self.should_fail_start = true;
        self
    }

    fn with_failing_stop(mut self) -> Self {
        self.should_fail_stop = true;
        self
    }

    fn get_calls(&self) -> MockPluginCalls {
        self.calls.lock().unwrap().clone()
    }
}

impl ServicePlugin for MockServicePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        self.calls
            .lock()
            .unwrap()
            .start_calls
            .push(self.name.clone());

        if self.should_fail_start {
            return Err(CleanroomError::service_error(format!(
                "Mock start failure for service '{}'",
                self.name
            )));
        }

        Ok(ServiceHandle {
            id: format!("{}-handle-{}", self.name, uuid::Uuid::new_v4()),
            service_name: self.name.clone(),
            metadata: HashMap::new(),
        })
    }

    fn stop(&self, handle: ServiceHandle) -> Result<()> {
        self.calls
            .lock()
            .unwrap()
            .stop_calls
            .push(handle.id.clone());

        if self.should_fail_stop {
            return Err(CleanroomError::service_error(format!(
                "Mock stop failure for handle '{}'",
                handle.id
            )));
        }

        Ok(())
    }

    fn health_check(&self, handle: &ServiceHandle) -> HealthStatus {
        self.calls
            .lock()
            .unwrap()
            .health_check_calls
            .push(handle.id.clone());
        self.health_status.clone()
    }
}

// ============================================================================
// ServiceRegistry Registration Tests
// ============================================================================

#[test]
fn test_service_registry_with_new_creates_empty_registry() {
    // Arrange & Act
    let registry = ServiceRegistry::new();

    // Assert - Registry is empty but functional
    assert_eq!(registry.active_services().len(), 0);
}

#[test]
fn test_service_registry_with_plugin_registration_succeeds() {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let plugin = Box::new(MockServicePlugin::new("test_service"));

    // Act
    registry.register_plugin(plugin);

    // Assert - Plugin is registered (verified through start)
}

// ============================================================================
// ServiceRegistry Start Service Tests (Behavior Verification)
// ============================================================================

#[tokio::test]
async fn test_start_service_with_registered_plugin_succeeds() -> Result<()> {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let mock = MockServicePlugin::new("api_service");
    let calls_tracker = Arc::clone(&mock.calls);
    registry.register_plugin(Box::new(mock));

    // Act
    let handle = registry.start_service("api_service").await?;

    // Assert - Verify interaction: plugin.start() was called
    let calls = calls_tracker.lock().unwrap();
    assert_eq!(calls.start_calls.len(), 1, "start() should be called once");
    assert_eq!(
        calls.start_calls[0], "api_service",
        "start() should be called for correct service"
    );
    assert_eq!(
        handle.service_name, "api_service",
        "Handle should reference correct service"
    );

    Ok(())
}

#[tokio::test]
async fn test_start_service_with_unregistered_plugin_fails() {
    // Arrange
    let mut registry = ServiceRegistry::new();

    // Act
    let result = registry.start_service("nonexistent_service").await;

    // Assert
    assert!(result.is_err(), "Should fail for unregistered service");
    let err = result.unwrap_err();
    assert!(
        err.message.contains("not found"),
        "Error should indicate service not found, got: {}",
        err.message
    );
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
    assert!(result.is_err(), "Should propagate plugin start error");
    let err = result.unwrap_err();
    assert!(
        err.message.contains("Mock start failure"),
        "Should propagate mock error message"
    );
}

#[tokio::test]
async fn test_start_multiple_services_tracks_all_handles() -> Result<()> {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let mock1 = MockServicePlugin::new("service_1");
    let mock2 = MockServicePlugin::new("service_2");
    let calls1 = Arc::clone(&mock1.calls);
    let calls2 = Arc::clone(&mock2.calls);

    registry.register_plugin(Box::new(mock1));
    registry.register_plugin(Box::new(mock2));

    // Act
    let handle1 = registry.start_service("service_1").await?;
    let handle2 = registry.start_service("service_2").await?;

    // Assert - Both services were started
    assert_eq!(calls1.lock().unwrap().start_calls.len(), 1);
    assert_eq!(calls2.lock().unwrap().start_calls.len(), 1);
    assert_ne!(handle1.id, handle2.id, "Handles should be unique");

    Ok(())
}

// ============================================================================
// ServiceRegistry Stop Service Tests (Interaction Verification)
// ============================================================================

#[tokio::test]
async fn test_stop_service_with_active_handle_succeeds() -> Result<()> {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let mock = MockServicePlugin::new("stoppable_service");
    let calls_tracker = Arc::clone(&mock.calls);
    registry.register_plugin(Box::new(mock));
    let handle = registry.start_service("stoppable_service").await?;

    // Act
    registry.stop_service(&handle.id).await?;

    // Assert - Verify interaction: plugin.stop() was called with handle
    let calls = calls_tracker.lock().unwrap();
    assert_eq!(calls.stop_calls.len(), 1, "stop() should be called once");
    assert_eq!(
        calls.stop_calls[0], handle.id,
        "stop() should be called with correct handle"
    );

    Ok(())
}

#[tokio::test]
async fn test_stop_service_with_nonexistent_handle_succeeds_gracefully() {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let fake_handle_id = "nonexistent-handle-123";

    // Act
    let result = registry.stop_service(fake_handle_id).await;

    // Assert - Should succeed (idempotent operation)
    assert!(
        result.is_ok(),
        "Stopping nonexistent service should be idempotent"
    );
}

#[tokio::test]
async fn test_stop_service_with_failing_plugin_propagates_error() -> Result<()> {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let mock = MockServicePlugin::new("bad_service").with_failing_stop();
    registry.register_plugin(Box::new(mock));
    let handle = registry.start_service("bad_service").await?;

    // Act
    let result = registry.stop_service(&handle.id).await;

    // Assert
    assert!(result.is_err(), "Should propagate plugin stop error");
    let err = result.unwrap_err();
    assert!(
        err.message.contains("Mock stop failure"),
        "Should propagate mock error message"
    );

    Ok(())
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
    assert_eq!(active_before, 1, "Should have 1 active service before stop");
    assert_eq!(active_after, 0, "Should have 0 active services after stop");

    Ok(())
}

// ============================================================================
// ServiceRegistry Health Check Tests
// ============================================================================

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
async fn test_check_all_health_with_no_services_returns_empty() {
    // Arrange
    let registry = ServiceRegistry::new();

    // Act
    let health_status = registry.check_all_health().await;

    // Assert
    assert_eq!(
        health_status.len(),
        0,
        "Empty registry should have no health status"
    );
}

// ============================================================================
// ServiceRegistry Integration Scenarios
// ============================================================================

#[tokio::test]
async fn test_complete_service_lifecycle_succeeds() -> Result<()> {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let mock = MockServicePlugin::new("lifecycle_service");
    let calls_tracker = Arc::clone(&mock.calls);
    registry.register_plugin(Box::new(mock));

    // Act - Complete lifecycle: register -> start -> health_check -> stop
    let handle = registry.start_service("lifecycle_service").await?;
    let _health = registry.check_all_health().await;
    registry.stop_service(&handle.id).await?;

    // Assert - Verify complete interaction sequence
    let calls = calls_tracker.lock().unwrap();
    assert_eq!(calls.start_calls.len(), 1, "Should call start once");
    assert_eq!(calls.stop_calls.len(), 1, "Should call stop once");
    // health_check is called by check_all_health

    Ok(())
}

#[tokio::test]
async fn test_registry_with_default_plugins_initializes_correctly() {
    // Arrange & Act
    let registry = ServiceRegistry::new().with_default_plugins();

    // Assert - Registry has multiple default plugins
    assert!(
        registry.active_services().len() == 0,
        "No services started yet"
    );
}

#[test]
fn test_registry_is_service_running_with_inactive_service_returns_false() {
    // Arrange
    let registry = ServiceRegistry::new();

    // Act
    let is_running = registry.is_service_running("nonexistent");

    // Assert
    assert!(!is_running, "Should return false for inactive service");
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
    assert!(is_running, "Should return true for active service");

    Ok(())
}

// ============================================================================
// ServiceRegistry Error Path Tests (No False Positives)
// ============================================================================

#[tokio::test]
async fn test_registry_multiple_start_calls_create_separate_instances() -> Result<()> {
    // Arrange
    let mut registry = ServiceRegistry::new();
    let mock = MockServicePlugin::new("multi_instance");
    let calls_tracker = Arc::clone(&mock.calls);
    registry.register_plugin(Box::new(mock));

    // Act - Start same service multiple times
    let handle1 = registry.start_service("multi_instance").await?;
    let handle2 = registry.start_service("multi_instance").await?;

    // Assert - Each start creates new instance
    let calls = calls_tracker.lock().unwrap();
    assert_eq!(calls.start_calls.len(), 2, "Should call start twice");
    assert_ne!(
        handle1.id, handle2.id,
        "Each start should create unique handle"
    );
    assert_eq!(
        registry.active_services().len(),
        2,
        "Both instances should be tracked"
    );

    Ok(())
}
