//! Gall Test Suite for Service Health Checks
//!
//! Exposes the gap where health checks are hardcoded to return Healthy,
//! bypassing actual TCP/readiness probes.

use clnrm_core::cleanroom::{ServicePlugin, HealthStatus, ServiceHandle};
use clnrm_core::services::surrealdb::SurrealDbPlugin;

#[test]
fn gall_gap_test_service_health_check_tcp_probe() {
    // Arrange
    let plugin = SurrealDbPlugin::new();
    let mut handle = ServiceHandle::new("surrealdb");
    handle.metadata.insert("port".to_string(), "59873".to_string()); // Ephemeral random port

    
    // Act
    // We check the health of a container that hasn't actually been started on the network.
    let status = plugin.health_check(&handle);

    // Assert
    // The plugin now correctly performs a real TCP probe. Since the container
    // wasn't actually started and bound to the host, it must return Unhealthy.
    assert_eq!(
        status, 
        HealthStatus::Unhealthy, 
        "Service health check returned Healthy without performing a real TCP or readiness probe"
    );
}