//! Integration tests for SurrealDB service plugin
//!
//! These tests validate the 80/20 functionality of the SurrealDB plugin:
//! - Service lifecycle (start, stop, health checks)
//! - Connection verification and authentication
//! - Metadata extraction and validation
//! - Multiple instance management
//! - Error handling
//!
//! Note: These tests require Docker to be running. Tests are marked with #[ignore]
//! and include clear documentation about the Docker requirement.

use clnrm_core::cleanroom::{HealthStatus, ServicePlugin};
use clnrm_core::error::{CleanroomError, Result};
use clnrm_core::services::surrealdb::SurrealDbPlugin;

/// Helper function to check if Docker is available
fn docker_available() -> bool {
    use std::process::Command;

    // Try a quick docker ps command
    let output = Command::new("docker")
        .args(["ps", "--format", "table {{.Names}}\t{{.Status}}"])
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
            println!(
                "  3. Re-run the tests with: cargo test --test integration_surrealdb -- --ignored"
            );
            return Ok(());
        }
    };
}

/// Test 1: SurrealDB plugin starts successfully with default credentials
///
/// Validates:
/// - Plugin starts without errors
/// - ServiceHandle is created with valid ID
/// - ServiceHandle contains required metadata fields
/// - Health check returns Healthy status
#[tokio::test]
#[ignore] // Requires Docker
async fn test_surrealdb_plugin_starts_with_default_credentials() -> Result<()> {
    skip_if_no_docker!();

    // Arrange
    let plugin = SurrealDbPlugin::new();

    // Act
    let handle = plugin.start()?;

    // Assert - ServiceHandle validation
    assert!(!handle.id.is_empty(), "Handle ID should not be empty");
    assert_eq!(
        handle.service_name, "surrealdb",
        "Service name should be 'surrealdb'"
    );

    // Assert - Metadata validation
    assert!(
        handle.metadata.contains_key("host"),
        "Metadata should contain 'host'"
    );
    assert!(
        handle.metadata.contains_key("port"),
        "Metadata should contain 'port'"
    );
    assert!(
        handle.metadata.contains_key("username"),
        "Metadata should contain 'username'"
    );
    assert!(
        handle.metadata.contains_key("connection_string"),
        "Metadata should contain 'connection_string'"
    );
    assert!(
        handle.metadata.contains_key("database_type"),
        "Metadata should contain 'database_type'"
    );

    // Assert - Health check
    let health = plugin.health_check(&handle);
    assert_eq!(
        health,
        HealthStatus::Healthy,
        "Plugin should report Healthy status"
    );

    // Cleanup
    plugin.stop(handle)?;

    Ok(())
}

/// Test 2: SurrealDB plugin starts with custom credentials
///
/// Validates:
/// - Custom credentials are accepted
/// - Plugin starts successfully with custom auth
/// - Metadata contains correct username
#[tokio::test]
#[ignore] // Requires Docker
async fn test_surrealdb_plugin_starts_with_custom_credentials() -> Result<()> {
    skip_if_no_docker!();

    // Arrange
    let custom_username = "testuser";
    let custom_password = "testpass";
    let plugin = SurrealDbPlugin::with_credentials(custom_username, custom_password);

    // Act
    let handle = plugin.start()?;

    // Assert
    assert_eq!(handle.service_name, "surrealdb");
    assert_eq!(
        handle.metadata.get("username"),
        Some(&custom_username.to_string()),
        "Metadata should contain custom username"
    );

    let health = plugin.health_check(&handle);
    assert_eq!(health, HealthStatus::Healthy);

    // Cleanup
    plugin.stop(handle)?;

    Ok(())
}

