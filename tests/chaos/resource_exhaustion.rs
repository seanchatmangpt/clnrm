//! Resource Exhaustion Tests
//!
//! Tests system behavior under memory pressure, CPU saturation,
//! disk space exhaustion, and file descriptor limits.

use clnrm_core::services::chaos_engine::{ChaosEnginePlugin, ChaosScenario};
use clnrm_core::error::Result;
use std::time::Instant;

/// Test memory exhaustion scenario
#[tokio::test]
async fn test_memory_exhaustion() -> Result<()> {
    let engine = ChaosEnginePlugin::new("memory_exhaustion");

    let scenario = ChaosScenario::MemoryExhaustion {
        duration_secs: 2,
        target_mb: 100,
    };

    let start = Instant::now();
    engine.run_scenario(&scenario).await?;
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() >= 2);
    println!("Memory exhaustion test completed in {}s", elapsed.as_secs());

    Ok(())
}

/// Test CPU saturation scenario
#[tokio::test]
async fn test_cpu_saturation() -> Result<()> {
    let engine = ChaosEnginePlugin::new("cpu_saturation");

    let scenario = ChaosScenario::CpuSaturation {
        duration_secs: 2,
        target_percent: 75,
    };

    let start = Instant::now();
    engine.run_scenario(&scenario).await?;
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() >= 2);
    println!("CPU saturation test completed in {}s", elapsed.as_secs());

    Ok(())
}

/// Test gradual memory pressure increase
#[tokio::test]
async fn test_gradual_memory_pressure() -> Result<()> {
    let engine = ChaosEnginePlugin::new("gradual_memory");

    // Simulate gradual memory pressure increase
    let memory_levels = vec![50, 100, 200, 400];

    for &mb in &memory_levels {
        let scenario = ChaosScenario::MemoryExhaustion {
            duration_secs: 1,
            target_mb: mb,
        };

        println!("Testing memory pressure: {}MB", mb);
        engine.run_scenario(&scenario).await?;
    }

    Ok(())
}

/// Test CPU spike patterns
#[tokio::test]
async fn test_cpu_spike_patterns() -> Result<()> {
    let engine = ChaosEnginePlugin::new("cpu_spikes");

    // Simulate CPU spikes at different intensity levels
    let cpu_levels = vec![25, 50, 75, 90];

    for &percent in &cpu_levels {
        let scenario = ChaosScenario::CpuSaturation {
            duration_secs: 1,
            target_percent: percent,
        };

        println!("Testing CPU saturation: {}%", percent);
        engine.run_scenario(&scenario).await?;
    }

    Ok(())
}

/// Test memory leak simulation
#[tokio::test]
async fn test_memory_leak_simulation() -> Result<()> {
    let engine = ChaosEnginePlugin::new("memory_leak");

    // Simulate progressive memory leak
    let mut total_allocated = 0;

    for i in 1..=5 {
        let scenario = ChaosScenario::MemoryExhaustion {
            duration_secs: 1,
            target_mb: i * 50,
        };

        total_allocated += i * 50;
        println!("Memory leak iteration {}: {} MB total", i, total_allocated);

        engine.run_scenario(&scenario).await?;
    }

    Ok(())
}

/// Test concurrent resource exhaustion
#[tokio::test]
async fn test_concurrent_resource_exhaustion() -> Result<()> {
    let engine = ChaosEnginePlugin::new("concurrent_exhaustion");

    // Run memory and CPU exhaustion concurrently
    let memory_scenario = ChaosScenario::MemoryExhaustion {
        duration_secs: 3,
        target_mb: 200,
    };

    let cpu_scenario = ChaosScenario::CpuSaturation {
        duration_secs: 3,
        target_percent: 70,
    };

    // Execute both scenarios
    let start = Instant::now();

    tokio::try_join!(
        engine.run_scenario(&memory_scenario),
        engine.run_scenario(&cpu_scenario)
    )?;

    let elapsed = start.elapsed();
    println!("Concurrent resource exhaustion completed in {}s", elapsed.as_secs());

    Ok(())
}

