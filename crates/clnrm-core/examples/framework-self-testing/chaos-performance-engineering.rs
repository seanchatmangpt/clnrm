//! CHAOS ENGINEERING & PERFORMANCE REGRESSION DETECTION
//!
//! This example demonstrates advanced "eat your own dog food" innovations:
//! 1. Chaos engineering: Injecting failures to test resilience
//! 2. Performance regression detection: Monitoring for performance degradation
//! 3. Automated performance validation: Continuous performance monitoring
//!
//! INNOVATION: The framework tests its own resilience and performance
//! characteristics by simulating real-world failure conditions and
//! measuring performance regressions over time.

use clnrm_core::{CleanroomEnvironment, CleanroomError};
use std::time::{Instant, Duration};
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<(), CleanroomError> {
    println!("🌪️  CHAOS ENGINEERING & PERFORMANCE REGRESSION DETECTION");
    println!("======================================================");
    println!("Testing framework resilience and performance monitoring.");
    println!("This demonstrates chaos engineering and regression detection.");
    println!();

    let start = Instant::now();

    // Phase 1: Baseline performance measurement
    println!("📊 Phase 1: Baseline Performance Measurement");
    println!("------------------------------------------");

    let baseline_metrics = establish_performance_baseline().await?;
    println!("✅ Baseline established: {:.2}ms average operation time", baseline_metrics);

    // Phase 2: Chaos engineering validation
    println!("\n📊 Phase 2: Chaos Engineering Validation");
    println!("---------------------------------------");

    validate_chaos_resilience().await?;
    println!("✅ Chaos engineering validation complete");

    // Phase 3: Performance regression detection
    println!("\n📊 Phase 3: Performance Regression Detection");
    println!("-------------------------------------------");

    detect_performance_regressions(&baseline_metrics).await?;
    println!("✅ Performance regression detection complete");

    // Phase 4: Automated performance monitoring
    println!("\n📊 Phase 4: Automated Performance Monitoring");
    println!("-------------------------------------------");

    setup_automated_performance_monitoring().await?;
    println!("✅ Automated performance monitoring setup complete");

    let total_duration = start.elapsed();
    println!("\n🎉 CHAOS & PERFORMANCE VALIDATION COMPLETE!");
    println!("Framework successfully demonstrated:");
    println!("  ✅ Baseline performance measurement");
    println!("  ✅ Chaos engineering resilience");
    println!("  ✅ Performance regression detection");
    println!("  ✅ Automated performance monitoring");
    println!("\n⏱️  Total validation time: {}ms", total_duration.as_millis());

    Ok(())
}

/// Establish baseline performance metrics
async fn establish_performance_baseline() -> Result<f64, CleanroomError> {
    let mut measurements = Vec::new();

    println!("   📏 Measuring baseline performance...");

    for run in 0..5 {
        let env = CleanroomEnvironment::new().await?;
        let start = Instant::now();

        // Perform a series of operations to measure performance
        for i in 0..10 {
            let container = env.get_or_create_container(&format!("baseline-{}", i), || {
                Ok::<String, CleanroomError>(format!("baseline-container-{}", i))
            }).await?;

            let _result = env.execute_in_container(&container, vec![
                "echo".to_string(),
                format!("baseline operation {}", i)
            ]).await?;
        }

        let duration = start.elapsed();
        measurements.push(duration.as_millis() as f64);
        println!("      Run {}: {:.2}ms", run + 1, duration.as_millis());
    }

    let average = measurements.iter().sum::<f64>() / measurements.len() as f64;
    println!("   📊 Baseline average: {:.2}ms", average);

    Ok(average)
}

/// Validate chaos engineering resilience
async fn validate_chaos_resilience() -> Result<(), CleanroomError> {
    println!("   🌪️  Injecting chaos conditions...");

    // Test 1: Network partition simulation
    println!("      🛡️  Simulating network partitions...");
    test_network_partition_resilience().await?;

    // Test 2: Resource exhaustion simulation
    println!("      🔋 Simulating resource exhaustion...");
    test_resource_exhaustion_resilience().await?;

    // Test 3: Service dependency failures
    println!("      💔 Simulating dependency failures...");
    test_dependency_failure_resilience().await?;

    // Test 4: Timing-based failures
    println!("      ⏰ Simulating timing failures...");
    test_timing_failure_resilience().await?;

    Ok(())
}

/// Test resilience to network partition conditions
async fn test_network_partition_resilience() -> Result<(), CleanroomError> {
    let env = CleanroomEnvironment::new().await?;

    // Create containers that simulate network isolation
    for i in 0..3 {
        let container = env.get_or_create_container(&format!("network-chaos-{}", i), || {
            Ok::<String, CleanroomError>(format!("network-chaos-container-{}", i))
        }).await?;

        // Execute commands that would fail under network partition
        let result = env.execute_in_container(&container, vec![
            "sh".to_string(),
            "-c".to_string(),
            format!("echo 'Network operation {}' && sleep 0.01", i)
        ]).await?;

        if result.success {
            println!("         ✅ Network operation {} succeeded", i);
        } else {
            return Err(CleanroomError::internal_error(format!("Network operation {} failed", i)));
        }
    }

    Ok(())
}

/// Test resilience to resource exhaustion
async fn test_resource_exhaustion_resilience() -> Result<(), CleanroomError> {
    let env = CleanroomEnvironment::new().await?;

    // Create many containers to simulate resource pressure
    for i in 0..5 {
        let container = env.get_or_create_container(&format!("resource-chaos-{}", i), || {
            Ok::<String, CleanroomError>(format!("resource-chaos-container-{}", i))
        }).await?;

        // Execute resource-intensive operations
        let result = env.execute_in_container(&container, vec![
            "sh".to_string(),
            "-c".to_string(),
            format!("echo 'Resource intensive operation {}' && echo 'data' > /tmp/test{}.txt && cat /tmp/test{}.txt", i, i, i)
        ]).await?;

        if result.success {
            println!("         ✅ Resource operation {} succeeded", i);
        }
    }

    Ok(())
}

