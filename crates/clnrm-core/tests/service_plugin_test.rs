//! Integration tests for service plugin system

use clnrm_core::cleanroom::{CleanroomEnvironment, HealthStatus, MockDatabasePlugin};
use clnrm_core::error::CleanroomError;
use clnrm_core::services::surrealdb::SurrealDbPlugin;

#[tokio::test]
async fn test_mock_database_plugin_lifecycle() -> Result<(), CleanroomError> {
    // Arrange
    let env = CleanroomEnvironment::new().await?;
    let plugin = Box::new(MockDatabasePlugin::new());

    // Act
    env.register_service(plugin).await?;
    let handle = env.start_service("mock_database").await?;

    // Assert
    assert_eq!(handle.service_name, "mock_database");
    assert!(handle.metadata.contains_key("port"));
    assert!(handle.metadata.contains_key("host"));

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_surrealdb_plugin_with_connection() -> Result<(), CleanroomError> {
    // Arrange
    let env = CleanroomEnvironment::new().await?;
    let plugin = Box::new(SurrealDbPlugin::new());

    // Act
    env.register_service(plugin).await?;
    let handle = env.start_service("surrealdb").await?;

    // Assert - verify we can actually connect
    assert!(handle.metadata.contains_key("connection_string"));

    let health = env.check_health().await;
    assert_eq!(health.get(&handle.id), Some(&HealthStatus::Healthy));

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_multiple_services_registration() -> Result<(), CleanroomError> {
    // Arrange
    let env = CleanroomEnvironment::new().await?;
    let mock_plugin = Box::new(MockDatabasePlugin::new());
    let surreal_plugin = Box::new(SurrealDbPlugin::new());

    // Act
    env.register_service(mock_plugin).await?;
    env.register_service(surreal_plugin).await?;

    // Assert - both services should be registered by trying to start them
    let mock_handle = env.start_service("mock_database").await?;
    env.stop_service(&mock_handle.id).await?;

    let surreal_handle = env.start_service("surrealdb").await?;
    env.stop_service(&surreal_handle.id).await?;

    Ok(())
}

#[tokio::test]
async fn test_service_health_check() -> Result<(), CleanroomError> {
    // Arrange
    let env = CleanroomEnvironment::new().await?;
    let plugin = Box::new(MockDatabasePlugin::new());

    // Act
    env.register_service(plugin).await?;
    let handle = env.start_service("mock_database").await?;

    // Assert
    let health = env.check_health().await;
    assert_eq!(health.get(&handle.id), Some(&HealthStatus::Healthy));

    // Cleanup
    env.stop_service(&handle.id).await?;
    Ok(())
}

#[tokio::test]
async fn test_service_stop_removes_from_registry() -> Result<(), CleanroomError> {
    // Arrange
    let env = CleanroomEnvironment::new().await?;
    let plugin = Box::new(MockDatabasePlugin::new());

    // Act
    env.register_service(plugin).await?;
    let handle = env.start_service("mock_database").await?;

    // Verify service is running
    let services = env.services().await;
    assert!(services.active_services().contains_key(&handle.id));

    // Stop service
    env.stop_service(&handle.id).await?;

    // Assert - service should be removed from active services
    let services = env.services().await;
    assert!(!services.active_services().contains_key(&handle.id));

    Ok(())
}