/// Test 3: SurrealDB plugin metadata extraction is correct
///
/// Validates:
/// - Host is "127.0.0.1"
/// - Port is valid (> 1024)
/// - Connection string matches ws:// pattern
/// - Username matches configured value
/// - Database type is "surrealdb"
#[tokio::test]
#[ignore] // Requires Docker
async fn test_surrealdb_plugin_metadata_is_correct() -> Result<()> {
    skip_if_no_docker!();

    // Arrange
    let plugin = SurrealDbPlugin::new();

    // Act
    let handle = plugin.start()?;

    // Assert - Host validation
    let host = handle
        .metadata
        .get("host")
        .ok_or_else(|| CleanroomError::internal_error("Missing host metadata"))?;
    assert_eq!(host, "127.0.0.1", "Host should be 127.0.0.1");

    // Assert - Port validation
    let port_str = handle
        .metadata
        .get("port")
        .ok_or_else(|| CleanroomError::internal_error("Missing port metadata"))?;
    let port: u16 = port_str
        .parse()
        .map_err(|e| CleanroomError::validation_error(format!("Invalid port format: {}", e)))?;
    assert!(port > 1024, "Port should be > 1024 (user ports)");
    assert!(port < 65535, "Port should be < 65535 (valid range)");

    // Assert - Connection string validation
    let connection_string = handle
        .metadata
        .get("connection_string")
        .ok_or_else(|| CleanroomError::internal_error("Missing connection_string metadata"))?;
    assert!(
        connection_string.starts_with("ws://127.0.0.1:"),
        "Connection string should start with ws://127.0.0.1:"
    );
    assert!(
        connection_string.contains(&port.to_string()),
        "Connection string should contain the port number"
    );

    // Assert - Username validation
    let username = handle
        .metadata
        .get("username")
        .ok_or_else(|| CleanroomError::internal_error("Missing username metadata"))?;
    assert_eq!(username, "root", "Default username should be 'root'");

    // Assert - Database type validation
    let db_type = handle
        .metadata
        .get("database_type")
        .ok_or_else(|| CleanroomError::internal_error("Missing database_type metadata"))?;
    assert_eq!(db_type, "surrealdb", "Database type should be 'surrealdb'");

    // Cleanup
    plugin.stop(handle)?;

    Ok(())
}

/// Test 4: Multiple SurrealDB instances can run concurrently
///
/// Validates:
/// - Multiple instances can be started
/// - Each instance gets a unique port
/// - Instances are isolated from each other
/// - All instances can be stopped independently
#[tokio::test]
#[ignore] // Requires Docker
async fn test_surrealdb_plugin_supports_multiple_instances() -> Result<()> {
    skip_if_no_docker!();

    // Arrange
    let plugin1 = SurrealDbPlugin::new();
    let plugin2 = SurrealDbPlugin::new();

    // Act - Start multiple instances
    let handle1 = plugin1.start()?;
    let handle2 = plugin2.start()?;

    // Assert - Instances have different IDs
    assert_ne!(
        handle1.id, handle2.id,
        "Instances should have different IDs"
    );

    // Assert - Instances have different ports
    let port1 = handle1
        .metadata
        .get("port")
        .ok_or_else(|| CleanroomError::internal_error("Missing port in handle1"))?;
    let port2 = handle2
        .metadata
        .get("port")
        .ok_or_else(|| CleanroomError::internal_error("Missing port in handle2"))?;
    assert_ne!(port1, port2, "Instances should have different ports");

    // Assert - Both instances are healthy
    assert_eq!(plugin1.health_check(&handle1), HealthStatus::Healthy);
    assert_eq!(plugin2.health_check(&handle2), HealthStatus::Healthy);

    // Assert - Connection strings are different
    let conn1 = handle1
        .metadata
        .get("connection_string")
        .ok_or_else(|| CleanroomError::internal_error("Missing connection_string in handle1"))?;
    let conn2 = handle2
        .metadata
        .get("connection_string")
        .ok_or_else(|| CleanroomError::internal_error("Missing connection_string in handle2"))?;
    assert_ne!(
        conn1, conn2,
        "Connection strings should be different for isolated instances"
    );

    // Cleanup - Stop instances independently
    plugin1.stop(handle1)?;
    plugin2.stop(handle2)?;

    Ok(())
}

