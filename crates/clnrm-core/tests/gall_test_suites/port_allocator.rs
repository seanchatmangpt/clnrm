//! Gall Test Suite for Port Allocator
//!
//! Validates `PortAllocator` handles concurrent requests without overlaps.

use clnrm_core::telemetry::live_check::port_allocator::PortAllocator;
use std::sync::Arc;
use tokio::task::JoinSet;

#[tokio::test]
async fn gall_test_concurrent_port_allocation() {
    // Arrange (Isolate)
    let allocator = Arc::new(PortAllocator::new().expect("Should initialize port allocator")); 
    let mut set = JoinSet::new();

    // Act (Ignite)
    // Request 10 ports concurrently
    for _ in 0..10 {
        let alloc_clone = allocator.clone();
        set.spawn(async move {
            alloc_clone.allocate_port().await.expect("Should allocate port")
        });
    }

    let mut allocated_ports = std::collections::HashSet::new();
    while let Some(res) = set.join_next().await {
        let port_lock = res.expect("Task failed");
        let port = port_lock.port();
        
        // Assert (Measure)
        // Ensure NO duplicate ports were handed out
        assert!(
            !allocated_ports.contains(&port),
            "Duplicate port allocated: {}",
            port
        );
        allocated_ports.insert(port);
        
        // Ensure ports are within bounds
        assert!((port >= 4317 && port <= 4327) || (port >= 5317 && port <= 5327) || (port >= 6317 && port <= 6337), "Port {} out of bounds", port);
    }

    assert_eq!(allocated_ports.len(), 10, "Should have allocated exactly 10 unique ports");
}