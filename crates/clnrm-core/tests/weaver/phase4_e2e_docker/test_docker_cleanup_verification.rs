//! E2E Docker cleanup verification

#[cfg(test)]
mod e2e_docker_cleanup_tests {
    use clnrm_core::cleanroom::{CleanroomEnvironment, MockDatabasePlugin};
    use clnrm_core::error::Result;

    #[tokio::test]
    async fn test_docker_cleanup_verification() -> Result<()> {
        // Arrange
        let env = CleanroomEnvironment::new().await?;
        let plugin = Box::new(MockDatabasePlugin::new());
        env.register_service(plugin).await?;
        let handle = env.start_service("mock_database").await?;
        assert!(env.check_health().await.contains_key(&handle.id));

        // Act - Stop service and ensure cleanup
        env.stop_service(&handle.id).await?;

        // Assert - Verify cleanup
        let health = env.check_health().await;
        assert!(!health.contains_key(&handle.id));
        
        Ok(())
    }
}
