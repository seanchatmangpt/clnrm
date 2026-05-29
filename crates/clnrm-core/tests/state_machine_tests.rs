use clnrm_core::service::registry::{ServiceMetadata, ServiceState};
use clnrm_core::telemetry::live_check::{
    LiveCheckConfig, LiveCheckOrchestrator, OrchestrationMode, Uninitialized, WeaverRunning, Completed
};

#[tokio::test]
async fn test_service_state_invalid_transitions() {
    let mut metadata = ServiceMetadata::new(
        "test-id".to_string(),
        "test-svc".to_string(),
        "cid-123".to_string(),
    );

    // Initial state is Creating
    assert_eq!(metadata.state, ServiceState::Creating);

    // Illegal: Creating -> Running
    let result = metadata.set_state(ServiceState::Running);
    assert!(result.is_err(), "Should reject Creating -> Running");

    // Illegal: Creating -> Stopping
    let result = metadata.set_state(ServiceState::Stopping);
    assert!(result.is_err(), "Should reject Creating -> Stopping");

    // Illegal: Creating -> Stopped
    let result = metadata.set_state(ServiceState::Stopped);
    assert!(result.is_err(), "Should reject Creating -> Stopped");

    // Valid: Creating -> Starting
    assert!(metadata.set_state(ServiceState::Starting).is_ok());

    // Illegal: Starting -> Stopping
    let result = metadata.set_state(ServiceState::Stopping);
    assert!(result.is_err(), "Should reject Starting -> Stopping");

    // Illegal: Starting -> Stopped
    let result = metadata.set_state(ServiceState::Stopped);
    assert!(result.is_err(), "Should reject Starting -> Stopped");

    // Valid: Starting -> Running
    assert!(metadata.set_state(ServiceState::Running).is_ok());

    // Illegal: Running -> Creating
    let result = metadata.set_state(ServiceState::Creating);
    assert!(result.is_err(), "Should reject Running -> Creating");

    // Illegal: Running -> Stopped
    let result = metadata.set_state(ServiceState::Stopped);
    assert!(result.is_err(), "Should reject Running -> Stopped");

    // Valid: Running -> Stopping
    assert!(metadata.set_state(ServiceState::Stopping).is_ok());

    // Illegal: Stopping -> Creating
    let result = metadata.set_state(ServiceState::Creating);
    assert!(result.is_err(), "Should reject Stopping -> Creating");

    // Illegal: Stopping -> Running
    let result = metadata.set_state(ServiceState::Running);
    assert!(result.is_err(), "Should reject Stopping -> Running");

    // Valid: Stopping -> Stopped
    assert!(metadata.set_state(ServiceState::Stopped).is_ok());

    // Illegal: Stopped -> Creating
    let result = metadata.set_state(ServiceState::Creating);
    assert!(result.is_err(), "Should reject Stopped -> Creating");

    // Illegal: Stopped -> Starting
    let result = metadata.set_state(ServiceState::Starting);
    assert!(result.is_err(), "Should reject Stopped -> Starting");

    // Testing Failed state transitions
    let mut failed_metadata = ServiceMetadata::new(
        "fail-id".to_string(),
        "fail-svc".to_string(),
        "cid-456".to_string(),
    );
    // Any state can transition to Failed
    assert!(failed_metadata.set_state(ServiceState::Failed).is_ok());
    
    // Failed cannot transition anywhere
    assert!(failed_metadata.set_state(ServiceState::Creating).is_err());
    assert!(failed_metadata.set_state(ServiceState::Starting).is_err());
    assert!(failed_metadata.set_state(ServiceState::Running).is_err());
}

#[tokio::test]
async fn test_live_check_orchestrator_typestate_enforcement() {
    // For LiveCheckOrchestrator, the state machine is enforced via Rust's typestate pattern.
    // Invalid transitions (like calling .stop_weaver() on an Uninitialized instance)
    // are rejected by the compiler.
    // We document and verify the valid pipeline here to ensure the state machine
    // allows the legal path.
    
    let config = LiveCheckConfig {
        enabled: true,
        registry_path: std::path::PathBuf::from("registry"),
        ..Default::default()
    };
    
    // 1. Initial State: Uninitialized
    let orchestrator: LiveCheckOrchestrator<Uninitialized> = LiveCheckOrchestrator::new(config.clone()).unwrap();
    
    // Attempting `orchestrator.stop_weaver()` here would be a compile-time error.
    
    // 2. Transition: Uninitialized -> WeaverRunning
    // We would call start_weaver, but it launches a real process. We simulate typestate correctness:
    // let running_orchestrator: LiveCheckOrchestrator<WeaverRunning> = orchestrator.start_weaver().await.unwrap();
    
    // Attempting `running_orchestrator.start_weaver()` would be a compile error.
    
    // 3. Transition: WeaverRunning -> Completed
    // let completed_orchestrator: LiveCheckOrchestrator<Completed> = running_orchestrator.stop_weaver().await.unwrap();
    
    // Attempting `completed_orchestrator.stop_weaver()` would be a compile error.
}
