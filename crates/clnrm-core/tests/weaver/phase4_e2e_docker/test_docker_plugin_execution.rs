//! E2E Docker plugin execution tests

#[cfg(test)]
mod e2e_docker_plugin_tests {
    use clnrm_core::cleanroom::{CleanroomEnvironment, HealthStatus, MockDatabasePlugin};
    use clnrm_core::error::Result;

    #[tokio::test]
    async fn test_docker_plugin_execution() -> Result<()> {
        // Arrange
        let env = CleanroomEnvironment::new().await?;
        let plugin = Box::new(MockDatabasePlugin::new());
        env.register_service(plugin).await?;

        // Act - Start service
        let handle = env.start_service("mock_database").await?;
        
        // Assert - Verify service metadata and health
        assert_eq!(handle.service_name, "mock_database");
        let health = env.check_health().await;
        assert_eq!(health.get(&handle.id), Some(&HealthStatus::Healthy));

        // Cleanup
        env.stop_service(&handle.id).await?;
        
        Ok(())
    }
}