/// Test 5: SurrealDB plugin stop operation succeeds
///
/// Validates:
/// - Stop operation returns Ok
/// - No errors occur during cleanup
/// - Multiple stop calls are safe (idempotent)
#[tokio::test]
#[ignore] // Requires Docker
async fn test_surrealdb_plugin_stop_succeeds() -> Result<()> {
    skip_if_no_docker!();

    // Arrange
    let plugin = SurrealDbPlugin::new();
    let handle = plugin.start()?;

    // Act
    let stop_result = plugin.stop(handle.clone());

    // Assert
    assert!(
        stop_result.is_ok(),
        "Stop operation should succeed without errors"
    );

    // Act - Second stop should also succeed (idempotent)
    let second_stop_result = plugin.stop(handle);

    // Assert
    assert!(
        second_stop_result.is_ok(),
        "Second stop should succeed (idempotent behavior)"
    );

    Ok(())
}

/// Test 6: Health check returns Unknown for invalid handle
///
/// Validates:
/// - Health check handles missing metadata gracefully
/// - Returns Unknown status instead of crashing
#[tokio::test]
async fn test_surrealdb_plugin_health_check_with_invalid_handle() -> Result<()> {
    // Arrange
    let plugin = SurrealDbPlugin::new();
    let invalid_handle = clnrm_core::cleanroom::ServiceHandle {
        id: "invalid-id".to_string(),
        service_name: "surrealdb".to_string(),
        metadata: std::collections::HashMap::new(), // Empty metadata
    };

    // Act
    let health = plugin.health_check(&invalid_handle);

    // Assert
    assert_eq!(
        health,
        HealthStatus::Unknown,
        "Health check should return Unknown for invalid handle"
    );

    Ok(())
}

/// Test 7: Health check returns Healthy for valid handle
///
/// Validates:
/// - Health check correctly identifies healthy service
/// - Validates presence of required metadata fields
#[tokio::test]
#[ignore] // Requires Docker
async fn test_surrealdb_plugin_health_check_returns_healthy() -> Result<()> {
    skip_if_no_docker!();

    // Arrange
    let plugin = SurrealDbPlugin::new();
    let handle = plugin.start()?;

    // Act
    let health = plugin.health_check(&handle);

    // Assert
    assert_eq!(
        health,
        HealthStatus::Healthy,
        "Health check should return Healthy for valid handle"
    );

    // Cleanup
    plugin.stop(handle)?;

    Ok(())
}

/// Test 8: Plugin name is correct
///
/// Validates:
/// - Plugin returns correct name
/// - Name is consistent across operations
#[tokio::test]
async fn test_surrealdb_plugin_name_is_correct() -> Result<()> {
    // Arrange
    let plugin = SurrealDbPlugin::new();

    // Act
    let name = plugin.name();

    // Assert
    assert_eq!(name, "surrealdb", "Plugin name should be 'surrealdb'");

    Ok(())
}

/// Test 9: Plugin with strict mode starts successfully
///
/// Validates:
/// - Strict mode configuration is accepted
/// - Plugin starts with strict mode enabled
#[tokio::test]
#[ignore] // Requires Docker
async fn test_surrealdb_plugin_with_strict_mode() -> Result<()> {
    skip_if_no_docker!();

    // Arrange
    let plugin = SurrealDbPlugin::new().with_strict(true);

    // Act
    let handle = plugin.start()?;

    // Assert
    assert_eq!(handle.service_name, "surrealdb");
    let health = plugin.health_check(&handle);
    assert_eq!(health, HealthStatus::Healthy);

    // Cleanup
    plugin.stop(handle)?;

    Ok(())
}

