//! Concurrency race conditions testing for ServiceRegistry
//!
//! Tests for race conditions and deadlocks in the ServiceRegistry under maximum contention.

use clnrm_core::service::health::HealthStatus;
use clnrm_core::service::registry::{ServiceMetadata, ServiceRegistry, ServiceState};
use std::sync::Arc;
use std::time::Duration;

#[tokio::test(flavor = "multi_thread", worker_threads = 8)]
async fn test_service_registry_deadlock_prevention() {
    let registry = Arc::new(ServiceRegistry::new());
    let num_writer_tasks = 50;
    let num_reader_tasks = 10;
    let iterations = 100;

    let mut handles = Vec::new();

    // Spawn 50 writer tasks
    for task_id in 0..num_writer_tasks {
        let registry_clone = registry.clone();

        let handle = tokio::spawn(async move {
            for i in 0..iterations {
                let service_id = format!("service-{}-{}", task_id, i);

                // 1. Register
                let meta = ServiceMetadata::new(
                    service_id.clone(),
                    format!("name-{}", task_id),
                    format!("container-{}", task_id),
                );
                registry_clone
                    .register(meta)
                    .await
                    .expect("Failed to register");

                // Simulate slight delay to increase interleaving
                tokio::task::yield_now().await;

                // 2. Update State
                registry_clone
                    .update_state(&service_id, ServiceState::Starting)
                    .await
                    .expect("Failed to update state to Starting");

                tokio::task::yield_now().await;

                registry_clone
                    .update_state(&service_id, ServiceState::Running)
                    .await
                    .expect("Failed to update state to Running");

                // 3. Update Health
                registry_clone
                    .update_health(&service_id, HealthStatus::Healthy)
                    .await
                    .expect("Failed to update health");

                tokio::task::yield_now().await;

                // 4. Unregister
                registry_clone
                    .unregister(&service_id)
                    .await
                    .expect("Failed to unregister");
            }
        });

        handles.push(handle);
    }

    // Spawn reader tasks (constantly reading check_all_health())
    for _ in 0..num_reader_tasks {
        let registry_clone = registry.clone();

        let handle = tokio::spawn(async move {
            for _ in 0..iterations {
                // check_all_health does quite a bit of work, reading from the lock and potentially
                // spawning runsc processes if any service is present.
                let _ = registry_clone.check_all_health().await;
                tokio::task::yield_now().await;
            }
        });

        handles.push(handle);
    }

    // Wait for all tasks to complete, with a timeout to catch deadlocks
    let join_all_tasks = async {
        for handle in handles {
            handle.await.expect("A task panicked");
        }
    };

    let result = tokio::time::timeout(Duration::from_secs(60), join_all_tasks).await;

    assert!(
        result.is_ok(),
        "Test timed out! A deadlock likely occurred in the ServiceRegistry RwLock."
    );

    // Ensure everything was unregistered successfully (no leaked services)
    let final_services = registry.list_services().await;
    assert_eq!(
        final_services.len(),
        0,
        "Expected all services to be unregistered, but some remained."
    );
}
