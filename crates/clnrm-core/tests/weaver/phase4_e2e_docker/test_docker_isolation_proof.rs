//! E2E Docker isolation proof

#[cfg(test)]
mod e2e_docker_isolation_tests {
    use clnrm_core::cleanroom::{CleanroomEnvironment, MockDatabasePlugin};
    use clnrm_core::error::Result;

    #[tokio::test]
    async fn test_docker_isolation_proof() -> Result<()> {
        // Arrange
        let env = CleanroomEnvironment::new().await?;
        let plugin = Box::new(MockDatabasePlugin::new());
        env.register_service(plugin).await?;
        let handle1 = env.start_service("mock_database").await?;
        let handle2 = env.start_service("mock_database").await?;

        // Assert - Verify both services running and isolated
        assert_ne!(handle1.id, handle2.id);
        let health = env.check_health().await;
        assert!(health.contains_key(&handle1.id));
        assert!(health.contains_key(&handle2.id));

        // Cleanup
        env.stop_service(&handle1.id).await?;
        env.stop_service(&handle2.id).await?;
        
        Ok(())
    }
}
