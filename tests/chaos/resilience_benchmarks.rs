//! Resilience Benchmarking Suite
//!
//! Benchmarks to measure system resilience metrics including
//! recovery time, throughput under chaos, error rates, and stability.

use clnrm_core::services::chaos_engine::{ChaosEnginePlugin, ChaosScenario};
use clnrm_core::error::Result;
use std::time::{Duration, Instant};

/// Resilience metrics
#[derive(Debug, Clone)]
struct ResilienceMetrics {
    total_operations: usize,
    successful_operations: usize,
    failed_operations: usize,
    avg_latency_ms: f64,
    max_latency_ms: u64,
    recovery_time_ms: u64,
    throughput_ops_per_sec: f64,
}

impl ResilienceMetrics {
    fn new() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            avg_latency_ms: 0.0,
            max_latency_ms: 0,
            recovery_time_ms: 0,
            throughput_ops_per_sec: 0.0,
        }
    }

    fn calculate_success_rate(&self) -> f64 {
        if self.total_operations == 0 {
            return 0.0;
        }
        (self.successful_operations as f64 / self.total_operations as f64) * 100.0
    }

    fn print(&self) {
        println!("\n=== Resilience Metrics ===");
        println!("Total operations: {}", self.total_operations);
        println!("Successful: {} ({:.2}%)", self.successful_operations, self.calculate_success_rate());
        println!("Failed: {}", self.failed_operations);
        println!("Avg latency: {:.2}ms", self.avg_latency_ms);
        println!("Max latency: {}ms", self.max_latency_ms);
        println!("Recovery time: {}ms", self.recovery_time_ms);
        println!("Throughput: {:.2} ops/sec", self.throughput_ops_per_sec);
        println!("==========================\n");
    }
}

/// Benchmark baseline performance
#[tokio::test]
async fn benchmark_baseline_performance() -> Result<()> {
    let engine = ChaosEnginePlugin::new("baseline");

    let iterations = 1000;
    let start = Instant::now();
    let mut latencies = Vec::new();

    for _ in 0..iterations {
        let op_start = Instant::now();

        // Simulate operation
        tokio::time::sleep(Duration::from_micros(100)).await;

        latencies.push(op_start.elapsed().as_micros() as u64);
    }

    let total_duration = start.elapsed();

    let mut metrics = ResilienceMetrics::new();
    metrics.total_operations = iterations;
    metrics.successful_operations = iterations;
    metrics.avg_latency_ms = latencies.iter().sum::<u64>() as f64 / (iterations as f64 * 1000.0);
    metrics.max_latency_ms = *latencies.iter().max().unwrap() / 1000;
    metrics.throughput_ops_per_sec = iterations as f64 / total_duration.as_secs_f64();

    println!("Baseline Performance Benchmark:");
    metrics.print();

    Ok(())
}

/// Benchmark performance under network chaos
#[tokio::test]
async fn benchmark_network_chaos_resilience() -> Result<()> {
    let engine = ChaosEnginePlugin::new("network_bench")
        .with_latency(100)
        .with_failure_rate(0.2);

    let iterations = 500;
    let mut latencies = Vec::new();
    let start = Instant::now();
    let mut successful = 0;
    let mut failed = 0;

    for i in 0..iterations {
        let op_start = Instant::now();

        // Inject chaos
        if engine.inject_failure(&format!("op_{}", i)).await? {
            failed += 1;
            latencies.push(op_start.elapsed().as_micros() as u64);
            continue;
        }

        let latency = engine.inject_latency(&format!("op_{}", i)).await?;

        successful += 1;
        latencies.push(op_start.elapsed().as_micros() as u64);
    }

    let total_duration = start.elapsed();

    let mut metrics = ResilienceMetrics::new();
    metrics.total_operations = iterations;
    metrics.successful_operations = successful;
    metrics.failed_operations = failed;
    metrics.avg_latency_ms = latencies.iter().sum::<u64>() as f64 / (iterations as f64 * 1000.0);
    metrics.max_latency_ms = *latencies.iter().max().unwrap() / 1000;
    metrics.throughput_ops_per_sec = successful as f64 / total_duration.as_secs_f64();

    println!("Network Chaos Resilience Benchmark:");
    metrics.print();

    assert!(metrics.calculate_success_rate() >= 70.0,
        "Success rate should be at least 70% under network chaos");

    Ok(())
}