/// Test resource recovery after exhaustion
#[tokio::test]
async fn test_resource_recovery() -> Result<()> {
    let engine = ChaosEnginePlugin::new("resource_recovery");

    // Phase 1: Exhaust resources
    let exhaust_scenario = ChaosScenario::MemoryExhaustion {
        duration_secs: 2,
        target_mb: 300,
    };

    println!("Phase 1: Exhausting resources...");
    engine.run_scenario(&exhaust_scenario).await?;

    // Phase 2: Wait for recovery
    println!("Phase 2: Allowing recovery...");
    tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

    // Phase 3: Verify system is responsive
    println!("Phase 3: Verifying recovery...");
    let test_scenario = ChaosScenario::RandomFailures {
        duration_secs: 1,
        failure_rate: 0.1,
    };
    engine.run_scenario(&test_scenario).await?;

    println!("Resource recovery test completed successfully");

    Ok(())
}

/// Test thrashing scenario (rapid allocation/deallocation)
#[tokio::test]
async fn test_thrashing_scenario() -> Result<()> {
    let engine = ChaosEnginePlugin::new("thrashing");

    // Rapidly alternate between high and low resource usage
    for i in 0..5 {
        let high_memory = ChaosScenario::MemoryExhaustion {
            duration_secs: 1,
            target_mb: 400,
        };

        println!("Thrashing iteration {}: High memory", i);
        engine.run_scenario(&high_memory).await?;

        // Brief recovery period
        tokio::time::sleep(tokio::time::Duration::from_millis(500)).await;
    }

    Ok(())
}

/// Test OOM (Out of Memory) simulation
#[tokio::test]
#[ignore] // Potentially dangerous test - run manually
async fn test_oom_simulation() -> Result<()> {
    let engine = ChaosEnginePlugin::new("oom_test");

    // WARNING: This test attempts to allocate very large amounts of memory
    let scenario = ChaosScenario::MemoryExhaustion {
        duration_secs: 1,
        target_mb: 2048, // 2GB
    };

    // This should either succeed or fail gracefully
    match engine.run_scenario(&scenario).await {
        Ok(_) => println!("OOM test completed without crashing"),
        Err(e) => println!("OOM test failed gracefully: {}", e),
    }

    Ok(())
}

/// Test disk space exhaustion simulation
#[tokio::test]
async fn test_disk_exhaustion() -> Result<()> {
    use std::fs::{File, create_dir_all};
    use std::io::Write;
    use tempfile::tempdir;

    let temp_dir = tempdir()?;
    let test_path = temp_dir.path().join("disk_test");
    create_dir_all(&test_path)?;

    // Simulate disk filling up
    let file_count = 100;
    let file_size = 1024 * 10; // 10KB per file

    println!("Simulating disk exhaustion with {} files", file_count);

    for i in 0..file_count {
        let file_path = test_path.join(format!("test_file_{}.dat", i));
        let mut file = File::create(file_path)?;
        let data = vec![0u8; file_size];
        file.write_all(&data)?;
    }

    println!("Created {} files totaling {}KB",
        file_count, (file_count * file_size) / 1024);

    // Cleanup is automatic when temp_dir goes out of scope
    Ok(())
}

/// Test file descriptor exhaustion
#[tokio::test]
async fn test_file_descriptor_exhaustion() -> Result<()> {
    use std::fs::File;
    use tempfile::tempdir;

    let temp_dir = tempdir()?;
    let mut files = Vec::new();

    // Try to open many files simultaneously
    let max_files = 100;

    println!("Testing file descriptor limits with {} files", max_files);

    for i in 0..max_files {
        let file_path = temp_dir.path().join(format!("fd_test_{}.txt", i));
        match File::create(&file_path) {
            Ok(f) => files.push(f),
            Err(e) => {
                println!("Reached file descriptor limit at {} files: {}", i, e);
                break;
            }
        }
    }

    println!("Successfully opened {} files", files.len());

    // Files are automatically closed when vector goes out of scope
    Ok(())
}

#[cfg(test)]
mod stress_tests {
    use super::*;

    /// Test sustained resource pressure
    #[tokio::test]
    #[ignore] // Long-running test
    async fn test_sustained_resource_pressure() -> Result<()> {
        let engine = ChaosEnginePlugin::new("sustained_pressure");

        let duration_secs = 30; // 30 seconds of sustained pressure

        let scenario = ChaosScenario::CpuSaturation {
            duration_secs,
            target_percent: 60,
        };

        println!("Running sustained resource pressure for {}s", duration_secs);

        let start = Instant::now();
        engine.run_scenario(&scenario).await?;
        let elapsed = start.elapsed();

        println!("Sustained pressure test completed after {}s", elapsed.as_secs());
        assert!(elapsed.as_secs() >= duration_secs);

        Ok(())
    }
}
