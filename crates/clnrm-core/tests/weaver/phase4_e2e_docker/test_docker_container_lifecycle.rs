//! E2E Docker container lifecycle tests

#[cfg(test)]
mod e2e_docker_container_lifecycle_tests {
    use clnrm_core::cleanroom::{CleanroomEnvironment, HealthStatus, MockDatabasePlugin};
    use clnrm_core::error::Result;

    #[tokio::test]
    async fn test_container_lifecycle() -> Result<()> {
        // Arrange
        let env = CleanroomEnvironment::new().await?;
        let plugin = Box::new(MockDatabasePlugin::new());
        env.register_service(plugin).await?;

        // Act - Start service
        let handle = env.start_service("mock_database").await?;
        
        // Assert - Verify container started
        let health = env.check_health().await;
        assert_eq!(health.get(&handle.id), Some(&HealthStatus::Healthy));

        // Act - Stop service
        env.stop_service(&handle.id).await?;

        // Assert - Verify container stopped
        let health = env.check_health().await;
        assert!(!health.contains_key(&handle.id));
        
        Ok(())
    }
}