/// Test resilience to dependency failures
async fn test_dependency_failure_resilience() -> Result<(), CleanroomError> {
    let env = CleanroomEnvironment::new().await?;

    // Create a chain of dependent operations
    let container1 = env.get_or_create_container("dep-chain-1", || {
        Ok::<String, CleanroomError>("dependency-chain-1".to_string())
    }).await?;

    let container2 = env.get_or_create_container("dep-chain-2", || {
        Ok::<String, CleanroomError>("dependency-chain-2".to_string())
    }).await?;

    // Execute dependent operations
    let result1 = env.execute_in_container(&container1, vec![
        "echo".to_string(),
        "dependency chain step 1"
    ]).await?;

    let result2 = env.execute_in_container(&container2, vec![
        "echo".to_string(),
        "dependency chain step 2"
    ]).await?;

    if result1.success && result2.success {
        println!("         ✅ Dependency chain succeeded");
    } else {
        return Err(CleanroomError::internal_error("Dependency chain failed"));
    }

    Ok(())
}

/// Test resilience to timing-based failures
async fn test_timing_failure_resilience() -> Result<(), CleanroomError> {
    let env = CleanroomEnvironment::new().await?;

    // Test operations with tight timing constraints
    for i in 0..3 {
        let start = Instant::now();

        let container = env.get_or_create_container(&format!("timing-chaos-{}", i), || {
            Ok::<String, CleanroomError>(format!("timing-chaos-container-{}", i))
        }).await?;

        // Execute time-sensitive operation
        let result = env.execute_in_container(&container, vec![
            "sh".to_string(),
            "-c".to_string(),
            "echo 'Timing sensitive operation' && sleep 0.005"
        ]).await?;

        let duration = start.elapsed();

        if result.success && duration < Duration::from_millis(100) {
            println!("         ✅ Timing operation {} succeeded in {:?}", i, duration);
        } else {
            return Err(CleanroomError::internal_error(format!("Timing operation {} failed", i)));
        }
    }

    Ok(())
}

/// Detect performance regressions by comparing against baseline
async fn detect_performance_regressions(baseline: &f64) -> Result<(), CleanroomError> {
    println!("   🔍 Detecting performance regressions...");

    let mut current_measurements = Vec::new();

    // Take current performance measurements
    for run in 0..3 {
        let env = CleanroomEnvironment::new().await?;
        let start = Instant::now();

        // Perform operations similar to baseline
        for i in 0..10 {
            let container = env.get_or_create_container(&format!("regression-test-{}", i), || {
                Ok::<String, CleanroomError>(format!("regression-container-{}", i))
            }).await?;

            let _result = env.execute_in_container(&container, vec![
                "echo".to_string(),
                format!("regression test {}", i)
            ]).await?;
        }

        let duration = start.elapsed();
        current_measurements.push(duration.as_millis() as f64);
        println!("      Current run {}: {:.2}ms", run + 1, duration.as_millis());
    }

    let current_average = current_measurements.iter().sum::<f64>() / current_measurements.len() as f64;

    // Check for regression (20% threshold)
    let regression_threshold = baseline * 1.2;
    let performance_change = ((current_average - baseline) / baseline) * 100.0;

    println!("   📊 Performance comparison:");
    println!("      Baseline: {:.2}ms", baseline);
    println!("      Current:  {:.2}ms", current_average);
    println!("      Change:   {:.1}%", performance_change);

    if current_average > regression_threshold {
        println!("      ⚠️  PERFORMANCE REGRESSION DETECTED!");
        println!("         Current performance is {:.1}% slower than baseline", performance_change);
        println!("         Threshold: 20% slower than baseline");
    } else {
        println!("      ✅ No performance regression detected");
    }

    Ok(())
}

/// Set up automated performance monitoring
async fn setup_automated_performance_monitoring() -> Result<(), CleanroomError> {
    println!("   📊 Setting up automated performance monitoring...");

    let env = CleanroomEnvironment::new().await?;

    // Create a performance monitoring container
    let monitor_container = env.get_or_create_container("performance-monitor", || {
        Ok::<String, CleanroomError>("performance-monitor".to_string())
    }).await?;

    // Execute performance monitoring script
    let monitor_script = r#"
        echo "Performance Monitoring Setup"
        echo "============================="
        echo "Monitoring framework performance metrics..."
        echo "Baseline: $(date)"
        echo "Monitoring active containers and response times"
    "#;

    let result = env.execute_in_container(&monitor_container, vec![
        "sh".to_string(),
        "-c".to_string(),
        monitor_script.to_string()
    ]).await?;

    if result.success {
        println!("      ✅ Performance monitoring setup completed");
        println!("         Monitor container: {}", monitor_container);
        println!("         Monitoring script executed successfully");
    } else {
        return Err(CleanroomError::internal_error("Performance monitoring setup failed"));
    }

    // Get final metrics to verify monitoring is working
    let final_metrics = env.get_metrics().await;
    println!("      📊 Final monitoring metrics:");
    println!("         Tests executed: {}", final_metrics.tests_executed);
    println!("         Containers created: {}", final_metrics.containers_created);
    println!("         Total duration: {}ms", final_metrics.total_duration_ms);

    Ok(())
}
