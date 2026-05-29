//! Gall Test Suite for Service Plugin Registry
//!
//! Validates Service state transitions without starting external daemons.

use clnrm_core::cleanroom::{CleanroomEnvironment, ServicePlugin, HealthStatus, ServiceHandle};
use clnrm_core::error::Result;

// A simple mock plugin that doesn't hit Docker
#[derive(Debug)]
struct MockPlugin {
    name: String,
}

impl ServicePlugin for MockPlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn start(&self) -> Result<ServiceHandle> {
        Ok(ServiceHandle::new(&self.name))
    }

    fn stop(&self, _handle: ServiceHandle) -> Result<()> {
        Ok(())
    }

    fn health_check(&self, _handle: &ServiceHandle) -> HealthStatus {
        HealthStatus::Healthy
    }
}

#[tokio::test]
async fn gall_test_service_registry_lifecycle() -> Result<()> {
    // Arrange
    let mut env = CleanroomEnvironment::with_config(None).await?;
    let plugin = Box::new(MockPlugin { name: "gall_mock".to_string() });

    // Act 1: Register
    env.register_service(plugin).await?;
    
    // Assert 1
    {
        let services = env.services().await;
        assert!(services.plugins.contains_key("gall_mock"), "Plugin should be registered");
    }

    // Act 2: Start
    let handle = env.start_service("gall_mock").await?;

    // Assert 2
    {
        let services = env.services().await;
        assert!(services.active_services().contains_key(&handle.id), "Service should be active");
        
        // Gall's Law check: is_registered logic used by execute_in_container
        let is_registered = services.active_services().values().any(|h| h.service_name == "gall_mock");
        assert!(is_registered, "Service should be recognized as actively registered");
    }

    // Act 3: Stop
    env.stop_service(&handle.id).await?;

    // Assert 3
    {
        let services = env.services().await;
        assert!(!services.active_services().contains_key(&handle.id), "Service should be removed from active registry");
    }

    Ok(())
}