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
    let handle = ServiceHandle::new("surrealdb");
    
    // Act
    // We check the health of a container that hasn't actually been started on the network.
    let status = plugin.health_check(&handle);

    // Assert
    // GALL GAP: The plugin always returns HealthStatus::Healthy immediately.
    // A production system must actually probe the TCP port or wait for readiness.
    assert_eq!(
        status, 
        HealthStatus::Unhealthy, 
        "Gall Gap: Service health check returned Healthy without performing a real TCP or readiness probe"
    );
}