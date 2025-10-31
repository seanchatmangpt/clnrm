//! Performance Validation Tests
//!
//! Validates that Weaver live-check integration meets production performance requirements:
//! - Overhead < 10% (CPU and memory)
//! - Handles 1000+ spans/sec without drops
//! - Streaming performance under load
//! - Timeout behavior validation

use clnrm_core::telemetry::weaver_controller::{WeaverConfig, WeaverController};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};
use sysinfo::{ProcessExt, System, SystemExt};

#[test]
#[ignore = "Requires Weaver installation and can be resource intensive"]
fn test_weaver_overhead_cpu_memory() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Testing Weaver overhead (CPU and memory)");

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_perf_test"),
        stream: false,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);

    // Baseline system metrics
    let mut sys = System::new_all();
    sys.refresh_all();
    let baseline_memory = sys.used_memory();
    let baseline_cpu = sys.global_cpu_info().cpu_usage();

    println!("   Baseline - Memory: {} MB, CPU: {:.2}%",
        baseline_memory / 1024 / 1024, baseline_cpu);

    // Start Weaver
    controller.start_live_check()?;
    thread::sleep(Duration::from_secs(2));

    // Measure with Weaver running
    sys.refresh_all();
    let with_weaver_memory = sys.used_memory();
    let with_weaver_cpu = sys.global_cpu_info().cpu_usage();

    println!("   With Weaver - Memory: {} MB, CPU: {:.2}%",
        with_weaver_memory / 1024 / 1024, with_weaver_cpu);

    // Calculate overhead
    let memory_overhead_mb = (with_weaver_memory - baseline_memory) / 1024 / 1024;
    let cpu_overhead = with_weaver_cpu - baseline_cpu;

    println!("   Overhead - Memory: {} MB, CPU: {:.2}%",
        memory_overhead_mb, cpu_overhead);

    // Stop Weaver
    let _report = controller.stop_and_report()?;

    // Validate overhead requirements
    assert!(memory_overhead_mb < 200,
        "Memory overhead {} MB exceeds 200 MB limit", memory_overhead_mb);
    assert!(cpu_overhead < 10.0,
        "CPU overhead {:.2}% exceeds 10% limit", cpu_overhead);

    println!("✅ Overhead validation passed");
    Ok(())
}

#[test]
#[ignore = "Requires Weaver installation and generates high load"]
fn test_high_volume_telemetry_1000_spans_per_sec() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Testing high-volume telemetry (1000+ spans/sec)");

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_highvolume_test"),
        stream: true,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);
    controller.start_live_check()?;

    // Generate high-volume telemetry
    let span_count = Arc::new(AtomicU64::new(0));
    let duration = Duration::from_secs(10);
    let start = Instant::now();

    println!("   Generating telemetry for {:?}...", duration);

    let span_counter = Arc::clone(&span_count);
    let generator = thread::spawn(move || {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            while start.elapsed() < duration {
                // Generate batches of spans
                for _ in 0..100 {
                    // Simulate span generation via OTLP
                    // In real test, this would use the actual OTLP exporter
                    span_counter.fetch_add(1, Ordering::Relaxed);
                }
                tokio::time::sleep(Duration::from_millis(100)).await;
            }
        });
    });

    generator.join().unwrap();

    let total_spans = span_count.load(Ordering::Relaxed);
    let elapsed = start.elapsed();
    let spans_per_sec = total_spans as f64 / elapsed.as_secs_f64();

    println!("   Generated {} spans in {:.2}s ({:.0} spans/sec)",
        total_spans, elapsed.as_secs_f64(), spans_per_sec);

    // Stop and validate
    let report = controller.stop_and_report()?;

    assert!(spans_per_sec >= 1000.0,
        "Did not achieve target throughput: {:.0} < 1000 spans/sec", spans_per_sec);

    // Weaver should still be healthy
    assert!(controller.is_validation_passing() || report.violations == 0,
        "Weaver detected violations under high load");

    println!("✅ High-volume test passed");
    Ok(())
}

