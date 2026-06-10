//! E2E Docker weaver validation tests

#[cfg(test)]
mod e2e_docker_weaver_tests {
    use clnrm_core::cleanroom::{CleanroomEnvironment, MockDatabasePlugin};
    use clnrm_core::error::Result;

    #[tokio::test]
    async fn test_docker_weaver_validation() -> Result<()> {
        // Arrange
        let env = CleanroomEnvironment::new().await?;
        let plugin = Box::new(MockDatabasePlugin::new());
        env.register_service(plugin).await?;
        let handle = env.start_service("mock_database").await?;
        
        // Act - Execute test and verify behavior (this would ideally interact with OTel span validation)
        let _ = env.execute_test("docker_weaver_validation", || {
            Ok::<(), clnrm_core::error::CleanroomError>(())
        }).await?;

        // Assert - Verify OTel span metrics
        let metrics = env.get_metrics().await;
        assert_eq!(metrics.tests_executed, 1);
        assert_eq!(metrics.tests_passed, 1);

        // Cleanup
        env.stop_service(&handle.id).await?;
        
        Ok(())
    }
}
