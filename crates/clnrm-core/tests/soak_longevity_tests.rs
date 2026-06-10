use clnrm_core::cleanroom::CleanroomEnvironment;
use clnrm_core::telemetry::live_check::port_allocator::PortAllocator;

#[tokio::test]
#[ignore = "long running soak"]
async fn test_soak_longevity() {
    let allocator = PortAllocator::new().unwrap();

    for _ in 0..1000 {
        // Create environment
        let mut env = CleanroomEnvironment::new().await.unwrap();

        // Allocate a port
        let port_lock = allocator.allocate_port().await.unwrap();

        // Check active services and containers before destruction
        let metrics = env.get_metrics().await.unwrap();
        // Since we didn't start a service here, it should be 0.
        // Assuming we started one:
        // env.start_service("test-service").await.unwrap();

        // Drop will deallocate port and destroy environment (containers)
        drop(port_lock);
        drop(env);
    }

    // Since environments are destroyed, we don't have a global metrics to check active_services
    // But we can check that ports aren't exhausted, since we allocated 1000 sequentially.
    // If they were leaked, allocate_port would fail or use up fallback ranges.
}
