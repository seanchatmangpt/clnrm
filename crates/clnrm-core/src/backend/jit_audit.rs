//! JIT Audit for Container Pool
//!
//! This module contains tests to verify Just-in-Time container creation
//! and quantify over-production waste in the container pool.

#[cfg(test)]
mod tests {
    use crate::backend::{ContainerPool, PoolConfig};
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_jit_container_creation_audit() {
        // Configure pool with a high min_idle to simulate potential over-production
        let config = PoolConfig {
            max_size: 20,
            min_idle: 10,
            ..Default::default()
        };

        tracing::info!(
            "Audit: Creating pool with min_idle = {} (JIT expected)",
            config.min_idle
        );
        let pool = ContainerPool::new(config.clone())
            .await
            .expect("Failed to create pool");

        // Phase 1: Pool Created - JIT check
        let stats = pool.stats();
        tracing::info!("Phase 1 (Post-Creation) Stats: {:?}", stats);

        let initial_waste = stats.overproduction_waste();
        tracing::info!(
            "Initial Over-production Waste: {} containers",
            initial_waste
        );

        // ASSERT JIT: No containers should be created yet
        assert_eq!(
            stats.created, 0,
            "JIT Violation: Containers created before request"
        );
        assert_eq!(
            stats.overproduction_waste(),
            0,
            "JIT Violation: Waste detected before request"
        );

        // Phase 2: Single Request
        tracing::info!("Audit: Acquiring 1 container");
        let handle = pool
            .acquire_handle()
            .await
            .expect("Failed to acquire handle");
        let stats = pool.stats();
        tracing::info!("Phase 2 (Single Request) Stats: {:?}", stats);

        assert_eq!(stats.created, 1, "Expected 1 container created JIT");
        assert_eq!(stats.max_active, 1);
        assert_eq!(
            stats.overproduction_waste(),
            0,
            "Sequential JIT should have 0 waste"
        );

        // Release it
        drop(handle);
        tokio::time::sleep(Duration::from_millis(100)).await; // Wait for auto-release

        // Phase 3: Multi-Request (Sequential Reuse)
        tracing::info!("Audit: Acquiring another container (should reuse)");
        let _handle2 = pool
            .acquire_handle()
            .await
            .expect("Failed to acquire handle");
        let stats = pool.stats();
        tracing::info!("Phase 3 (Sequential Reuse) Stats: {:?}", stats);

        assert_eq!(stats.created, 1, "Should have reused existing container");
        assert_eq!(stats.hits, 1);
        assert_eq!(stats.overproduction_waste(), 0);

        // Phase 4: Concurrent Requests (Expanding capacity JIT)
        tracing::info!("Audit: Acquiring 3 concurrent containers");
        let _h1 = pool.acquire_handle().await.expect("Failed to acquire");
        let _h2 = pool.acquire_handle().await.expect("Failed to acquire");
        let _h3 = pool.acquire_handle().await.expect("Failed to acquire");
        let stats = pool.stats();
        tracing::info!("Phase 4 (Concurrent Load) Stats: {:?}", stats);

        assert_eq!(
            stats.created, 3,
            "Should have created 2 more containers JIT"
        );
        assert_eq!(stats.max_active, 3);
        assert_eq!(stats.overproduction_waste(), 0);

        // Quantification
        tracing::info!("--- JIT AUDIT REPORT ---");
        tracing::info!("Target: Just-in-Time Creation");
        tracing::info!("Actual: Just-in-Time Creation");
        tracing::info!("Configured min_idle: {}", config.min_idle);
        tracing::info!("Containers Created: {}", stats.created);
        tracing::info!("Peak Concurrent Demand: {}", stats.max_active);
        tracing::info!(
            "Over-production Waste: {} containers",
            stats.overproduction_waste()
        );
        tracing::info!(
            "Waste Percentage: {:.1}%",
            (stats.overproduction_waste() as f64 / stats.created as f64) * 100.0
        );
        tracing::info!("------------------------");

        // Cleanup
        pool.shutdown().await.expect("Failed to shutdown pool");
    }
}