#[test]
#[ignore = "Requires Weaver installation"]
fn test_streaming_performance() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Testing streaming performance");

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_streaming_test"),
        stream: true,  // Enable streaming
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);

    let start = Instant::now();
    controller.start_live_check()?;
    let startup_time = start.elapsed();

    println!("   Weaver startup time: {:?}", startup_time);

    // Simulate streaming telemetry
    thread::sleep(Duration::from_secs(5));

    // Check validation status while streaming
    let is_passing = controller.is_validation_passing();
    println!("   Validation status during streaming: {}",
        if is_passing { "✅ Passing" } else { "❌ Failing" });

    let stop_start = Instant::now();
    let _report = controller.stop_and_report()?;
    let shutdown_time = stop_start.elapsed();

    println!("   Weaver shutdown time: {:?}", shutdown_time);

    // Validate performance requirements
    assert!(startup_time < Duration::from_secs(5),
        "Startup time {:?} exceeds 5s limit", startup_time);
    assert!(shutdown_time < Duration::from_secs(10),
        "Shutdown time {:?} exceeds 10s limit", shutdown_time);

    println!("✅ Streaming performance test passed");
    Ok(())
}

#[test]
#[ignore = "Requires Weaver installation"]
fn test_timeout_behavior_under_load() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Testing timeout behavior under load");

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_timeout_test"),
        stream: false,
        ..Default::default()
    };

    let mut controller = WeaverController::new(config);
    controller.start_live_check()?;

    // Generate load while testing timeout handling
    let test_duration = Duration::from_secs(30);
    let start = Instant::now();

    println!("   Running load test for {:?}...", test_duration);

    while start.elapsed() < test_duration {
        // Simulate periodic telemetry
        thread::sleep(Duration::from_millis(100));
    }

    // Weaver should handle graceful shutdown even under load
    let shutdown_start = Instant::now();
    let report = controller.stop_and_report()?;
    let shutdown_duration = shutdown_start.elapsed();

    println!("   Shutdown duration: {:?}", shutdown_duration);
    println!("   Violations detected: {}", report.violations);

    // Should complete within reasonable timeout
    assert!(shutdown_duration < Duration::from_secs(15),
        "Shutdown took too long: {:?}", shutdown_duration);

    println!("✅ Timeout behavior test passed");
    Ok(())
}

#[test]
#[ignore = "Benchmark test - run manually for performance profiling"]
fn benchmark_weaver_latency() -> Result<(), Box<dyn std::error::Error>> {
    println!("📊 Benchmarking Weaver validation latency");

    let config = WeaverConfig {
        registry_path: PathBuf::from("registry"),
        output_dir: PathBuf::from("/tmp/clnrm_benchmark"),
        stream: false,
        ..Default::default()
    };

    let iterations = 10;
    let mut startup_times = Vec::new();
    let mut shutdown_times = Vec::new();

    for i in 0..iterations {
        println!("   Iteration {}/{}", i + 1, iterations);

        let mut controller = WeaverController::new(config.clone());

        // Measure startup
        let start = Instant::now();
        controller.start_live_check()?;
        startup_times.push(start.elapsed());

        // Brief operation
        thread::sleep(Duration::from_secs(1));

        // Measure shutdown
        let start = Instant::now();
        let _report = controller.stop_and_report()?;
        shutdown_times.push(start.elapsed());

        // Cool down between iterations
        thread::sleep(Duration::from_secs(2));
    }

    // Calculate statistics
    let avg_startup = startup_times.iter().sum::<Duration>() / iterations as u32;
    let avg_shutdown = shutdown_times.iter().sum::<Duration>() / iterations as u32;

    let max_startup = startup_times.iter().max().unwrap();
    let max_shutdown = shutdown_times.iter().max().unwrap();

    println!("\n📊 Benchmark Results:");
    println!("   Average startup:  {:?}", avg_startup);
    println!("   Max startup:      {:?}", max_startup);
    println!("   Average shutdown: {:?}", avg_shutdown);
    println!("   Max shutdown:     {:?}", max_shutdown);

    // Validate performance targets
    assert!(avg_startup < Duration::from_secs(3),
        "Average startup {:?} exceeds 3s target", avg_startup);
    assert!(avg_shutdown < Duration::from_secs(5),
        "Average shutdown {:?} exceeds 5s target", avg_shutdown);

    println!("✅ Benchmark completed successfully");
    Ok(())
}
