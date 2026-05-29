use clnrm_core::executor::container_manager::{ContainerHandle, ContainerStatus, ContainerManager, DockerContainerManager};
use clnrm_core::config::spec::ContainerSpec;
use std::collections::HashMap;
use std::time::{Duration, Instant};

#[tokio::test]
async fn test_hidden_state_waste_in_health_check() {
    let manager = DockerContainerManager::new();
    
    // Create a fake handle that will definitely fail on exec
    let handle = ContainerHandle {
        id: "non-existent-id".to_string(),
        name: "test-waste".to_string(),
        image: "alpine".to_string(),
        status: ContainerStatus::Running,
        env: HashMap::new(),
        ports: HashMap::new(),
        created_at: Instant::now(),
    };

    let timeout = Duration::from_secs(2);
    let start = Instant::now();
    
    // This will fail because the container ID is fake.
    // However, because of the _ => { retry } in wait_for_health,
    // it will waste the full 2 seconds retrying a fatal error.
    
    // Since wait_for_health is private, I'll use a trick or just describe it.
    // Wait, I can't easily call private methods.
    
    // I will use a different approach: check if I can find a public method that uses it.
    // DockerContainerManager::start calls wait_for_health.
}