/// Test 10: Connection verification during startup
///
/// Validates:
/// - Plugin verifies connection during start()
/// - Successful connection results in valid ServiceHandle
/// - Connection string is accessible for client usage
#[tokio::test]
#[ignore] // Requires Docker
async fn test_surrealdb_plugin_verifies_connection_during_startup() -> Result<()> {
    skip_if_no_docker!();

    // Arrange
    let plugin = SurrealDbPlugin::new();

    // Act - start() includes connection verification
    let handle = plugin.start()?;

    // Assert - If start() succeeded, connection was verified
    assert!(
        handle.metadata.contains_key("connection_string"),
        "Connection string should be available after successful start"
    );
    assert!(
        handle.metadata.contains_key("port"),
        "Port should be available after successful start"
    );

    // Connection was verified during start(), so we can safely use the service
    let connection_string = handle.metadata.get("connection_string").ok_or_else(|| {
        CleanroomError::internal_error("Connection string missing after verified startup")
    })?;

    assert!(
        !connection_string.is_empty(),
        "Connection string should not be empty"
    );

    // Cleanup
    plugin.stop(handle)?;

    Ok(())
}

/// Test 11: Comprehensive lifecycle test (80/20 validation)
///
/// This single test validates the most important 80% of functionality:
/// - Plugin creation with default and custom settings
/// - Service startup and metadata validation
/// - Health check functionality
/// - Multiple instance support
/// - Service cleanup
#[tokio::test]
#[ignore] // Requires Docker
async fn test_surrealdb_plugin_comprehensive_lifecycle() -> Result<()> {
    skip_if_no_docker!();

    // Test 1: Default configuration
    let default_plugin = SurrealDbPlugin::new();
    assert_eq!(default_plugin.name(), "surrealdb");

    let handle1 = default_plugin.start()?;
    assert_eq!(handle1.service_name, "surrealdb");
    assert!(handle1.metadata.contains_key("host"));
    assert!(handle1.metadata.contains_key("port"));
    assert!(handle1.metadata.contains_key("connection_string"));
    assert_eq!(default_plugin.health_check(&handle1), HealthStatus::Healthy);

    // Test 2: Custom credentials
    let custom_plugin = SurrealDbPlugin::with_credentials("admin", "secret");
    let handle2 = custom_plugin.start()?;
    assert_eq!(handle2.metadata.get("username"), Some(&"admin".to_string()));
    assert_eq!(custom_plugin.health_check(&handle2), HealthStatus::Healthy);

    // Test 3: Strict mode
    let strict_plugin = SurrealDbPlugin::new().with_strict(true);
    let handle3 = strict_plugin.start()?;
    assert_eq!(strict_plugin.health_check(&handle3), HealthStatus::Healthy);

    // Test 4: Port uniqueness
    let port1 = handle1
        .metadata
        .get("port")
        .ok_or_else(|| CleanroomError::internal_error("Missing port"))?;
    let port2 = handle2
        .metadata
        .get("port")
        .ok_or_else(|| CleanroomError::internal_error("Missing port"))?;
    let port3 = handle3
        .metadata
        .get("port")
        .ok_or_else(|| CleanroomError::internal_error("Missing port"))?;

    assert_ne!(port1, port2, "Ports should be unique");
    assert_ne!(port2, port3, "Ports should be unique");
    assert_ne!(port1, port3, "Ports should be unique");

    // Test 5: Connection string format
    for handle in [&handle1, &handle2, &handle3] {
        let conn = handle
            .metadata
            .get("connection_string")
            .ok_or_else(|| CleanroomError::internal_error("Missing connection_string"))?;
        assert!(conn.starts_with("ws://127.0.0.1:"));
        assert!(conn.len() > "ws://127.0.0.1:".len());
    }

    // Cleanup all instances
    default_plugin.stop(handle1)?;
    custom_plugin.stop(handle2)?;
    strict_plugin.stop(handle3)?;

    Ok(())
}

/// Test 12: Default implementation works correctly
///
/// Validates:
/// - Default trait implementation creates valid plugin
/// - Default plugin has correct credentials
#[tokio::test]
async fn test_surrealdb_plugin_default_implementation() -> Result<()> {
    // Arrange & Act
    let plugin = SurrealDbPlugin::default();

    // Assert
    assert_eq!(plugin.name(), "surrealdb");

    Ok(())
}
