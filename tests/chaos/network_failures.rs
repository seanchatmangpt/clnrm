//! Network Failure and Latency Injection Tests
//!
//! Tests system resilience against network failures, partitions,
//! latency spikes, packet loss, and connection timeouts.

use clnrm_core::services::chaos_engine::{ChaosEnginePlugin, ChaosScenario};
use clnrm_core::error::Result;
use std::time::{Duration, Instant};
use tokio::time::sleep;

/// Test network latency injection
#[tokio::test]
async fn test_network_latency_injection() -> Result<()> {
    let engine = ChaosEnginePlugin::new("latency_test")
        .with_latency(500);

    // Inject latency and measure
    let start = Instant::now();
    let latency = engine.inject_latency("test_service").await?;
    let elapsed = start.elapsed();

    // Verify latency was injected (if random triggered it)
    if latency > 0 {
        assert!(elapsed.as_millis() >= latency as u128);
        println!("Network latency injected: {}ms, actual: {}ms", latency, elapsed.as_millis());
    }

    Ok(())
}

/// Test latency spike scenarios
#[tokio::test]
async fn test_latency_spikes() -> Result<()> {
    let engine = ChaosEnginePlugin::new("latency_spikes");

    let scenario = ChaosScenario::LatencySpikes {
        duration_secs: 3,
        max_latency_ms: 1000,
    };

    let start = Instant::now();
    engine.run_scenario(&scenario).await?;
    let elapsed = start.elapsed();

    // Should take at least the duration specified
    assert!(elapsed.as_secs() >= 3);

    let metrics = engine.get_metrics().await;
    println!("Latency spikes - Total latency injected: {}ms", metrics.latency_injected_ms);

    Ok(())
}

/// Test network partition creation
#[tokio::test]
async fn test_network_partition() -> Result<()> {
    let engine = ChaosEnginePlugin::new("network_partition");

    let services = vec![
        "service_a".to_string(),
        "service_b".to_string(),
        "service_c".to_string(),
    ];

    engine.create_network_partition(&services).await?;

    let metrics = engine.get_metrics().await;

    // Verify partition might have been created (based on probability)
    if metrics.network_partitions > 0 {
        assert!(metrics.affected_services.len() >= 3);
        println!("Network partition created affecting {} services", metrics.affected_services.len());
    }

    Ok(())
}

/// Test network partition scenario with duration
#[tokio::test]
async fn test_network_partition_scenario() -> Result<()> {
    let engine = ChaosEnginePlugin::new("partition_scenario");

    let scenario = ChaosScenario::NetworkPartition {
        duration_secs: 2,
        affected_services: vec![
            "database".to_string(),
            "cache".to_string(),
            "api".to_string(),
        ],
    };

    let start = Instant::now();
    engine.run_scenario(&scenario).await?;
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() >= 2);

    let metrics = engine.get_metrics().await;
    assert!(metrics.network_partitions >= 1);
    assert!(metrics.affected_services.contains(&"database".to_string()));

    println!("Network partition scenario completed - Partitions: {}", metrics.network_partitions);

    Ok(())
}

/// Test cascading network failures
#[tokio::test]
async fn test_cascading_network_failures() -> Result<()> {
    let engine = ChaosEnginePlugin::new("cascading_failures");

    let scenario = ChaosScenario::CascadingFailures {
        trigger_service: "load_balancer".to_string(),
        propagation_delay_ms: 100,
    };

    let start = Instant::now();
    engine.run_scenario(&scenario).await?;
    let elapsed = start.elapsed();

    // Should take at least the propagation delay
    assert!(elapsed.as_millis() >= 100);

    let metrics = engine.get_metrics().await;

    // Cascading failures should affect multiple services
    assert!(metrics.failures_injected >= 1);
    assert!(metrics.affected_services.len() > 1);

    println!("Cascading failures - Services affected: {}", metrics.affected_services.len());

    Ok(())
}

/// Test network timeout simulation
#[tokio::test]
async fn test_network_timeout_simulation() -> Result<()> {
    let engine = ChaosEnginePlugin::new("timeout_test")
        .with_latency(2000);

    // Simulate operation with timeout
    let timeout_duration = Duration::from_secs(1);
    let operation_start = Instant::now();

    let result = tokio::time::timeout(
        timeout_duration,
        engine.inject_latency("slow_service")
    ).await;

    // Should timeout if latency was injected and exceeded timeout
    if result.is_err() {
        println!("Network timeout triggered successfully");
        assert!(operation_start.elapsed() >= timeout_duration);
    }

    Ok(())
}

/// Test intermittent network failures
#[tokio::test]
async fn test_intermittent_network_failures() -> Result<()> {
    let engine = ChaosEnginePlugin::new("intermittent_failures")
        .with_failure_rate(0.5);

    let mut failures = 0;
    let iterations = 20;

    for i in 0..iterations {
        if engine.inject_failure(&format!("service_{}", i)).await? {
            failures += 1;
        }
    }

    // With 50% failure rate, we should see some failures
    println!("Intermittent failures: {}/{} iterations", failures, iterations);
    assert!(failures > 0 && failures < iterations);

    Ok(())
}

/// Test connection pool exhaustion
#[tokio::test]
async fn test_connection_pool_exhaustion() -> Result<()> {
    let engine = ChaosEnginePlugin::new("connection_pool")
        .with_failure_rate(0.8);

    // Simulate concurrent connections
    let mut tasks = vec![];

    for i in 0..50 {
        let engine_clone = engine.clone();
        let task = tokio::spawn(async move {
            engine_clone.inject_failure(&format!("connection_{}", i)).await
        });
        tasks.push(task);
    }

    // Wait for all connections
    let results = futures_util::future::join_all(tasks).await;

    let failures: usize = results.iter()
        .filter_map(|r| r.as_ref().ok())
        .filter_map(|r| r.as_ref().ok())
        .filter(|&&failed| failed)
        .count();

    println!("Connection pool exhaustion - {} out of 50 failed", failures);
    assert!(failures > 0);

    Ok(())
}

/// Test DNS resolution failures
#[tokio::test]
async fn test_dns_resolution_failures() -> Result<()> {
    let engine = ChaosEnginePlugin::new("dns_failures")
        .with_failure_rate(0.3);

    let dns_services = vec!["auth.service", "db.service", "cache.service"];
    let mut failed_resolutions = 0;

    for service in &dns_services {
        if engine.inject_failure(service).await? {
            failed_resolutions += 1;
        }
    }

    println!("DNS resolution failures: {}/{}", failed_resolutions, dns_services.len());

    Ok(())
}

/// Test split-brain network scenario
#[tokio::test]
async fn test_split_brain_scenario() -> Result<()> {
    let engine = ChaosEnginePlugin::new("split_brain");

    // Partition network into two groups
    let group_a = vec!["node_1".to_string(), "node_2".to_string()];
    let group_b = vec!["node_3".to_string(), "node_4".to_string()];

    engine.create_network_partition(&group_a).await?;
    sleep(Duration::from_millis(100)).await;
    engine.create_network_partition(&group_b).await?;

    let metrics = engine.get_metrics().await;

    if metrics.network_partitions > 0 {
        println!("Split-brain scenario - Partitions created: {}", metrics.network_partitions);
        assert!(metrics.affected_services.len() >= 2);
    }

    Ok(())
}