/// Benchmark recovery time after failure
#[tokio::test]
async fn benchmark_recovery_time() -> Result<()> {
    let engine = ChaosEnginePlugin::new("recovery_bench");

    let mut recovery_times = Vec::new();

    for iteration in 0..10 {
        // Inject failure
        let scenario = ChaosScenario::RandomFailures {
            duration_secs: 1,
            failure_rate: 0.9,
        };

        println!("Iteration {}: Injecting failure...", iteration);
        engine.run_scenario(&scenario).await?;

        // Measure recovery time
        let recovery_start = Instant::now();

        // Attempt recovery
        let mut recovered = false;
        while recovery_start.elapsed() < Duration::from_secs(5) {
            if !engine.inject_failure(&format!("recovery_{}", iteration)).await? {
                recovered = true;
                break;
            }
            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        let recovery_time = recovery_start.elapsed();

        if recovered {
            println!("Iteration {}: Recovered in {:?}", iteration, recovery_time);
            recovery_times.push(recovery_time.as_millis() as u64);
        } else {
            println!("Iteration {}: Failed to recover", iteration);
        }
    }

    let avg_recovery = recovery_times.iter().sum::<u64>() as f64 / recovery_times.len() as f64;
    let max_recovery = *recovery_times.iter().max().unwrap();

    println!("\nRecovery Time Benchmark:");
    println!("  Avg recovery time: {:.2}ms", avg_recovery);
    println!("  Max recovery time: {}ms", max_recovery);
    println!("  Successful recoveries: {}/10", recovery_times.len());

    assert!(avg_recovery < 1000.0,
        "Average recovery time should be under 1 second");

    Ok(())
}

/// Benchmark throughput degradation under chaos
#[tokio::test]
async fn benchmark_throughput_degradation() -> Result<()> {
    let chaos_levels = vec![
        (0.0, "No chaos"),
        (0.2, "Light chaos (20%)"),
        (0.5, "Moderate chaos (50%)"),
        (0.8, "Heavy chaos (80%)"),
    ];

    println!("\nThroughput Degradation Benchmark:");

    for (failure_rate, label) in chaos_levels {
        let engine = ChaosEnginePlugin::new("throughput_bench")
            .with_failure_rate(failure_rate);

        let duration = Duration::from_secs(2);
        let start = Instant::now();
        let mut operations = 0;
        let mut successful = 0;

        while start.elapsed() < duration {
            operations += 1;

            if !engine.inject_failure(&format!("op_{}", operations)).await? {
                successful += 1;
            }

            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        let throughput = successful as f64 / start.elapsed().as_secs_f64();
        let success_rate = (successful as f64 / operations as f64) * 100.0;

        println!("\n  {} - Failure rate: {:.0}%", label, failure_rate * 100.0);
        println!("    Throughput: {:.2} ops/sec", throughput);
        println!("    Success rate: {:.1}%", success_rate);
    }

    Ok(())
}

/// Benchmark resource exhaustion impact
#[tokio::test]
async fn benchmark_resource_exhaustion_impact() -> Result<()> {
    let engine = ChaosEnginePlugin::new("resource_bench");

    let resource_scenarios = vec![
        ChaosScenario::MemoryExhaustion { duration_secs: 2, target_mb: 100 },
        ChaosScenario::CpuSaturation { duration_secs: 2, target_percent: 70 },
    ];

    println!("\nResource Exhaustion Impact Benchmark:");

    for scenario in resource_scenarios {
        let scenario_name = format!("{:?}", scenario);

        // Measure baseline
        let baseline_start = Instant::now();
        let mut baseline_ops = 0;

        while baseline_start.elapsed() < Duration::from_secs(1) {
            baseline_ops += 1;
            tokio::time::sleep(Duration::from_micros(100)).await;
        }

        // Measure under chaos
        let chaos_task = tokio::spawn({
            let engine = engine.clone();
            let scenario = scenario.clone();
            async move {
                engine.run_scenario(&scenario).await
            }
        });

        tokio::time::sleep(Duration::from_millis(100)).await; // Let chaos start

        let chaos_start = Instant::now();
        let mut chaos_ops = 0;

        while chaos_start.elapsed() < Duration::from_secs(1) {
            chaos_ops += 1;
            tokio::time::sleep(Duration::from_micros(100)).await;
        }

        chaos_task.await.unwrap()?;

        let degradation = ((baseline_ops - chaos_ops) as f64 / baseline_ops as f64) * 100.0;

        println!("\n  Scenario: {}", scenario_name);
        println!("    Baseline: {} ops", baseline_ops);
        println!("    Under chaos: {} ops", chaos_ops);
        println!("    Degradation: {:.1}%", degradation);
    }

    Ok(())
}

/// Benchmark cascading failure propagation
#[tokio::test]
async fn benchmark_cascading_failure_propagation() -> Result<()> {
    let engine = ChaosEnginePlugin::new("cascade_bench");

    let propagation_delays = vec![10, 50, 100, 200, 500];

    println!("\nCascading Failure Propagation Benchmark:");

    for delay_ms in propagation_delays {
        let scenario = ChaosScenario::CascadingFailures {
            trigger_service: "root".to_string(),
            propagation_delay_ms: delay_ms,
        };

        let start = Instant::now();
        engine.run_scenario(&scenario).await?;
        let total_time = start.elapsed();

        let metrics = engine.get_metrics().await;

        println!("\n  Propagation delay: {}ms", delay_ms);
        println!("    Total time: {:?}", total_time);
        println!("    Services affected: {}", metrics.affected_services.len());
        println!("    Failures injected: {}", metrics.failures_injected);
    }

    Ok(())
}

/// Benchmark concurrent chaos scenarios
#[tokio::test]
async fn benchmark_concurrent_chaos() -> Result<()> {
    let engine = ChaosEnginePlugin::new("concurrent_bench")
        .with_failure_rate(0.3)
        .with_latency(100);

    let concurrent_levels = vec![1, 5, 10, 20, 50];

    println!("\nConcurrent Chaos Benchmark:");

    for concurrency in concurrent_levels {
        let mut tasks = vec![];
        let start = Instant::now();

        for i in 0..concurrency {
            let engine = engine.clone();
            let task = tokio::spawn(async move {
                let mut ops = 0;
                let mut successful = 0;

                for j in 0..20 {
                    ops += 1;

                    if !engine.inject_failure(&format!("task_{}_{}", i, j)).await.unwrap_or(true) {
                        engine.inject_latency(&format!("task_{}_{}", i, j)).await.ok();
                        successful += 1;
                    }
                }

                (ops, successful)
            });

            tasks.push(task);
        }

        let results = futures_util::future::join_all(tasks).await;

        let total_ops: usize = results.iter().map(|r| r.as_ref().unwrap().0).sum();
        let total_successful: usize = results.iter().map(|r| r.as_ref().unwrap().1).sum();
        let duration = start.elapsed();

        let throughput = total_successful as f64 / duration.as_secs_f64();
        let success_rate = (total_successful as f64 / total_ops as f64) * 100.0;

        println!("\n  Concurrency: {}", concurrency);
        println!("    Operations: {}", total_ops);
        println!("    Successful: {} ({:.1}%)", total_successful, success_rate);
        println!("    Throughput: {:.2} ops/sec", throughput);
        println!("    Duration: {:?}", duration);
    }

    Ok(())
}

/// Benchmark system stability over time
#[tokio::test]
#[ignore] // Long-running test
async fn benchmark_stability_over_time() -> Result<()> {
    let engine = ChaosEnginePlugin::new("stability_bench")
        .with_failure_rate(0.3);

    let test_duration = Duration::from_secs(30);
    let sample_interval = Duration::from_secs(5);

    println!("\nStability Over Time Benchmark:");
    println!("Duration: {:?}", test_duration);
    println!("Sample interval: {:?}\n", sample_interval);

    let start = Instant::now();
    let mut samples = Vec::new();

    while start.elapsed() < test_duration {
        let sample_start = Instant::now();
        let mut ops = 0;
        let mut successful = 0;

        while sample_start.elapsed() < sample_interval {
            ops += 1;

            if !engine.inject_failure(&format!("op_{}", ops)).await? {
                successful += 1;
            }

            tokio::time::sleep(Duration::from_millis(50)).await;
        }

        let success_rate = (successful as f64 / ops as f64) * 100.0;
        samples.push(success_rate);

        println!("Sample {}: {:.1}% success rate ({}/{})",
            samples.len(), success_rate, successful, ops);
    }

    let avg_success_rate = samples.iter().sum::<f64>() / samples.len() as f64;
    let stability_variance = samples.iter()
        .map(|&x| (x - avg_success_rate).powi(2))
        .sum::<f64>() / samples.len() as f64;

    println!("\nStability Metrics:");
    println!("  Average success rate: {:.1}%", avg_success_rate);
    println!("  Variance: {:.2}", stability_variance);
    println!("  Samples: {}", samples.len());

    assert!(stability_variance < 100.0,
        "System should maintain stable performance");

    Ok(())
}
